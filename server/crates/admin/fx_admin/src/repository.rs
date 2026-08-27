use async_trait::async_trait;
use fx_core::AppResult;
use fx_user_core::AccountStatus;

use crate::{AdminUserDetail, AdminUserPage, AdminUserQuery};

#[async_trait]
pub trait AdminUserRepository: Send + Sync {
    async fn page_users(&self, query: &AdminUserQuery) -> AppResult<AdminUserPage>;
    async fn user_detail(&self, account_id: i64) -> AppResult<AdminUserDetail>;
    async fn set_user_status(
        &self,
        account_id: i64,
        status: AccountStatus,
        actor: &str,
        reason: Option<&str>,
    ) -> AppResult<()>;
}
