use crate::{db, AppState};
use async_stream::stream;
use axum::{
    body::Body,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use serde::Deserialize;
use std::sync::Arc;
use axum::http::header;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, BufReader};

#[derive(Deserialize)]
pub struct SongQuery {
    song: String,
    artist: String,
    is_mobile: String,
}

pub async fn handler(Query(query): Query<SongQuery>, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Get the file path using the song and artist names
    let file_path = db::get_song_path_by_song_name_and_artist_name(&state.db, &query.song, &query.artist).await;
    if file_path.is_none() {
        return Response::builder()
            .status(404)
            .body(Body::from("song not found"))
            .unwrap();
    }
    let file_path = file_path.unwrap();

    match OpenOptions::new().read(true).open(file_path).await {
        Ok(file) => {
            // Use BufReader for efficient chunked reading
            let mut file = BufReader::new(file);

            // Create a stream from the file
            let stream = stream! {
                let mut buffer = vec![0; 1024]; // Chunk size of 1024 bytes
                loop {
                    match file.read(&mut buffer).await {
                        Ok(0) => break, // End of file
                        Ok(n) => yield Ok::<_, std::io::Error>(Bytes::copy_from_slice(&buffer[..n])), // Yield the bytes as Bytes
                        Err(e) => yield Err(e), // Handle read error
                    }
                }
            };

            // Set headers to prevent caching and stream audio
            Response::builder()
                .header(header::CACHE_CONTROL, "no-store, no-cache, must-revalidate")
                .header(header::PRAGMA, "no-cache")
                .header("Content-Type", "audio/mpeg")
                .body(Body::from_stream(stream)) // Wrap the stream into the response body
                .unwrap()
        }
        Err(e) => {
            Response::builder()
                .status(404)
                .body(Body::from(format!("File not found: {}", e)))
                .unwrap()
        }
    }
}
