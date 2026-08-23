//! 定义认证 Provider 协议以及验证码渠道注册与分发机制。

use std::sync::Arc;

use async_trait::async_trait;
use fx_core::{AppError, AppResult};

use crate::{AuthContext, AuthIdentity, AuthInput};

#[async_trait]
pub trait AuthProvider: Send + Sync {
    fn kind(&self) -> &'static str;

    async fn authenticate(&self, input: AuthInput, ctx: AuthContext) -> AppResult<AuthIdentity>;
}

#[async_trait]
pub trait VerificationCodeProvider: AuthProvider {
    async fn request_code(
        &self,
        identifier: &str,
        scene: &str,
        ctx: AuthContext,
    ) -> AppResult<Option<String>>;

    async fn verify_code(
        &self,
        identifier: &str,
        code: &str,
        scene: &str,
        ctx: AuthContext,
    ) -> AppResult<()>;
}

#[derive(Default)]
pub struct VerificationCodeProviders {
    providers: Vec<Arc<dyn VerificationCodeProvider>>,
}

impl VerificationCodeProviders {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_provider(mut self, provider: Arc<dyn VerificationCodeProvider>) -> Self {
        self.providers.push(provider);
        self
    }

    pub async fn request_code(
        &self,
        channel: &str,
        identifier: &str,
        scene: &str,
        ctx: AuthContext,
    ) -> AppResult<Option<String>> {
        let provider = self
            .providers
            .iter()
            .find(|provider| provider.kind() == channel)
            .ok_or_else(|| {
                AppError::bad_request_code("AUTH_CHANNEL_UNSUPPORTED", "不支持的验证码渠道")
            })?;
        provider.request_code(identifier, scene, ctx).await
    }

    pub async fn verify_code(
        &self,
        channel: &str,
        identifier: &str,
        code: &str,
        scene: &str,
        ctx: AuthContext,
    ) -> AppResult<()> {
        let provider = self
            .providers
            .iter()
            .find(|provider| provider.kind() == channel)
            .ok_or_else(|| {
                AppError::bad_request_code("AUTH_CHANNEL_UNSUPPORTED", "不支持的验证码渠道")
            })?;
        provider.verify_code(identifier, code, scene, ctx).await
    }
}
