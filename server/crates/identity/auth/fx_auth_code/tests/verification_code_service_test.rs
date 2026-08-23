//! 验证默认验证码服务的生成、发送、冷却与一次性消费行为。

mod support;

use std::sync::Arc;

use chrono::Duration;
use fx_auth_code::{
    DefaultVerificationCodeService, VerificationCodePolicy, VerificationCodeService,
};
use fx_auth_core::AuthContext;

use support::{MemorySender, MemoryStore};

// 调试模式返回验证码，且同一验证码只能成功消费一次。
#[tokio::test]
async fn debug_mode_exposes_code_and_allows_single_consumption() {
    let store = Arc::new(MemoryStore::default());
    let sender = Arc::new(MemorySender::default());
    let service = DefaultVerificationCodeService::new(
        store.clone(),
        sender.clone(),
        VerificationCodePolicy {
            cooldown: Duration::zero(),
            expose_code: true,
            ..Default::default()
        },
    );
    let ctx = AuthContext::default();

    let code = service
        .issue("email", "user@example.com", "login", &ctx)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|char| char.is_ascii_digit()));
    assert_eq!(sender.sent_count(), 0);
    service
        .verify("email", "user@example.com", &code, "login", &ctx)
        .await
        .unwrap();
    assert!(
        service
            .verify("email", "user@example.com", &code, "login", &ctx)
            .await
            .is_err()
    );
}

// 正式模式通过发送器发送验证码，并在冷却期内拒绝重复请求。
#[tokio::test]
async fn production_mode_sends_code_and_rejects_during_cooldown() {
    let store = Arc::new(MemoryStore::default());
    let sender = Arc::new(MemorySender::default());
    let service = DefaultVerificationCodeService::new(
        store,
        sender.clone(),
        VerificationCodePolicy::default(),
    );
    let ctx = AuthContext::default();

    let result = service
        .issue("sms", "18700000000", "login", &ctx)
        .await
        .unwrap();

    assert!(result.is_none());
    assert_eq!(sender.sent_count(), 1);
    assert!(
        service
            .issue("sms", "18700000000", "login", &ctx)
            .await
            .is_err()
    );
    assert_eq!(sender.sent_count(), 1);
}

// 错误输入不能提前消费仍然有效的验证码。
#[tokio::test]
async fn invalid_code_does_not_consume_active_code() {
    let store = Arc::new(MemoryStore::default());
    let service = DefaultVerificationCodeService::new(
        store,
        Arc::new(MemorySender::default()),
        VerificationCodePolicy {
            cooldown: Duration::zero(),
            expose_code: true,
            ..Default::default()
        },
    );
    let ctx = AuthContext::default();
    let code = service
        .issue("email", "user@example.com", "login", &ctx)
        .await
        .unwrap()
        .unwrap();

    assert!(
        service
            .verify("email", "user@example.com", "wrong", "login", &ctx)
            .await
            .is_err()
    );
    service
        .verify("email", "user@example.com", &code, "login", &ctx)
        .await
        .unwrap();
}

// 同一邮箱的验证码不能跨业务场景使用。
#[tokio::test]
async fn code_cannot_be_consumed_by_another_scene() {
    let service = DefaultVerificationCodeService::new(
        Arc::new(MemoryStore::default()),
        Arc::new(MemorySender::default()),
        VerificationCodePolicy {
            cooldown: Duration::zero(),
            expose_code: true,
            ..Default::default()
        },
    );
    let ctx = AuthContext::default();
    let code = service
        .issue("email", "user@example.com", "bind_email", &ctx)
        .await
        .unwrap()
        .unwrap();

    assert!(
        service
            .verify("email", "user@example.com", &code, "reset_password", &ctx,)
            .await
            .is_err()
    );
    service
        .verify("email", "user@example.com", &code, "bind_email", &ctx)
        .await
        .unwrap();
}
