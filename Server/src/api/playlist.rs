use crate::api::io_util::{ApiError, ApiResponse};
use crate::db::action;
use crate::AppState;
use axum::extract::State;
use axum::Json;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use serde_json::{json, Value};
use std::sync::Arc;
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Index {
    pub index_start: usize,
    pub index_end: usize,
}

pub async fn get_private(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<Index>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db, auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }
    use uuid::Uuid;
    use crate::db::action;
    let session_id = Uuid::parse_str(auth.token()).map_err(|_| ApiError::Unauthorized)?;
    let playlists = action::playlist::get_private_info(&state.db, &session_id, params.index_start, params.index_end).await
        .map_err(|_| ApiError::InternalServerError("Failed to fetch private playlists".to_string()))?;
    let data: Vec<_> = playlists.into_iter().map(|pl| json!({
        "name": pl.name,
        "owner": "me",
        "isPublic": false
    })).collect();
    Ok(Json(ApiResponse {
        success: true,
        message: "private playlists".to_string(),
        data: Some(json!(data)),
    }))
}

pub async fn get_public(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<Index>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db, auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }
    let playlists = action::playlist::get_public_info(&state.db, params.index_start, params.index_end).await
        .map_err(|_| ApiError::InternalServerError("Failed to fetch public playlists".to_string()))?;
    let data: Vec<_> = playlists.into_iter().map(|pl| json!({
        "name": pl.name,
        "owner": pl.username,
        "isPublic": true
    })).collect();
    Ok(Json(ApiResponse {
        success: true,
        message: "public playlists".to_string(),
        data: Some(json!(data)),
    }))
}

pub async fn get_private_total(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db, auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }
    use uuid::Uuid;
    let session_id = Uuid::parse_str(auth.token()).map_err(|_| ApiError::Unauthorized)?;
    let total = action::playlist::get_private_count(&state.db, &session_id).await
        .map_err(|_| ApiError::InternalServerError("Failed to get playlist count".to_string()))?;
    Ok(Json(ApiResponse {
        success: true,
        message: "Got Total".to_string(),
        data: Some(json!({ "total": total })),
    }))
}

pub async fn get_public_total(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db, auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }
    let total = action::playlist::get_public_count(&state.db).await
        .map_err(|_| ApiError::InternalServerError("Failed to get public playlist count".to_string()))?;
    Ok(Json(ApiResponse {
        success: true,
        message: "Got Total".to_string(),
        data: Some(json!({ "total": total })),
    }))
} 