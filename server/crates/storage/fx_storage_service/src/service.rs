//! 存储服务编排层(通用上传/去重/删除流程)。
//!
//! 职责:在 `StorageBackend` 与三个注入端口之上编排上传、去重、引用计数与删除;
//!      产出内容寻址 `StorageObject` 与 `UploadResult`,不绑定 owner。
//! 边界:泛型 `<B: StorageBackend>` 静态分发后端;端口以 trait object 注入,宿主决定实现。
//! 约束:不得出现 user_id/accounts/业务链接 等宿主概念;配额经端口透传 `Scope`。
//!      业务引用关系(谁引用了此文件)由业务域自管链接表 + 调 `inc_ref/dec_ref`,核心不感知。

use std::sync::Arc;

use fx_storage_core::{QuotaSnapshot, Scope, StorageBackend, StorageError, StorageObject};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config::{StorageServiceConfig, date_path, ext_of, mime_from_ext, truncate_filename};
use crate::ports::{
    FileObjectStore, ImageTransform, MetadataExtractor, NewStorageObject, QuotaPolicy, VideoMeta,
};

/// 上传统一结果。`url` 由后端 `url_for` 解析,核心不关心本地/对象存储。
#[derive(Debug, serde::Serialize)]
pub struct UploadResult {
    pub file_id: i64,
    /// 私有原图读取路径；客户端应优先使用它而非公开 URL。
    pub content_path: String,
    /// 私有缩略图读取路径。
    pub thumbnail_path: Option<String>,
    /// 兼容旧客户端的公开 URL；Canvas v3 不会保存该字段。
    pub url: String,
    pub thumb_url: Option<String>,
    pub size: i64,
    pub is_dedup: bool,
    pub mime_type: String,
    pub mime_category: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub original_name: Option<String>,
    pub quota: QuotaSnapshot,
}

/// 删除结果。是否可删除由宿主业务先检查资源引用；本服务只执行物理删除。
#[derive(Debug, serde::Serialize)]
pub struct DeleteResult {
    pub physical_deleted: bool,
    pub freed_bytes: i64,
    pub snapshot: QuotaSnapshot,
}

/// 云空间文件信息。`url` 与 `thumb_url` 由后端解析,不暴露后端类型。
#[derive(Debug, serde::Serialize)]
pub struct FileInfo {
    pub id: i64,
    pub hash: String,
    pub content_path: String,
    pub thumbnail_path: Option<String>,
    pub url: String,
    pub thumb_url: Option<String>,
    pub size: i64,
    pub mime_type: String,
    pub mime_category: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub original_name: Option<String>,
    pub ref_count: i32,
}

/// 云空间分页结果。
#[derive(Debug, serde::Serialize)]
pub struct FileListResult {
    pub items: Vec<FileInfo>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

/// 已鉴权资源内容。HTTP 层只负责写响应头和字节流。
pub struct FileContent {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

/// 配额变更回调签名:`(scope, snapshot)`
type QuotaChanged = Arc<dyn Fn(&Scope, QuotaSnapshot) + Send + Sync>;

pub struct StorageService<B: StorageBackend> {
    backend: B,
    store: Arc<dyn FileObjectStore>,
    quota: Arc<dyn QuotaPolicy>,
    metadata: Arc<dyn MetadataExtractor>,
    config: StorageServiceConfig,
    on_quota_changed: Option<QuotaChanged>,
}

impl<B: StorageBackend> StorageService<B> {
    pub fn new(
        backend: B,
        store: Arc<dyn FileObjectStore>,
        quota: Arc<dyn QuotaPolicy>,
        metadata: Arc<dyn MetadataExtractor>,
        config: StorageServiceConfig,
    ) -> Self {
        Self {
            backend,
            store,
            quota,
            metadata,
            config,
            on_quota_changed: None,
        }
    }

    pub fn with_quota_changed(mut self, cb: QuotaChanged) -> Self {
        self.on_quota_changed = Some(cb);
        self
    }

    pub fn max_image_size(&self) -> u64 {
        self.config.max_image_size
    }
    pub fn max_video_size(&self) -> u64 {
        self.config.max_video_size
    }
    pub fn max_file_size(&self) -> u64 {
        self.config.max_file_size
    }

