use std::sync::Arc;

use fx_core::AppResult;
use fx_user_core::AccountStatus;

use crate::{AdminUserDetail, AdminUserPage, AdminUserQuery, AdminUserRepository};

#[derive(Clone)]
pub struct AdminUserService {
    repository: Arc<dyn AdminUserRepository>,
}

impl AdminUserService {
    pub fn new(repository: Arc<dyn AdminUserRepository>) -> Self {
        Self { repository }
    }

    pub async fn page_users(&self, query: &AdminUserQuery) -> AppResult<AdminUserPage> {
        self.repository.page_users(&query.normalized()).await
    }

    pub async fn user_detail(&self, account_id: i64) -> AppResult<AdminUserDetail> {
        self.repository.user_detail(account_id).await
    }

    pub async fn set_user_status(
        &self,
        account_id: i64,
        status: AccountStatus,
        actor: &str,
        reason: Option<&str>,
    ) -> AppResult<()> {
        self.repository
            .set_user_status(account_id, status, actor, reason)
            .await
    }
}
