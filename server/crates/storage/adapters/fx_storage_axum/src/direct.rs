//! OSS 直传宿主路由(薄):装配 `DirectUploadService` + 挂路由。
//!
//! 直传编排(object_key 生成/配额预检/签发/确认存在性+落库+扣配额)已下沉到
//! `fx_storage::DirectUploadService`;本文件只做 HTTP 解析与错误映射。
//! 仅当 OSS env 配齐时由 main.rs 挂载。

use std::sync::Arc;

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use fx_server_web::{ApiResult, api_ok};
use fx_storage_service::{
    ConfirmRequest, ConfirmResult, DirectUploadService, IssueGrantRequest, UploadGrant,
};
use serde::Deserialize;

use crate::{ScopeResolver, map_storage_err};

pub struct DirectUploadState<
    B: fx_storage_service::StorageBackend,
    I: fx_storage_service::DirectUploadIssuer,
> {
    pub service: Arc<DirectUploadService<B, I>>,
    pub scopes: Arc<dyn ScopeResolver>,
}

impl<B: fx_storage_service::StorageBackend, I: fx_storage_service::DirectUploadIssuer> Clone
    for DirectUploadState<B, I>
{
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            scopes: self.scopes.clone(),
        }
    }
}

pub fn direct_routes<B, I>(state: DirectUploadState<B, I>) -> Router
where
    B: fx_storage_service::StorageBackend,
    I: fx_storage_service::DirectUploadIssuer + 'static,
{
    Router::new()
        .route("/storage/upload-token", post(upload_token::<B, I>))
        .route("/storage/confirm-upload", post(confirm_upload::<B, I>))
        .with_state(state)
}

#[derive(Deserialize)]
struct UploadTokenBody {
    file_name: String,
    file_size: i64,
    mime_type: String,
    thumb_size: Option<i64>,
}

async fn upload_token<B, I>(
    State(st): State<DirectUploadState<B, I>>,
    headers: HeaderMap,
    Json(body): Json<UploadTokenBody>,
) -> ApiResult<UploadGrant>
where
    B: fx_storage_service::StorageBackend,
    I: fx_storage_service::DirectUploadIssuer + 'static,
{
    let scope = st.scopes.resolve(&headers)?;
    let grant = st
        .service
        .issue_grant(
            &scope,
            IssueGrantRequest {
                file_name: body.file_name,
                file_size: body.file_size,
                mime_type: body.mime_type,
                thumb_size: body.thumb_size,
            },
        )
        .await
        .map_err(map_storage_err)?;
    Ok(api_ok(grant))
}

#[derive(Deserialize)]
struct ConfirmUploadBody {
    object_key: String,
    file_size: i64,
    mime_type: String,
    mime_category: String,
    hash: String,
    thumb_object_key: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    duration_ms: Option<i64>,
    original_name: Option<String>,
}

async fn confirm_upload<B, I>(
    State(st): State<DirectUploadState<B, I>>,
    headers: HeaderMap,
    Json(body): Json<ConfirmUploadBody>,
) -> ApiResult<ConfirmResult>
where
    B: fx_storage_service::StorageBackend,
    I: fx_storage_service::DirectUploadIssuer + 'static,
{
    let scope = st.scopes.resolve(&headers)?;
    let result = st
        .service
        .confirm(
            &scope,
            ConfirmRequest {
                object_key: body.object_key,
                file_size: body.file_size,
                mime_type: body.mime_type,
                mime_category: body.mime_category,
                hash: body.hash,
                thumb_object_key: body.thumb_object_key,
                width: body.width,
                height: body.height,
                duration_ms: body.duration_ms,
                original_name: body.original_name,
            },
        )
        .await
        .map_err(map_storage_err)?;
    Ok(api_ok(result))
}
