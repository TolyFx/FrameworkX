use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, StatusCode, header::ACCESS_CONTROL_ALLOW_ORIGIN},
    response::{IntoResponse, Response},
    routing::get,
};
use fx_core::{AppError as CoreError, ErrorKind};
use serde::Serialize;
use std::{net::UdpSocket, path::PathBuf};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::EnvFilter;

pub use axum;
pub use fx_core;
pub use serde;
pub use serde_json;
pub use sqlx;

pub type AppResult<T> = Result<T, AppError>;
pub type ApiResult<T> = AppResult<Json<ApiResponse<T>>>;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: &'static str,
    pub message: String,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self::with_message(data, "请求成功")
    }

    pub fn with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            code: "SUCCESS",
            message: message.into(),
            data,
        }
    }
}

pub fn api_ok<T>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse::success(data))
}

pub fn api_message<T>(data: T, message: impl Into<String>) -> Json<ApiResponse<T>> {
    Json(ApiResponse::with_message(data, message))
}

pub fn api_empty(message: impl Into<String>) -> Json<ApiResponse<()>> {
    api_message((), message)
}

#[derive(Debug)]
pub struct AppError(pub CoreError);

impl AppError {
    pub fn new(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self(CoreError::new(kind_from_status(status), code, message))
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self(CoreError::bad_request(message))
    }
    pub fn bad_request_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self(CoreError::bad_request_code(code, message))
    }
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self(CoreError::unauthorized(message))
    }
    pub fn unauthorized_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self(CoreError::unauthorized_code(code, message))
    }
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self(CoreError::forbidden(message))
    }
    pub fn forbidden_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self(CoreError::forbidden_code(code, message))
    }
    pub fn not_found(message: impl Into<String>) -> Self {
        Self(CoreError::not_found(message))
    }
    pub fn not_found_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self(CoreError::not_found_code(code, message))
    }
    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self(CoreError::conflict(code, message))
    }
    pub fn unprocessable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self(CoreError::unprocessable(code, message))
    }
    pub fn too_many_requests(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self(CoreError::too_many_requests(code, message))
    }
    pub fn internal(error: impl std::fmt::Display, context: &str) -> Self {
        Self(CoreError::internal(error, context))
    }
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.0 = self.0.with_detail(detail);
        self
    }
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.0 = self.0.with_data(data);
        self
    }
}

impl From<CoreError> for AppError {
    fn from(error: CoreError) -> Self {
        Self(error)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::internal(error, "database")
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
    status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = status_from_kind(self.0.kind);
        if let Some(detail) = &self.0.detail {
            eprintln!("[fx_server_web] {detail}");
        }
        let expose_detail = std::env::var("FX_ERROR_DETAIL")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let body = ErrorResponse {
            code: self.0.code,
            message: self.0.message,
            status: status.as_u16(),
            detail: expose_detail.then_some(self.0.detail).flatten(),
            data: self.0.data,
        };
        (status, Json(body)).into_response()
    }
}

fn status_from_kind(kind: ErrorKind) -> StatusCode {
    match kind {
        ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
        ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
        ErrorKind::Forbidden => StatusCode::FORBIDDEN,
        ErrorKind::NotFound => StatusCode::NOT_FOUND,
        ErrorKind::Conflict => StatusCode::CONFLICT,
        ErrorKind::Unprocessable => StatusCode::UNPROCESSABLE_ENTITY,
        ErrorKind::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
        ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn kind_from_status(status: StatusCode) -> ErrorKind {
    match status {
        StatusCode::UNAUTHORIZED => ErrorKind::Unauthorized,
        StatusCode::FORBIDDEN => ErrorKind::Forbidden,
        StatusCode::NOT_FOUND => ErrorKind::NotFound,
        StatusCode::CONFLICT => ErrorKind::Conflict,
        StatusCode::UNPROCESSABLE_ENTITY => ErrorKind::Unprocessable,
        StatusCode::TOO_MANY_REQUESTS => ErrorKind::TooManyRequests,
        value if value.is_server_error() => ErrorKind::Internal,
        _ => ErrorKind::BadRequest,
    }
}

#[derive(Debug, Clone)]
pub struct FxServerConfig {
    pub host: String,
    pub port: u16,
    pub database_url: Option<String>,
    pub version: String,
}

impl FxServerConfig {
    pub fn from_env() -> Self {
        load_server_env();
        let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "9600".into())
            .parse()
            .expect("SERVER_PORT must be a valid port number");
        Self {
            host,
            port,
            database_url: std::env::var("DATABASE_URL").ok(),
            version: "0.1.0".into(),
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
}

#[derive(Debug, Clone)]
pub struct FxServer {
    config: FxServerConfig,
}

impl FxServer {
    pub fn new(config: FxServerConfig) -> Self {
        Self { config }
    }

    pub async fn run(self, app: Router) {
        init_tracing();
        let port = self.config.port;
        let app = app
            .merge(version_routes(self.config.version.clone()))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(
                        DefaultMakeSpan::new()
                            .level(Level::INFO)
                            .include_headers(false),
                    )
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            );
        println!("🚀 Server listening on:");
        println!("   Local:   http://127.0.0.1:{port}");
        println!("   Network: http://{}:{port}", get_local_ip());
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.config.host, port))
            .await
            .expect("bind server");
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve app");
    }

    pub async fn pg_pool(&self) -> Option<sqlx::PgPool> {
        let database_url = self.config.database_url.as_ref()?;
        Some(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(10)
                .connect(database_url)
                .await
                .expect("Failed to connect to database"),
        )
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info"));
    match tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init()
    {
        Ok(()) => tracing::info!("HTTP tracing 已启用"),
        Err(error) => eprintln!("[fx_server_web] tracing 初始化失败: {error}"),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VersionInfo {
    pub version: String,
}

#[derive(Debug, Clone)]
struct VersionState {
    version: String,
}

pub fn version_routes(version: impl Into<String>) -> Router {
    Router::new()
        .route("/v", get(version_handler))
        .with_state(VersionState {
            version: version.into(),
        })
}

async fn version_handler(State(state): State<VersionState>) -> impl IntoResponse {
    (
        [(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"))],
        api_ok(VersionInfo {
            version: state.version,
        }),
    )
}

pub fn load_server_env() {
    if let Some(path) = find_server_env_file() {
        let _ = dotenvy::from_path(path);
    }
}

pub fn find_server_env_file() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(".fx").join(".env").join(".server.env");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn get_local_ip() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("8.8.8.8:80")?;
            socket.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_error_maps_to_http_status() {
        let error = AppError::from(CoreError::conflict("DUPLICATE", "重复"));
        assert_eq!(status_from_kind(error.0.kind), StatusCode::CONFLICT);
    }
}
