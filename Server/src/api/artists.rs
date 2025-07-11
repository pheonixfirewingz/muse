use std::sync::Arc;
use axum::extract::{Query, State};
use axum::Json;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::error;
use crate::api::io_util::{ApiError, ApiResponse};
use crate::{db, AppState};

#[derive(Serialize, Deserialize)]
pub struct ArtistIndex {
    index_start: usize,
    index_end: usize,
}

#[derive(Serialize)]
struct ArtistData {
    artist_name: String
}

pub async fn get_artists(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<ArtistIndex>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !db::actions::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    let mut artists_data = match db::actions::get_db_artist_info(&state.db, true).await {
        Ok(artists_data) => artists_data,
        Err(_) => {
            error!("No artists found in database");
            return Err(ApiError::NotFound("No artists found".to_string()));
        }
    };
    let start: usize = params.index_start.max(0);
    let end: usize = params.index_end.min(artists_data.len() - 1);
    if start <= end && end <= artists_data.len() {
        artists_data = artists_data[start..end].to_vec();
    } else {
        return Err(ApiError::BadRequest("bad start and end parameters".to_string()));
    }

    let mut artists: Vec<ArtistData> = Vec::new();
    for artist in artists_data {
        artists.push(ArtistData {
            artist_name:artist.get_name().to_string()
        });
    }

    Ok(Json::from(ApiResponse {
        success: true,
        message: "artists".to_string(),
        data: Some(json!(artists)),
    }))
}


pub async fn get_artist_total(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if !db::actions::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    let artist_total = match db::actions::get_db_artist_count(&state.db).await {
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
            "total": artist_total,
        })),
    }))
}