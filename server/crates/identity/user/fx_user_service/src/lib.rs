use std::sync::Arc;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use fx_auth_core::{AuthContext, AuthIdentity, AuthInput, AuthProvider};
use fx_core::{AppError, AppResult};

pub use fx_user_core::{
    AccountIdentifierCheck, EmptyUserHooks, LoginResponse, TokenProvider, UserContext, UserHooks,
    UserProfile, UserRepository, UserResponse,
};

#[derive(Clone)]
pub struct FxUserService {
    repo: Arc<dyn UserRepository>,
    auth_providers: Arc<Vec<Arc<dyn AuthProvider>>>,
    token_provider: Arc<dyn TokenProvider>,
    hooks: Arc<dyn UserHooks>,
    registration_enabled: bool,
}

impl FxUserService {
    pub async fn create_scan_session(
        &self,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        self.repo.create_scan_session(token, expires_at).await
    }

    pub async fn scan_session(&self, token: &str) -> AppResult<Option<fx_user_core::ScanSession>> {
        self.repo.find_scan_session(token).await
    }

    pub async fn mark_scan_scanned(&self, token: &str, account_id: i64) -> AppResult<bool> {
        self.repo.mark_scan_scanned(token, account_id).await
    }

    pub async fn mark_scan_confirmed(&self, token: &str, account_id: i64) -> AppResult<bool> {
        self.repo.mark_scan_confirmed(token, account_id).await
    }

    pub async fn cancel_scan(&self, token: &str, account_id: i64) -> AppResult<bool> {
        self.repo.cancel_scan(token, account_id).await
    }
    pub fn new(repo: Arc<dyn UserRepository>, token_provider: Arc<dyn TokenProvider>) -> Self {
        Self {
            repo,
            auth_providers: Arc::new(Vec::new()),
            token_provider,
            hooks: Arc::new(EmptyUserHooks),
            registration_enabled: true,
        }
    }

    pub fn with_auth_provider(mut self, provider: Arc<dyn AuthProvider>) -> Self {
        Arc::make_mut(&mut self.auth_providers).push(provider);
        self
    }

    pub fn with_hooks(mut self, hooks: Arc<dyn UserHooks>) -> Self {
        self.hooks = hooks;
        self
    }

    pub fn with_registration_enabled(mut self, enabled: bool) -> Self {
        self.registration_enabled = enabled;
        self
    }

    pub async fn authenticate(
        &self,
        input: AuthInput,
        ctx: AuthContext,
    ) -> AppResult<AuthIdentity> {
        let provider = self
            .auth_providers
            .iter()
            .find(|provider| provider.kind() == input.kind)
            .ok_or_else(|| AppError::bad_request("不支持的认证方式"))?;
        provider.authenticate(input, ctx).await
    }

    pub async fn find_or_create_account(&self, identity: &AuthIdentity) -> AppResult<UserContext> {
        if let Some(credential) = self
            .repo
            .find_credential(&identity.auth_type, &identity.identifier)
            .await?
        {
            let account = self
                .repo
                .find_account_by_id(credential.account_id)
                .await?
                .ok_or_else(|| AppError::not_found("账号不存在"))?;
            return Ok(UserContext {
                account_id: account.id,
                status: account.status,
            });
        }

        if !self.registration_enabled {
            return Err(AppError::forbidden("当前未开放新用户注册"));
        }

        let account = self.repo.create_account_with_identity(identity).await?;
        self.hooks.on_user_created(account.id).await?;
        self.hooks
            .on_credential_bound(account.id, &identity.auth_type)
            .await?;
        Ok(UserContext {
            account_id: account.id,
            status: account.status,
        })
    }

    pub async fn issue_login(
        &self,
        user: UserContext,
        context: &AuthContext,
    ) -> AppResult<LoginResponse> {
        if !user.status.is_active() {
            return Err(AppError::forbidden("账号不可用"));
        }

        let token = self.token_provider.issue(&user)?;
        self.repo.record_login(user.account_id, context).await?;
        self.hooks.on_user_logged_in(user.account_id).await?;
        let profile = self
            .repo
            .find_profile(user.account_id)
            .await?
            .ok_or_else(|| AppError::not_found("用户资料不存在"))?;
        Ok(LoginResponse {
            token,
            user_id: user.account_id,
            has_password: profile.has_password,
            user: UserResponse::from(profile),
        })
    }

    pub async fn current_profile(&self, user: &UserContext) -> AppResult<UserProfile> {
        if !user.status.is_active() {
            return Err(AppError::forbidden("账号不可用"));
        }
        self.repo
            .find_profile(user.account_id)
            .await?
            .ok_or_else(|| AppError::not_found("用户资料不存在"))
    }

    pub async fn update_profile(
        &self,
        user: &UserContext,
        nickname: Option<&str>,
        avatar: Option<&str>,
        signature: Option<&str>,
    ) -> AppResult<UserProfile> {
        if !user.status.is_active() {
            return Err(AppError::forbidden("账号不可用"));
        }
        if nickname.is_some_and(|value| value.trim().is_empty() || value.chars().count() > 50) {
            return Err(AppError::bad_request("名字不能为空且不能超过 50 个字符"));
        }
        if signature.is_some_and(|value| value.chars().count() > 100) {
            return Err(AppError::bad_request("签名不能超过 100 个字符"));
        }
        self.repo
            .update_profile(user.account_id, nickname, avatar, signature)
            .await
    }

