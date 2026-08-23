//! 本地文件系统存储后端。
//!
//! 职责:实现 `StorageBackend`,基于 `tokio::fs` 读写 `base_path` 下相对路径。
//! 边界:只依赖 `fx_storage_core`;无 OSS/网络依赖。
//! 约束:`put` 自动 `create_dir_all`;`url_for` 返回 `{url_prefix}/{path}`,
//!      由宿主用 `ServeDir` 在 `url_prefix` 处静态托管。

use std::path::PathBuf;

use fx_storage_core::StorageBackend;
use tokio::fs;

#[derive(Clone)]
pub struct LocalFs {
    base_path: PathBuf,
    url_prefix: String,
}

impl LocalFs {
    pub fn new(base_path: PathBuf, url_prefix: String) -> Self {
        Self {
            base_path,
            url_prefix,
        }
    }
}

impl StorageBackend for LocalFs {
    async fn put(&self, path: &str, data: &[u8]) -> Result<(), std::io::Error> {
        let full = self.base_path.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(full, data).await
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        fs::read(self.base_path.join(path)).await
    }

    async fn delete(&self, path: &str) -> Result<(), std::io::Error> {
        fs::remove_file(self.base_path.join(path)).await
    }

    async fn exists(&self, path: &str) -> Result<bool, std::io::Error> {
        Ok(self.base_path.join(path).exists())
    }

    fn url_for(&self, path: &str) -> String {
        format!("{}/{}", self.url_prefix, path)
    }
}
