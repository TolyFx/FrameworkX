//! 提供 GitHub OAuth 授权码认证并输出统一外部身份。

mod model;

use std::time::Duration;

use async_trait::async_trait;
use fx_auth_core::{AuthContext, AuthIdentity, AuthInput, AuthProvider};
use fx_core::{AppError, AppResult};
use reqwest::Client;

use model::{GitHubUser, TokenResponse};

pub struct GitHubAuthProvider {
    client: Client,
    client_id: String,
    client_secret: String,
}

impl GitHubAuthProvider {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self::with_client(
            client_id,
            client_secret,
            Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to create github oauth client"),
        )
    }

    pub fn with_client(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        client: Client,
    ) -> Self {
        Self {
            client,
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        }
    }
}

#[async_trait]
impl AuthProvider for GitHubAuthProvider {
    fn kind(&self) -> &'static str {
        "github"
    }

    async fn authenticate(&self, input: AuthInput, _ctx: AuthContext) -> AppResult<AuthIdentity> {
        let token = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "client_id": self.client_id,
                "client_secret": self.client_secret,
                "code": input.credential,
            }))
            .send()
            .await
            .map_err(|error| AppError::internal(error, "github oauth"))?
            .json::<TokenResponse>()
            .await
            .map_err(|error| AppError::internal(error, "github token"))?
            .access_token
            .ok_or_else(|| AppError::unauthorized("GitHub 授权失败"))?;
        let response = self
            .client
            .get("https://api.github.com/user")
            .bearer_auth(token)
            .header("User-Agent", "fx-auth-github")
            .send()
            .await
            .map_err(|error| AppError::internal(error, "github user"))?;
        if !response.status().is_success() {
            return Err(AppError::unauthorized("GitHub 授权信息无效"));
        }
        let user = response
            .json::<GitHubUser>()
            .await
            .map_err(|error| AppError::internal(error, "github user"))?;
        Ok(AuthIdentity {
            auth_type: self.kind().into(),
            identifier: user.id.to_string(),
            display_name: Some(user.name.unwrap_or(user.login)),
            avatar: user.avatar_url,
        })
    }
}
