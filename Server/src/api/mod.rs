pub mod io_util;
pub mod login;
mod songs;
mod artists;
mod playlist;
mod user;
mod stream;

use crate::AppState;
use axum::routing::{get, post};
use axum::Router;
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use std::path::Path;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/register", post(login::register))
        .route("/api/login",post(login::login))
        .route("/api/stream", get(stream::song))
        .route("/api/songs",get(songs::get))
        .route("/api/songs/total",get(songs::get_total))
        .route("/api/songs/cover",get(songs::get_image))
        .route("/api/songs/search", get(songs::search))
        .route("/api/artists",get(artists::get))
        .route("/api/artists/total",get(artists::get_total))
        .route("/api/artists/cover",get(artists::get_image))
        .route("/api/artists/songs",get(artists::get_songs))
        .route("/api/playlists/private", get(playlist::get_private))
        .route("/api/playlists/public", get(playlist::get_public))
        .route("/api/playlists/private/total", get(playlist::get_private_total))
        .route("/api/playlists/public/total", get(playlist::get_public_total))
        .route("/api/user", get(user::get_info).put(user::update_info))
        .route("/api/user/delete", post(user::delete_account))
}

async fn health_check() -> Json<serde_json::Value> {
    // Check if HTTPS certificates exist
    let cert_path = std::env::var("HTTPS_CERT_PATH").unwrap_or_else(|_| "certs/cert.pem".to_string());
    let key_path = std::env::var("HTTPS_KEY_PATH").unwrap_or_else(|_| "certs/key.pem".to_string());
    let https_enabled = Path::new(&cert_path).exists() && Path::new(&key_path).exists();

    Json(json!({
        "status": "OK",
        "protocols": {
            "http": true,
            "https": https_enabled
        },
        "server": "Muse Music Server",
        "version": "1.0.0"
    }))
}