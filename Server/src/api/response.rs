use serde::Serialize;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::collections::HashMap;
use time::OffsetDateTime;

/// Standard API success response envelope
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(with = "time::serde::iso8601")]
    pub timestamp: OffsetDateTime,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(message: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            timestamp: OffsetDateTime::now_utc(),
        }
    }

    #[allow(dead_code)]
    pub fn success_no_data(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

impl ApiResponse<()> {
    pub fn no_data(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

/// Standard API error response envelope
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub success: bool,
    pub message: String,
    pub code: u16,
    #[serde(with = "time::serde::iso8601")]
    pub timestamp: OffsetDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, String>>,
}

impl ApiError {
    pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            code: code.as_u16(),
            timestamp: OffsetDateTime::now_utc(),
            errors: None,
        }
    }

    pub fn with_errors(
        code: StatusCode,
        message: impl Into<String>,
        errors: HashMap<String, String>,
    ) -> Self {
        Self {
            success: false,
            message: message.into(),
            code: code.as_u16(),
            timestamp: OffsetDateTime::now_utc(),
            errors: Some(errors),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    #[allow(dead_code)]
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    #[allow(dead_code)]
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

/// Helper type for responses
pub type ApiResult<T> = Result<Json<ApiResponse<T>>, ApiError>;
pub type ApiResultNoData = Result<Json<ApiResponse<()>>, ApiError>;
