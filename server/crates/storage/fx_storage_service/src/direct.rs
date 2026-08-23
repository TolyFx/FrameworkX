//! 直传编排层(可复用):签发直传凭证 + 确认上传,不绑死云厂商。
//!
//! 职责:在 `StorageBackend` + `DirectUploadIssuer` 端口之上编排直传通用流程
//!      (object_key 路径生成 / 配额预检 / 签发 / 确认存在性 + 落库 + 扣配额)。
//! 边界:云厂商特有的 STS 签名(如阿里云 AssumeRole + `acs:oss` 资源串)由 `DirectUploadIssuer`
//!      实现提供(见 `fx_storage_sts::AliyunStsIssuer`),核心只消费 `StsCredentials`。
//! 约束:owner 段替换冒号(scope `user:5` → `user-5`),保 OSS 路径与 STS 资源 ARN 安全。

use std::sync::Arc;

use async_trait::async_trait;
use fx_storage_core::{QuotaSnapshot, Scope, StorageBackend, StorageError, StorageObject};
use uuid::Uuid;

use crate::config::{date_path, ext_of};
use crate::ports::{FileObjectStore, NewStorageObject, QuotaPolicy};

/// 直传签发端口:为给定对象 resources 签发临时上传凭证。
///
/// 云厂商特有(STS 协议、资源 ARN 格式)由实现封装;`resources` 为相对 object_key,
/// 实现负责包装成对应云的资源标识。
#[async_trait]
pub trait DirectUploadIssuer: Send + Sync {
    async fn issue_grant(
        &self,
        session: &str,
        resources: &[String],
        duration_secs: u32,
    ) -> Result<StsCredentials, StorageError>;
}

/// 签发结果:临时凭证 + 云目标(bucket/endpoint)
#[derive(Debug, Clone, serde::Serialize)]
pub struct StsCredentials {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub security_token: String,
    pub expiration: String,
    pub bucket: String,
    pub endpoint: String,
}

/// 签发请求(宿主从 HTTP body 构造)
#[derive(Debug, Clone)]
pub struct IssueGrantRequest {
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub thumb_size: Option<i64>,
}

/// 签发返回的直传凭证(含服务端生成的 object_key 与访问 URL)
#[derive(Debug, serde::Serialize)]
pub struct UploadGrant {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub security_token: String,
    pub expiration: String,
    pub bucket: String,
    pub endpoint: String,
    pub object_key: String,
    pub thumb_object_key: Option<String>,
    pub url: String,
    pub thumb_url: Option<String>,
}

/// 确认请求(客户端直传 OSS 后调用)
#[derive(Debug, Clone)]
pub struct ConfirmRequest {
    pub object_key: String,
    pub file_size: i64,
    pub mime_type: String,
    pub mime_category: String,
    pub hash: String,
    pub thumb_object_key: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub original_name: Option<String>,
}

/// 确认结果
#[derive(Debug, serde::Serialize)]
pub struct ConfirmResult {
    pub file_id: i64,
    pub url: String,
    pub thumb_url: Option<String>,
    pub quota: QuotaSnapshot,
}

/// 直传编排服务。`B` 为对象存储后端(直传目标,如 `OssBackend`),
/// `I` 为签发端口实现(如 `AliyunStsIssuer`)。
pub struct DirectUploadService<B: StorageBackend, I: DirectUploadIssuer> {
    backend: B,
    issuer: I,
    store: Arc<dyn FileObjectStore>,
    quota: Arc<dyn QuotaPolicy>,
}

impl<B: StorageBackend, I: DirectUploadIssuer> DirectUploadService<B, I> {
    pub fn new(
        backend: B,
        issuer: I,
        store: Arc<dyn FileObjectStore>,
        quota: Arc<dyn QuotaPolicy>,
    ) -> Self {
        Self {
            backend,
            issuer,
            store,
            quota,
        }
    }

    /// 签发直传凭证:配额预检 + 生成 object_key + 调 issuer 签发
    pub async fn issue_grant(
        &self,
        scope: &Scope,
        req: IssueGrantRequest,
    ) -> Result<UploadGrant, StorageError> {
        self.quota.check(scope, req.file_size).await?;

        let owner = scope.as_ref().replace(':', "-");
        let ext = ext_of(&req.file_name, "bin");
        let date = date_path();
        let uid = Uuid::new_v4();
        let category_dir = match req.mime_type.split('/').next().unwrap_or("file") {
            "image" => "original",
            "video" => "video",
            _ => "file",
        };
        let object_key = format!("users/{}/{}/{}/{}.{}", owner, category_dir, date, uid, ext);
        let thumb_object_key = req
            .thumb_size
            .map(|_| format!("users/{}/thumb/{}/{}.webp", owner, date, uid));

        let mut resources = vec![object_key.clone()];
        if let Some(ref thumb) = thumb_object_key {
            resources.push(thumb.clone());
        }
        let session = format!("upload-{}", owner);
        let creds = self.issuer.issue_grant(&session, &resources, 900).await?;

        let url = self.backend.url_for(&object_key);
        let thumb_url = thumb_object_key.as_ref().map(|k| self.backend.url_for(k));

        Ok(UploadGrant {
            access_key_id: creds.access_key_id,
            access_key_secret: creds.access_key_secret,
            security_token: creds.security_token,
            expiration: creds.expiration,
            bucket: creds.bucket,
            endpoint: creds.endpoint,
            object_key,
            thumb_object_key,
            url,
            thumb_url,
        })
    }

    /// 确认直传:归属校验 + 存在性校验 + 落库 + 扣配额
    pub async fn confirm(
        &self,
        scope: &Scope,
        req: ConfirmRequest,
    ) -> Result<ConfirmResult, StorageError> {
        let owner = scope.as_ref().replace(':', "-");
        let expected_prefix = format!("users/{}/", owner);
        if !req.object_key.starts_with(&expected_prefix) {
            return Err(StorageError::Forbidden(
                "object_key 不属于当前 scope".into(),
            ));
        }
        if !self.backend.exists(&req.object_key).await? {
            return Err(StorageError::NotOnStore(req.object_key.clone()));
        }

        let obj: StorageObject = self
            .store
            .insert(
                NewStorageObject {
                    hash: req.hash,
                    storage_path: req.object_key.clone(),
                    size: req.file_size,
                    mime_type: req.mime_type,
                    mime_category: req.mime_category,
                    width: req.width,
                    height: req.height,
                    duration_ms: req.duration_ms,
                    thumb_path: req.thumb_object_key.clone(),
                    original_name: req.original_name,
                    upload_id: None,
                },
                scope,
            )
            .await?;

        let snap = self.quota.consume(scope, req.file_size).await?;

        Ok(ConfirmResult {
            file_id: obj.id,
            url: self.backend.url_for(&req.object_key),
            thumb_url: req
                .thumb_object_key
                .as_ref()
                .map(|k| self.backend.url_for(k)),
            quota: snap,
        })
    }
}
