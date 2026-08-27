use async_trait::async_trait;
use fx_auth_core::{AuthContext, AuthIdentity};
use fx_core::AppResult;

use crate::model::{Account, AuthCredential, ScanSession, UserProfile};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_account_by_id(&self, account_id: i64) -> AppResult<Option<Account>>;

    async fn find_credential(
        &self,
        auth_type: &str,
        identifier: &str,
    ) -> AppResult<Option<AuthCredential>>;

    async fn find_profile(&self, account_id: i64) -> AppResult<Option<UserProfile>>;

    async fn update_profile(
        &self,
        account_id: i64,
        nickname: Option<&str>,
        avatar: Option<&str>,
        signature: Option<&str>,
    ) -> AppResult<UserProfile>;

    async fn create_account_with_identity(&self, identity: &AuthIdentity) -> AppResult<Account>;

    async fn record_login(&self, account_id: i64, context: &AuthContext) -> AppResult<()>;

    async fn password_hash(&self, account_id: i64) -> AppResult<Option<String>>;

    async fn save_password_hash(&self, account_id: i64, hash: &str) -> AppResult<()>;

    async fn bind_credential(
        &self,
        account_id: i64,
        auth_type: &str,
        identifier: &str,
    ) -> AppResult<()>;

    async fn delete_account(&self, account_id: i64) -> AppResult<()>;

    async fn create_scan_session(
        &self,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()>;

    async fn find_scan_session(&self, token: &str) -> AppResult<Option<ScanSession>>;

    async fn mark_scan_scanned(&self, token: &str, account_id: i64) -> AppResult<bool>;

    async fn mark_scan_confirmed(&self, token: &str, account_id: i64) -> AppResult<bool>;

    async fn cancel_scan(&self, token: &str, account_id: i64) -> AppResult<bool>;
}
