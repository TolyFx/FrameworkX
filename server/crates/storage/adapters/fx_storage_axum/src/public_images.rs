//! 独立的图片读取入口：不创建存储服务、上传接口、用户或数据库。
//! 原图委托 ServeDir；有尺寸参数时在受并发限制的阻塞线程生成临时变体。

use std::{path::PathBuf, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, Request, State, rejection::QueryRejection},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use fx_server_web::{AppError, AppResult};
use fx_storage_service::{MetadataExtractor, StorageError};
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::{ContentQuery, map_storage_err, parse_image_transform};

/// 宿主只注入本地目录与图片处理器；默认源文件上限 25 MiB、同时转换 2 张。
#[derive(Clone)]
pub struct PublicImageState {
    root: PathBuf,
    images: Arc<dyn MetadataExtractor>,
    max_source_size: u64,
    permits: Arc<Semaphore>,
}

impl PublicImageState {
    pub fn new(root: impl Into<PathBuf>, images: Arc<dyn MetadataExtractor>) -> Self {
        Self {
            root: root.into(),
            images,
            max_source_size: 25 * 1024 * 1024,
            permits: Arc::new(Semaphore::new(2)),
        }
    }

    pub fn with_max_source_size(mut self, bytes: u64) -> Self {
        self.max_source_size = bytes;
        self
    }

    /// 至少保留一个转换槽；配置在创建路由前完成。
    pub fn with_concurrency(mut self, count: usize) -> Self {
        self.permits = Arc::new(Semaphore::new(count.max(1)));
        self
    }
}

/// 可挂载在任意前缀，文件路径相对于 root，无 /original 或上传目录约定。
pub fn public_image_routes(state: PublicImageState) -> Router {
    Router::new()
        .route("/{*path}", get(get_image))
        .with_state(state)
}

async fn get_image(
    State(state): State<PublicImageState>,
    Path(path): Path<String>,
    query: Result<Query<ContentQuery>, QueryRejection>,
    request: Request,
) -> AppResult<Response> {
    let Query(query) = query.map_err(|_| AppError::bad_request("图片参数格式无效"))?;
    // 在读取文件前校验尺寸、格式、质量；与 FlutterUnit 共用同一参数协议。
    let transform = if query.w.is_some() || query.h.is_some() {
        Some(parse_image_transform(&query)?)
    } else {
        None
    };
    let root = tokio::fs::canonicalize(&state.root)
        .await
        .map_err(|error| AppError::internal(error, "image root"))?;
    let file = tokio::fs::canonicalize(root.join(path))
        .await
        .map_err(|_| AppError::not_found("图片不存在"))?;
    if !file.starts_with(&root) {
        return Err(AppError::not_found("图片不存在"));
    }
    let metadata = tokio::fs::metadata(&file)
        .await
        .map_err(|_| AppError::not_found("图片不存在"))?;
    if !metadata.is_file() {
        return Err(AppError::not_found("图片不存在"));
    }
    super::public_local::mime_from_path(&file)?;
    let Some(transform) = transform else {
        // 保留原图 HEAD、Range、Last-Modified 和条件请求的既有语义。
        return Ok(ServeDir::new(root)
            .oneshot(request)
            .await
            .unwrap()
            .into_response());
    };
    if metadata.len() > state.max_source_size {
        return Ok(image_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "IMAGE_SOURCE_TOO_LARGE",
            "图片源文件过大",
        ));
    }
    let permit = match state.permits.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            let mut response = image_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "IMAGE_TRANSFORM_BUSY",
                "图片转换繁忙，请稍后重试",
            );
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
            return Ok(response);
        }
    };
    let bytes = tokio::fs::read(&file)
        .await
        .map_err(|error| AppError::internal(error, "read image"))?;
    if bytes.len() as u64 > state.max_source_size {
        return Ok(image_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "IMAGE_SOURCE_TOO_LARGE",
            "图片源文件过大",
        ));
    }
    let runtime = tokio::runtime::Handle::current();
    let output = tokio::task::spawn_blocking(move || {
        // 槽位跟随实际工作线程，客户端取消请求也不会提前释放并发预算。
        let _permit = permit;
        runtime.block_on(state.images.transform_image(&bytes, transform))
    })
    .await
    .map_err(|error| AppError::internal(error, "image transform worker"))?
    .map_err(|error| match error {
        StorageError::Image(message) => {
            AppError::bad_request_code("IMAGE_TRANSFORM_INVALID", message)
        }
        other => map_storage_err(other),
    })?;
    let etag = format!("\"{:x}\"", Sha256::digest(&output.bytes));
    let not_modified = request
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value.split(',').any(|candidate| {
                let candidate = candidate.trim();
                candidate == "*" || candidate.strip_prefix("W/").unwrap_or(candidate) == etag
            })
        });
    let mut response = if not_modified {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        let length = output.bytes.len();
        let mut response = output.bytes.into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(output.mime_type),
        );
        response
            .headers_mut()
            .insert(header::CONTENT_LENGTH, HeaderValue::from(length));
        response
    };
    response
        .headers_mut()
        .insert(header::ETAG, HeaderValue::from_str(&etag).unwrap());
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    Ok(response)
}

// CoreError 目前不保留 413/503；在适配器内明确写入状态，不改变全局错误协议。
fn image_error_response(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "code": code, "message": message, "status": status.as_u16(),
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use fx_storage_image::ImageExtractor;

    #[tokio::test]
    async fn busy_conversion_returns_retry_after_but_original_is_available() {
        let directory = tempfile::TempDir::new().unwrap();
        std::fs::write(directory.path().join("a.png"), b"fixture").unwrap();
        let state = PublicImageState::new(directory.path(), Arc::new(ImageExtractor::new(400, 80)))
            .with_concurrency(1);
        let _occupied = state.permits.clone().acquire_owned().await.unwrap();
        let router = public_image_routes(state);
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/a.png?w=20")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers()[header::RETRY_AFTER], "1");
        let original = router
            .oneshot(
                Request::builder()
                    .uri("/a.png")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(original.status(), StatusCode::OK);
    }
}
