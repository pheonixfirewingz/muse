use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use crate::api::response::ApiError;
use crate::api::auth::AppState;

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    pub artist: String,
    pub name: String,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "mp3".to_string()
}

pub async fn stream_song(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<StreamQuery>,
) -> Result<Response, ApiError> {
    // Search for the song by artist name and title
    let songs = state.db.search_songs(&params.name, 0, 100).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to search songs: {}", e)))?;
    
    // Find the song matching both artist name and title
    let song = songs.into_iter()
        .find(|s| s.artist_name.eq_ignore_ascii_case(&params.artist) && 
                   s.title.eq_ignore_ascii_case(&params.name))
        .ok_or_else(|| ApiError::not_found("Song not found"))?;
    
    // Get file metadata
    let file_path = &song.file_path;
    let metadata = tokio::fs::metadata(file_path).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to read file metadata: {}", e)))?;
    
    let file_size = metadata.len();
    
    // Determine content type based on format
    let content_type = match params.format.to_lowercase().as_str() {
        "mp3" => "audio/mpeg",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        _ => "application/octet-stream",
    };
    
    // Check for Range header to support partial content requests
    if let Some(range_header) = headers.get(header::RANGE) {
        if let Ok(range_str) = range_header.to_str() {
            // Parse range header (e.g., "bytes=0-1023")
            if let Some(range) = parse_range_header(range_str, file_size) {
                let (start, end) = range;
                let content_length = end - start + 1;
                
                // Open file and seek to start position
                let file = File::open(file_path).await
                    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to open file: {}", e)))?;
                
                let mut file = tokio::io::BufReader::new(file);
                tokio::io::AsyncSeekExt::seek(&mut file, std::io::SeekFrom::Start(start)).await
                    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to seek file: {}", e)))?;
                
                // Create a stream that reads only the requested range
                let limited_stream = tokio::io::AsyncReadExt::take(file, content_length);
                let stream = ReaderStream::new(limited_stream);
                
                return Ok((
                    StatusCode::PARTIAL_CONTENT,
                    [
                        (header::CONTENT_TYPE, content_type),
                        (header::CONTENT_LENGTH, &content_length.to_string()),
                        (header::CONTENT_RANGE, &format!("bytes {}-{}/{}", start, end, file_size)),
                        (header::ACCEPT_RANGES, "bytes"),
                    ],
                    Body::from_stream(stream),
                ).into_response());
            }
        }
    }
    
    // No range request or invalid range - stream entire file
    let file = File::open(file_path).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to open file: {}", e)))?;
    
    let stream = ReaderStream::new(file);
    
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_LENGTH, &file_size.to_string()),
            (header::ACCEPT_RANGES, "bytes"),
        ],
        Body::from_stream(stream),
    ).into_response())
}

/// Parse HTTP Range header
/// Returns (start, end) byte positions, or None if invalid
fn parse_range_header(range: &str, file_size: u64) -> Option<(u64, u64)> {
    // Expected format: "bytes=start-end" or "bytes=start-"
    let range = range.strip_prefix("bytes=")?;
    
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let start = parts[0].parse::<u64>().ok()?;
    let end = if parts[1].is_empty() {
        file_size - 1
    } else {
        parts[1].parse::<u64>().ok()?.min(file_size - 1)
    };
    
    if start > end || start >= file_size {
        return None;
    }
    
    Some((start, end))
}
