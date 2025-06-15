use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;
use crate::db::schema::music_brainz::{get_artist_image, get_album_image};
use crate::AppState;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/artist/{name}", get(get_artist_image_handler))
        .route("/album/{artist}/{album}", get(get_album_image_handler))
        .layer(CompressionLayer::new())
}

async fn get_artist_image_handler(
    State(_state): State<Arc<AppState>>,
    axum::extract::Path(artist_name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match get_artist_image(&artist_name).await {
        Some(image) => Json(json!({
            "success": true,
            "url": image.url,
            "data": image.data
        })),
        None => Json(json!({
            "success": false,
            "error": "Image not found"
        }))
    }
}

async fn get_album_image_handler(
    State(_state): State<Arc<AppState>>,
    axum::extract::Path((artist_name, album_name)): axum::extract::Path<(String, String)>,
) -> Json<serde_json::Value> {
    match get_album_image(&artist_name, &album_name).await {
        Some(image) => Json(json!({
            "success": true,
            "url": image.url,
            "data": image.data
        })),
        None => Json(json!({
            "success": false,
            "error": "Image not found"
        }))
    }
} 