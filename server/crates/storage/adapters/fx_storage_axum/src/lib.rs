//! Axum adapter for the reusable storage HTTP contract.
//!
//! The host only resolves request credentials into an opaque storage [`Scope`]. Product-specific
//! reference checks and enriched asset catalog endpoints remain in the host domain.

mod direct;
mod public_local;

use std::sync::Arc;

use axum::{
    Router,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use fx_server_web::{ApiResult, AppError, AppResult, api_ok};
use fx_storage_service::{
    ImageFit, ImageOutputFormat, ImageTransform, QuotaSnapshot, Scope, StorageBackend,
    StorageError, StorageService, UploadResult, VideoMeta,
};
use serde::Deserialize;
use uuid::Uuid;

pub use direct::{DirectUploadState, direct_routes};
pub use public_local::{PublicLocalImageState, public_local_image_routes};

/// Host boundary that turns request authentication into an opaque storage scope.
pub trait ScopeResolver: Send + Sync + 'static {
    fn resolve(&self, headers: &HeaderMap) -> AppResult<Scope>;
}

type ResolveBearer = dyn Fn(&str) -> AppResult<Scope> + Send + Sync;

/// Default resolver that owns Bearer header parsing. Hosts only provide token validation and the
/// mapping from their identity model to an opaque storage scope.
pub struct BearerScopeResolver {
    resolve_bearer: Arc<ResolveBearer>,
}

impl BearerScopeResolver {
    pub fn new(resolve_bearer: impl Fn(&str) -> AppResult<Scope> + Send + Sync + 'static) -> Self {
        Self {
            resolve_bearer: Arc::new(resolve_bearer),
        }
    }
}

impl ScopeResolver for BearerScopeResolver {
    fn resolve(&self, headers: &HeaderMap) -> AppResult<Scope> {
        let token = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::unauthorized("缺少有效登录凭据"))?;
        (self.resolve_bearer)(token)
    }
}

pub struct StorageHttpState<B: StorageBackend> {
    pub service: Arc<StorageService<B>>,
    pub scopes: Arc<dyn ScopeResolver>,
}

impl<B: StorageBackend> Clone for StorageHttpState<B> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            scopes: self.scopes.clone(),
        }
    }
}

/// Generic storage routes. Asset listing, detail and deletion are deliberately left to the host
/// because those operations commonly depend on product reference policies.
pub fn storage_routes<B: StorageBackend>(state: StorageHttpState<B>) -> Router {
    let image_limit = state.service.max_image_size() as usize;
    let file_limit = state.service.max_file_size() as usize;
    let video_limit = state.service.max_video_size() as usize;
    Router::new()
        .route(
            "/storage/upload/image",
            post(upload_image::<B>).layer(DefaultBodyLimit::max(image_limit)),
        )
        .route(
            "/storage/upload/video",
            post(upload_video::<B>).layer(DefaultBodyLimit::max(video_limit)),
        )
        .route(
            "/storage/upload/file",
            post(upload_file::<B>).layer(DefaultBodyLimit::max(file_limit)),
        )
        .route("/storage/quota", get(get_quota::<B>))
        .route("/storage/check", get(check_hash::<B>))
        .route("/storage/files/{id}/content", get(get_file_content::<B>))
        .with_state(state)
}

pub fn map_storage_err(error: StorageError) -> AppError {
    match error {
        StorageError::QuotaExceeded { .. } => AppError::new(
            axum::http::StatusCode::FORBIDDEN,
            "STORAGE_QUOTA_EXCEEDED",
            "云空间不足",
        ),
        StorageError::FileTooLarge { size, max } => {
            AppError::bad_request(format!("文件过大: {size} bytes, 最大 {max} bytes"))
        }
        StorageError::UnsupportedType(value) => {
            AppError::bad_request(format!("文件类型不支持: {value}"))
        }
        StorageError::InvalidHash => AppError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "ASSET_HASH_INVALID",
            "文件 hash 必须是 64 位小写 SHA-256",
        ),
        StorageError::HashMismatch => AppError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "ASSET_HASH_MISMATCH",
            "文件内容与 hash 不一致",
        ),
        StorageError::Forbidden(message) => AppError::forbidden(message),
        StorageError::NotOnStore(key) => {
            AppError::bad_request(format!("文件未上传成功，请重试: {key}"))
        }
        StorageError::NotFound => AppError::not_found("文件不存在"),
        other => AppError::internal(other, "storage"),
    }
}

