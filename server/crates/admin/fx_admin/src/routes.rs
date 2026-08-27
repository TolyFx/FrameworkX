use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, header::AUTHORIZATION},
    routing::{get, post},
};
use fx_server_web::{ApiResult, AppError, api_empty, api_ok};
use fx_user_core::AccountStatus;
use serde::Deserialize;

use crate::{AdminUserDetail, AdminUserPage, AdminUserQuery, AdminUserService};

#[derive(Clone)]
pub struct AdminAccess {
    token: String,
    actor: String,
}

impl AdminAccess {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            actor: "static-admin".into(),
        }
    }

    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = actor.into();
        self
    }

    pub fn authorize(&self, headers: &HeaderMap) -> Result<(), AppError> {
        let received = headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::unauthorized("缺少管理员凭据"))?;
        if received != self.token {
            return Err(AppError::forbidden("管理员凭据无效"));
        }
        Ok(())
    }

    pub fn actor(&self) -> &str {
        &self.actor
    }
}

#[derive(Clone)]
pub struct AdminApiState {
    service: AdminUserService,
    access: AdminAccess,
}

impl AdminApiState {
    pub fn new(service: AdminUserService, access: AdminAccess) -> Self {
        Self { service, access }
    }
}

pub fn admin_routes(state: AdminApiState) -> Router {
    Router::new()
        .route("/admin/api/v1/health", get(health))
        .route("/admin/api/v1/users", get(users))
        .route("/admin/api/v1/users/{account_id}", get(user_detail))
        .route(
            "/admin/api/v1/users/{account_id}/actions/enable",
            post(enable_user),
        )
        .route(
            "/admin/api/v1/users/{account_id}/actions/disable",
            post(disable_user),
        )
        .with_state(state)
}

async fn health(State(state): State<AdminApiState>, headers: HeaderMap) -> ApiResult<&'static str> {
    require_admin(&headers, &state)?;
    Ok(api_ok("OK"))
}

async fn users(
    State(state): State<AdminApiState>,
    headers: HeaderMap,
    Query(query): Query<AdminUserQuery>,
) -> ApiResult<AdminUserPage> {
    require_admin(&headers, &state)?;
    Ok(api_ok(state.service.page_users(&query).await?))
}

async fn user_detail(
    State(state): State<AdminApiState>,
    headers: HeaderMap,
    Path(account_id): Path<i64>,
) -> ApiResult<AdminUserDetail> {
    require_admin(&headers, &state)?;
    Ok(api_ok(state.service.user_detail(account_id).await?))
}

#[derive(Default, Deserialize)]
struct StatusActionRequest {
    reason: Option<String>,
}

async fn enable_user(
    State(state): State<AdminApiState>,
    headers: HeaderMap,
    Path(account_id): Path<i64>,
    payload: Option<Json<StatusActionRequest>>,
) -> ApiResult<()> {
    set_status(state, headers, account_id, AccountStatus::Active, payload).await
}

async fn disable_user(
    State(state): State<AdminApiState>,
    headers: HeaderMap,
    Path(account_id): Path<i64>,
    payload: Option<Json<StatusActionRequest>>,
) -> ApiResult<()> {
    set_status(state, headers, account_id, AccountStatus::Disabled, payload).await
}

async fn set_status(
    state: AdminApiState,
    headers: HeaderMap,
    account_id: i64,
    status: AccountStatus,
    payload: Option<Json<StatusActionRequest>>,
) -> ApiResult<()> {
    require_admin(&headers, &state)?;
    state
        .service
        .set_user_status(
            account_id,
            status,
            state.access.actor(),
            payload.as_ref().and_then(|value| value.reason.as_deref()),
        )
        .await?;
    Ok(api_empty("用户状态更新成功"))
}

fn require_admin(headers: &HeaderMap, state: &AdminApiState) -> Result<(), AppError> {
    state.access.authorize(headers)
}
