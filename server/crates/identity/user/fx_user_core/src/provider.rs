use async_trait::async_trait;
use fx_core::AppResult;

use crate::model::UserContext;

pub trait TokenProvider: Send + Sync {
    fn issue(&self, user: &UserContext) -> AppResult<String>;
    fn resolve(&self, token: &str) -> AppResult<UserContext>;
    fn revoke(&self, token: &str) -> AppResult<()>;
}

#[async_trait]
pub trait UserHooks: Send + Sync {
    async fn on_user_created(&self, _user_id: i64) -> AppResult<()> {
        Ok(())
    }

    async fn on_user_logged_in(&self, _user_id: i64) -> AppResult<()> {
        Ok(())
    }

    async fn on_credential_bound(&self, _user_id: i64, _kind: &str) -> AppResult<()> {
        Ok(())
    }

    async fn on_user_deleted(&self, _user_id: i64) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct EmptyUserHooks;

#[async_trait]
impl UserHooks for EmptyUserHooks {}
