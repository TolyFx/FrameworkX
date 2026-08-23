//! 提供 Apple identity token 验签并输出统一外部身份。

mod model;

use std::time::Duration;

use async_trait::async_trait;
use fx_auth_core::{AuthContext, AuthIdentity, AuthInput, AuthProvider};
use fx_core::{AppError, AppResult};
use jsonwebtoken::{DecodingKey, Validation, decode};
use reqwest::Client;

use model::{AppleClaims, AppleJwks};

pub struct AppleAuthProvider {
    client: Client,
    audience: String,
}

impl AppleAuthProvider {
    pub fn new(audience: impl Into<String>) -> Self {
        Self::with_client(
            audience,
            Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to create apple auth client"),
        )
    }

    pub fn with_client(audience: impl Into<String>, client: Client) -> Self {
        Self {
            client,
            audience: audience.into(),
        }
    }
}

#[async_trait]
impl AuthProvider for AppleAuthProvider {
    fn kind(&self) -> &'static str {
        "apple"
    }

    async fn authenticate(&self, input: AuthInput, _ctx: AuthContext) -> AppResult<AuthIdentity> {
        let header = jsonwebtoken::decode_header(&input.credential)
            .map_err(|_| AppError::unauthorized("Apple 授权信息无效"))?;
        let kid = header
            .kid
            .ok_or_else(|| AppError::unauthorized("Apple 授权信息无效"))?;
        let keys = self
            .client
            .get("https://appleid.apple.com/auth/keys")
            .send()
            .await
            .map_err(|error| AppError::internal(error, "apple keys"))?
            .json::<AppleJwks>()
            .await
            .map_err(|error| AppError::internal(error, "apple keys"))?;
        let key = keys
            .keys
            .iter()
            .find(|key| key.kid == kid)
            .ok_or_else(|| AppError::unauthorized("Apple 公钥不存在"))?;
        let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
            .map_err(|error| AppError::internal(error, "apple key"))?;
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_issuer(&["https://appleid.apple.com"]);
        validation.set_audience(&[&self.audience]);
        let claims = decode::<AppleClaims>(&input.credential, &decoding_key, &validation)
            .map_err(|_| AppError::unauthorized("Apple 授权验证失败"))?
            .claims;
        let display_name = claims
            .email
            .as_deref()
            .and_then(|email| email.split('@').next())
            .unwrap_or("Apple 用户")
            .to_string();
        Ok(AuthIdentity {
            auth_type: self.kind().into(),
            identifier: claims.sub,
            display_name: Some(display_name),
            avatar: None,
        })
    }
}
