//! 阿里云 OSS 存储后端(S3 兼容协议)。
//!
//! 职责:实现 `StorageBackend`,基于 `aws-sdk-s3` 进行 put/get/delete/exists;
//!      额外暴露 `client()`/`bucket()` 供 `fx_storage_sts` 直传与 `head_object` 校验使用。
//! 边界:只依赖 `fx_storage_core` 与 AWS SDK;不依赖宿主业务。
//! 约束:公网访问 URL 由 `bucket.endpoint` 拼接,`url_for` 直接返回此前缀 + path。

use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use fx_storage_core::StorageBackend;

#[derive(Clone)]
pub struct OssBackend {
    client: Client,
    bucket: String,
    /// 公网访问 URL 前缀,如 `https://flash-im-storage.oss-cn-beijing.aliyuncs.com`
    pub url_prefix: String,
    /// OSS endpoint,供直传客户端签名(如 `https://oss-cn-beijing.aliyuncs.com`)
    pub endpoint: String,
}

pub struct OssConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub region: String,
}

impl OssConfig {
    /// 从环境变量读取 OSS 配置,返回 `None` 表示未配置(宿主据此决定是否启用直传)
    pub fn from_env() -> Option<Self> {
        let endpoint = std::env::var("OSS_ENDPOINT").ok()?;
        let bucket = std::env::var("OSS_BUCKET").ok()?;
        let access_key_id = std::env::var("OSS_ACCESS_KEY_ID").ok()?;
        let access_key_secret = std::env::var("OSS_ACCESS_KEY_SECRET").ok()?;
        let region = std::env::var("OSS_REGION").unwrap_or_else(|_| "cn-beijing".to_string());
        Some(Self {
            endpoint,
            bucket,
            access_key_id,
            access_key_secret,
            region,
        })
    }
}

impl OssBackend {
    pub fn new(config: OssConfig) -> Self {
        let credentials = Credentials::new(
            &config.access_key_id,
            &config.access_key_secret,
            None,
            None,
            "fx-storage-oss",
        );

        let url_prefix = format!(
            "https://{}.{}",
            config.bucket,
            config.endpoint.trim_start_matches("https://")
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .region(Region::new(config.region))
            .endpoint_url(&config.endpoint)
            .credentials_provider(credentials)
            .force_path_style(false)
            .behavior_version_latest()
            .build();

        Self {
            client: Client::from_conf(s3_config),
            bucket: config.bucket,
            endpoint: config.endpoint,
            url_prefix,
        }
    }

    /// 供 `fx_storage_sts` 直传与 `head_object` 等高级操作使用
    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

impl StorageBackend for OssBackend {
    async fn put(&self, path: &str, data: &[u8]) -> Result<(), std::io::Error> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(path)
            .body(ByteStream::from(data.to_vec()))
            .send()
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let bytes = resp
            .body
            .collect()
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(bytes.to_vec())
    }

    async fn delete(&self, path: &str) -> Result<(), std::io::Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool, std::io::Error> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let service_err = e.into_service_error();
                if service_err.is_not_found() {
                    Ok(false)
                } else {
                    Err(std::io::Error::other(service_err.to_string()))
                }
            }
        }
    }

    fn url_for(&self, path: &str) -> String {
        format!("{}/{}", self.url_prefix, path)
    }
}
