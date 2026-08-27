//! 提供微信小程序登录凭证校验并输出统一外部身份。
//!
//! 本包只负责 `js_code -> openid`，不创建账号、不保存 session_key，也不依赖宿主框架。

mod model;

use std::time::Duration;

use async_trait::async_trait;
use fx_auth_core::{AuthContext, AuthIdentity, AuthInput, AuthProvider};
use fx_core::{AppError, AppResult};
use reqwest::Client;

use model::WeChatSessionResponse;

const WECHAT_CODE_SESSION_URL: &str = "https://api.weixin.qq.com/sns/jscode2session";

pub struct WeChatAuthProvider {
    client: Client,
    app_id: String,
    app_secret: String,
}

impl WeChatAuthProvider {
    pub fn new(app_id: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self::with_client(
            app_id,
            app_secret,
            Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to create wechat auth client"),
        )
    }

    pub fn with_client(
        app_id: impl Into<String>,
        app_secret: impl Into<String>,
        client: Client,
    ) -> Self {
        Self {
            client,
            app_id: app_id.into(),
            app_secret: app_secret.into(),
        }
    }
}

#[async_trait]
impl AuthProvider for WeChatAuthProvider {
    fn kind(&self) -> &'static str {
        "wechat"
    }

    async fn authenticate(&self, input: AuthInput, _ctx: AuthContext) -> AppResult<AuthIdentity> {
        let code = input.credential.trim();
        if code.is_empty() {
            return Err(AppError::bad_request_code(
                "AUTH_WECHAT_CODE_REQUIRED",
                "微信登录凭证不能为空",
            ));
        }
        if self.app_id.trim().is_empty() || self.app_secret.trim().is_empty() {
            return Err(AppError::internal(
                "wechat app id or secret is empty",
                "wechat auth config",
            ));
        }
        let response = self
            .client
            .get(WECHAT_CODE_SESSION_URL)
            .query(&[
                ("appid", self.app_id.as_str()),
                ("secret", self.app_secret.as_str()),
                ("js_code", code),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|error| AppError::internal(error, "wechat code session"))?
            .error_for_status()
            .map_err(|error| AppError::internal(error, "wechat code session"))?
            .json::<WeChatSessionResponse>()
            .await
            .map_err(|error| AppError::internal(error, "wechat code session"))?;
        let openid = response.into_openid().map_err(|_| {
            AppError::unauthorized_code("AUTH_WECHAT_CODE_INVALID", "微信登录凭证无效")
        })?;
        let display_name = input
            .identifier
            .trim()
            .is_empty()
            .then_some("微信用户".to_owned())
            .unwrap_or_else(|| input.identifier.trim().to_owned());
        Ok(AuthIdentity {
            auth_type: self.kind().to_owned(),
            identifier: openid,
            display_name: Some(display_name),
            avatar: None,
        })
    }
}
