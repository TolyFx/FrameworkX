//! 定义 Apple 公钥与 identity token 的内部响应模型。

use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct AppleJwks {
    pub(super) keys: Vec<AppleJwk>,
}

#[derive(Deserialize)]
pub(super) struct AppleJwk {
    pub(super) kid: String,
    pub(super) n: String,
    pub(super) e: String,
}

#[derive(Clone, Deserialize)]
pub(super) struct AppleClaims {
    pub(super) sub: String,
    pub(super) email: Option<String>,
}
