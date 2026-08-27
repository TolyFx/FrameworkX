//! 存储服务抽象端口(宿主注入实现)。
//!
//! 职责:定义文件对象持久化、配额策略、元数据提取三个端口,
//!      由 `StorageService` 编排,由宿主(如未来的 `fx_storage_pg`)实现。
//! 边界:端口仅依赖 `fx_storage_core` 的类型与错误;不绑定具体数据库表或业务模型。
//! 约束:所有方法返回 `StorageError`,宿主实现需将底层错误映射为对应变体。

use async_trait::async_trait;
use fx_storage_core::{QuotaSnapshot, Scope, StorageError, StorageObject};
use uuid::Uuid;

/// 文件对象持久化端口。
///
/// 宿主可在实现中追加 owner/scope 列与外键,核心不感知——`scope` 仅用于归属过滤与配额关联。
#[async_trait]
pub trait FileObjectStore: Send + Sync {
    /// 仅在当前归属内按 hash 查找，禁止跨账号去重命中。
    async fn find_by_hash(
        &self,
        scope: &Scope,
        hash: &str,
    ) -> Result<Option<StorageObject>, StorageError>;

    /// 按客户端上传幂等键查找当前归属的既有结果。
    async fn find_by_upload_id(
        &self,
        scope: &Scope,
        upload_id: Uuid,
    ) -> Result<Option<StorageObject>, StorageError>;

    async fn insert(
        &self,
        obj: NewStorageObject,
        scope: &Scope,
    ) -> Result<StorageObject, StorageError>;

    /// 读取文件(带 scope 归属校验)
    async fn get(&self, id: i64, scope: &Scope) -> Result<Option<StorageObject>, StorageError>;

    /// 分页读取文件(带 scope 归属校验),`category` 由宿主按 mime_category 过滤
    async fn list(
        &self,
        scope: &Scope,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StorageObject>, StorageError>;

    /// 统计文件总数(带 scope 归属校验),与 `list` 使用相同过滤条件
    async fn count(&self, scope: &Scope, category: Option<&str>) -> Result<i64, StorageError>;

    async fn delete(&self, id: i64) -> Result<(), StorageError>;
    async fn inc_ref(&self, id: i64) -> Result<(), StorageError>;
    async fn dec_ref(&self, id: i64) -> Result<(), StorageError>;
}

/// 配额策略端口。
///
/// 宿主可实现不限额(`NoopQuotaPolicy`)或按 scope 配额策略,
/// 核心只消费 `QuotaSnapshot`,不关心阈值来源。
#[async_trait]
pub trait QuotaPolicy: Send + Sync {
    /// 预检,不扣减;超限时返回 `QuotaExceeded`
    async fn check(&self, scope: &Scope, size: i64) -> Result<QuotaSnapshot, StorageError>;
    async fn consume(&self, scope: &Scope, size: i64) -> Result<QuotaSnapshot, StorageError>;
    async fn release(&self, scope: &Scope, size: i64) -> Result<QuotaSnapshot, StorageError>;
    async fn current(&self, scope: &Scope) -> Result<QuotaSnapshot, StorageError>;
}

/// 图像元数据提取结果(含缩略图字节)
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub thumb: Vec<u8>,
    pub thumb_ext: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFit {
    Contain,
    Cover,
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageOutputFormat {
    WebP,
    Jpeg,
    Png,
}

#[derive(Debug, Clone, Copy)]
pub struct ImageTransform {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fit: ImageFit,
    pub quality: u8,
    pub format: ImageOutputFormat,
}

pub struct TransformedImage {
    pub bytes: Vec<u8>,
    pub mime_type: &'static str,
}

/// 视频元数据(由客户端预提取并随请求上传,服务端不解码视频)
#[derive(Debug, Clone, Copy)]
pub struct VideoMeta {
    pub duration_ms: u64,
    pub width: u32,
    pub height: u32,
}

/// 元数据/缩略图提取端口。默认实现见 `fx_storage_image`。
#[async_trait]
pub trait MetadataExtractor: Send + Sync {
    async fn extract_image(&self, data: &[u8]) -> Result<ImageMetadata, StorageError>;

    async fn transform_image(
        &self,
        _data: &[u8],
        _transform: ImageTransform,
    ) -> Result<TransformedImage, StorageError> {
        Err(StorageError::UnsupportedType(
            "动态图片变体未配置".to_owned(),
        ))
    }
}

/// 待持久化的文件对象(owner 无关,scope 单独传入)。
#[derive(Debug, Clone)]
pub struct NewStorageObject {
    pub hash: String,
    pub storage_path: String,
    pub size: i64,
    pub mime_type: String,
    pub mime_category: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub thumb_path: Option<String>,
    pub original_name: Option<String>,
    /// 仅客户端上传链路携带，用于请求丢失响应后的安全重试。
    pub upload_id: Option<Uuid>,
}
