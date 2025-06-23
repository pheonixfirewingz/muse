use std::io::SeekFrom;
use std::ops::RangeInclusive;
use crate::{db, web, AppState};
use axum::extract::{Query, State, Form};
use axum::response::{IntoResponse, Response, Redirect};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use axum::body::{Body, Bytes};
use axum::http::{header, HeaderMap, StatusCode};
use futures::{stream, TryStreamExt};
use rustrict::CensorStr;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncSeekExt, BufReader};
use tracing::{error, info};
use tower_cookies::Cookies;
use uuid::Uuid;
use tokio::process::Command;
use tokio_util::io::ReaderStream;
use which::which;
use crate::web::api::ApiError::{BadRequest, InternalServerError, NotFound, Unauthorized};

#[derive(Debug, Deserialize)]
struct ArtistImageQuery {
    artist_name: String,
}

#[derive(Debug, Deserialize)]
struct SongImageQuery {
    artist_name: String,
    song_name: String,
}

#[derive(Debug, Deserialize)]
struct SongStreamQuery {
    artist_name: String,
    song_name: String,
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreatePlaylistRequest {
    name: String,
    public: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AddToPlaylistRequest {
    playlist_name: String,
    song_name: String,
    artist_name: String,
}

#[derive(Debug, Deserialize)]
struct CreatePlaylistAndAddRequest {
    playlist_name: String,
    song_name: String,
    artist_name: String,
    public: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct DeletePlaylist {
    playlist_name: String,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T: Serialize = Value> {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
struct PlaylistData {
    playlist_uuid: String,
}

enum ApiError {
    Unauthorized,
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Unauthorized => (StatusCode::UNAUTHORIZED, "Not authenticated".to_string()),
            BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let response = ApiResponse::<()> {
            success: false,
            message,
            data: None,
        };

        (status, Json(response)).into_response()
    }
}

fn get_session_id_from_cookies(cookies: &Cookies) -> Result<Uuid, ApiError>
{
    match web::get_session_id_from_cookies(cookies) { 
        Ok(uuid) => Ok(uuid),
        Err(Some(msg)) => Err(BadRequest(msg)),
        Err(None) => Err(Unauthorized),
    }
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

async fn get_artist_image(
    Query(params): Query<ArtistImageQuery>,
    cookies: Cookies,
) -> Result<Redirect, ApiError> {
    let _session_id = get_session_id_from_cookies(&cookies)?;
    match db::thirdparty::get_artist_image_url(&params.artist_name).await {
        Ok(Some(image_url)) => Ok(Redirect::to(&image_url)),
        _ => Err(NotFound("Artist image not found".to_string())),
    }
}

async fn get_song_image(
    Query(params): Query<SongImageQuery>,
    cookies: Cookies,
) -> Result<Redirect, ApiError> {
    let _session_id = get_session_id_from_cookies(&cookies)?;
    match db::thirdparty::get_song_image_url(&params.artist_name, &params.song_name).await {
        Ok(Some(image_url)) => Ok(Redirect::to(&image_url)),
        _ => Err(NotFound("Song image not found".to_string())),
    }
}

async fn get_user_playlists(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    let session_id = get_session_id_from_cookies(&cookies)?;
    let playlists = db::actions::get_db_user_playlists_info(&state.db, &session_id)
        .await
        .map_err(|e| InternalServerError(format!("Failed to get playlists: {}", e)))?;

    Ok(Json(ApiResponse {
        success: true,
        message: "Playlists retrieved successfully".to_string(),
        data: Some(serde_json::json!(playlists)),
    }))
}

async fn get_public_playlists(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    let playlists = db::actions::get_db_public_playlists_info(&state.db)
        .await
        .map_err(|e| InternalServerError(format!("Failed to get public playlists: {}", e)))?;

    Ok(Json(ApiResponse {
        success: true,
        message: "Public playlists retrieved successfully".to_string(),
        data: Some(serde_json::json!(playlists)),
    }))
}

async fn create_playlist(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Form(request): Form<CreatePlaylistRequest>,
) -> Result<Json<ApiResponse<PlaylistData>>, ApiError> {
    let session_id = get_session_id_from_cookies(&cookies)?;

    let playlist_uuid = db::actions::create_playlist_for_user(
        &state.db, &session_id, &request.name, request.public.unwrap_or(false)
    )
    .await
    .map_err(|e| BadRequest(format!("Failed to create playlist: {}", e)))?;

    info!("Created playlist '{}'", request.name);
    
    Ok(Json(ApiResponse {
        success: true,
        message: format!("Playlist '{}' created successfully", request.name),
        data: Some(PlaylistData { playlist_uuid: playlist_uuid.to_string() }),
    }))
}

async fn add_to_playlist(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Form(request): Form<AddToPlaylistRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let session_id = get_session_id_from_cookies(&cookies)?;

    let added = db::actions::add_song_to_playlist(
        &state.db, &session_id, &request.playlist_name, &request.song_name, &request.artist_name
    )
    .await
    .map_err(|e| BadRequest(format!("Failed to add song to playlist: {}", e)))?;

    let message = if added {
        format!("Added '{}' to playlist '{}'", request.song_name, request.playlist_name)
    } else {
        format!("Song '{}' is already in playlist '{}'", request.song_name, request.playlist_name)
    };

    info!("{}", message);
    
    Ok(Json(ApiResponse { success: true, message, data: None }))
}

async fn create_playlist_and_add(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Form(request): Form<CreatePlaylistAndAddRequest>,
) -> Result<Json<ApiResponse<PlaylistData>>, ApiError> {
    let session_id = get_session_id_from_cookies(&cookies)?;

    if request.playlist_name.is_inappropriate() {
        return Err(BadRequest("playlist name is inappropriate".to_string()));
    }

    let playlist_uuid = db::actions::create_playlist_and_add_song(
        &state.db, &session_id, &request.playlist_name, &request.song_name, &request.artist_name, request.public.unwrap_or(false)
    )
    .await.map_err(|e| BadRequest(format!("Failed to create playlist and add song: {}", e)))?;

    info!("Created playlist '{}' and added song '{}'", request.playlist_name, request.song_name);
    
    Ok(Json(ApiResponse {
        success: true,
        message: format!("Created playlist '{}' and added '{}'", request.playlist_name, request.song_name),
        data: Some(PlaylistData { playlist_uuid: playlist_uuid.to_string() }),
    }))
}

async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Form(request): Form<DeletePlaylist>,
) -> Result<(), ApiError> {
    let session_id = get_session_id_from_cookies(&cookies)?;
    db::actions::delete_playlist(&state.db,&request.playlist_name,&session_id)
    .await.map_err(|e| BadRequest(format!("Failed to delete playlist: {}", e)))
}

async fn stream_song(
    Query(params): Query<SongStreamQuery>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    cookies: Cookies,
) -> Result<Response, ApiError> {
    let _session_id = get_session_id_from_cookies(&cookies)?;
    // Detect a platform from headers (very basic example)
    let user_agent = headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()).unwrap_or("");
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
                    Some((Ok::<_, std::io::Error>(Bytes::copy_from_slice(&buffer[..n])), (reader, remaining)))
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
    let stream = ReaderStream::new(stdout).map_ok(Bytes::from);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::ACCEPT_RANGES, "none")
        .body(Body::from_stream(stream))
        .map_err(|_| InternalServerError("Failed to build streaming response".to_string()))?)
}

// ============================================================================
// Router Configuration
// ============================================================================

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/images/artist", get(get_artist_image))
        .route("/api/images/song", get(get_song_image))
        .route("/api/playlists", get(get_user_playlists).post(create_playlist))
        .route("/api/playlists/songs", post(add_to_playlist))
        .route("/api/playlists/create_and_add", post(create_playlist_and_add))
        .route("/api/playlists/delete", post(delete_playlist))
        .route("/api/playlists/public", get(get_public_playlists))
        .route("/api/stream", get(stream_song))
}