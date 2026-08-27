//! 文件存储核心层。
//!
//! 职责:定义可复用存储模块对外稳定的核心协议与类型,不含任何用户/IM/宿主耦合。
//! 边界:本 crate 不依赖 flash-core 或任何宿主模块;只产出内容寻址的 `StorageObject`
//!      与不透明 `Scope`,owner/配额/引用目标全部上浮到宿主。
//! 约束:`StorageBackend` 由具体后端 crate(fx_storage_local / fx_storage_oss)实现;
//!      此处不得引入 io 之外的运行时或业务依赖。

use std::future::Future;
use std::sync::Arc;

/// 归属的不透明标识。核心不解释其语义,宿主可定义为 `"user:42"` / `"tenant:acme"` 等。
/// 之所以用 `Arc<str>` 而非具体用户 id,是为了让核心保持用户无关。
pub type Scope = Arc<str>;

/// 统一存储错误。宿主端口实现需将底层(数据库/对象存储)错误映射为对应变体。
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("文件 hash 格式无效")]
    InvalidHash,

    #[error("文件内容与 hash 不一致")]
    HashMismatch,

    #[error("文件类型不支持: {0}")]
    UnsupportedType(String),

    #[error("文件过大: {size} bytes, 最大 {max} bytes")]
    FileTooLarge { size: u64, max: u64 },

    #[error("配额不足")]
    QuotaExceeded { used_bytes: i64, quota_bytes: i64 },

    #[error("文件不存在")]
    NotFound,

    #[error("无权访问: {0}")]
    Forbidden(String),

    #[error("文件未上传到存储: {0}")]
    NotOnStore(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("图片处理错误: {0}")]
    Image(String),

    #[error("存储后端错误: {0}")]
    Backend(String),

    #[error("数据存储错误: {0}")]
    Store(String),
}

/// 存储后端抽象。具体实现见 `fx_storage_local` / `fx_storage_oss`。
///
/// 每个 backend 自带 `url_for`,消除靠路径前缀猜测 URL 的耦合
/// (取代原 app-storage 中按 `users/` 前缀选择 URL 的做法)。
pub trait StorageBackend: Send + Sync + 'static {
    fn put(
        &self,
        path: &str,
        data: &[u8],
    ) -> impl Future<Output = Result<(), std::io::Error>> + Send;
    fn get(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, std::io::Error>> + Send;
    fn delete(&self, path: &str) -> impl Future<Output = Result<(), std::io::Error>> + Send;
    fn exists(&self, path: &str) -> impl Future<Output = Result<bool, std::io::Error>> + Send;
    fn url_for(&self, path: &str) -> String;
}

/// 内容寻址文件对象(无 owner)。
///
/// owner 关联由宿主在 `FileObjectStore` 实现侧完成,核心不感知——这是「用户无关」的边界。
#[derive(Debug, Clone, serde::Serialize)]
pub struct StorageObject {
    pub id: i64,
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
    pub ref_count: i32,
}

/// 配额快照。配额阈值与已用量均由宿主策略决定,核心只透传。
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct QuotaSnapshot {
    pub used_bytes: i64,
    pub quota_bytes: i64,
}
