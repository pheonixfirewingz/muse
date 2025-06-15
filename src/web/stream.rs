use crate::{db, AppState};
use async_stream::stream;
use axum::{
    body::{Body, Bytes},
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::{ops::RangeInclusive, sync::Arc};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncSeekExt, BufReader, SeekFrom};

#[derive(Deserialize)]
pub struct SongQuery {
    song: String,
    artist: String,
}

// Helper function to parse "bytes=start-end" range header
fn parse_range(header_value: &str, total_size: u64) -> Option<RangeInclusive<u64>> {
    if !header_value.starts_with("bytes=") {
        return None;
    }
    let range_part = &header_value[6..];
    let parts: Vec<&str> = range_part.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    let start = parts[0].parse::<u64>().ok()?;
    let end = if parts[1].is_empty() {
        total_size - 1
    } else {
        parts[1].parse::<u64>().ok()?
    };
    if start > end || end >= total_size {
        return None;
    }
    Some(start..=end)
}

pub async fn handler(
    Query(query): Query<SongQuery>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let file_path = db::get_song_path_by_song_name_and_artist_name(&state.db, &query.song, &query.artist).await;
    if file_path.is_none() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("song not found"))
            .unwrap();
    }
    let file_path = file_path.unwrap();

    let file = match OpenOptions::new().read(true).open(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!("File not found: {}", e)))
                .unwrap();
        }
    };

    let metadata = match file.metadata().await {
        Ok(m) => m,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Could not get file metadata: {}", e)))
                .unwrap();
        }
    };
    let total_size = metadata.len();

    // Parse Range header if present
    let range_header = headers.get(header::RANGE).and_then(|v| v.to_str().ok());
    let (status, range) = if let Some(range_header) = range_header {
        match parse_range(range_header, total_size) {
            Some(r) => (StatusCode::PARTIAL_CONTENT, r),
            None => (StatusCode::RANGE_NOT_SATISFIABLE, 0..=0), // Invalid range
        }
    } else {
        (StatusCode::OK, 0..=total_size - 1)
    };

    if status == StatusCode::RANGE_NOT_SATISFIABLE {
        return Response::builder()
            .status(StatusCode::RANGE_NOT_SATISFIABLE)
            .header(header::CONTENT_RANGE, format!("bytes */{}", total_size))
            .body(Body::empty())
            .unwrap();
    }

    // Clone range for stream usage so original stays accessible
    let range_for_stream = range.clone();

    let stream = stream! {
        let mut file = BufReader::new(file);

        if let Err(e) = file.seek(SeekFrom::Start(*range_for_stream.start())).await {
            yield Err(std::io::Error::new(std::io::ErrorKind::Other, format!("seek error: {}", e)));
            return;
        }

        let mut left = (*range_for_stream.end() - *range_for_stream.start() + 1) as usize;
        let mut buffer = vec![0; 8192];

        while left > 0 {
            let read_len = buffer.len().min(left);
            match file.read(&mut buffer[..read_len]).await {
                Ok(0) => break,
                Ok(n) => {
                    left -= n;
                    yield Ok::<_, std::io::Error>(Bytes::copy_from_slice(&buffer[..n]));
                }
                Err(e) => {
                    yield Err(e);
                    break;
                }
            }
        }
    };

    let content_length = (*range.end() - *range.start() + 1).to_string();
    let content_range = format!("bytes {}-{}/{}", range.start(), range.end(), total_size);

    Response::builder()
        .status(status)
        .header(header::CACHE_CONTROL, "no-store, no-cache, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .header(header::CONTENT_LENGTH, content_length)
        .header(header::CONTENT_RANGE, content_range)
        .body(Body::from_stream(stream))
        .unwrap()
}
