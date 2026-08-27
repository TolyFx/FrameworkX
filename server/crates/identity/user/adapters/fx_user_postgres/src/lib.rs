use async_trait::async_trait;
use fx_auth_core::{AuthContext, AuthIdentity};
use fx_core::{AppError, AppResult};
use fx_user_core::{
    Account, AccountStatus, AuthCredential, ScanSession, UserProfile, UserRepository,
};

fn db_error(error: sqlx::Error) -> AppError {
    AppError::internal(error, "fx_user_postgres")
}

#[derive(Clone)]
pub struct PgUserRepository {
    db: sqlx::PgPool,
}

impl PgUserRepository {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_account_by_id(&self, account_id: i64) -> AppResult<Option<Account>> {
        let row: Option<(i64, i16)> =
            sqlx::query_as("SELECT id, status FROM accounts WHERE id = $1")
                .bind(account_id)
                .fetch_optional(&self.db)
                .await
                .map_err(db_error)?;

        Ok(row.map(|(id, status)| Account {
            id,
            status: account_status_from_db(status),
        }))
    }

    async fn find_credential(
        &self,
        auth_type: &str,
        identifier: &str,
    ) -> AppResult<Option<AuthCredential>> {
        let row: Option<(i64, String, String, bool)> = sqlx::query_as(
            "SELECT account_id, auth_type, identifier, verified \
             FROM auth_credentials \
             WHERE auth_type = $1 AND identifier = $2",
        )
        .bind(auth_type)
        .bind(identifier)
        .fetch_optional(&self.db)
        .await
        .map_err(db_error)?;

        Ok(row.map(
            |(account_id, auth_type, identifier, verified)| AuthCredential {
                account_id,
                auth_type,
                identifier,
                verified,
            },
        ))
    }

    async fn find_profile(&self, account_id: i64) -> AppResult<Option<UserProfile>> {
        let row: Option<(i64, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, bool)> =
            sqlx::query_as(
                "SELECT p.account_id, p.nickname, p.avatar, p.signature, p.bio, \
                 (SELECT identifier FROM auth_credentials WHERE account_id = p.account_id AND auth_type = 'email' LIMIT 1), \
                 (SELECT identifier FROM auth_credentials WHERE account_id = p.account_id AND auth_type = 'sms' LIMIT 1), \
                 c.identifier, c.auth_type, \
                 EXISTS(SELECT 1 FROM auth_credentials pc WHERE pc.account_id = p.account_id AND pc.credential IS NOT NULL) \
                 FROM user_profiles p LEFT JOIN LATERAL (SELECT identifier, auth_type FROM auth_credentials \
                 WHERE account_id = p.account_id ORDER BY (auth_type = 'email') DESC, created_at LIMIT 1) c ON TRUE \
                 WHERE p.account_id = $1",
            )
            .bind(account_id)
            .fetch_optional(&self.db)
            .await.map_err(db_error)?;

        Ok(row.map(
            |(
                account_id,
                nickname,
                avatar,
                signature,
                bio,
                email,
                phone,
                identifier,
                auth_type,
                has_password,
            )| UserProfile {
                account_id,
                nickname,
                avatar,
                signature,
                bio,
                email,
                phone,
                identifier,
                auth_type,
                has_password,
            },
        ))
    }

    async fn update_profile(
        &self,
        account_id: i64,
        nickname: Option<&str>,
        avatar: Option<&str>,
        signature: Option<&str>,
    ) -> AppResult<UserProfile> {
        sqlx::query(
            "UPDATE user_profiles SET \
             nickname = COALESCE($2, nickname), avatar = COALESCE($3, avatar), \
             signature = COALESCE($4, signature), updated_at = NOW() \
             WHERE account_id = $1",
        )
        .bind(account_id)
        .bind(nickname)
        .bind(avatar)
        .bind(signature)
        .execute(&self.db)
        .await
        .map_err(db_error)?;
        self.find_profile(account_id)
            .await?
            .ok_or_else(|| fx_core::AppError::not_found("用户资料不存在"))
    }

    async fn create_account_with_identity(&self, identity: &AuthIdentity) -> AppResult<Account> {
        let mut tx = self.db.begin().await.map_err(db_error)?;
        let (account_id,): (i64,) = sqlx::query_as(
            "INSERT INTO accounts (status, created_at, updated_at) \
             VALUES (0, NOW(), NOW()) RETURNING id",
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;

        let nickname = identity
            .display_name
            .clone()
            .unwrap_or_else(|| format!("用户{account_id}"));
        let avatar = identity
            .avatar
            .clone()
            .unwrap_or_else(|| format!("identicon:{account_id}"));
        sqlx::query(
            "INSERT INTO user_profiles (account_id, nickname, avatar, updated_at) \
             VALUES ($1, $2, $3, NOW())",
        )
        .bind(account_id)
        .bind(nickname)
        .bind(avatar)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;

        sqlx::query(
            "INSERT INTO auth_credentials \
             (account_id, auth_type, identifier, verified, created_at) \
             VALUES ($1, $2, $3, true, NOW())",
        )
        .bind(account_id)
        .bind(&identity.auth_type)
        .bind(&identity.identifier)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;

        tx.commit().await.map_err(db_error)?;

        Ok(Account {
            id: account_id,
            status: AccountStatus::Active,
        })
    }

    async fn record_login(&self, account_id: i64, context: &AuthContext) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO login_logs \
             (account_id, ip, platform, device_name, device_id, app_version) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(account_id)
        .bind(context.ip.as_deref())
        .bind(context.platform.as_deref())
        .bind(context.device_name.as_deref())
        .bind(context.device_id.as_deref())
        .bind(context.app_version.as_deref())
        .execute(&self.db)
        .await
        .map_err(db_error)?;
        Ok(())
    }

