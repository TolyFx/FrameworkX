//! 本地与 OSS 双后端路由。
//!
//! 职责:实现 `StorageBackend`,把 put/get/delete/exists/url_for 按 `users/` 前缀路由到
//!      OSS(直传文件)或本地(中转文件),让 `StorageService` 的删除与 URL 解析对两类文件都正确——
//!      修复「直传 OSS 文件 `DELETE` 删不掉 OSS 对象」与「列表 URL 错指本地」两个问题。
//! 边界:FrameworkX 默认后端组合；直传 object key 由 storage service 统一生成 `users/` 前缀。
//! 约束:核心 `StorageService` 对此无感知,只调 `backend.delete`/`url_for`;未配 OSS 时 `oss=None`,
//!      `users/` 分支不会被命中(无直传文件)。

use fx_storage_core::StorageBackend;
use fx_storage_local::LocalFs;
use fx_storage_oss::OssBackend;

/// 双后端:`local` 始终存在;`oss` 仅在 OSS env 配齐时存在。
#[derive(Clone)]
pub struct DualBackend {
    pub local: LocalFs,
    pub oss: Option<OssBackend>,
}

impl DualBackend {
    pub fn new(local: LocalFs, oss: Option<OssBackend>) -> Self {
        Self { local, oss }
    }

    /// 直传文件以 `users/` 开头(见 direct.rs 的 object_key 生成)
    fn is_oss(path: &str) -> bool {
        path.starts_with("users/")
    }
}

impl StorageBackend for DualBackend {
    async fn put(&self, path: &str, data: &[u8]) -> Result<(), std::io::Error> {
        match (Self::is_oss(path), &self.oss) {
            (true, Some(oss)) => oss.put(path, data).await,
            (true, None) => Ok(()), // 无 OSS 后端且无直传文件,不应命中
            (false, _) => self.local.put(path, data).await,
        }
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        match (Self::is_oss(path), &self.oss) {
            (true, Some(oss)) => oss.get(path).await,
            (true, None) => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "无 OSS 后端",
            )),
            (false, _) => self.local.get(path).await,
        }
    }

    async fn delete(&self, path: &str) -> Result<(), std::io::Error> {
        match (Self::is_oss(path), &self.oss) {
            (true, Some(oss)) => oss.delete(path).await,
            (true, None) => Ok(()),
            (false, _) => self.local.delete(path).await,
        }
    }

    async fn exists(&self, path: &str) -> Result<bool, std::io::Error> {
        match (Self::is_oss(path), &self.oss) {
            (true, Some(oss)) => oss.exists(path).await,
            (true, None) => Ok(false),
            (false, _) => self.local.exists(path).await,
        }
    }

    fn url_for(&self, path: &str) -> String {
        match (Self::is_oss(path), &self.oss) {
            (true, Some(oss)) => oss.url_for(path),
            (true, None) => self.local.url_for(path), // 不应命中;回退本地前缀
            (false, _) => self.local.url_for(path),
        }
    }
}
