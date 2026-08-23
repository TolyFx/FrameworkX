//! 定义验证码存储、发送与流程服务的宿主扩展端口。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use fx_auth_core::AuthContext;
use fx_core::AppResult;

use crate::{ActiveVerificationCode, NewVerificationCode};

#[async_trait]
pub trait VerificationCodeStore: Send + Sync {
    async fn latest_issued_at(
        &self,
        identifier: &str,
        request_ip: Option<&str>,
    ) -> AppResult<Option<DateTime<Utc>>>;

    async fn save(&self, code: NewVerificationCode) -> AppResult<()>;

    async fn find_active(
        &self,
        identifier: &str,
        channel: &str,
        scene: &str,
        now: DateTime<Utc>,
    ) -> AppResult<Option<ActiveVerificationCode>>;

    async fn mark_used(&self, id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait VerificationCodeSender: Send + Sync {
    fn name(&self, channel: &str) -> Option<String>;

    async fn send(
        &self,
        channel: &str,
        identifier: &str,
        code: &str,
        ctx: &AuthContext,
    ) -> AppResult<()>;
}

#[async_trait]
pub trait VerificationCodeService: Send + Sync {
    async fn issue(
        &self,
        channel: &str,
        identifier: &str,
        scene: &str,
        ctx: &AuthContext,
    ) -> AppResult<Option<String>>;

    async fn verify(
        &self,
        channel: &str,
        identifier: &str,
        code: &str,
        scene: &str,
        ctx: &AuthContext,
    ) -> AppResult<()>;
}