    async fn password_hash(&self, account_id: i64) -> AppResult<Option<String>> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT credential FROM auth_credentials WHERE account_id = $1 ORDER BY created_at LIMIT 1",
        )
        .bind(account_id)
        .fetch_optional(&self.db)
        .await.map_err(db_error)?;
        Ok(row.and_then(|(hash,)| hash))
    }

    async fn save_password_hash(&self, account_id: i64, hash: &str) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE auth_credentials SET credential = $2 WHERE id = \
             (SELECT id FROM auth_credentials WHERE account_id = $1 ORDER BY created_at LIMIT 1)",
        )
        .bind(account_id)
        .bind(hash)
        .execute(&self.db)
        .await
        .map_err(db_error)?;
        if result.rows_affected() == 0 {
            return Err(fx_core::AppError::not_found("用户认证凭据不存在"));
        }
        Ok(())
    }

    async fn bind_credential(
        &self,
        account_id: i64,
        auth_type: &str,
        identifier: &str,
    ) -> AppResult<()> {
        let mut tx = self.db.begin().await.map_err(db_error)?;
        let result = sqlx::query(
            "UPDATE auth_credentials SET identifier = $2, verified = true \
             WHERE account_id = $1 AND auth_type = $3",
        )
        .bind(account_id)
        .bind(identifier)
        .bind(auth_type)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        if result.rows_affected() == 0 {
            sqlx::query(
                "INSERT INTO auth_credentials \
                 (account_id, auth_type, identifier, verified, created_at) \
                 VALUES ($1, $2, $3, true, NOW())",
            )
            .bind(account_id)
            .bind(auth_type)
            .bind(identifier)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        }
        tx.commit().await.map_err(db_error)?;
        Ok(())
    }

    async fn delete_account(&self, account_id: i64) -> AppResult<()> {
        let mut tx = self.db.begin().await.map_err(db_error)?;
        sqlx::query("DELETE FROM scan_sessions WHERE user_id = $1")
            .bind(account_id)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        sqlx::query("DELETE FROM login_logs WHERE account_id = $1")
            .bind(account_id)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        sqlx::query("DELETE FROM user_profiles WHERE account_id = $1")
            .bind(account_id)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        sqlx::query("DELETE FROM auth_credentials WHERE account_id = $1")
            .bind(account_id)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        let result = sqlx::query(
            "UPDATE accounts SET status = 2, updated_at = NOW() WHERE id = $1 AND status <> 2",
        )
        .bind(account_id)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        if result.rows_affected() == 0 {
            return Err(fx_core::AppError::not_found("用户不存在"));
        }
        tx.commit().await.map_err(db_error)?;
        Ok(())
    }

    async fn create_scan_session(
        &self,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        sqlx::query("INSERT INTO scan_sessions (token, status, expires_at) VALUES ($1, 0, $2)")
            .bind(token)
            .bind(expires_at)
            .execute(&self.db)
            .await
            .map_err(db_error)?;
        Ok(())
    }

    async fn find_scan_session(&self, token: &str) -> AppResult<Option<ScanSession>> {
        let row: Option<(i16, Option<i64>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            "SELECT status, user_id, expires_at FROM scan_sessions WHERE token = $1",
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await
        .map_err(db_error)?;
        Ok(row.map(|(status, user_id, expires_at)| ScanSession {
            status,
            user_id,
            expires_at,
        }))
    }

    async fn mark_scan_scanned(&self, token: &str, account_id: i64) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE scan_sessions SET status = 1, user_id = $2 \
             WHERE token = $1 AND status = 0 AND expires_at > NOW()",
        )
        .bind(token)
        .bind(account_id)
        .execute(&self.db)
        .await
        .map_err(db_error)?;
        Ok(result.rows_affected() == 1)
    }

    async fn mark_scan_confirmed(&self, token: &str, account_id: i64) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE scan_sessions SET status = 2 \
             WHERE token = $1 AND status = 1 AND user_id = $2 AND expires_at > NOW()",
        )
        .bind(token)
        .bind(account_id)
        .execute(&self.db)
        .await
        .map_err(db_error)?;
        Ok(result.rows_affected() == 1)
    }

    async fn cancel_scan(&self, token: &str, account_id: i64) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE scan_sessions SET status = 3 \
             WHERE token = $1 AND status = 1 AND user_id = $2",
        )
        .bind(token)
        .bind(account_id)
        .execute(&self.db)
        .await
        .map_err(db_error)?;
        Ok(result.rows_affected() == 1)
    }
}

fn account_status_from_db(status: i16) -> AccountStatus {
    match status {
        1 => AccountStatus::Disabled,
        2 => AccountStatus::Deleted,
        _ => AccountStatus::Active,
    }
}
