use std::sync::Arc;
use axum::extract::{State};
use axum::Json;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use serde_json::json;
use uuid::Uuid;
use crate::AppState;
use crate::api::io_util::{ApiError, ApiResponse};
use crate::db::action::user::{get_user_info_from_session_id, update_user_info_from_session_id, UpdateUserInfo};
use crate::db::action;
use axum::extract::Json as AxumJson;
use serde::Deserialize;
use crate::db::action::user;

#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    password: String,
}

pub async fn get_info(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ApiResponse>, ApiError> {
    if !action::is_valid_user(&state.db, auth.token()).await.unwrap_or(false) {
        return Err(ApiError::Unauthorized);
    }
    let session_id = Uuid::parse_str(auth.token()).map_err(|_| ApiError::Unauthorized)?;
    match get_user_info_from_session_id(&state.db, &session_id).await {
        Some(user_info) => Ok(Json(ApiResponse {
            success: true,
            message: "User info fetched".to_string(),
            data: Some(json!(user_info)),
        })),
        None => Err(ApiError::Unauthorized),
    }
}

pub async fn update_info(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(update): Json<UpdateUserInfo>,
) -> Result<Json<ApiResponse>, ApiError> {
    if !action::is_valid_user(&state.db, auth.token()).await.unwrap_or(false) {
        return Err(ApiError::Unauthorized);
    }
    let session_id = Uuid::parse_str(auth.token()).map_err(|_| ApiError::Unauthorized)?;
    match update_user_info_from_session_id(&state.db, &session_id, update).await {
        Ok(()) => Ok(Json(ApiResponse {
            success: true,
            message: "User info updated".to_string(),
            data: None,
        })),
        Err(e) => Err(ApiError::BadRequest(e)),
    }
}

pub async fn delete_account(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    AxumJson(req): AxumJson<DeleteAccountRequest>,
) -> Result<Json<ApiResponse>, ApiError> {
    if !action::is_valid_user(&state.db, auth.token()).await.unwrap_or(false) {
        return Err(ApiError::Unauthorized);
    }
    match user::delete_user_with_password(&state.db, auth.token(), &req.password).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            message: "Account deleted successfully".to_string(),
            data: None,
        })),
        Err(e) => {
            if e == "Incorrect password" {
                Err(ApiError::BadRequest(e))
            } else if e == "Invalid session" || e == "Invalid session token" || e == "User not found" {
                Err(ApiError::Unauthorized)
            } else {
                Err(ApiError::InternalServerError(e))
            }
        }
    }
} 