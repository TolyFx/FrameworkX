//! 提供基于手机短信验证码的认证 Provider。
//!
//! 验证码生命周期由 `fx_auth_code` 管理，短信发送实现由宿主注入。

use std::sync::Arc;

use async_trait::async_trait;
use fx_auth_code::VerificationCodeService;
use fx_auth_core::{AuthContext, AuthIdentity, AuthInput, AuthProvider, VerificationCodeProvider};
use fx_core::{AppError, AppResult};

pub struct SmsAuthProvider {
    codes: Arc<dyn VerificationCodeService>,
}

impl SmsAuthProvider {
    pub fn new(codes: Arc<dyn VerificationCodeService>) -> Self {
        Self { codes }
    }
}

#[async_trait]
impl VerificationCodeProvider for SmsAuthProvider {
    async fn request_code(
        &self,
        identifier: &str,
        scene: &str,
        ctx: AuthContext,
    ) -> AppResult<Option<String>> {
        let phone = identifier.trim();
        if phone.is_empty() {
            return Err(AppError::bad_request("手机号不能为空"));
        }
        self.codes.issue(self.kind(), phone, scene, &ctx).await
    }

    async fn verify_code(
        &self,
        identifier: &str,
        code: &str,
        scene: &str,
        ctx: AuthContext,
    ) -> AppResult<()> {
        let phone = identifier.trim();
        if phone.is_empty() {
            return Err(AppError::bad_request("手机号不能为空"));
        }
        self.codes
            .verify(self.kind(), phone, code, scene, &ctx)
            .await
    }
}

#[async_trait]
impl AuthProvider for SmsAuthProvider {
    fn kind(&self) -> &'static str {
        "sms"
    }

    async fn authenticate(&self, input: AuthInput, ctx: AuthContext) -> AppResult<AuthIdentity> {
        let phone = input.identifier.trim();
        if phone.is_empty() {
            return Err(AppError::bad_request("手机号不能为空"));
        }
        self.codes
            .verify(self.kind(), phone, &input.credential, "login", &ctx)
            .await?;
        Ok(AuthIdentity {
            auth_type: self.kind().into(),
            identifier: phone.into(),
            display_name: Some(format!(
                "用户{}",
                phone.chars().rev().take(4).collect::<String>()
            )),
            avatar: None,
        })
    }
}
