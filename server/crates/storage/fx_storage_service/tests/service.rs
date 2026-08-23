//! fx_storage 编排层契约测试。
//!
//! 职责:用内存 fake 端口 + LocalFs(tempdir) 验证 StorageService 的去重、配额、删除、
//!      删除编排与回调,不依赖数据库/对象存储/真实图像解码(fake MetadataExtractor)。
//! 边界:只测可复用层编排逻辑;owner/scope 语义由 fake 忽略(属宿主关注)。
//! 约束:fake 端口用 std::sync::Mutex 做内部可变,临界区不跨 await。

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicI64, Ordering},
};

use async_trait::async_trait;
use fx_storage_service::{
    FileObjectStore, ImageMetadata, MetadataExtractor, NewStorageObject, QuotaPolicy,
    QuotaSnapshot, Scope, StorageError, StorageObject, StorageService, StorageServiceConfig,
    VideoMeta,
};
use fx_storage_local::LocalFs;

// ─── 不透明 scope 构造 helper ───

fn scope(s: &str) -> Scope {
    Arc::from(s)
}

// ─── fake 端口 ───

struct FakeStore {
    files: Mutex<Vec<StorageObject>>,
    next_id: AtomicI64,
}

impl FakeStore {
    fn new() -> Self {
        Self {
            files: Mutex::new(Vec::new()),
            next_id: AtomicI64::new(1),
        }
    }
    fn count(&self) -> usize {
        self.files.lock().unwrap().len()
    }
    fn only(&self) -> StorageObject {
        self.files.lock().unwrap()[0].clone()
    }
    fn get(&self, id: i64) -> Option<StorageObject> {
        self.files
            .lock()
            .unwrap()
            .iter()
            .find(|f| f.id == id)
            .cloned()
    }
}

#[async_trait]
impl FileObjectStore for FakeStore {
    async fn find_by_hash(
        &self,
        _scope: &Scope,
        hash: &str,
    ) -> Result<Option<StorageObject>, StorageError> {
        Ok(self
            .files
            .lock()
            .unwrap()
            .iter()
            .find(|f| f.hash == hash)
            .cloned())
    }

    async fn find_by_upload_id(
        &self,
        _scope: &Scope,
        _upload_id: uuid::Uuid,
    ) -> Result<Option<StorageObject>, StorageError> {
        Ok(None)
    }

