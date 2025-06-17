use futures::stream;
use tokio::io::{AsyncReadExt, AsyncSeekExt, BufReader, SeekFrom};
use axum::{
    body::{Body, Bytes},
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::{ops::RangeInclusive, sync::Arc};
use tokio::fs::OpenOptions;
use crate::{db, AppState};
use serde::Deserialize;
use tracing::error;

#[derive(Deserialize)]
pub struct SongQuery {
    pub song: String,
    pub artist: String,
}

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
//TODO: support other formats we will need to know what the client supports
pub async fn handler(
    Query(query): Query<SongQuery>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let file_path: String = match db::actions::get_song_file_path(&state.db,&query.song, &query.artist).await {
        Ok(file_path) => file_path,
        Err(e) => {
            error!("could not find song in database: {}",e);
            return Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("Song Dose not exist")).unwrap();
        }
    };

    let file = match OpenOptions::new().read(true).open(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("File open error: {}", e);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Song not available"))
                .unwrap();
        }
    };

    let metadata = match file.metadata().await {
        Ok(m) => m,
        Err(e) => {
            error!("Metadata error: {}", e);
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal server error"))
                .unwrap();
        }
    };

    let total_size = metadata.len();

    let range_header = headers.get(header::RANGE).and_then(|v| v.to_str().ok());
    let (status, range) = if let Some(range_header) = range_header {
        match parse_range(range_header, total_size) {
            Some(r) => (StatusCode::PARTIAL_CONTENT, r),
            None => (StatusCode::RANGE_NOT_SATISFIABLE, 0..=0),
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

    let start = *range.start();
    let end = *range.end();
    let content_length = end - start + 1;
    let content_range = format!("bytes {}-{}/{}", start, end, total_size);

    let mut reader = BufReader::new(file);
    if let Err(e) = reader.seek(SeekFrom::Start(start)).await {
        error!("Seek failed: {}", e);
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal server error"))
            .unwrap();
    }

    let stream = stream::unfold((reader, content_length), |(mut reader, mut remaining)| async move {
        if remaining == 0 {
            return None;
        }

        let mut buffer = [0u8; 8192];
        let read_len = buffer.len().min(remaining as usize);

        match reader.read(&mut buffer[..read_len]).await {
            Ok(0) => None,
            Ok(n) => {
                remaining -= n as u64;
                let chunk = Bytes::copy_from_slice(&buffer[..n]);
                Some((Ok(chunk), (reader, remaining)))
            }
            Err(e) => {
                error!("Read error: {}", e);
                Some((Err(std::io::Error::new(std::io::ErrorKind::Other, "read error")), (reader, 0)))
            }
        }
    });

    Response::builder()
        .status(status)
        .header(header::CACHE_CONTROL, "no-store, no-cache, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .header(header::CONTENT_LENGTH, content_length.to_string())
        .header(header::CONTENT_RANGE, content_range)
        .body(Body::from_stream(stream))
        .unwrap()
}