    /// 按后端规则解析 storage_path 的访问 URL(委托给 `StorageBackend::url_for`)
    pub fn url_for(&self, path: &str) -> String {
        self.backend.url_for(path)
    }

    fn notify(&self, scope: &Scope, snap: QuotaSnapshot) {
        if let Some(cb) = &self.on_quota_changed {
            cb(scope, snap);
        }
    }

    /// 秒传预检(不递增引用计数)
    pub async fn check_file_exists(
        &self,
        scope: &Scope,
        hash: &str,
    ) -> Result<Option<StorageObject>, StorageError> {
        self.store.find_by_hash(scope, hash).await
    }

    /// 仅配额预检,不扣减
    pub async fn check_quota_only(&self, scope: &Scope, size: i64) -> Result<(), StorageError> {
        self.quota.check(scope, size).await.map(|_| ())
    }

    /// 查询当前配额快照(委托给 QuotaPolicy::current)
    pub async fn quota_snapshot(&self, scope: &Scope) -> Result<QuotaSnapshot, StorageError> {
        self.quota.current(scope).await
    }

    /// 分页列出当前 scope 的文件。`category` 对应 mime_category,如 image/video/file/audio。
    pub async fn list_files(
        &self,
        scope: &Scope,
        category: Option<&str>,
        page: i64,
        limit: i64,
    ) -> Result<FileListResult, StorageError> {
        let page = page.max(1);
        let limit = limit.clamp(1, 50);
        let offset = (page - 1) * limit;
        let total = self.store.count(scope, category).await?;
        let files = self.store.list(scope, category, limit, offset).await?;
        let items = files
            .into_iter()
            .map(|file| file_info(file, &self.backend))
            .collect();
        Ok(FileListResult {
            items,
            total,
            page,
            limit,
        })
    }

    /// 读取当前 scope 的文件详情。
    pub async fn file_detail(&self, id: i64, scope: &Scope) -> Result<FileInfo, StorageError> {
        let file = self
            .store
            .get(id, scope)
            .await?
            .ok_or(StorageError::NotFound)?;
        Ok(file_info(file, &self.backend))
    }

    /// 读取当前归属的原图或缩略图，避免把私有资产交给公开静态 URL。
    pub async fn read_file(
        &self,
        id: i64,
        scope: &Scope,
        thumbnail: bool,
    ) -> Result<FileContent, StorageError> {
        let file: StorageObject = self
            .store
            .get(id, scope)
            .await?
            .ok_or(StorageError::NotFound)?;
        let path: &str = if thumbnail {
            file.thumb_path.as_deref().ok_or(StorageError::NotFound)?
        } else {
            &file.storage_path
        };
        let bytes: Vec<u8> = self.backend.get(path).await?;
        let mime_type: String = if thumbnail {
            "image/webp".to_string()
        } else {
            file.mime_type
        };
        Ok(FileContent { bytes, mime_type })
    }

    /// 读取当前 scope 的本地或远端原图，并按调用方已校验的规格动态生成图片变体。
    pub async fn read_image_variant(
        &self,
        id: i64,
        scope: &Scope,
        transform: ImageTransform,
    ) -> Result<FileContent, StorageError> {
        let file = self
            .store
            .get(id, scope)
            .await?
            .ok_or(StorageError::NotFound)?;
        if file.mime_category != "image" {
            return Err(StorageError::UnsupportedType(file.mime_type));
        }
        let source = self.backend.get(&file.storage_path).await?;
        let transformed = self.metadata.transform_image(&source, transform).await?;
        Ok(FileContent {
            bytes: transformed.bytes,
            mime_type: transformed.mime_type.to_owned(),
        })
    }

    /// 当前归属内的去重不改变引用计数；画板引用由业务链接表单独维护。
    async fn check_dedup(
        &self,
        scope: &Scope,
        hash: &str,
    ) -> Result<Option<StorageObject>, StorageError> {
        self.store.find_by_hash(scope, hash).await
    }

