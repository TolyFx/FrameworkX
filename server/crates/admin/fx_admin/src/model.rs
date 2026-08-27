use chrono::{DateTime, Utc};
use fx_user_core::AccountStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AdminUserQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    #[serde(default)]
    pub query: String,
}

impl AdminUserQuery {
    pub fn normalized(&self) -> Self {
        Self {
            page: self.page.max(1),
            page_size: self.page_size.clamp(1, 100),
            query: self.query.trim().to_owned(),
        }
    }
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    30
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminUserSummary {
    pub id: i64,
    pub nickname: String,
    pub avatar: Option<String>,
    pub signature: Option<String>,
    pub account: String,
    pub auth_type: String,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminUserPage {
    pub items: Vec<AdminUserSummary>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminUserDetail {
    pub account: AdminUserAccount,
    pub profile: AdminUserProfile,
    pub credentials: Vec<AdminUserCredential>,
    pub recent_logins: Vec<AdminLoginRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminUserAccount {
    pub id: i64,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AdminUserProfile {
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub signature: Option<String>,
    pub bio: Option<String>,
    pub gender: Option<i16>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminUserCredential {
    pub auth_type: String,
    pub identifier: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminLoginRecord {
    pub login_at: DateTime<Utc>,
    pub ip: Option<String>,
    pub platform: Option<String>,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
    pub app_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::AdminUserQuery;

    #[test]
    fn user_query_is_trimmed_and_bounded() {
        let query = AdminUserQuery {
            page: 0,
            page_size: 999,
            query: "  toly  ".into(),
        }
        .normalized();
        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 100);
        assert_eq!(query.query, "toly");
    }
}
