//! 实现默认验证码生成、冷却、发送、校验与消费流程。

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use fx_auth_core::AuthContext;
use fx_core::{AppError, AppResult};
use rand::Rng;

use crate::{
    NewVerificationCode, VerificationCodePolicy, VerificationCodeSender, VerificationCodeService,
    VerificationCodeStore,
};

pub struct DefaultVerificationCodeService {
    store: Arc<dyn VerificationCodeStore>,
    sender: Arc<dyn VerificationCodeSender>,
    policy: VerificationCodePolicy,
}

impl DefaultVerificationCodeService {
    pub fn new(
        store: Arc<dyn VerificationCodeStore>,
        sender: Arc<dyn VerificationCodeSender>,
        policy: VerificationCodePolicy,
    ) -> Self {
        Self {
            store,
            sender,
            policy,
        }
    }

    fn generate_code(&self) -> String {
        let length = self.policy.length.clamp(1, 9);
        let upper = 10_u32.pow(length);
        format!(
            "{:0width$}",
            rand::rng().random_range(0..upper),
            width = length as usize
        )
    }
}

#[async_trait]
impl VerificationCodeService for DefaultVerificationCodeService {
    async fn issue(
        &self,
        channel: &str,
        identifier: &str,
        scene: &str,
        ctx: &AuthContext,
    ) -> AppResult<Option<String>> {
        let now = Utc::now();
        let latest = self
            .store
            .latest_issued_at(identifier, ctx.ip.as_deref())
            .await?;
        if let Some(created_at) = latest
            && now - created_at < self.policy.cooldown
        {
            return Err(AppError::too_many_requests(
                "AUTH_CODE_RATE_LIMITED",
                "验证码发送过于频繁，请稍后再试",
            ));
        }

        let code = self.generate_code();
        self.store
            .save(NewVerificationCode {
                identifier: identifier.into(),
                channel: channel.into(),
                scene: scene.into(),
                code: code.clone(),
                expires_at: now + self.policy.ttl,
                request_ip: ctx.ip.clone(),
                sender: self.sender.name(channel),
            })
            .await?;

        if self.policy.expose_code {
            return Ok(Some(code));
        }

        self.sender.send(channel, identifier, &code, ctx).await?;
        Ok(None)
    }

    async fn verify(
        &self,
        channel: &str,
        identifier: &str,
        code: &str,
        scene: &str,
        _ctx: &AuthContext,
    ) -> AppResult<()> {
        let active = self
            .store
            .find_active(identifier, channel, scene, Utc::now())
            .await?
            .ok_or_else(|| {
                AppError::bad_request_code("AUTH_CODE_EXPIRED", "验证码不存在或已失效")
            })?;
        if active.code != code {
            return Err(AppError::unauthorized_code(
                "AUTH_CODE_INVALID",
                "验证码错误",
            ));
        }
        self.store.mark_used(active.id).await
    }
}
