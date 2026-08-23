//! 定义 GitHub OAuth 与用户接口的内部响应模型。

use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct TokenResponse {
    pub(super) access_token: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct GitHubUser {
    pub(super) id: i64,
    pub(super) login: String,
    pub(super) name: Option<String>,
    pub(super) avatar_url: Option<String>,
}