    /// 上传图片(含去重 + 配额检查 + 服务端缩略图)
    pub async fn upload_image(
        &self,
        scope: &Scope,
        data: &[u8],
        filename: &str,
        hash: &str,
    ) -> Result<UploadResult, StorageError> {
        self.upload_image_with_id(scope, data, filename, hash, Uuid::new_v4())
            .await
    }

    /// 上传图片并接受稳定幂等键；同一 owner 重试会得到同一资产。
    pub async fn upload_image_with_id(
        &self,
        scope: &Scope,
        data: &[u8],
        filename: &str,
        hash: &str,
        upload_id: Uuid,
    ) -> Result<UploadResult, StorageError> {
        validate_sha256(hash, data)?;
        if let Some(file) = self.store.find_by_upload_id(scope, upload_id).await? {
            let quota: QuotaSnapshot = self.quota.current(scope).await?;
            return Ok(dedup_result(file, &self.backend, quota));
        }
        if let Some(file) = self.check_dedup(scope, hash).await? {
            let quota: QuotaSnapshot = self.quota.current(scope).await?;
            return Ok(dedup_result(file, &self.backend, quota));
        }

        let size = data.len() as i64;
        if size as u64 > self.config.max_image_size {
            return Err(StorageError::FileTooLarge {
                size: size as u64,
                max: self.config.max_image_size,
            });
        }

        let ext = ext_of(filename, "jpg");
        let format = match ext.as_str() {
            "jpg" | "jpeg" => "jpg",
            "png" => "png",
            "gif" => "gif",
            "webp" => "webp",
            _ => return Err(StorageError::UnsupportedType(ext)),
        };

        let meta = self.metadata.extract_image(data).await?;
        let date = date_path();
        let uid = Uuid::new_v4();
        let original_rel = format!("original/{}/{}.{}", date, uid, ext);
        let thumb_rel = format!("thumb/{}/{}.{}", date, uid, meta.thumb_ext);

        let snap: QuotaSnapshot = self.quota.consume(scope, size).await?;
        if let Err(error) = self.backend.put(&original_rel, data).await {
            let _result: QuotaSnapshot = self.quota.release(scope, size).await?;
            return Err(error.into());
        }
        if let Err(error) = self.backend.put(&thumb_rel, &meta.thumb).await {
            let _result: Result<(), std::io::Error> = self.backend.delete(&original_rel).await;
            let _snapshot: QuotaSnapshot = self.quota.release(scope, size).await?;
            return Err(error.into());
        }

        let mime_type = format!("image/{}", format);
        let original_name = truncate_filename(filename, 255);
        let insert_result = self
            .store
            .insert(
                NewStorageObject {
                    hash: hash.to_string(),
                    storage_path: original_rel.clone(),
                    size,
                    mime_type: mime_type.clone(),
                    mime_category: "image".to_string(),
                    width: Some(meta.width as i32),
                    height: Some(meta.height as i32),
                    duration_ms: None,
                    thumb_path: Some(thumb_rel.clone()),
                    original_name: Some(original_name.clone()),
                    upload_id: Some(upload_id),
                },
                scope,
            )
            .await;
        let obj: StorageObject = match insert_result {
            Ok(value) => value,
            Err(error) => {
                let _original_deleted: Result<(), std::io::Error> =
                    self.backend.delete(&original_rel).await;
                let _thumb_deleted: Result<(), std::io::Error> =
                    self.backend.delete(&thumb_rel).await;
                let _released: Result<QuotaSnapshot, StorageError> =
                    self.quota.release(scope, size).await;
                if let Some(existing) = self.store.find_by_hash(scope, hash).await? {
                    let quota: QuotaSnapshot = self.quota.current(scope).await?;
                    return Ok(dedup_result(existing, &self.backend, quota));
                }
                return Err(error);
            }
        };
        self.notify(scope, snap);

        Ok(UploadResult {
            file_id: obj.id,
            content_path: format!("/storage/files/{}/content", obj.id),
            thumbnail_path: Some(format!(
                "/storage/files/{}/content?variant=thumbnail",
                obj.id
            )),
            url: self.backend.url_for(&original_rel),
            thumb_url: Some(self.backend.url_for(&thumb_rel)),
            size,
            is_dedup: false,
            mime_type,
            mime_category: "image".to_string(),
            width: Some(meta.width as i32),
            height: Some(meta.height as i32),
            duration_ms: None,
            original_name: Some(original_name),
            quota: snap,
        })
    }

