use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{ConnectInfo, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use fx_core::AppResult;
use fx_server_web::{ApiResult, AppError, api_empty, api_message, api_ok};
use serde::{Deserialize, Serialize};

use fx_auth_core::{AuthContext, AuthIdentity, AuthInput, VerificationCodeProviders};
use fx_user_core::{
    AccountIdentifierCheck, AccountStatus, LoginResponse, TokenProvider, UserContext, UserResponse,
};
use fx_user_service::FxUserService;

#[derive(Clone)]
struct UserApiState {
    service: FxUserService,
    codes: Arc<VerificationCodeProviders>,
    tokens: Arc<dyn TokenProvider>,
}

pub fn user_routes(
    service: FxUserService,
    codes: Arc<VerificationCodeProviders>,
    tokens: Arc<dyn TokenProvider>,
) -> Router {
    Router::new()
        .route("/user/health", get(health))
        .route("/auth/code", post(request_code))
        .route("/auth/login", post(login))
        .route("/auth/password/reset", post(reset_password))
        .route("/auth/logout", post(logout))
        .route("/auth/scan/create", post(scan_create))
        .route("/auth/scan/status", get(scan_status))
        .route("/auth/scan/confirm", post(scan_confirm))
        .route("/auth/scan/cancel", post(scan_cancel))
        .route("/user/profile", get(profile).put(update_profile))
        .route("/user/account/check", post(check_account))
        .route("/user/email", axum::routing::put(bind_email))
        .route("/user/phone", axum::routing::put(bind_phone))
        .route("/user/password", post(set_password).put(change_password))
        .route("/user/account", axum::routing::delete(delete_account))
        .with_state(UserApiState {
            service,
            codes,
            tokens,
        })
}

#[derive(Deserialize)]
struct CodeRequest {
    channel: String,
    identifier: String,
    #[serde(default = "default_code_scene")]
    scene: String,
}

fn default_code_scene() -> String {
    "login".into()
}

#[derive(Serialize)]
struct CodeResponse {
    sent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
}

async fn request_code(
    State(state): State<UserApiState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<CodeRequest>,
) -> ApiResult<CodeResponse> {
    if !matches!(
        request.scene.as_str(),
        "login" | "reset_password" | "bind_email" | "bind_phone"
    ) {
        return Err(
            AppError::bad_request_code("AUTH_CODE_SCENE_INVALID", "不支持的验证码场景").into(),
        );
    }
    let identifier = request.identifier.trim();
    if matches!(request.scene.as_str(), "bind_email" | "bind_phone") {
        let user = resolve_user(&headers, state.tokens.as_ref())?;
        let account_type = if request.scene == "bind_phone" {
            "phone"
        } else {
            "email"
        };
        let expected_channel = if account_type == "phone" {
            "sms"
        } else {
            "email"
        };
        if request.channel != expected_channel {
            return Err(AppError::bad_request_code(
                "AUTH_CODE_CHANNEL_INVALID",
                "验证码通道与使用场景不匹配",
            )
            .into());
        }
        let check = state
            .service
            .check_account(&user, account_type, identifier)
            .await?;
        if !check.available {
            let (code, message) = if account_type == "phone" {
                ("AUTH_PHONE_ALREADY_BOUND", "该手机号已被其他账号绑定")
            } else {
                ("AUTH_EMAIL_ALREADY_BOUND", "该邮箱已被其他账号绑定")
            };
            return Err(AppError::conflict(code, message).into());
        }
    }
    let code = state
        .codes
        .request_code(
            &request.channel,
            identifier,
            &request.scene,
            AuthContext {
                ip: Some(address.ip().to_string()),
                ..Default::default()
            },
        )
        .await?;
    Ok(api_ok(CodeResponse { sent: true, code }))
}

#[derive(Deserialize)]
struct LoginRequest {
    #[serde(rename = "type")]
    kind: String,
    identifier: String,
    credential: String,
    #[serde(default)]
    device_info: DeviceInfo,
}

#[derive(Default, Deserialize)]
struct DeviceInfo {
    platform: Option<String>,
    device_name: Option<String>,
    device_id: Option<String>,
    app_version: Option<String>,
}

async fn login(
    State(state): State<UserApiState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(request): Json<LoginRequest>,
) -> ApiResult<LoginResponse> {
    let context = AuthContext {
        ip: Some(address.ip().to_string()),
        platform: request.device_info.platform,
        device_name: request.device_info.device_name,
        device_id: request.device_info.device_id,
        app_version: request.device_info.app_version,
    };
    let identity = state
        .service
        .authenticate(
            AuthInput {
                kind: request.kind,
                identifier: request.identifier,
                credential: request.credential,
            },
            context.clone(),
        )
        .await?;
    let user = state.service.find_or_create_account(&identity).await?;
    Ok(api_ok(state.service.issue_login(user, &context).await?))
}

#[derive(Deserialize)]
struct SetPasswordRequest {
    new_password: String,
}

async fn set_password(
    State(state): State<UserApiState>,
    headers: HeaderMap,
    Json(request): Json<SetPasswordRequest>,
) -> ApiResult<()> {
    let user = resolve_user(&headers, state.tokens.as_ref())?;
    state
        .service
        .set_password(&user, None, &request.new_password)
        .await?;
    Ok(api_empty("密码设置成功"))
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

async fn change_password(
    State(state): State<UserApiState>,
    headers: HeaderMap,
    Json(request): Json<ChangePasswordRequest>,
) -> ApiResult<()> {
    let user = resolve_user(&headers, state.tokens.as_ref())?;
    state
        .service
        .set_password(&user, Some(&request.old_password), &request.new_password)
        .await?;
    Ok(api_empty("密码修改成功"))
}

#[derive(Deserialize)]
struct ResetPasswordRequest {
    email: String,
    code: String,
    new_password: String,
}

/// 校验一次性邮箱验证码，并为已存在账号设置新密码。
async fn reset_password(
    State(state): State<UserApiState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(request): Json<ResetPasswordRequest>,
) -> ApiResult<()> {
    let context = AuthContext {
        ip: Some(address.ip().to_string()),
        ..Default::default()
    };
    let email = request.email.trim();
    state
        .codes
        .verify_code("email", email, &request.code, "reset_password", context)
        .await?;
    let identity = AuthIdentity {
        auth_type: "email".into(),
        identifier: email.into(),
        display_name: None,
        avatar: None,
    };
    state
        .service
        .reset_password(&identity, &request.new_password)
        .await?;
    Ok(api_empty("密码重置成功"))
}

#[derive(Deserialize)]
struct BindEmailRequest {
    email: String,
    code: String,
}

#[derive(Deserialize)]
struct BindPhoneRequest {
    phone: String,
    code: String,
}

#[derive(Deserialize)]
struct CheckAccountRequest {
    #[serde(rename = "type")]
    kind: String,
    identifier: String,
}

/// 检查账号标识是否可供当前登录用户绑定。
async fn check_account(
    State(state): State<UserApiState>,
    headers: HeaderMap,
    Json(request): Json<CheckAccountRequest>,
) -> ApiResult<AccountIdentifierCheck> {
    let user = resolve_user(&headers, state.tokens.as_ref())?;
    let result = state
        .service
        .check_account(&user, &request.kind, &request.identifier)
        .await?;
    Ok(api_ok(result))
}

/// 校验绑定邮箱专用验证码，并将邮箱加入当前账号的认证凭据。
async fn bind_email(
    State(state): State<UserApiState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<BindEmailRequest>,
) -> ApiResult<UserResponse> {
    let user = resolve_user(&headers, state.tokens.as_ref())?;
    let email = request.email.trim();
    state
        .codes
        .verify_code(
            "email",
            email,
            &request.code,
            "bind_email",
            AuthContext {
                ip: Some(address.ip().to_string()),
                ..Default::default()
            },
        )
        .await?;
    let profile = state.service.bind_email(&user, email).await?;
    Ok(api_ok(UserResponse::from(profile)))
}

/// 校验绑定手机号专用验证码，并将手机号加入当前账号的认证凭据。
async fn bind_phone(
    State(state): State<UserApiState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<BindPhoneRequest>,
) -> ApiResult<UserResponse> {
    let user = resolve_user(&headers, state.tokens.as_ref())?;
    let phone = request.phone.trim();
    state
        .codes
        .verify_code(
            "sms",
            phone,
            &request.code,
            "bind_phone",
            AuthContext {
                ip: Some(address.ip().to_string()),
                ..Default::default()
            },
        )
        .await?;
    let profile = state.service.bind_phone(&user, phone).await?;
    Ok(api_ok(UserResponse::from(profile)))
}

#[derive(Deserialize)]
struct DeleteAccountRequest {
    password: String,
}

async fn delete_account(
    State(state): State<UserApiState>,
    headers: HeaderMap,
    Json(request): Json<DeleteAccountRequest>,
) -> ApiResult<()> {
    let user = resolve_user(&headers, state.tokens.as_ref())?;
    state
        .service
        .delete_account(&user, &request.password)
        .await?;
    Ok(api_empty("账号注销成功"))
}

async fn profile(State(state): State<UserApiState>, headers: HeaderMap) -> ApiResult<UserResponse> {
    let user = resolve_user(&headers, state.tokens.as_ref())?;
    let profile = state.service.current_profile(&user).await?;
    Ok(api_ok(UserResponse::from(profile)))
}

#[derive(Deserialize)]
struct ProfileUpdate {
    display_name: Option<String>,
    avatar: Option<String>,
    signature: Option<String>,
}

async fn update_profile(
    State(state): State<UserApiState>,
    headers: HeaderMap,
    Json(request): Json<ProfileUpdate>,
) -> ApiResult<UserResponse> {
    let user = resolve_user(&headers, state.tokens.as_ref())?;
    let profile = state
        .service
        .update_profile(
            &user,
            request.display_name.as_deref(),
            request.avatar.as_deref(),
            request.signature.as_deref(),
        )
        .await?;
    Ok(api_ok(UserResponse::from(profile)))
}

async fn logout(State(state): State<UserApiState>, headers: HeaderMap) -> ApiResult<()> {
    let token = bearer_token(&headers)?;
    state.tokens.revoke(token)?;
    Ok(api_empty("退出登录成功"))
}

#[derive(Serialize)]
struct ScanCreateResponse {
    token: String,
    qr_content: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

async fn scan_create(State(state): State<UserApiState>) -> ApiResult<ScanCreateResponse> {
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);
    state
        .service
        .create_scan_session(&token, expires_at)
        .await?;
    Ok(api_ok(ScanCreateResponse {
        qr_content: format!("fx://auth/scan/{token}"),
        token,
        expires_at,
    }))
}

#[derive(Deserialize)]
struct ScanStatusQuery {
    token: String,
}

async fn scan_status(
    State(state): State<UserApiState>,
    Query(query): Query<ScanStatusQuery>,
) -> ApiResult<serde_json::Value> {
    let session = state
        .service
        .scan_session(&query.token)
        .await?
        .ok_or_else(|| AppError::not_found("扫码会话不存在"))?;
    if chrono::Utc::now() > session.expires_at && session.status < 2 {
        return Ok(api_ok(serde_json::json!({"status": "expired"})));
    }
    let name = match session.status {
        0 => "pending",
        1 => "scanned",
        2 => "confirmed",
        3 => "cancelled",
        _ => "unknown",
    };
    if session.status == 2 {
        let account_id = session
            .user_id
            .ok_or_else(|| AppError::bad_request("扫码用户不存在"))?;
        let token = state.tokens.issue(&UserContext {
            account_id,
            status: AccountStatus::Active,
        })?;
        return Ok(api_ok(serde_json::json!({
            "status": name,
            "token": token,
            "user_id": account_id,
        })));
    }
    Ok(api_ok(serde_json::json!({"status": name})))
}

#[derive(Deserialize)]
struct ScanActionRequest {
    scan_token: String,
    #[serde(default)]
    action: String,
}

async fn scan_confirm(
    State(state): State<UserApiState>,
    headers: HeaderMap,
    Json(request): Json<ScanActionRequest>,
) -> ApiResult<()> {
    let user = resolve_user(&headers, state.tokens.as_ref())?;
    let session = state
        .service
        .scan_session(&request.scan_token)
        .await?
        .ok_or_else(|| AppError::bad_request("扫码会话不存在"))?;
    if chrono::Utc::now() > session.expires_at {
        return Err(AppError::bad_request("二维码已过期"));
    }
    let changed = match request.action.as_str() {
        "scan" => {
            state
                .service
                .mark_scan_scanned(&request.scan_token, user.account_id)
                .await?
        }
        "confirm" => {
            state
                .service
                .mark_scan_confirmed(&request.scan_token, user.account_id)
                .await?
        }
        _ => return Err(AppError::bad_request("扫码状态不匹配")),
    };
    if !changed {
        return Err(AppError::bad_request("扫码状态不匹配"));
    }
    Ok(api_empty("扫码状态更新成功"))
}

async fn scan_cancel(
    State(state): State<UserApiState>,
    headers: HeaderMap,
    Json(request): Json<ScanActionRequest>,
) -> ApiResult<()> {
    let user = resolve_user(&headers, state.tokens.as_ref())?;
    if !state
        .service
        .cancel_scan(&request.scan_token, user.account_id)
        .await?
    {
        return Err(AppError::bad_request("扫码状态不匹配"));
    }
    Ok(api_empty("扫码登录已取消"))
}

fn resolve_user(headers: &HeaderMap, tokens: &dyn TokenProvider) -> AppResult<UserContext> {
    tokens.resolve(bearer_token(headers)?)
}

fn bearer_token(headers: &HeaderMap) -> AppResult<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| fx_core::AppError::unauthorized("缺少有效登录凭据"))
}

async fn health() -> ApiResult<serde_json::Value> {
    Ok(api_message(
        serde_json::json!({
            "module": "fx_user",
            "status": "ok"
        }),
        "用户模块运行正常",
    ))
}