    async fn insert(
        &self,
        obj: NewStorageObject,
        _scope: &Scope,
    ) -> Result<StorageObject, StorageError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let file = StorageObject {
            id,
            hash: obj.hash,
            storage_path: obj.storage_path,
            size: obj.size,
            mime_type: obj.mime_type,
            mime_category: obj.mime_category,
            width: obj.width,
            height: obj.height,
            duration_ms: obj.duration_ms,
            thumb_path: obj.thumb_path,
            original_name: obj.original_name,
            ref_count: 1,
        };
        self.files.lock().unwrap().push(file.clone());
        Ok(file)
    }

    async fn get(&self, id: i64, _scope: &Scope) -> Result<Option<StorageObject>, StorageError> {
        Ok(self.get(id))
    }

    async fn list(
        &self,
        _scope: &Scope,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StorageObject>, StorageError> {
        let files = self.files.lock().unwrap();
        let mut items: Vec<StorageObject> = files
            .iter()
            .filter(|f| category.is_none_or(|category| f.mime_category == category))
            .cloned()
            .collect();
        items.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(items
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }

    async fn count(&self, _scope: &Scope, category: Option<&str>) -> Result<i64, StorageError> {
        let count = self
            .files
            .lock()
            .unwrap()
            .iter()
            .filter(|f| category.is_none_or(|category| f.mime_category == category))
            .count();
        Ok(count as i64)
    }

    async fn delete(&self, id: i64) -> Result<(), StorageError> {
        self.files.lock().unwrap().retain(|f| f.id != id);
        Ok(())
    }

    async fn inc_ref(&self, id: i64) -> Result<(), StorageError> {
        if let Some(f) = self.files.lock().unwrap().iter_mut().find(|f| f.id == id) {
            f.ref_count += 1;
        }
        Ok(())
    }

    async fn dec_ref(&self, id: i64) -> Result<(), StorageError> {
        if let Some(f) = self.files.lock().unwrap().iter_mut().find(|f| f.id == id) {
            f.ref_count = (f.ref_count - 1).max(0);
        }
        Ok(())
    }
}

struct FakeQuota {
    state: Mutex<QuotaSnapshot>,
}

impl FakeQuota {
    fn new(used: i64, quota: i64) -> Self {
        Self {
            state: Mutex::new(QuotaSnapshot {
                used_bytes: used,
                quota_bytes: quota,
            }),
        }
    }
}

#[async_trait]
impl QuotaPolicy for FakeQuota {
    async fn check(&self, _scope: &Scope, size: i64) -> Result<QuotaSnapshot, StorageError> {
        let s = *self.state.lock().unwrap();
        if s.used_bytes + size > s.quota_bytes {
            return Err(StorageError::QuotaExceeded {
                used_bytes: s.used_bytes,
                quota_bytes: s.quota_bytes,
            });
        }
        Ok(s)
    }

    async fn consume(&self, _scope: &Scope, size: i64) -> Result<QuotaSnapshot, StorageError> {
        let mut s = self.state.lock().unwrap();
        if s.used_bytes + size > s.quota_bytes {
            return Err(StorageError::QuotaExceeded {
                used_bytes: s.used_bytes,
                quota_bytes: s.quota_bytes,
            });
        }
        s.used_bytes += size;
        Ok(*s)
    }

    async fn release(&self, _scope: &Scope, size: i64) -> Result<QuotaSnapshot, StorageError> {
        let mut s = self.state.lock().unwrap();
        s.used_bytes = (s.used_bytes - size).max(0);
        Ok(*s)
    }

    async fn current(&self, _scope: &Scope) -> Result<QuotaSnapshot, StorageError> {
        Ok(*self.state.lock().unwrap())
    }
}

struct FakeMeta {
    width: u32,
    height: u32,
}

#[async_trait]
impl MetadataExtractor for FakeMeta {
    async fn extract_image(&self, _data: &[u8]) -> Result<ImageMetadata, StorageError> {
        Ok(ImageMetadata {
            width: self.width,
            height: self.height,
            thumb: vec![0u8; 16],
            thumb_ext: "webp",
        })
    }
}

// ─── 装配 helper ───

fn build(
    tmp: &tempfile::TempDir,
    used: i64,
    quota: i64,
    max_image: u64,
) -> (Arc<StorageService<LocalFs>>, Arc<FakeStore>, Arc<FakeQuota>) {
    let cfg = StorageServiceConfig {
        max_image_size: max_image,
        ..Default::default()
    };
    let backend = LocalFs::new(tmp.path().to_path_buf(), "/uploads".into());
    let store = Arc::new(FakeStore::new());
    let quota = Arc::new(FakeQuota::new(used, quota));
    let meta = Arc::new(FakeMeta {
        width: 800,
        height: 600,
    });
    let svc = Arc::new(StorageService::new(
        backend,
        store.clone(),
        quota.clone(),
        meta,
        cfg,
    ));
    (svc, store, quota)
}

// ─── 测试用例 ───

#[tokio::test]
async fn upload_image_new_stores_and_consumes_quota() {
    let tmp = tempfile::tempdir().unwrap();
    let (svc, store, quota) = build(&tmp, 0, 1_000_000, 1_000_000);
    let data = b"fake-image-bytes";

    let r = svc
        .upload_image(&scope("user:1"), data, "a.jpg", "hash-a")
        .await
        .unwrap();

    assert!(!r.is_dedup);
    assert!(r.url.starts_with("/uploads/original/"));
    assert!(
        r.thumb_url
            .as_deref()
            .unwrap()
            .starts_with("/uploads/thumb/")
    );
    assert_eq!(store.count(), 1);
    // 原图 + 缩略图应落盘
    assert!(
        tmp.path()
            .join(r.url.trim_start_matches("/uploads/"))
            .exists()
    );
    assert_eq!(
        quota.current(&scope("user:1")).await.unwrap().used_bytes,
        data.len() as i64
    );
}

#[tokio::test]
async fn upload_image_dedup_reuses_owner_asset_without_second_store_or_quota() {
    let tmp = tempfile::tempdir().unwrap();
    let (svc, store, quota) = build(&tmp, 0, 1_000_000, 1_000_000);

    svc.upload_image(&scope("u"), b"data", "a.jpg", "hash-a")
        .await
        .unwrap();
    let before_files = store.count();
    let before_used = quota.current(&scope("u")).await.unwrap().used_bytes;

    let r = svc
        .upload_image(&scope("u"), b"data-dup", "a.jpg", "hash-a")
        .await
        .unwrap();

    assert!(r.is_dedup);
    assert_eq!(store.count(), before_files); // 无新记录
    assert_eq!(
        quota.current(&scope("u")).await.unwrap().used_bytes,
        before_used
    ); // 配额未再扣
    assert_eq!(store.get(r.file_id).unwrap().ref_count, 1);
}

#[tokio::test]
async fn upload_image_too_large_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let (svc, store, _quota) = build(&tmp, 0, 1_000_000, 10);
    let data = vec![0u8; 20];

    let err = svc
        .upload_image(&scope("u"), &data, "a.jpg", "h")
        .await
        .unwrap_err();

    assert!(matches!(err, StorageError::FileTooLarge { .. }));
    assert_eq!(store.count(), 0); // 未落库
}

