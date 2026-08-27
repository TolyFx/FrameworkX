use serde::Serialize;

/// FrameworkX 应用服务的统一结果类型。
pub type AppResult<T> = Result<T, AppError>;

/// 与具体传输协议无关的错误类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    Unprocessable,
    TooManyRequests,
    Internal,
}

/// 跨领域共享的应用错误，不依赖 Axum 或数据库驱动。
#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub kind: ErrorKind,
    pub code: String,
    pub message: String,
    pub detail: Option<String>,
    pub data: Option<serde_json::Value>,
}

impl AppError {
    pub fn new(kind: ErrorKind, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind,
            code: code.into(),
            message: message.into(),
            detail: None,
            data: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::BadRequest, "BAD_REQUEST", message)
    }

    pub fn bad_request_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::BadRequest, code, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Unauthorized, "UNAUTHORIZED", message)
    }

    pub fn unauthorized_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Unauthorized, code, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Forbidden, "FORBIDDEN", message)
    }

    pub fn forbidden_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Forbidden, code, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, "NOT_FOUND", message)
    }

    pub fn not_found_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, code, message)
    }

    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Conflict, code, message)
    }

    pub fn unprocessable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Unprocessable, code, message)
    }

    pub fn too_many_requests(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TooManyRequests, code, message)
    }

    pub fn internal(error: impl std::fmt::Display, context: &str) -> Self {
        Self::new(
            ErrorKind::Internal,
            "INTERNAL_SERVER_ERROR",
            "Internal Server Error",
        )
        .with_detail(format!("[{context}] {error}"))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}
