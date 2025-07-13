use crate::api::io_util::ApiError;
use crate::api::io_util::ApiError::InternalServerError;
use crate::AppState;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use serde::Deserialize;
use std::ops::RangeInclusive;
use std::sync::Arc;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use crate::api::songs::Index;
use crate::db::action;

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    artist_name: String,
    song_name: String,
    format: Option<String>,
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

pub async fn song(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(_params): Query<StreamQuery>,
) -> Result<Response, ApiError> {
    if !action::is_valid_user(&state.db,auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }
    // Detect a platform from headers (very basic example)
   /* let user_agent = headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()).unwrap_or("");
    // Use a format from a query if provided, otherwise auto-detect
    let (preferred_formats, target_format, content_type) = if let Some(ref fmt) = params.format {
        match fmt.as_str() {
            "m4a" | "aac" => (&["m4a", "aac", "mp3"][..], "m4a", "audio/mp4"),
            "mp3" => (&["mp3", "m4a", "aac"][..], "mp3", "audio/mpeg"),
            _ => (&["mp3", "m4a", "aac"][..], "mp3", "audio/mpeg"),
        }
    } else if user_agent.contains("iPhone") || user_agent.contains("iPad") || user_agent.contains("Macintosh") {
        (&["m4a", "aac", "mp3"][..], "m4a", "audio/mp4")
    } else {
        (&["mp3", "m4a", "aac"][..], "mp3", "audio/mpeg")
    };
    let (file_path, format) = db::actions::get_song_file_path(&state.db, &params.song_name, &params.artist_name, Some(preferred_formats))
        .await
        .map_err(|e| {
            error!("Could not find song in database: {}", e);
            NotFound("Song does not exist".to_string())
        })?;

    // If the file is already in the best format, stream directly (with range support)
    if format == target_format {
        let file = OpenOptions::new().read(true).open(&file_path).await.map_err(|e| {
            error!("File open error: {}", e);
            NotFound("Song not available".to_string())
        })?;

        let metadata = file.metadata().await.map_err(|e| {
            error!("Metadata error: {}", e);
            InternalServerError("Failed to read file metadata".to_string())
        })?;

        let total_size = metadata.len();
        let range_header = headers.get(header::RANGE).and_then(|v| v.to_str().ok());
        let (status, range) = if let Some(range_header) = range_header {
            match parse_range(range_header, total_size) {
                Some(r) => (StatusCode::PARTIAL_CONTENT, r),
                None => return Err(BadRequest("Invalid range".to_string())),
            }
        } else {
            (StatusCode::OK, 0..=total_size - 1)
        };

        let start = *range.start();
        let end = *range.end();
        let content_length = end - start + 1;
        let content_range = format!("bytes {}-{}/{}", start, end, total_size);

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(start)).await.map_err(|e| {
            error!("Seek failed: {}", e);
            InternalServerError("Failed to seek in file".to_string())
        })?;

        let stream = stream::unfold((reader, content_length), |(mut reader, mut remaining)| async move {
            if remaining == 0 { return None; }
            let mut buffer = [0u8; 8192];
            let read_len = buffer.len().min(remaining as usize);
            match reader.read(&mut buffer[..read_len]).await {
                Ok(0) => None,
                Ok(n) => {
                    remaining -= n as u64;
                    Some((Ok::<_, std::io::Error>(buffer[..n].to_vec()), (reader, remaining)))
                }
                Err(e) => {
                    error!("Read error: {}", e);
                    Some((Err(e), (reader, 0)))
                }
            }
        });

        return Response::builder()
            .status(status)
            .header(header::ACCEPT_RANGES, "bytes")
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, content_length.to_string())
            .header(header::CONTENT_RANGE, content_range)
            .body(Body::from_stream(stream))
            .map_err(|_| InternalServerError("Failed to build streaming response".to_string()));
    }

    // Otherwise, use ffmpeg to convert on the fly (no range support)
    let ffmpeg_path = which("ffmpeg").map_err(|_| InternalServerError("ffmpeg not found in PATH".to_string()))?;
    let mut cmd = Command::new(ffmpeg_path);
    cmd.arg("-i").arg(&file_path)
        .arg("-f");
    if target_format == "mp3" {
        cmd.arg("mp3");
        cmd.arg("-acodec").arg("libmp3lame");
    } else {
        cmd.arg("ipod"); // ipod muxer for m4a
        cmd.arg("-acodec").arg("aac");
        cmd.arg("-b:a").arg("192k");
    }
    cmd.arg("-movflags").arg("frag_keyframe+empty_moov");
    cmd.arg("-vn").arg("-sn").arg("-dn");
    cmd.arg("-ar").arg("44100");
    cmd.arg("-ac").arg("2");
    cmd.arg("-y");
    cmd.arg("-"); // Output to stdout

    let mut child = cmd.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null()).spawn()
        .map_err(|e| InternalServerError(format!("Failed to spawn ffmpeg: {e}")))?;
    let stdout = child.stdout.take().ok_or_else(|| InternalServerError("Failed to capture ffmpeg stdout".to_string()))?;
    let stream = stream::unfold(stdout, |mut stdout| async move {
        let mut buffer = [0u8; 8192];
        match stdout.read(&mut buffer).await {
            Ok(0) => None,
            Ok(n) => Some((Ok::<_, std::io::Error>(buffer[..n].to_vec()), stdout)),
            Err(e) => Some((Err(e), stdout))
        }
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::ACCEPT_RANGES, "none")
        .body(Body::from_stream(stream))
        .map_err(|_| InternalServerError("Failed to build streaming response".to_string()))?)*/
    Ok(Response::builder().status(StatusCode::OK).body(Body::empty()).map_err(|_| InternalServerError("Failed to build streaming response".to_string()))?)
}