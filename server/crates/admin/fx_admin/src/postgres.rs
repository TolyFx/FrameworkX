use async_trait::async_trait;
use fx_core::{AppError, AppResult};
use fx_user_core::AccountStatus;

use crate::{
    AdminLoginRecord, AdminUserAccount, AdminUserCredential, AdminUserDetail, AdminUserPage,
    AdminUserProfile, AdminUserQuery, AdminUserRepository, AdminUserSummary,
};

#[derive(Clone)]
pub struct PgAdminUserRepository {
    db: sqlx::PgPool,
}

impl PgAdminUserRepository {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AdminUserRepository for PgAdminUserRepository {
    async fn page_users(&self, query: &AdminUserQuery) -> AppResult<AdminUserPage> {
        let pattern = format!("%{}%", query.query);
        let (total,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM accounts a
             LEFT JOIN user_profiles p ON p.account_id = a.id
             WHERE $1 = '%%' OR COALESCE(p.nickname, '') ILIKE $1 OR CAST(a.id AS TEXT) ILIKE $1
                OR EXISTS (SELECT 1 FROM auth_credentials c WHERE c.account_id = a.id AND c.identifier ILIKE $1)",
        )
        .bind(&pattern)
        .fetch_one(&self.db)
        .await
        .map_err(db_error)?;
        let rows: Vec<(i64, String, Option<String>, Option<String>, String, String, i16, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            "SELECT a.id, COALESCE(p.nickname, '未命名用户'), p.avatar, p.signature,
                    COALESCE(c.identifier, ''), COALESCE(c.auth_type, ''), a.status, a.created_at
             FROM accounts a
             LEFT JOIN user_profiles p ON p.account_id = a.id
             LEFT JOIN LATERAL (
                SELECT identifier, auth_type FROM auth_credentials WHERE account_id = a.id ORDER BY created_at LIMIT 1
             ) c ON TRUE
             WHERE $1 = '%%' OR COALESCE(p.nickname, '') ILIKE $1 OR CAST(a.id AS TEXT) ILIKE $1
                OR EXISTS (SELECT 1 FROM auth_credentials ac WHERE ac.account_id = a.id AND ac.identifier ILIKE $1)
             ORDER BY a.created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(query.page_size)
        .bind((query.page - 1) * query.page_size)
        .fetch_all(&self.db)
        .await
        .map_err(db_error)?;
        Ok(AdminUserPage {
            items: rows
                .into_iter()
                .map(|row| AdminUserSummary {
                    id: row.0,
                    nickname: row.1,
                    avatar: row.2,
                    signature: row.3,
                    account: row.4,
                    auth_type: row.5,
                    status: status_from_db(row.6),
                    created_at: row.7,
                })
                .collect(),
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    async fn user_detail(&self, account_id: i64) -> AppResult<AdminUserDetail> {
        let account: (
            i64,
            i16,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        ) = sqlx::query_as("SELECT id, status, created_at, updated_at FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(&self.db)
            .await
            .map_err(db_error)?
            .ok_or_else(|| AppError::not_found("用户不存在"))?;
        let profile: Option<(Option<String>, Option<String>, Option<String>, Option<String>, Option<i16>, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
            "SELECT nickname, avatar, signature, bio, gender, updated_at FROM user_profiles WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_optional(&self.db)
        .await
        .map_err(db_error)?;
        let credentials: Vec<(String, String, bool, chrono::DateTime<chrono::Utc>)> =
            sqlx::query_as("SELECT auth_type, identifier, verified, created_at FROM auth_credentials WHERE account_id = $1 ORDER BY created_at")
                .bind(account_id)
                .fetch_all(&self.db)
                .await
                .map_err(db_error)?;
        let logins: Vec<(chrono::DateTime<chrono::Utc>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT login_at, ip, platform, device_name, device_id, app_version FROM login_logs WHERE account_id = $1 ORDER BY login_at DESC LIMIT 30")
                .bind(account_id)
                .fetch_all(&self.db)
                .await
                .map_err(db_error)?;
        Ok(AdminUserDetail {
            account: AdminUserAccount {
                id: account.0,
                status: status_from_db(account.1),
                created_at: account.2,
                updated_at: account.3,
            },
            profile: profile
                .map(|row| AdminUserProfile {
                    nickname: row.0,
                    avatar: row.1,
                    signature: row.2,
                    bio: row.3,
                    gender: row.4,
                    updated_at: row.5,
                })
                .unwrap_or_default(),
            credentials: credentials
                .into_iter()
                .map(|row| AdminUserCredential {
                    auth_type: row.0,
                    identifier: row.1,
                    verified: row.2,
                    created_at: row.3,
                })
                .collect(),
            recent_logins: logins
                .into_iter()
                .map(|row| AdminLoginRecord {
                    login_at: row.0,
                    ip: row.1,
                    platform: row.2,
                    device_name: row.3,
                    device_id: row.4,
                    app_version: row.5,
                })
                .collect(),
        })
    }

    async fn set_user_status(
        &self,
        account_id: i64,
        status: AccountStatus,
        actor: &str,
        reason: Option<&str>,
    ) -> AppResult<()> {
        let mut tx = self.db.begin().await.map_err(db_error)?;
        let result =
            sqlx::query("UPDATE accounts SET status = $2, updated_at = NOW() WHERE id = $1")
                .bind(account_id)
                .bind(status_to_db(status))
                .execute(&mut *tx)
                .await
                .map_err(db_error)?;
        if result.rows_affected() == 0 {
            return Err(AppError::not_found("用户不存在"));
        }
        sqlx::query(
            "INSERT INTO admin_audit_logs (actor, action, target_type, target_id, reason, result)
             VALUES ($1, $2, 'user', $3, $4, 'success')",
        )
        .bind(actor)
        .bind(format!("users.status.{}", status_name(status)))
        .bind(account_id.to_string())
        .bind(reason)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(())
    }
}

fn status_from_db(value: i16) -> AccountStatus {
    match value {
        1 => AccountStatus::Disabled,
        2 => AccountStatus::Deleted,
        _ => AccountStatus::Active,
    }
}

fn status_to_db(status: AccountStatus) -> i16 {
    match status {
        AccountStatus::Active => 0,
        AccountStatus::Disabled => 1,
        AccountStatus::Deleted => 2,
    }
}

fn status_name(status: AccountStatus) -> &'static str {
    match status {
        AccountStatus::Active => "active",
        AccountStatus::Disabled => "disabled",
        AccountStatus::Deleted => "deleted",
    }
}

fn db_error(error: sqlx::Error) -> AppError {
    AppError::internal(error, "fx_admin_postgres")
}
