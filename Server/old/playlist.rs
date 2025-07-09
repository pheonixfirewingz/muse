use std::sync::Arc;
use axum::extract::State;
use axum::{Form, Json};
use rustrict::CensorStr;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tower_cookies::Cookies;
use tracing::info;
use crate::{db, AppState};
use crate::api::{get_session_id_from_cookies, ApiResponse};
use crate::api::error::ApiError;
use crate::api::error::ApiError::{BadRequest, InternalServerError};

#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    name: String,
    public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddToPlaylistRequest {
    playlist_name: String,
    song_name: String,
    artist_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlaylistAndAddRequest {
    playlist_name: String,
    song_name: String,
    artist_name: String,
    public: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PlaylistData {
    playlist_uuid: String,
}


#[derive(Debug, Deserialize)]
pub struct DeletePlaylist {
    playlist_name: String,
}

#[derive(Deserialize)]
pub struct ReorderSongsRequest {
    playlist_name: String,
    song_order: Vec<SongOrderEntry>, // song title and artist in new order
}

#[derive(Deserialize)]
pub struct SongOrderEntry {
    title: String,
    artist: String,
}

pub async fn get_user_playlists(
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
        data: Some(json!(playlists)),
    }))
}

pub async fn get_public_playlists(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    let playlists = db::actions::get_db_public_playlists_info(&state.db)
        .await
        .map_err(|e| InternalServerError(format!("Failed to get public playlists: {}", e)))?;

    Ok(Json(ApiResponse {
        success: true,
        message: "Public playlists retrieved successfully".to_string(),
        data: Some(json!(playlists)),
    }))
}

pub async fn create_playlist(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Form(request): Form<CreatePlaylistRequest>,
) -> Result<Json<ApiResponse<PlaylistData>>, ApiError> {
    let session_id = get_session_id_from_cookies(&cookies)?;
    let is_public = request.public.unwrap_or(false);

    if is_public {
        if request.name.is_inappropriate() {
            return Err(BadRequest("inappropriate name for public playlist".to_string()))
        }
    }

    let playlist_uuid = db::actions::create_playlist_for_user(
        &state.db, &session_id, &request.name, is_public
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

pub async fn add_to_playlist(
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

pub async fn create_playlist_and_add(
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

pub async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Form(request): Form<DeletePlaylist>,
) -> Result<(), ApiError> {
    let session_id = get_session_id_from_cookies(&cookies)?;
    db::actions::delete_playlist(&state.db,&request.playlist_name,&session_id)
        .await.map_err(|e| BadRequest(format!("Failed to delete playlist: {}", e)))
}

pub async fn reorder_playlist_songs(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Json(payload): Json<ReorderSongsRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let session_id = get_session_id_from_cookies(&cookies)?;
    let song_order: Vec<(String, String)> = payload.song_order
        .into_iter()
        .map(|entry| (entry.title, entry.artist))
        .collect();
    db::actions::reorder_songs_in_playlist(&state.db, &session_id, &payload.playlist_name, &song_order)
        .await
        .map_err(|e| BadRequest(format!("Failed to reorder songs: {}", e)))?;
    Ok(Json(ApiResponse {
        success: true,
        message: "Playlist order updated".to_string(),
        data: None,
    }))
}