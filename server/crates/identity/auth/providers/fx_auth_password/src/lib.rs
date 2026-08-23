//! 提供基于 Argon2 密码摘要的认证 Provider。
//!
//! 密码记录的查询由宿主存储实现负责。

use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use fx_auth_core::{AuthContext, AuthIdentity, AuthInput, AuthProvider};
use fx_core::{AppError, AppResult};

pub struct PasswordRecord {
    pub auth_type: String,
    pub identifier: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
}

#[async_trait]
pub trait PasswordRecordStore: Send + Sync {
    async fn find(&self, identifier: &str) -> AppResult<Option<PasswordRecord>>;
}

pub struct PasswordAuthProvider {
    store: Arc<dyn PasswordRecordStore>,
}

impl PasswordAuthProvider {
    pub fn new(store: Arc<dyn PasswordRecordStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl AuthProvider for PasswordAuthProvider {
    fn kind(&self) -> &'static str {
        "password"
    }

    async fn authenticate(&self, input: AuthInput, _ctx: AuthContext) -> AppResult<AuthIdentity> {
        let record = self
            .store
            .find(input.identifier.trim())
            .await?
            .ok_or_else(|| {
                AppError::unauthorized_code("AUTH_CREDENTIAL_INVALID", "账号或密码错误")
            })?;
        let hash = PasswordHash::new(&record.password_hash)
            .map_err(|error| AppError::internal(error, "password hash"))?;
        Argon2::default()
            .verify_password(input.credential.as_bytes(), &hash)
            .map_err(|_| {
                AppError::unauthorized_code("AUTH_CREDENTIAL_INVALID", "账号或密码错误")
            })?;
        Ok(AuthIdentity {
            auth_type: record.auth_type,
            identifier: record.identifier.clone(),
            display_name: record.display_name.or(Some(record.identifier)),
            avatar: record.avatar,
        })
    }
}