async fn upload_image<B: StorageBackend>(
    State(state): State<StorageHttpState<B>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> ApiResult<UploadResult> {
    let scope = state.scopes.resolve(&headers)?;
    let mut file = None;
    let mut hash = None;
    let mut upload_id = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "file" => {
                let name = field.file_name().unwrap_or("image.jpg").to_owned();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::bad_request("读取文件失败"))?;
                file = Some((bytes.to_vec(), name));
            }
            "hash" => hash = non_empty(field.text().await.unwrap_or_default()),
            "upload_id" => upload_id = non_empty(field.text().await.unwrap_or_default()),
            _ => {}
        }
    }
    let (bytes, name) = file.ok_or_else(|| AppError::bad_request("缺少文件"))?;
    let hash = hash.ok_or_else(|| AppError::bad_request("缺少 hash"))?;
    let upload_id = upload_id
        .as_deref()
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| {
            AppError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "ASSET_UPLOAD_ID_INVALID",
                "upload_id 无效",
            )
        })?;
    let result = state
        .service
        .upload_image_with_id(&scope, &bytes, &name, &hash, upload_id)
        .await
        .map_err(map_storage_err)?;
    Ok(api_ok(result))
}

async fn upload_file<B: StorageBackend>(
    State(state): State<StorageHttpState<B>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> ApiResult<UploadResult> {
    let scope = state.scopes.resolve(&headers)?;
    let mut file = None;
    let mut hash = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "file" => {
                let name = field.file_name().unwrap_or("file.bin").to_owned();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::bad_request("读取文件失败"))?;
                file = Some((bytes.to_vec(), name));
            }
            "hash" => hash = non_empty(field.text().await.unwrap_or_default()),
            _ => {}
        }
    }
    let (bytes, name) = file.ok_or_else(|| AppError::bad_request("缺少文件"))?;
    let hash = hash.ok_or_else(|| AppError::bad_request("缺少 hash"))?;
    let result = state
        .service
        .upload_file(&scope, &bytes, &name, &hash)
        .await
        .map_err(map_storage_err)?;
    Ok(api_ok(result))
}

async fn upload_video<B: StorageBackend>(
    State(state): State<StorageHttpState<B>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> ApiResult<UploadResult> {
    let scope = state.scopes.resolve(&headers)?;
    let mut video = None;
    let mut thumbnail = None;
    let mut hash = None;
    let mut duration_ms = 0;
    let mut width = 0;
    let mut height = 0;
    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "video" => {
                let name = field.file_name().unwrap_or("video.mp4").to_owned();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::bad_request("读取视频失败"))?;
                video = Some((bytes.to_vec(), name));
            }
            "thumbnail" => {
                thumbnail = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| AppError::bad_request("读取缩略图失败"))?
                        .to_vec(),
                );
            }
            "hash" => hash = non_empty(field.text().await.unwrap_or_default()),
            "duration_ms" => duration_ms = parse_number(field.text().await),
            "width" => width = parse_number(field.text().await),
            "height" => height = parse_number(field.text().await),
            _ => {}
        }
    }
    let (bytes, name) = video.ok_or_else(|| AppError::bad_request("缺少视频文件"))?;
    let thumbnail = thumbnail.ok_or_else(|| AppError::bad_request("缺少缩略图"))?;
    let hash = hash.ok_or_else(|| AppError::bad_request("缺少 hash"))?;
    let result = state
        .service
        .upload_video(
            &scope,
            &bytes,
            &name,
            &thumbnail,
            &hash,
            VideoMeta {
                duration_ms,
                width,
                height,
            },
        )
        .await
        .map_err(map_storage_err)?;
    Ok(api_ok(result))
}

async fn get_quota<B: StorageBackend>(
    State(state): State<StorageHttpState<B>>,
    headers: HeaderMap,
) -> ApiResult<QuotaSnapshot> {
    let scope = state.scopes.resolve(&headers)?;
    let result = state
        .service
        .quota_snapshot(&scope)
        .await
        .map_err(map_storage_err)?;
    Ok(api_ok(result))
}

#[derive(Deserialize)]
struct CheckQuery {
    hash: String,
    size: Option<i64>,
}

async fn check_hash<B: StorageBackend>(
    State(state): State<StorageHttpState<B>>,
    headers: HeaderMap,
    Query(query): Query<CheckQuery>,
) -> ApiResult<serde_json::Value> {
    let scope = state.scopes.resolve(&headers)?;
    if let Some(file) = state
        .service
        .check_file_exists(&scope, &query.hash)
        .await
        .map_err(map_storage_err)?
    {
        return Ok(api_ok(serde_json::json!({
            "exists": true,
            "file_id": file.id,
            "url": state.service.url_for(&file.storage_path),
            "thumb_url": file.thumb_path.as_deref().map(|path| state.service.url_for(path)),
            "size": file.size,
            "width": file.width,
            "height": file.height,
            "duration_ms": file.duration_ms,
            "mime_type": file.mime_type,
        })));
    }
    if let Some(size) = query.size {
        state
            .service
            .check_quota_only(&scope, size)
            .await
            .map_err(map_storage_err)?;
    }
    Err(AppError::not_found("文件未上传过，可以上传"))
}

