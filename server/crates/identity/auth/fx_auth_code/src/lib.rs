//! 提供验证码认证的通用生命周期、领域模型与扩展端口。
//!
//! 本包不负责具体数据库、发送供应商或宿主业务策略。

mod model;
mod ports;
mod service;

pub use model::{ActiveVerificationCode, NewVerificationCode, VerificationCodePolicy};
pub use ports::{VerificationCodeSender, VerificationCodeService, VerificationCodeStore};
pub use service::DefaultVerificationCodeService;
