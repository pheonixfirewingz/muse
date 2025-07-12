use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::Response;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::error;
use crate::api::io_util::{ApiError, ApiResponse};
use crate::{AppState};
use crate::api::io_util::ApiError::InternalServerError;
use crate::db::action;
use crate::db::thirdparty::fetch_and_cache_artist_image;

#[derive(Serialize,Deserialize)]
pub struct Index {
    index_start: usize,
    index_end: usize,
}

#[derive(Serialize,Deserialize)]
pub struct Data {
    name: String
}

#[derive(Serialize,Deserialize)]
pub struct ImageQuery {
    pub name: String,
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<Index>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    let artists_data = match action::artist::get_info(&state.db, true).await {
        Ok(artists_data) => artists_data,
        Err(_) => {
            error!("No artists found in database");
            return Err(ApiError::NotFound("No artists found".to_string()));
        }
    };
    let start: usize = params.index_start.clamp(0,artists_data.len() - 1);
    let end: usize = params.index_end.clamp(start,artists_data.len() - 1);
    let mut artists: Vec<Data> = Vec::new();
    for artist in artists_data[start..end].to_vec() {
        artists.push(Data {
            name:artist.artist_name
        });
    }

    Ok(Json::from(ApiResponse {
        success: true,
        message: "artists".to_string(),
        data: Some(json!(artists)),
    }))
}


pub async fn get_total(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    let total = match action::artist::get_count(&state.db).await {
        Ok(artists_data) => artists_data,
        Err(_) => {
            return Ok(Json::from(ApiResponse {
                success: false,
                message: "no artists register with this server".to_string(),
                data: None
            }));
        }
    };

    Ok(Json::from(ApiResponse {
        success: true,
        message: "Got Total".to_string(),
        data: Some(json!({
            "total": total,
        })),
    }))
}

pub async fn get_image(State(state): State<Arc<AppState>>,
                              Query(params): Query<ImageQuery>,
                              TypedHeader(auth): TypedHeader<Authorization<Bearer>>) -> Result<Response, ApiError> {
    if !action::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    match fetch_and_cache_artist_image(&params.name).await {
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

pub async fn get_songs(State(state): State<Arc<AppState>>,
                              TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
                              Query(params): Query<Data>
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }
    if params.name.is_empty() {
        return Err(ApiError::BadRequest("empty artist name".to_string()));
    }
    
    let songs_data = match action::song::get_info_by_artist(&state.db, &params.name, true).await {
        Ok(songs) => songs,
        Err(_) => {
            return Err(ApiError::BadRequest("could not find songs in storage".to_string()));
        }
    };
    let mut songs: Vec<String> = Vec::new();
    for song in songs_data {
        songs.push(song.song_name);
    }

    Ok(Json::from(ApiResponse {
        success: true,
        message: "Got Total".to_string(),
        data: Some(json!(songs)),
    }))
}