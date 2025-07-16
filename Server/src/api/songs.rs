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
use crate::db::thirdparty::fetch_and_cache_song_image;

#[derive(Serialize, Deserialize)]
pub struct Index {
    pub index_start: usize,
    pub index_end: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub name: String,
    pub artist_name: String
}

#[derive(Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<Index>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    // Clamp indices to DB length
    let total = action::song::get_count(&state.db).await.unwrap_or_else(|_| 0);


    let start = params.index_start.clamp(0, total);
    let end = params.index_end.clamp(start, total);
    if end < start {
        return Err(ApiError::BadRequest("index_end must be >= index_start".to_string()));
    }

    let songs_data = match action::song::get_info(&state.db, start, end).await {
        Ok(songs_data) => songs_data,
        Err(e) => {
            error!("No songs found in database: {:?}",e);
            return Err(ApiError::NotFound("No songs found".to_string()));
        }
    };
    let mut songs: Vec<Data> = Vec::new();
    for song in songs_data {
        songs.push(Data {
            name:song.song_name,
            artist_name:song.artist_name,
        });
    }
    
    Ok(Json::from(ApiResponse {
        success: true,
        message: "songs".to_string(),
        data: Some(json!(songs)),
    }))
}

pub async fn get_image(
    State(state): State<Arc<AppState>>,
    Query(params): Query<Data>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Response<Body>, ApiError> {
    if !action::is_valid_user(&state.db, auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    match fetch_and_cache_song_image(&params.artist_name, &params.name).await {
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

pub async fn get_total(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    let song_total = match action::song::get_count(&state.db).await {
        Ok(songs_data) => songs_data,
        Err(_) => {
            return Ok(Json::from(ApiResponse {
                success: false,
                message: "no songs register with this server".to_string(),
                data: None
            }));
        }
    };
    
    Ok(Json::from(ApiResponse {
        success: true,
        message: "Got Total".to_string(),
        data: Some(json!({
            "total": song_total,
        })),
    }))
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db, auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }
    let songs_data = match action::fuzzy_search(&state.db, &params.query).await {
        Ok(songs_data) => songs_data,
        Err(e) => {
            error!("Fuzzy search failed: {:?}", e);
            return Err(ApiError::InternalServerError("Fuzzy search failed".to_string()));
        }
    };
    let mut songs: Vec<Data> = Vec::new();
    for song in songs_data {
        songs.push(Data {
            name: song.song_name,
            artist_name: song.artist_name,
        });
    }
    Ok(Json::from(ApiResponse {
        success: true,
        message: "fuzzy search results".to_string(),
        data: Some(json!(songs)),
    }))
}