#[tokio::test]
async fn upload_image_unsupported_type_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let (svc, _store, _quota) = build(&tmp, 0, 1_000_000, 1_000_000);

    let err = svc
        .upload_image(&scope("u"), b"x", "a.bmp", "h")
        .await
        .unwrap_err();

    assert!(matches!(err, StorageError::UnsupportedType(_)));
}

#[tokio::test]
async fn upload_image_quota_exceeded_aborts_before_store() {
    let tmp = tempfile::tempdir().unwrap();
    // 已用 990KB,配额 1MB;再传 20KB 会超
    let (svc, store, _quota) = build(&tmp, 990_000, 1_000_000, 1_000_000);
    let data = vec![0u8; 20_000];

    let err = svc
        .upload_image(&scope("u"), &data, "a.jpg", "h")
        .await
        .unwrap_err();

    assert!(matches!(err, StorageError::QuotaExceeded { .. }));
    assert_eq!(store.count(), 0); // check 在 put 之前,未落库
    assert!(tmp.path().read_dir().unwrap().count() == 0); // 未落盘
}

#[tokio::test]
async fn upload_file_new_stores_without_thumb() {
    let tmp = tempfile::tempdir().unwrap();
    let (svc, _store, quota) = build(&tmp, 0, 1_000_000, 1_000_000);

    let r = svc
        .upload_file(&scope("u"), b"hello", "doc.pdf", "h")
        .await
        .unwrap();

    assert!(!r.is_dedup);
    assert_eq!(r.mime_category, "file");
    assert_eq!(r.mime_type, "application/pdf");
    assert!(r.thumb_url.is_none());
    assert_eq!(quota.current(&scope("u")).await.unwrap().used_bytes, 5);
}

#[tokio::test]
async fn upload_video_stores_video_and_client_thumb() {
    let tmp = tempfile::tempdir().unwrap();
    let (svc, _store, quota) = build(&tmp, 0, 1_000_000, 50_000_000);
    let meta = VideoMeta {
        duration_ms: 1000,
        width: 640,
        height: 480,
    };

    let r = svc
        .upload_video(&scope("u"), b"videobytes", "v.mp4", b"thumbjpg", "h", meta)
        .await
        .unwrap();

    assert!(!r.is_dedup);
    assert_eq!(r.mime_category, "video");
    assert_eq!(r.duration_ms, Some(1000));
    assert!(r.thumb_url.is_some());
    assert_eq!(
        quota.current(&scope("u")).await.unwrap().used_bytes,
        b"videobytes".len() as i64
    );
}