#[derive(Deserialize)]
pub(crate) struct ContentQuery {
    pub(crate) variant: Option<String>,
    pub(crate) w: Option<u32>,
    pub(crate) h: Option<u32>,
    pub(crate) fit: Option<String>,
    pub(crate) q: Option<u8>,
    pub(crate) format: Option<String>,
}

async fn get_file_content<B: StorageBackend>(
    State(state): State<StorageHttpState<B>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(query): Query<ContentQuery>,
) -> AppResult<Response> {
    let scope = state.scopes.resolve(&headers)?;
    let content = if query.w.is_some() || query.h.is_some() {
        let transform = parse_image_transform(&query)?;
        state
            .service
            .read_image_variant(id, &scope, transform)
            .await
            .map_err(map_storage_err)?
    } else {
        state
            .service
            .read_file(
                id,
                &scope,
                matches!(query.variant.as_deref(), Some("thumbnail")),
            )
            .await
            .map_err(map_storage_err)?
    };
    let content_type = HeaderValue::from_str(&content.mime_type)
        .map_err(|_| AppError::internal("invalid asset mime type", "storage content"))?;
    let mut response = content.bytes.into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, content_type);
    Ok(response)
}

pub(crate) fn parse_image_transform(query: &ContentQuery) -> AppResult<ImageTransform> {
    const MAX_DIMENSION: u32 = 4096;
    const MAX_PIXELS: u64 = 16_000_000;
    let width = query.w;
    let height = query.h;
    if width == Some(0)
        || height == Some(0)
        || width.is_some_and(|value| value > MAX_DIMENSION)
        || height.is_some_and(|value| value > MAX_DIMENSION)
        || width.unwrap_or(1) as u64 * height.unwrap_or(1) as u64 > MAX_PIXELS
    {
        return Err(AppError::bad_request("图片尺寸超出允许范围"));
    }
    let fit = match query.fit.as_deref().unwrap_or("contain") {
        "contain" => ImageFit::Contain,
        "cover" => ImageFit::Cover,
        "fill" => ImageFit::Fill,
        _ => return Err(AppError::bad_request("fit 仅支持 contain、cover 或 fill")),
    };
    let quality = query.q.unwrap_or(80);
    if !(40..=95).contains(&quality) {
        return Err(AppError::bad_request("q 必须在 40 到 95 之间"));
    }
    let format = match query.format.as_deref().unwrap_or("webp") {
        "webp" => ImageOutputFormat::WebP,
        "jpeg" | "jpg" => ImageOutputFormat::Jpeg,
        "png" => ImageOutputFormat::Png,
        _ => return Err(AppError::bad_request("format 仅支持 webp、jpeg 或 png")),
    };
    Ok(ImageTransform {
        width,
        height,
        fit,
        quality,
        format,
    })
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn parse_number<T: Default + std::str::FromStr>(
    value: Result<String, axum::extract::multipart::MultipartError>,
) -> T {
    value
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_resolver_maps_token_to_scope() {
        let resolver = BearerScopeResolver::new(|token| Ok(Arc::from(format!("tenant:{token}"))));
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer abc".parse().unwrap(),
        );

        assert_eq!(resolver.resolve(&headers).unwrap().as_ref(), "tenant:abc");
    }

    #[test]
    fn bearer_resolver_rejects_missing_credentials() {
        let resolver = BearerScopeResolver::new(|_| Ok(Arc::from("unused")));

        assert!(resolver.resolve(&HeaderMap::new()).is_err());
    }

    #[test]
    fn dynamic_image_query_accepts_bounded_cover_transform() {
        let transform = parse_image_transform(&ContentQuery {
            variant: None,
            w: Some(240),
            h: Some(135),
            fit: Some("cover".to_owned()),
            q: Some(80),
            format: Some("webp".to_owned()),
        })
        .unwrap();

        assert_eq!(transform.width, Some(240));
        assert_eq!(transform.height, Some(135));
        assert_eq!(transform.fit, ImageFit::Cover);
        assert_eq!(transform.format, ImageOutputFormat::WebP);
    }

    #[test]
    fn dynamic_image_query_rejects_oversized_dimensions() {
        let result = parse_image_transform(&ContentQuery {
            variant: None,
            w: Some(4097),
            h: Some(100),
            fit: None,
            q: None,
            format: None,
        });

        assert!(result.is_err());
    }
}
