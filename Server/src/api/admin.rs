use axum::{extract::{Json, Query, State}};
use serde::{Deserialize, Serialize};
use crate::api::response::{ApiResponse, ApiResult, ApiResultNoData, ApiError};
use crate::api::users::UserInfo;
use crate::api::auth::AppState;
use crate::music::MusicScanner;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub index_start: usize,
    pub index_end: usize,
}

#[derive(Debug, Deserialize)]
pub struct EditUserRequest {
    pub username: String,
    pub new_email: Option<String>,
    pub is_admin: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserRequest {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct EditSongRequest {
    pub artist_name: String,
    pub song_name: String,
    pub new_album: Option<String>,
    pub new_genre: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSongRequest {
    pub artist_name: String,
    pub song_name: String,
}

#[derive(Debug, Deserialize)]
pub struct EditPlaylistRequest {
    pub name: String,
    pub new_name: Option<String>,
    pub new_visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeletePlaylistRequest {
    pub name: String,
}


#[derive(Debug, Serialize)]
pub struct AdminPlaylistInfo {
    pub name: String,
    pub owner: String,
    pub is_public: bool,
    pub song_count: usize,
}

pub async fn get_all_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationQuery>
) -> ApiResult<Vec<UserInfo>> {
    // Calculate offset and limit from index_start and index_end
    let offset = params.index_start;
    let limit = params.index_end.saturating_sub(params.index_start).max(1);
    
    let users = state.db.get_all_users(offset, limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get users: {}", e);
            ApiError::internal_server_error(format!("Failed to retrieve users: {}", e))
        })?;
    
    let user_infos: Vec<UserInfo> = users.into_iter().map(|user| UserInfo {
        username: user.username,
        email: user.email,
        is_admin: user.is_admin,
        created_at: user.created_at.format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "Invalid date".to_string()),
    }).collect();
    
    Ok(Json(ApiResponse::success("users", user_infos)))
}

pub async fn edit_user(
    State(state): State<AppState>,
    Json(payload): Json<EditUserRequest>
) -> ApiResultNoData {
    // Get the user first to ensure they exist
    let user = state.db.get_user_by_username(&payload.username)
        .await
        .map_err(|e| {
            tracing::error!("Failed to find user: {}", e);
            ApiError::not_found(format!("User not found: {}", e))
        })?;
    
    // Update email if provided
    if let Some(new_email) = payload.new_email {
        state.db.update_user_email(&payload.username, &new_email)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update user email: {}", e);
                ApiError::bad_request(format!("Failed to update email: {}", e))
            })?;
    }
    
    // Update admin status if provided
    if let Some(is_admin) = payload.is_admin {
        state.db.update_user_admin_status(&user.id, is_admin)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update user admin status: {}", e);
                ApiError::internal_server_error(format!("Failed to update admin status: {}", e))
            })?;
    }
    
    Ok(Json(ApiResponse::no_data("User updated successfully")))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Json(payload): Json<DeleteUserRequest>
) -> ApiResultNoData {
    state.db.delete_user_by_username(&payload.username)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete user: {}", e);
            ApiError::internal_server_error(format!("Failed to delete user: {}", e))
        })?;
    
    Ok(Json(ApiResponse::no_data("User deleted successfully")))
}

pub async fn add_song(Json(_payload): Json<serde_json::Value>) -> ApiResultNoData {
    // Songs are added automatically via the music scanner
    // This endpoint is not implemented as manual song addition is not supported
    Err(ApiError::bad_request("Manual song addition is not supported. Songs are automatically added via the music scanner. Use POST /api/admin/songs/scan to scan for new songs."))
}

pub async fn edit_song(
    State(state): State<AppState>,
    Json(payload): Json<EditSongRequest>
) -> ApiResultNoData {
    // Get artist by name
    let artist = state.db.get_artist_by_name(&payload.artist_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to find artist: {}", e);
            ApiError::not_found(format!("Artist not found: {}", e))
        })?;
    
    // Get songs by this artist
    let songs = state.db.get_songs_by_artist(&artist.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get songs: {}", e);
            ApiError::internal_server_error(format!("Failed to retrieve songs: {}", e))
        })?;
    
    // Find the song with matching title
    let song = songs.iter()
        .find(|s| s.title == payload.song_name)
        .ok_or_else(|| {
            tracing::error!("Song '{}' not found for artist '{}'", payload.song_name, payload.artist_name);
            ApiError::not_found(format!("Song '{}' not found", payload.song_name))
        })?;
    
    // Update song metadata (album is supported, genre is not currently in the database model)
    state.db.update_song_metadata(
        &song.id,
        payload.new_album.as_deref(),
        None, // duration - not provided in request
        None  // cover_path - not provided in request
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update song metadata: {}", e);
        ApiError::internal_server_error(format!("Failed to update song: {}", e))
    })?;
    
    // Note: new_genre is ignored as the current database schema doesn't support genre field
    if payload.new_genre.is_some() {
        tracing::warn!("Genre update requested but not supported in current schema");
    }
    
    Ok(Json(ApiResponse::no_data("Song metadata updated successfully")))
}

