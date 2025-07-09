pub(crate) mod io_util; 
pub mod login;
mod songs;

use crate::api::io_util::{ApiError, ApiResponse};
use crate::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;
use serde_json::{json, Value};

async fn api_version(State(_state): State<Arc<AppState>>) -> Result<Json<ApiResponse<Value>>, ApiError> {
    Ok(Json::from(ApiResponse {
        success: true,
        message: "API VERSION".to_string(),
        data: Some(json!({
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/muse_server_version",get(api_version))
        .route("/api/register", post(login::register))
        .route("/api/login",post(login::login))
        .route("/api/songs",get(songs::get_songs))
}