//! 公开本机图片访问与动态变体路由。
//!
//! 只读取宿主明确配置的根目录，拒绝路径穿越与符号链接越界。无尺寸参数返回原图，
//! 携带 `w` 或 `h` 时调用图片扩展生成临时变体，不写回原文件。

use std::{path::PathBuf, sync::Arc};

use axum::{
    Router,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use fx_server_web::{AppError, AppResult};
use fx_storage_service::MetadataExtractor;

use crate::{ContentQuery, map_storage_err, parse_image_transform};

#[derive(Clone)]
pub struct PublicLocalImageState {
    pub root: PathBuf,
    pub images: Arc<dyn MetadataExtractor>,
    pub max_source_size: u64,
}

pub fn public_local_image_routes(state: PublicLocalImageState) -> Router {
    Router::new()
        .route("/original/{*path}", get(get_public_image))
        .with_state(state)
}

async fn get_public_image(
    State(state): State<PublicLocalImageState>,
    Path(path): Path<String>,
    Query(query): Query<ContentQuery>,
) -> AppResult<Response> {
    let root = tokio::fs::canonicalize(&state.root)
        .await
        .map_err(|error| AppError::internal(error, "public image root"))?;
    let requested = root.join("original").join(&path);
    let file = tokio::fs::canonicalize(&requested)
        .await
        .map_err(|_| AppError::not_found("图片不存在"))?;
    if !file.starts_with(&root) {
        return Err(AppError::not_found("图片不存在"));
    }
    let metadata = tokio::fs::metadata(&file)
        .await
        .map_err(|_| AppError::not_found("图片不存在"))?;
    if !metadata.is_file() || metadata.len() > state.max_source_size {
        return Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "IMAGE_SOURCE_TOO_LARGE",
            "图片源文件过大",
        ));
    }
    let bytes = tokio::fs::read(&file)
        .await
        .map_err(|_| AppError::not_found("图片不存在"))?;

    let (bytes, mime_type) = if query.w.is_some() || query.h.is_some() {
        let transformed = state
            .images
            .transform_image(&bytes, parse_image_transform(&query)?)
            .await
            .map_err(map_storage_err)?;
        (transformed.bytes, transformed.mime_type)
    } else {
        (bytes, mime_from_path(&file)?)
    };
    let mut response = bytes.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime_type)
            .map_err(|error| AppError::internal(error, "public image mime"))?,
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    Ok(response)
}

fn mime_from_path(path: &std::path::Path) -> AppResult<&'static str> {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "png" => Ok("image/png"),
        "gif" => Ok("image/gif"),
        "webp" => Ok("image/webp"),
        _ => Err(AppError::bad_request("仅支持公开访问图片文件")),
    }
}

#[cfg(test)]
mod tests {
    use super::mime_from_path;
    use std::path::Path;

    #[test]
    fn public_image_mime_accepts_supported_extensions() {
        assert_eq!(mime_from_path(Path::new("a.jpeg")).unwrap(), "image/jpeg");
        assert_eq!(mime_from_path(Path::new("a.webp")).unwrap(), "image/webp");
    }

    #[test]
    fn public_image_mime_rejects_non_images() {
        assert!(mime_from_path(Path::new("secret.env")).is_err());
    }
}
