use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Active,
    Disabled,
    Deleted,
}

impl AccountStatus {
    pub fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub status: AccountStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub account_id: i64,
    pub nickname: String,
    pub avatar: Option<String>,
    pub signature: Option<String>,
    pub bio: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub identifier: Option<String>,
    pub auth_type: Option<String>,
    pub has_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredential {
    pub account_id: i64,
    pub auth_type: String,
    pub identifier: String,
    pub verified: bool,
}

/// 当前用户视角下的账号标识检查结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountIdentifierCheck {
    pub exists: bool,
    pub owned_by_current_account: bool,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub account_id: i64,
    pub status: AccountStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: i64,
    pub has_password: bool,
    pub user: UserResponse,
}

/// 面向客户端的用户资料响应。
///
/// 该结构同时用于登录响应中的 `user` 字段和 `/user/profile`，避免两处
/// 用户资料字段逐渐分叉。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub user_id: i64,
    pub nickname: String,
    pub avatar: Option<String>,
    pub profile: serde_json::Value,
}

impl From<UserProfile> for UserResponse {
    fn from(profile: UserProfile) -> Self {
        Self {
            user_id: profile.account_id,
            nickname: profile.nickname,
            avatar: profile.avatar,
            profile: serde_json::json!({
                "signature": profile.signature,
                "bio": profile.bio,
                "email": profile.email,
                "phone": profile.phone,
                "identifier": profile.identifier,
                "auth_type": profile.auth_type,
                "has_password": profile.has_password,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanSession {
    pub status: i16,
    pub user_id: Option<i64>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
