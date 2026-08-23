//! 提供基于邮箱验证码的认证 Provider。
//!
//! 验证码生命周期由 `fx_auth_code` 管理，邮件发送实现由宿主注入。

use std::sync::Arc;

use async_trait::async_trait;
use fx_auth_code::VerificationCodeService;
use fx_auth_core::{AuthContext, AuthIdentity, AuthInput, AuthProvider, VerificationCodeProvider};
use fx_core::{AppError, AppResult};

pub struct EmailAuthProvider {
    codes: Arc<dyn VerificationCodeService>,
}

impl EmailAuthProvider {
    pub fn new(codes: Arc<dyn VerificationCodeService>) -> Self {
        Self { codes }
    }
}

#[async_trait]
impl VerificationCodeProvider for EmailAuthProvider {
    async fn request_code(
        &self,
        identifier: &str,
        scene: &str,
        ctx: AuthContext,
    ) -> AppResult<Option<String>> {
        let email = identifier.trim();
        if !email.contains('@') {
            return Err(AppError::bad_request_code(
                "AUTH_EMAIL_INVALID",
                "邮箱格式不正确",
            ));
        }
        self.codes.issue(self.kind(), email, scene, &ctx).await
    }

    async fn verify_code(
        &self,
        identifier: &str,
        code: &str,
        scene: &str,
        ctx: AuthContext,
    ) -> AppResult<()> {
        let email = identifier.trim();
        if !email.contains('@') {
            return Err(AppError::bad_request_code(
                "AUTH_EMAIL_INVALID",
                "邮箱格式不正确",
            ));
        }
        self.codes
            .verify(self.kind(), email, code, scene, &ctx)
            .await
    }
}

#[async_trait]
impl AuthProvider for EmailAuthProvider {
    fn kind(&self) -> &'static str {
        "email"
    }

    async fn authenticate(&self, input: AuthInput, ctx: AuthContext) -> AppResult<AuthIdentity> {
        let email = input.identifier.trim();
        if !email.contains('@') {
            return Err(AppError::bad_request_code(
                "AUTH_EMAIL_INVALID",
                "邮箱格式不正确",
            ));
        }
        self.codes
            .verify(self.kind(), email, &input.credential, "login", &ctx)
            .await?;
        Ok(AuthIdentity {
            auth_type: self.kind().into(),
            identifier: email.into(),
            display_name: Some(email.split('@').next().unwrap_or("用户").into()),
            avatar: None,
        })
    }
}