    /// 上传视频(缩略图与元数据由客户端提供)
    pub async fn upload_video(
        &self,
        scope: &Scope,
        video: &[u8],
        video_filename: &str,
        thumb: &[u8],
        hash: &str,
        meta: VideoMeta,
    ) -> Result<UploadResult, StorageError> {
        if let Some(file) = self.check_dedup(scope, hash).await? {
            let quota: QuotaSnapshot = self.quota.current(scope).await?;
            return Ok(dedup_result(file, &self.backend, quota));
        }

        let size = video.len() as i64;
        if size as u64 > self.config.max_video_size {
            return Err(StorageError::FileTooLarge {
                size: size as u64,
                max: self.config.max_video_size,
            });
        }

        let ext = ext_of(video_filename, "mp4");
        match ext.as_str() {
            "mp4" | "mov" | "avi" => {}
            _ => return Err(StorageError::UnsupportedType(ext)),
        }

        self.quota.check(scope, size).await?;

        let date = date_path();
        let uid = Uuid::new_v4();
        let video_rel = format!("video/{}/{}.{}", date, uid, ext);
        let thumb_rel = format!("thumb/{}/{}.jpg", date, uid);

        self.backend.put(&video_rel, video).await?;
        self.backend.put(&thumb_rel, thumb).await?;

        let mime_type = format!("video/{}", ext);
        let original_name = truncate_filename(video_filename, 255);
        let obj = self
            .store
            .insert(
                NewStorageObject {
                    hash: hash.to_string(),
                    storage_path: video_rel.clone(),
                    size,
                    mime_type: mime_type.clone(),
                    mime_category: "video".to_string(),
                    width: Some(meta.width as i32),
                    height: Some(meta.height as i32),
                    duration_ms: Some(meta.duration_ms as i64),
                    thumb_path: Some(thumb_rel.clone()),
                    original_name: Some(original_name.clone()),
                    upload_id: None,
                },
                scope,
            )
            .await?;

        let snap = self.quota.consume(scope, size).await?;
        self.notify(scope, snap);

        Ok(UploadResult {
            file_id: obj.id,
            content_path: format!("/storage/files/{}/content", obj.id),
            thumbnail_path: Some(format!(
                "/storage/files/{}/content?variant=thumbnail",
                obj.id
            )),
            url: self.backend.url_for(&video_rel),
            thumb_url: Some(self.backend.url_for(&thumb_rel)),
            size,
            is_dedup: false,
            mime_type,
            mime_category: "video".to_string(),
            width: Some(meta.width as i32),
            height: Some(meta.height as i32),
            duration_ms: Some(meta.duration_ms as i64),
            original_name: Some(original_name),
            quota: snap,
        })
    }

    /// 上传通用文件(无缩略图、按扩展名推断分类)
    pub async fn upload_file(
        &self,
        scope: &Scope,
        data: &[u8],
        filename: &str,
        hash: &str,
    ) -> Result<UploadResult, StorageError> {
        if let Some(file) = self.check_dedup(scope, hash).await? {
            let quota: QuotaSnapshot = self.quota.current(scope).await?;
            return Ok(dedup_result(file, &self.backend, quota));
        }

        let size = data.len() as i64;
        if size as u64 > self.config.max_file_size {
            return Err(StorageError::FileTooLarge {
                size: size as u64,
                max: self.config.max_file_size,
            });
        }

        self.quota.check(scope, size).await?;

        let ext = ext_of(filename, "bin");
        let date = date_path();
        let uid = Uuid::new_v4();
        let file_rel = format!("file/{}/{}.{}", date, uid, ext);

        self.backend.put(&file_rel, data).await?;

        let mime_category = match ext.as_str() {
            "mp3" | "wav" | "aac" | "ogg" | "m4a" => "audio",
            _ => "file",
        };
        let mime_type = mime_from_ext(&ext);
        let original_name = truncate_filename(filename, 255);
        let obj = self
            .store
            .insert(
                NewStorageObject {
                    hash: hash.to_string(),
                    storage_path: file_rel.clone(),
                    size,
                    mime_type: mime_type.clone(),
                    mime_category: mime_category.to_string(),
                    width: None,
                    height: None,
                    duration_ms: None,
                    thumb_path: None,
                    original_name: Some(original_name.clone()),
                    upload_id: None,
                },
                scope,
            )
            .await?;

        let snap = self.quota.consume(scope, size).await?;
        self.notify(scope, snap);

        Ok(UploadResult {
            file_id: obj.id,
            content_path: format!("/storage/files/{}/content", obj.id),
            thumbnail_path: None,
            url: self.backend.url_for(&file_rel),
            thumb_url: None,
            size,
            is_dedup: false,
            mime_type,
            mime_category: mime_category.to_string(),
            width: None,
            height: None,
            duration_ms: None,
            original_name: Some(original_name),
            quota: snap,
        })
    }