#[tokio::test]
async fn delete_file_does_not_treat_refcount_as_delete_authority() {
    let tmp = tempfile::tempdir().unwrap();
    let (svc, store, quota) = build(&tmp, 0, 1_000_000, 1_000_000);
    // 画板引用关系由业务域维护，`StorageService` 不以冗余计数判断删除资格。
    svc.upload_image(&scope("u"), b"d", "a.jpg", "h")
        .await
        .unwrap();
    svc.upload_image(&scope("u"), b"d", "a.jpg", "h")
        .await
        .unwrap();
    let id = store.only().id;
    store.inc_ref(id).await.unwrap();
    let r = svc.delete_file(id, &scope("u")).await.unwrap();

    assert!(r.physical_deleted);
    assert_eq!(r.freed_bytes, 1);
    assert!(store.get(id).is_none());
    assert_eq!(quota.current(&scope("u")).await.unwrap().used_bytes, 0);
}

#[tokio::test]
async fn delete_file_last_ref_physical_deletes_and_releases_quota() {
    let tmp = tempfile::tempdir().unwrap();
    let (svc, store, quota) = build(&tmp, 0, 1_000_000, 1_000_000);
    svc.upload_image(&scope("u"), b"d", "a.jpg", "h")
        .await
        .unwrap();
    let id = store.only().id;
    let path_rel = store.only().storage_path.clone();
    let thumb_rel = store.only().thumb_path.clone().unwrap();

    let r = svc.delete_file(id, &scope("u")).await.unwrap();

    assert!(r.physical_deleted);
    assert_eq!(r.freed_bytes, 1);
    assert!(store.get(id).is_none()); // 行已删
    assert!(!tmp.path().join(&path_rel).exists()); // 原图已删
    assert!(!tmp.path().join(&thumb_rel).exists()); // 缩略图已删
    assert_eq!(quota.current(&scope("u")).await.unwrap().used_bytes, 0); // 配额已退
}

#[tokio::test]
async fn check_file_exists_does_not_increment_ref() {
    let tmp = tempfile::tempdir().unwrap();
    let (svc, store, _quota) = build(&tmp, 0, 1_000_000, 1_000_000);
    svc.upload_image(&scope("u"), b"d", "a.jpg", "h")
        .await
        .unwrap();

    let found = svc.check_file_exists(&scope("u"), "h").await.unwrap();

    assert!(found.is_some());
    assert_eq!(store.only().ref_count, 1); // 预检不递增
}

#[tokio::test]
async fn quota_snapshot_delegates_to_policy() {
    let tmp = tempfile::tempdir().unwrap();
    let (svc, _store, _quota) = build(&tmp, 990, 1_000_000, 1_000_000);

    let snap = svc.quota_snapshot(&scope("u")).await.unwrap();

    assert_eq!(snap.used_bytes, 990);
    assert_eq!(snap.quota_bytes, 1_000_000);
}

#[tokio::test]
async fn quota_changed_callback_fires_on_consume() {
    let tmp = tempfile::tempdir().unwrap();
    let backend = LocalFs::new(tmp.path().to_path_buf(), "/uploads".into());
    let store = Arc::new(FakeStore::new());
    let quota = Arc::new(FakeQuota::new(0, 1_000_000));
    let meta = Arc::new(FakeMeta {
        width: 1,
        height: 1,
    });
    let fired = Arc::new(Mutex::new(None::<QuotaSnapshot>));
    let fired2 = fired.clone();

    let svc = StorageService::new(backend, store, quota, meta, StorageServiceConfig::default())
        .with_quota_changed(Arc::new(move |_scope, snap| {
            *fired2.lock().unwrap() = Some(snap);
        }));

    svc.upload_image(&scope("u"), b"data", "a.jpg", "h")
        .await
        .unwrap();

    let snap = fired.lock().unwrap().clone().unwrap();
    assert_eq!(snap.used_bytes, 4);
}
