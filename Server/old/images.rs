use axum::body::Body;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::Response;
use serde::Deserialize;
use tower_cookies::Cookies;
use crate::db::thirdparty::{fetch_and_cache_artist_image, fetch_and_cache_song_image};
use crate::api::error::ApiError;
use crate::api::error::ApiError::InternalServerError;

#[derive(Debug, Deserialize)]
pub struct ArtistImageQuery {
    artist_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SongImageQuery {
    artist_name: String,
    song_name: String,
}

pub async fn get_artist_image(
    Query(params): Query<ArtistImageQuery>,
    _cookies: Cookies,
) -> Result<Response, ApiError> {
    match fetch_and_cache_artist_image(&params.artist_name).await {
        Ok(Some(data)) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "image/avif")
                .body(Body::from(data))
                .map_err(|_| InternalServerError("Failed to return cached artist image".to_string()))?;
            Ok(response)
        }
        Ok(None) => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Artist image not found"))
                .unwrap();
            Ok(response)
        }
        Err(e) => {
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Artist image error: {}", e)))
                .unwrap();
            Ok(response)
        }
    }
}

pub async fn get_song_image(
    Query(params): Query<SongImageQuery>,
    _cookies: Cookies,
) -> Result<Response, ApiError> {
    match fetch_and_cache_song_image(&params.artist_name, &params.song_name).await {
        Ok(Some(data)) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "image/avif")
                .body(Body::from(data))
                .map_err(|_| InternalServerError("Failed to return cached song image".to_string()))?;
            Ok(response)
        }
        Ok(None) => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Song image not found"))
                .unwrap();
            Ok(response)
        }
        Err(e) => {
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Song image error: {}", e)))
                .unwrap();
            Ok(response)
        }
    }
}