use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::Value;
use crate::api::io_util::ApiError::{Unauthorized, BadRequest, InternalServerError, NotFound};
use crate::api::login::ErrorResponse;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize = Value> {
    pub(crate) success: bool,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<T>,
}

pub enum ApiError {
    Unauthorized,
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Unauthorized => (StatusCode::UNAUTHORIZED, "Not authenticated".to_string()),
            BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let response = ApiResponse::<()> {
            success: false,
            message,
            data: None,
        };

        (status, Json(response)).into_response()
    }
}

pub fn check_header_is_json(headers: &HeaderMap) -> Result<(),Json<ErrorResponse>> {
    let content_type = headers.get("content-type")
        .and_then(|v| v.to_str().ok()).unwrap_or("").trim();
    if !content_type.contains("application/json") {
        return Err(Json(ErrorResponse {
            success: false,
            message: "Invalid Content-Type. Expected application/json".to_string(),
            errors: None,
        }));
    }
    Ok(())
}