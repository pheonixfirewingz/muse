use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use serde::Deserialize;
use crate::db::thirdparty::{fetch_and_cache_artist_image, fetch_and_cache_song_image};
use crate::api::io_util::ApiError;
use crate::api::io_util::ApiError::InternalServerError;
use crate::{db, AppState};

#[derive(Debug, Deserialize)]
pub struct ArtistImageQuery {
    artist_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SongImageQuery {
    artist_name: String,
    song_name: String,
}

pub async fn get_artist_image(State(state): State<Arc<AppState>>,
                              Query(params): Query<ArtistImageQuery>,
                              TypedHeader(auth): TypedHeader<Authorization<Bearer>>) -> Result<Response, ApiError> {
    if !db::actions::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

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
                .status(StatusCode::OK)
                .body(Body::empty())
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
    State(state): State<Arc<AppState>>,
    Query(params): Query<SongImageQuery>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Response<Body>, ApiError> {
    if !db::actions::is_valid_user(&state.db, auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

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
                .status(StatusCode::OK)
                .body(Body::empty())
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