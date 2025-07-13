use crate::{db, AppState};
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use tower_cookies::Cookies;
use crate::util::cache::{load_cache, store_cache};
use crate::util::cache::img_ext::ImageFormat;
use crate::api::error::ApiError;
use crate::api::error::ApiError::InternalServerError;
use crate::api::util::get_session_id_from_cookies;
use axum_extra::extract::Multipart;
use axum::Json;

mod error;
mod stream;
mod playlist;
mod util;
mod images;
pub mod login;

#[derive(Debug, Serialize)]
struct ApiResponse<T: Serialize = Value> {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

async fn get_user_image(
    State(state): State<Arc<AppState>>,
    _headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, ApiError> {
    let session_id = get_session_id_from_cookies(&cookies)?;
    let user_id = match db::actions::get_user_id_from_session_id(&session_id, &state.db).await {
        Ok(id) => id,
        _ => return Err(InternalServerError("user not found".to_string())),
    };

    match load_cache::<Vec<u8>, ImageFormat>(&user_id.to_string(), "user/images").await {
        Ok(Some(entry)) => {
            let data = match entry.get_data() { 
                Some(data) => data,
                None => return Err(InternalServerError("Internal server error failed to get found image".to_string())),
            }.clone();
            
            let response = match Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "image/avif")
                .body(Body::from(data)) {
                Ok(response) => response,
                Err(_) => return Err(InternalServerError("Internal server error failed to return found image".to_string())),
            };

            Ok(response)
        }
        Ok(None) | Err(_) => {
            Ok(Redirect::permanent("/assets/images/place_holder.webp").into_response())
        }
    }
}

async fn update_user_image(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse>, ApiError> {
    let session_id = get_session_id_from_cookies(&cookies)?;
    let user_id = match db::actions::get_user_id_from_session_id(&session_id, &state.db).await {
        Ok(id) => id,
        _ => return Err(InternalServerError("user not found".to_string())),
    };

    // Find the image field in the multipart data
    let mut image_data: Option<Vec<u8>> = None;
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        InternalServerError(format!("Failed to read multipart field: {}", e))
    })? {
        let field_name = field.name().unwrap_or("").to_string();
        
        if field_name == "image" || field_name == "avatar" || field_name == "file" {
            let data = field.bytes().await.map_err(|e| {
                InternalServerError(format!("Failed to read image data: {}", e))
            })?;
            
            if !data.is_empty() {
                image_data = Some(data.to_vec());
                break;
            }
        }
    }

    let image_data = image_data.ok_or_else(|| {
        InternalServerError("No image data found in request".to_string())
    })?;

    // Validate image size (max 10MB)
    if image_data.len() > 10 * 1024 * 1024 {
        return Err(InternalServerError("Image too large. Maximum size is 10MB".to_string()));
    }

    // Create cache entry with the image data
    let cache_entry = crate::util::cache::CacheEntry::new_hit(image_data.clone(), image_data.len() as u64);
    
    // Store the image in cache
    match store_cache::<Vec<u8>, ImageFormat>(&cache_entry, &user_id.to_string(), "user/images").await {
        Ok(_) => {
            Ok(Json(ApiResponse {
                success: true,
                message: "User image updated successfully".to_string(),
                data: None,
            }))
        }
        Err(e) => {
            Err(InternalServerError(format!("Failed to store image: {}", e)))
        }
    }
}
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/muse_server_version",get(api_version))
        .route("/api/user/image", get(get_user_image).post(update_user_image))
        .route("/api/images/artist", get(images::get_artist_image))
        .route("/api/images/song", get(images::get_song_image))
        .route("/api/playlists", get(playlist::get_user_playlists).post(playlist::create_playlist))
        .route("/api/playlists/songs", post(playlist::add_to_playlist))
        .route("/api/playlists/create_and_add", post(playlist::create_playlist_and_add))
        .route("/api/playlists/delete", post(playlist::delete_playlist))
        .route("/api/playlists/public", get(playlist::get_public_playlists))
        .route("/api/playlists/reorder_songs", post(playlist::reorder_playlist_songs))
        .route("/api/stream", get(stream::stream_song))
        .route("/api/songs", get(get_all_songs))
        .route("/api/register", post(login::register))
        .route("/api/login",post(login::login))
}