//! 定义验证码生命周期使用的策略与数据模型。

use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub struct VerificationCodePolicy {
    pub scene: String,
    pub length: u32,
    pub ttl: Duration,
    pub cooldown: Duration,
    pub expose_code: bool,
}

impl Default for VerificationCodePolicy {
    fn default() -> Self {
        Self {
            scene: "login".into(),
            length: 6,
            ttl: Duration::minutes(5),
            cooldown: Duration::seconds(60),
            expose_code: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewVerificationCode {
    pub identifier: String,
    pub channel: String,
    pub scene: String,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub request_ip: Option<String>,
    pub sender: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActiveVerificationCode {
    pub id: i64,
    pub code: String,
}
