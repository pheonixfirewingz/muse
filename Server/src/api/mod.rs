pub(crate) mod io_util; 
pub mod login;
mod songs;
mod images;
mod artists;

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
        .route("/api/songs/total",get(songs::get_song_total))
        .route("/api/songs/cover",get(images::get_song_image))
        .route("/api/artists",get(artists::get_artists))
        .route("/api/artists/total",get(artists::get_artist_total))
        .route("/api/artists/cover",get(images::get_artist_image))
}