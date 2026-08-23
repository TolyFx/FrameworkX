//! 阿里云 STS `AssumeRole` 临时凭证签发 + 直传签发端口实现。
//!
//! 职责:`StsConfig` 用主账号 AK/SK 调 STS `AssumeRole` 签发带 Policy 的临时凭证;
//!      `AliyunStsIssuer` 实现 `fx_storage::DirectUploadIssuer`,把 object resources 包装成
//!      阿里云 `acs:oss` 资源串并签发,供 `DirectUploadService` 编排直传。
//! 边界:阿里云特有(STS 协议、资源 ARN)封装于此;依赖 `fx_storage`(端口),不依赖宿主。
//! 约束:`duration_secs` 区间 [900, 3600];签名采用阿里云 SignatureV1(HMAC-SHA1)。

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::Utc;
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::Deserialize;
use sha1::Sha1;

/// STS 配置(主账号 AK/SK + 目标角色 ARN)
#[derive(Debug, Clone)]
pub struct StsConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub role_arn: String,
}

/// STS 临时凭证(直传时客户端作为临时身份)
#[derive(Debug, Clone, serde::Serialize)]
pub struct StsToken {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub security_token: String,
    pub expiration: String,
}

impl StsConfig {
    /// 从环境变量读取
    pub fn from_env() -> Option<Self> {
        let access_key_id = std::env::var("OSS_ACCESS_KEY_ID").ok()?;
        let access_key_secret = std::env::var("OSS_ACCESS_KEY_SECRET").ok()?;
        let role_arn = std::env::var("OSS_STS_ROLE_ARN").ok()?;
        Some(Self {
            access_key_id,
            access_key_secret,
            role_arn,
        })
    }

    /// 调用 `AssumeRole` 获取临时凭证。
    ///
    /// - `session_name`: 会话标识,如 `"upload-user-1"`
    /// - `policy_json`: 可选内联策略 JSON,进一步把权限收敛到具体资源
    /// - `duration_secs`: Token 有效期(秒),最小 900,最大 3600
    pub async fn assume_role(
        &self,
        session_name: &str,
        policy_json: Option<&str>,
        duration_secs: u32,
    ) -> Result<StsToken, String> {
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce: u64 = rand::rng().random();
        let duration_str = duration_secs.to_string();
        let nonce_str = nonce.to_string();

        let mut params: Vec<(&str, &str)> = vec![
            ("Action", "AssumeRole"),
            ("Format", "JSON"),
            ("Version", "2015-04-01"),
            ("AccessKeyId", &self.access_key_id),
            ("SignatureMethod", "HMAC-SHA1"),
            ("SignatureVersion", "1.0"),
            ("SignatureNonce", &nonce_str),
            ("Timestamp", &timestamp),
            ("RoleArn", &self.role_arn),
            ("RoleSessionName", session_name),
            ("DurationSeconds", &duration_str),
        ];

        if let Some(policy) = policy_json {
            params.push(("Policy", policy));
        }

        params.sort_by_key(|&(k, _)| k);

        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let string_to_sign = format!("POST&{}&{}", encode("/"), encode(&query_string));

        let signing_key = format!("{}&", self.access_key_secret);
        let mut mac = Hmac::<Sha1>::new_from_slice(signing_key.as_bytes())
            .map_err(|e| format!("HMAC error: {}", e))?;
        mac.update(string_to_sign.as_bytes());
        let signature = BASE64.encode(mac.finalize().into_bytes());

        let mut body_params = params
            .iter()
            .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
            .collect::<Vec<_>>();
        body_params.push(format!("Signature={}", encode(&signature)));
        let body = body_params.join("&");

        let client = reqwest::Client::new();
        let resp = client
            .post("https://sts.aliyuncs.com/")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("STS request failed: {}", e))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Read body failed: {}", e))?;

        if !status.is_success() {
            return Err(format!("STS returned {}: {}", status, text));
        }

        let response: AssumeRoleResponse = serde_json::from_str(&text)
            .map_err(|e| format!("Parse STS response failed: {} body: {}", e, text))?;

        Ok(StsToken {
            access_key_id: response.credentials.access_key_id,
            access_key_secret: response.credentials.access_key_secret,
            security_token: response.credentials.security_token,
            expiration: response.credentials.expiration,
        })
    }
}

/// URL 编码(阿里云要求的严格编码:RFC 3986 unreserved 字符不编码,其余全编码)
fn encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AssumeRoleResponse {
    credentials: AssumeRoleCredentials,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AssumeRoleCredentials {
    access_key_id: String,
    access_key_secret: String,
    security_token: String,
    expiration: String,
}

// ─── 直传签发端口实现 ───

use async_trait::async_trait;
use fx_storage_service::{DirectUploadIssuer, StorageError, StsCredentials};

/// 阿里云直传签发器:实现 `fx_storage::DirectUploadIssuer`,
/// 把 object resources 包装成 `acs:oss:*:*:<bucket>/<key>` 资源串并签发。
pub struct AliyunStsIssuer {
    pub sts: StsConfig,
    pub bucket: String,
    pub endpoint: String,
}

impl AliyunStsIssuer {
    pub fn new(sts: StsConfig, bucket: String, endpoint: String) -> Self {
        Self {
            sts,
            bucket,
            endpoint,
        }
    }
}

#[async_trait]
impl DirectUploadIssuer for AliyunStsIssuer {
    async fn issue_grant(
        &self,
        session: &str,
        resources: &[String],
        duration_secs: u32,
    ) -> Result<StsCredentials, StorageError> {
        let resources: Vec<String> = resources
            .iter()
            .map(|r| format!("acs:oss:*:*:{}/{}", self.bucket, r))
            .collect();
        let policy = serde_json::json!({
            "Version": "1",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["oss:PutObject"],
                "Resource": resources
            }]
        });
        let token = self
            .sts
            .assume_role(session, Some(&policy.to_string()), duration_secs)
            .await
            .map_err(|e| StorageError::Backend(format!("STS 签发失败: {e}")))?;
        Ok(StsCredentials {
            access_key_id: token.access_key_id,
            access_key_secret: token.access_key_secret,
            security_token: token.security_token,
            expiration: token.expiration,
            bucket: self.bucket.clone(),
            endpoint: self.endpoint.clone(),
        })
    }
}