    /// 递增引用计数。业务域(IM 消息、帖子等)在自己链接表新增一条引用时调用。
    pub async fn inc_ref(&self, file_id: i64) -> Result<(), StorageError> {
        self.store.inc_ref(file_id).await
    }

    /// 递减引用计数(不低于 0)。业务域移除自己的引用时调用。
    pub async fn dec_ref(&self, file_id: i64) -> Result<(), StorageError> {
        self.store.dec_ref(file_id).await
    }

    /// 物理删除文件并释放配额。
    ///
    /// 业务引用关系由宿主检查；不能以冗余 `ref_count` 作为删除授权依据。
    pub async fn delete_file(&self, id: i64, scope: &Scope) -> Result<DeleteResult, StorageError> {
        let file = self
            .store
            .get(id, scope)
            .await?
            .ok_or(StorageError::NotFound)?;

        let size = file.size;
        let _ = self.backend.delete(&file.storage_path).await;
        if let Some(t) = &file.thumb_path {
            let _ = self.backend.delete(t).await;
        }
        self.store.delete(id).await?;
        let snap = self.quota.release(scope, size).await?;
        self.notify(scope, snap);

        Ok(DeleteResult {
            physical_deleted: true,
            freed_bytes: size,
            snapshot: snap,
        })
    }
}

fn validate_sha256(expected: &str, data: &[u8]) -> Result<(), StorageError> {
    let valid_format = expected.len() == 64
        && expected
            .bytes()
            .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value));
    if !valid_format {
        return Err(StorageError::InvalidHash);
    }
    let actual = format!("{:x}", Sha256::digest(data));
    if actual != expected {
        return Err(StorageError::HashMismatch);
    }
    Ok(())
}

/// 由命中去重的既有对象构造 `UploadResult`
fn dedup_result<B: StorageBackend>(
    file: StorageObject,
    backend: &B,
    quota: QuotaSnapshot,
) -> UploadResult {
    UploadResult {
        file_id: file.id,
        content_path: format!("/storage/files/{}/content", file.id),
        thumbnail_path: file
            .thumb_path
            .as_ref()
            .map(|_| format!("/storage/files/{}/content?variant=thumbnail", file.id)),
        url: backend.url_for(&file.storage_path),
        thumb_url: file.thumb_path.as_deref().map(|p| backend.url_for(p)),
        size: file.size,
        is_dedup: true,
        mime_type: file.mime_type,
        mime_category: file.mime_category,
        width: file.width,
        height: file.height,
        duration_ms: file.duration_ms,
        original_name: file.original_name,
        quota,
    }
}

fn file_info<B: StorageBackend>(file: StorageObject, backend: &B) -> FileInfo {
    FileInfo {
        id: file.id,
        hash: file.hash,
        content_path: format!("/storage/files/{}/content", file.id),
        thumbnail_path: file
            .thumb_path
            .as_ref()
            .map(|_| format!("/storage/files/{}/content?variant=thumbnail", file.id)),
        url: backend.url_for(&file.storage_path),
        thumb_url: file.thumb_path.as_deref().map(|p| backend.url_for(p)),
        size: file.size,
        mime_type: file.mime_type,
        mime_category: file.mime_category,
        width: file.width,
        height: file.height,
        duration_ms: file.duration_ms,
        original_name: file.original_name,
        ref_count: file.ref_count,
    }
}