pub async fn delete_song(
    State(state): State<AppState>,
    Json(payload): Json<DeleteSongRequest>
) -> ApiResultNoData {
    // Get artist by name
    let artist = state.db.get_artist_by_name(&payload.artist_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to find artist: {}", e);
            ApiError::not_found(format!("Artist not found: {}", e))
        })?;
    
    // Get songs by this artist
    let songs = state.db.get_songs_by_artist(&artist.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get songs: {}", e);
            ApiError::internal_server_error(format!("Failed to retrieve songs: {}", e))
        })?;
    
    // Find the song with matching title
    let song = songs.iter()
        .find(|s| s.title == payload.song_name)
        .ok_or_else(|| {
            tracing::error!("Song '{}' not found for artist '{}'", payload.song_name, payload.artist_name);
            ApiError::not_found(format!("Song '{}' not found", payload.song_name))
        })?;
    
    // Delete the song
    state.db.delete_song_by_id(&song.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete song: {}", e);
            ApiError::internal_server_error(format!("Failed to delete song: {}", e))
        })?;
    
    Ok(Json(ApiResponse::no_data("Song deleted successfully")))
}

pub async fn get_all_playlists(
    State(state): State<AppState>,
    Query(params): Query<PaginationQuery>
) -> ApiResult<Vec<AdminPlaylistInfo>> {
    // Calculate offset and limit from index_start and index_end
    let offset = params.index_start;
    let limit = params.index_end.saturating_sub(params.index_start).max(1);
    
    // Get all playlists from database
    let playlists = state.db.get_all_playlists(offset, limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get playlists: {}", e);
            ApiError::internal_server_error(format!("Failed to retrieve playlists: {}", e))
        })?;
    
    // Convert to AdminPlaylistInfo with song count
    let mut admin_playlists = Vec::new();
    for playlist in playlists {
        // Get song count for this playlist
        let songs = state.db.get_playlist_songs(&playlist.id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get playlist songs: {}", e);
                ApiError::internal_server_error(format!("Failed to retrieve playlist songs: {}", e))
            })?;
        
        admin_playlists.push(AdminPlaylistInfo {
            name: playlist.name,
            owner: playlist.owner_username,
            is_public: playlist.is_public,
            song_count: songs.len(),
        });
    }
    
    Ok(Json(ApiResponse::success("playlists", admin_playlists)))
}

pub async fn edit_playlist(
    State(state): State<AppState>,
    Json(payload): Json<EditPlaylistRequest>
) -> ApiResultNoData {
    // Find playlist by name (admin can see all playlists)
    // We need to get all playlists and find the one with matching name
    // This is inefficient but works for now - could add a db method later
    let all_playlists = state.db.get_all_playlists(0, 10000)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get playlists: {}", e);
            ApiError::internal_server_error(format!("Failed to retrieve playlists: {}", e))
        })?;
    
    let playlist = all_playlists.iter()
        .find(|p| p.name == payload.name)
        .ok_or_else(|| {
            tracing::error!("Playlist '{}' not found", payload.name);
            ApiError::not_found(format!("Playlist '{}' not found", payload.name))
        })?;
    
    // Update name if provided
    if let Some(new_name) = payload.new_name {
        state.db.update_playlist_name(&playlist.id, &new_name)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update playlist name: {}", e);
                ApiError::internal_server_error(format!("Failed to update playlist name: {}", e))
            })?;
    }
    
    // Update visibility if provided
    if let Some(new_visibility) = payload.new_visibility {
        let is_public = new_visibility.eq_ignore_ascii_case("public");
        state.db.update_playlist_visibility(&playlist.id, is_public)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update playlist visibility: {}", e);
                ApiError::internal_server_error(format!("Failed to update playlist visibility: {}", e))
            })?;
    }
    
    Ok(Json(ApiResponse::no_data("Playlist updated successfully")))
}

pub async fn delete_playlist(
    State(state): State<AppState>,
    Json(payload): Json<DeletePlaylistRequest>
) -> ApiResultNoData {
    // Find playlist by name (admin can delete any playlist)
    let all_playlists = state.db.get_all_playlists(0, 10000)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get playlists: {}", e);
            ApiError::internal_server_error(format!("Failed to retrieve playlists: {}", e))
        })?;
    
    let playlist = all_playlists.iter()
        .find(|p| p.name == payload.name)
        .ok_or_else(|| {
            tracing::error!("Playlist '{}' not found", payload.name);
            ApiError::not_found(format!("Playlist '{}' not found", payload.name))
        })?;
    
    // Delete the playlist (admin version bypasses owner check)
    state.db.delete_playlist_by_id(&playlist.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete playlist: {}", e);
            ApiError::internal_server_error(format!("Failed to delete playlist: {}", e))
        })?;
    
    Ok(Json(ApiResponse::no_data("Playlist deleted successfully")))
}

#[derive(Debug, Serialize)]
pub struct ScanMusicResult {
    pub total_files: usize,
    pub registered: usize,
    pub updated: usize,
    pub skipped: usize,
    pub removed: usize,
    pub errors: usize,
}

/// POST /api/admin/songs/scan
/// Scan the runtime/music directory and register all audio files
/// Also removes songs whose files no longer exist
pub async fn scan_music_directory(
    State(state): State<AppState>,
) -> ApiResult<ScanMusicResult> {
    tracing::info!("Starting music directory scan");
    
    // Create the music scanner
    let scanner = MusicScanner::new(state.db.clone(), "runtime/music");
    
    // Perform the scan
    let result = scanner.scan_and_register().await
        .map_err(|e| {
            tracing::error!("Failed to scan music directory: {}", e);
            ApiError::internal_server_error(format!("Failed to scan music directory: {}", e))
        })?;
    
    let scan_result = ScanMusicResult {
        total_files: result.total_files,
        registered: result.registered,
        updated: result.updated,
        skipped: result.skipped,
        removed: result.removed,
        errors: result.errors,
    };
    
    Ok(Json(ApiResponse::success("Music scan completed", scan_result)))
}
