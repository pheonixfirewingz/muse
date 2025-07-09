use std::sync::Arc;
use axum::extract::{Query, State};
use axum::Json;
use axum_extra::headers::{Authorization};
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::error;
use crate::api::io_util::{ApiError, ApiResponse};
use crate::{db, AppState};

#[derive(Serialize, Deserialize)]
pub struct SongIndex {
    index_start: usize,
    index_end: usize,
}

#[derive(Serialize)]
struct SongData {
    song_name: String,
    artist_name: String
}

pub async fn get_songs(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<SongIndex>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !db::actions::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    let mut songs_data = match db::actions::get_db_song_info(&state.db, true).await {
        Ok(songs_data) => songs_data,
        Err(_) => {
            error!("No songs found in database");
            return Err(ApiError::NotFound("No songs found".to_string()));
        }
    };
    let start: usize = params.index_start;
    let end: usize = params.index_end;
    if start <= end && end <= songs_data.len() {
        songs_data = songs_data[start..end].to_vec();
    } else {
        return Err(ApiError::BadRequest("bad start and end parameters".to_string()));
    }

    let mut songs: Vec<SongData> = Vec::new();
    for song in songs_data {
        songs.push(SongData {
            song_name:song.get_song_name().to_string(),
            artist_name:song.get_artist_name().to_string(),
        });
    }
    
    Ok(Json::from(ApiResponse {
        success: true,
        message: "songs".to_string(),
        data: Some(json!(songs)),
    }))
}