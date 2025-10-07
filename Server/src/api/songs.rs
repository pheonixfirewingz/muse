use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::api::response::{ApiError, ApiResponse, ApiResult};
use crate::api::auth::AppState;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub index_start: usize,
    pub index_end: usize,
}

#[derive(Debug, Deserialize)]
pub struct SongInfoQuery {
    pub artist_name: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct SongBasic {
    pub name: String,
    pub artist_name: String,
}

#[derive(Debug, Serialize)]
pub struct SongInfo {
    pub name: String,
    pub artist_name: String,
    pub album: String,
    pub duration: u32,
    pub bitrate: u32,
    pub genre: String,
}

#[derive(Debug, Serialize)]
pub struct TotalCount {
    pub total: usize,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/songs?index_start=X&index_end=Y
/// Get paginated list of songs
pub async fn get_songs(
    State(state): State<AppState>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult<Vec<SongBasic>> {
    // Calculate offset and limit from the pagination query
    let offset = params.index_start;
    let limit = params.index_end.saturating_sub(params.index_start);
    
    // Query database for songs in the specified range
    let songs = state.db.get_songs(offset, limit).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to fetch songs: {}", e)))?;
    
    // Convert database Song models to SongBasic response type
    let song_basics: Vec<SongBasic> = songs.into_iter()
        .map(|song| SongBasic {
            name: song.title,
            artist_name: song.artist_name,
        })
        .collect();

    Ok(Json(ApiResponse::success("songs", song_basics)))
}

/// GET /api/songs/info?artist_name=X&name=Y
/// Get detailed information about a specific song
pub async fn get_song_info(
    State(state): State<AppState>,
    Query(params): Query<SongInfoQuery>,
) -> ApiResult<SongInfo> {
    // Search for the song by artist name and title
    let songs = state.db.search_songs(&params.name, 0, 100).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to search songs: {}", e)))?;
    
    // Find the song matching both artist name and title
    let song = songs.into_iter()
        .find(|s| s.artist_name.eq_ignore_ascii_case(&params.artist_name) && 
                   s.title.eq_ignore_ascii_case(&params.name))
        .ok_or_else(|| ApiError::not_found("Song not found"))?;
    
    let song_info = SongInfo {
        name: song.title,
        artist_name: song.artist_name,
        album: song.album.unwrap_or_else(|| "Unknown Album".to_string()),
        duration: song.duration.unwrap_or(0) as u32,
        bitrate: 0, // Bitrate info not stored in DB, would need file analysis
        genre: "Unknown".to_string(), // Genre not in current DB schema
    };

    Ok(Json(ApiResponse::success("Song info", song_info)))
}

/// GET /api/songs/total
/// Get total count of songs in the library
pub async fn get_total_songs(
    State(state): State<AppState>,
) -> ApiResult<TotalCount> {
    // Query database for total song count
    let total_count = state.db.get_total_songs().await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get total songs: {}", e)))?;
    
    let total = TotalCount { total: total_count };

    Ok(Json(ApiResponse::success("Got Total", total)))
}

/// GET /api/songs/cover?artist_name=X&name=Y
/// Get cover image for a specific song
pub async fn get_song_cover(
    State(state): State<AppState>,
    Query(params): Query<SongInfoQuery>,
) -> Result<Response, ApiError> {
    // Search for the song by artist name and title
    let songs = state.db.search_songs(&params.name, 0, 100).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to search songs: {}", e)))?;
    
    // Find the song matching both artist name and title
    let song = songs.into_iter()
        .find(|s| s.artist_name.eq_ignore_ascii_case(&params.artist_name) && 
                   s.title.eq_ignore_ascii_case(&params.name))
        .ok_or_else(|| ApiError::not_found("Song not found"))?;
    
    // Check if song has cover image
    let cover_path = song.cover_image_path
        .ok_or_else(|| ApiError::not_found("Cover image not found"))?;
    
    // Read the cover image file
    let image_bytes = tokio::fs::read(&cover_path).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to read cover image: {}", e)))?;
    
    // Return the image with appropriate content type
    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "image/avif")],
        image_bytes,
    ).into_response())
}

/// GET /api/songs/search?query=X
/// Fuzzy search for songs by query string
pub async fn search_songs(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> ApiResult<Vec<SongBasic>> {
    // Perform fuzzy search on song titles
    let songs = state.db.search_songs(&params.query, 0, 50).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to search songs: {}", e)))?;
    
    // Convert to SongBasic response type
    let results: Vec<SongBasic> = songs.into_iter()
        .map(|song| SongBasic {
            name: song.title,
            artist_name: song.artist_name,
        })
        .collect();

    Ok(Json(ApiResponse::success("fuzzy search results", results)))
}
