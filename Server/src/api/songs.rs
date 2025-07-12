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

pub async fn get(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<Index>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !action::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    //TODO: this is memory intensive lets do what we do for playlists 
    let songs_data = match action::song::get_info(&state.db, true).await {
        Ok(songs_data) => songs_data,
        Err(e) => {
            error!("No songs found in database: {:?}",e);
            return Err(ApiError::NotFound("No songs found".to_string()));
        }
    };
    let start: usize = params.index_start.clamp(0,songs_data.len());
    let end: usize = params.index_end.clamp(start,songs_data.len());
    let mut songs: Vec<Data> = Vec::new();
    for song in songs_data[start..end].to_vec() {
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