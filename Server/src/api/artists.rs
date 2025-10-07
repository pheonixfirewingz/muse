use axum::{
    extract::{Json, Query, State},
    response::Response,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::api::auth::AppState;
use crate::api::response::{ApiError, ApiResponse, ApiResult};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub index_start: usize,
    pub index_end: usize,
}

#[derive(Debug, Deserialize)]
pub struct ArtistNameQuery {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ArtistBasic {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct TotalCount {
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct SongBasic {
    pub id: String,
    pub title: String,
    pub artist_name: String,
}

/// GET /api/artists?index_start=0&index_end=50
/// Get paginated list of artists
pub async fn get_artists(
    State(state): State<AppState>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult<Vec<ArtistBasic>> {
    // Calculate offset and limit from index_start and index_end
    let offset = params.index_start;
    let limit = params.index_end.saturating_sub(params.index_start);
    
    // Get artists from database
    let artists = state.db.get_artists(offset, limit).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e)))?;
    
    // Convert to response format
    let artist_list: Vec<ArtistBasic> = artists.into_iter().map(|artist| ArtistBasic {
        id: artist.id,
        name: artist.name,
    }).collect();
    
    Ok(Json(ApiResponse::success("artists", artist_list)))
}

/// GET /api/artists/total
/// Get total number of artists
pub async fn get_total_artists(
    State(state): State<AppState>,
) -> ApiResult<TotalCount> {
    let total = state.db.get_total_artists().await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e)))?;
    
    Ok(Json(ApiResponse::success("Got Total", TotalCount { total })))
}

/// GET /api/artists/cover?name=ArtistName
/// Get artist cover image
pub async fn get_artist_cover(
    State(state): State<AppState>,
    Query(params): Query<ArtistNameQuery>,
) -> Result<Response, ApiError> {
    // Get artist from database
    let artist = state.db.get_artist_by_name(&params.name).await
        .map_err(|_| ApiError::not_found("Artist not found"))?;
    
    // Check if artist has a cover image
    let cover_path = artist.cover_image_path
        .ok_or_else(|| ApiError::not_found("Artist cover image not found"))?;
    
    // Read the image file
    let image_data = fs::read(&cover_path).await
        .map_err(|_| ApiError::not_found("Artist cover image file not found"))?;
    
    // Determine content type based on file extension
    let content_type = if cover_path.ends_with(".png") {
        "image/png"
    } else if cover_path.ends_with(".jpg") || cover_path.ends_with(".jpeg") {
        "image/jpeg"
    } else if cover_path.ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    };
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .body(image_data.into())
        .unwrap())
}

/// GET /api/artists/songs?name=ArtistName
/// Get all songs by a specific artist
pub async fn get_artist_songs(
    State(state): State<AppState>,
    Query(params): Query<ArtistNameQuery>,
) -> ApiResult<Vec<SongBasic>> {
    // Get artist from database
    let artist = state.db.get_artist_by_name(&params.name).await
        .map_err(|_| ApiError::not_found("Artist not found"))?;
    
    // Get songs by this artist
    let songs = state.db.get_songs_by_artist(&artist.id).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e)))?;
    
    // Convert to response format
    let song_list: Vec<SongBasic> = songs.into_iter().map(|song| SongBasic {
        id: song.id,
        title: song.title,
        artist_name: song.artist_name,
    }).collect();
    
    Ok(Json(ApiResponse::success("artist songs", song_list)))
}