    pub async fn set_password(
        &self,
        user: &UserContext,
        old_password: Option<&str>,
        new_password: &str,
    ) -> AppResult<()> {
        if new_password.chars().count() < 6 {
            return Err(AppError::bad_request("密码至少需要 6 位"));
        }
        let existing = self.repo.password_hash(user.account_id).await?;
        match (existing.as_deref(), old_password) {
            (Some(_), None) => return Err(AppError::bad_request("已设置密码")),
            (None, Some(_)) => return Err(AppError::bad_request("请先设置密码")),
            (Some(hash), Some(old)) => {
                let parsed = PasswordHash::new(hash)
                    .map_err(|error| AppError::internal(error, "password hash"))?;
                Argon2::default()
                    .verify_password(old.as_bytes(), &parsed)
                    .map_err(|_| AppError::unauthorized("旧密码错误"))?;
            }
            (None, None) => {}
        }
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|error| AppError::internal(error, "password hash"))?
            .to_string();
        self.repo.save_password_hash(user.account_id, &hash).await
    }

    /// 在邮箱验证码已经验证通过后，为既有账号重置密码。
    pub async fn reset_password(
        &self,
        identity: &AuthIdentity,
        new_password: &str,
    ) -> AppResult<()> {
        if new_password.chars().count() < 6 {
            return Err(AppError::bad_request("密码至少需要 6 位"));
        }
        let credential = self
            .repo
            .find_credential(&identity.auth_type, &identity.identifier)
            .await?
            .ok_or_else(|| AppError::not_found("账号不存在"))?;
        let account = self
            .repo
            .find_account_by_id(credential.account_id)
            .await?
            .ok_or_else(|| AppError::not_found("账号不存在"))?;
        if !account.status.is_active() {
            return Err(AppError::forbidden("账号不可用"));
        }
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|error| AppError::internal(error, "password hash"))?
            .to_string();
        self.repo.save_password_hash(account.id, &hash).await
    }

    /// 将已验证邮箱绑定到当前账号，拒绝占用其他账号的邮箱。
    pub async fn bind_email(&self, user: &UserContext, email: &str) -> AppResult<UserProfile> {
        let email = email.trim();
        if !email.contains('@') {
            return Err(AppError::bad_request("邮箱格式不正确"));
        }
        let check = self.check_account(user, "email", email).await?;
        if !check.available {
            return Err(AppError::conflict(
                "AUTH_EMAIL_ALREADY_BOUND",
                "该邮箱已被其他账号绑定",
            ));
        }
        self.repo
            .bind_credential(user.account_id, "email", email)
            .await?;
        self.current_profile(user).await
    }

    /// 将已验证手机号绑定到当前账号，拒绝占用其他账号的手机号。
    pub async fn bind_phone(&self, user: &UserContext, phone: &str) -> AppResult<UserProfile> {
        let phone = phone.trim();
        if !is_valid_phone(phone) {
            return Err(AppError::bad_request_code(
                "AUTH_PHONE_INVALID",
                "手机号格式不正确",
            ));
        }
        let check = self.check_account(user, "phone", phone).await?;
        if !check.available {
            return Err(AppError::conflict(
                "AUTH_PHONE_ALREADY_BOUND",
                "该手机号已被其他账号绑定",
            ));
        }
        self.repo
            .bind_credential(user.account_id, "sms", phone)
            .await?;
        self.current_profile(user).await
    }

    /// 检查账号标识是否存在，以及是否可供当前用户绑定。
    pub async fn check_account(
        &self,
        user: &UserContext,
        auth_type: &str,
        identifier: &str,
    ) -> AppResult<AccountIdentifierCheck> {
        let auth_type = auth_type.trim();
        let identifier = identifier.trim();
        let credential_type = match auth_type {
            "email" => "email",
            "phone" => "sms",
            _ => {
                return Err(AppError::bad_request_code(
                    "AUTH_ACCOUNT_TYPE_UNSUPPORTED",
                    "暂不支持该账号类型",
                ));
            }
        };
        if auth_type == "email" && !identifier.contains('@') {
            return Err(AppError::bad_request_code(
                "AUTH_EMAIL_INVALID",
                "邮箱格式不正确",
            ));
        }
        if auth_type == "phone" && !is_valid_phone(identifier) {
            return Err(AppError::bad_request_code(
                "AUTH_PHONE_INVALID",
                "手机号格式不正确",
            ));
        }
        let credential = self
            .repo
            .find_credential(credential_type, identifier)
            .await?;
        let exists = credential.is_some();
        let owned_by_current_account = credential
            .as_ref()
            .is_some_and(|value| value.account_id == user.account_id);
        Ok(AccountIdentifierCheck {
            exists,
            owned_by_current_account,
            available: !exists || owned_by_current_account,
        })
    }

    pub async fn delete_account(&self, user: &UserContext, password: &str) -> AppResult<()> {
        let hash = self
            .repo
            .password_hash(user.account_id)
            .await?
            .ok_or_else(|| AppError::bad_request("请先设置密码"))?;
        let parsed =
            PasswordHash::new(&hash).map_err(|error| AppError::internal(error, "password hash"))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AppError::unauthorized("密码错误"))?;
        self.repo.delete_account(user.account_id).await?;
        self.hooks.on_user_deleted(user.account_id).await
    }
}

fn is_valid_phone(phone: &str) -> bool {
    phone.len() == 11 && phone.starts_with('1') && phone.chars().all(|value| value.is_ascii_digit())
}
