use axum::{
    extract::{Json, Query, State, Extension},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::api::response::{ApiResponse, ApiError};
use crate::api::auth::AppState;
use crate::auth::Claims;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub index_start: usize,
    pub index_end: usize,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistNameQuery {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    #[serde(rename = "isPublic")]
    pub is_public: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddSongToPlaylistRequest {
    pub playlist: String,
    pub song: String,
    pub artist: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoveSongFromPlaylistRequest {
    pub playlist: String,
    pub song: String,
}

#[derive(Debug, Deserialize)]
pub struct SharePlaylistRequest {
    pub playlist_name: String,
    pub target_user: String,
}

#[derive(Debug, Serialize)]
pub struct PlaylistBasic {
    pub name: String,
    pub is_public: bool,
    pub owner: String,
}

#[derive(Debug, Serialize)]
pub struct SharedPlaylistInfo {
    pub name: String,
    pub owner: String,
    pub shared_by: String,
}

pub async fn get_private_playlists(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<PlaylistBasic>>>, ApiError> {
    let playlists = state.db.get_user_playlists(&claims.sub, params.index_start, params.index_end - params.index_start).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to fetch playlists: {}", e)))?;
    
    let playlist_basics: Vec<PlaylistBasic> = playlists.into_iter()
        .map(|p| PlaylistBasic {
            name: p.name,
            is_public: p.is_public,
            owner: p.owner_username,
        })
        .collect();
    
    Ok(Json(ApiResponse::success("private playlists", playlist_basics)))
}

pub async fn get_public_playlists(
    State(state): State<AppState>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<PlaylistBasic>>>, ApiError> {
    let playlists = state.db.get_public_playlists(params.index_start, params.index_end - params.index_start).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to fetch playlists: {}", e)))?;
    
    let playlist_basics: Vec<PlaylistBasic> = playlists.into_iter()
        .map(|p| PlaylistBasic {
            name: p.name,
            is_public: p.is_public,
            owner: p.owner_username,
        })
        .collect();
    
    Ok(Json(ApiResponse::success("public playlists", playlist_basics)))
}

pub async fn get_shared_playlists(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<SharedPlaylistInfo>>>, ApiError> {
    let shared_playlists = state.db.get_shared_playlists(&claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to fetch shared playlists: {}", e)))?;
    
    let mut shared_info = Vec::new();
    for (playlist, share) in shared_playlists {
        let shared_by_user = state.db.get_user_by_id(&share.shared_by_user_id).await
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to fetch user: {}", e)))?;
        
        shared_info.push(SharedPlaylistInfo {
            name: playlist.name,
            owner: playlist.owner_username,
            shared_by: shared_by_user.username,
        });
    }
    
    Ok(Json(ApiResponse::success("shared playlists", shared_info)))
}

pub async fn create_playlist(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreatePlaylistRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    state.db.create_playlist(&payload.name, &claims.sub, payload.is_public).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create playlist: {}", e)))?;
    
    Ok(Json(ApiResponse::no_data("Playlist created successfully")))
}

pub async fn add_song_to_playlist(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AddSongToPlaylistRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // Get playlist by name and owner
    let playlist = state.db.get_playlist_by_name_and_owner(&payload.playlist, &claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::NOT_FOUND, &format!("Playlist not found: {}", e)))?;
    
    // Get song by title and artist
    let artist = state.db.get_artist_by_name(&payload.artist).await
        .map_err(|e| ApiError::new(StatusCode::NOT_FOUND, &format!("Artist not found: {}", e)))?;
    
    let songs = state.db.get_songs_by_artist(&artist.id).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to fetch songs: {}", e)))?;
    
    let song = songs.iter().find(|s| s.title == payload.song)
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "Song not found"))?;
    
    // Add song to playlist
    state.db.add_song_to_playlist(&playlist.id, &song.id).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to add song to playlist: {}", e)))?;
    
    Ok(Json(ApiResponse::no_data("Song added to playlist")))
}

pub async fn remove_song_from_playlist(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<RemoveSongFromPlaylistRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // Get playlist by name and owner
    let playlist = state.db.get_playlist_by_name_and_owner(&payload.playlist, &claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::NOT_FOUND, &format!("Playlist not found: {}", e)))?;
    
    // Find song by title (simplified - in production you'd want more specific song identification)
    let songs = state.db.get_playlist_songs(&playlist.id).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to fetch playlist songs: {}", e)))?;
    
    let song = songs.iter().find(|s| s.title == payload.song)
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "Song not found in playlist"))?;
    
    // Remove song from playlist
    state.db.remove_song_from_playlist(&playlist.id, &song.id).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to remove song from playlist: {}", e)))?;
    
    Ok(Json(ApiResponse::no_data("Song removed from playlist")))
}

pub async fn delete_playlist(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PlaylistNameQuery>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // Get playlist to get its ID
    let playlist = state.db.get_playlist_by_name_and_owner(&params.name, &claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::NOT_FOUND, &format!("Playlist not found: {}", e)))?;
    
    state.db.delete_playlist(&playlist.id, &claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to delete playlist: {}", e)))?;
    
    Ok(Json(ApiResponse::no_data("Playlist deleted successfully")))
}

pub async fn share_playlist(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<SharePlaylistRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // Get playlist by name and owner
    let playlist = state.db.get_playlist_by_name_and_owner(&payload.playlist_name, &claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::NOT_FOUND, &format!("Playlist not found: {}", e)))?;
    
    // Get target user
    let target_user = state.db.get_user_by_username(&payload.target_user).await
        .map_err(|e| ApiError::new(StatusCode::NOT_FOUND, &format!("User not found: {}", e)))?;
    
    // Share playlist
    state.db.share_playlist(&playlist.id, &target_user.id, &claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to share playlist: {}", e)))?;
    
    Ok(Json(ApiResponse::no_data("Playlist shared successfully")))
}

pub async fn revoke_playlist_share(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<SharePlaylistRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // Get playlist by name and owner
    let playlist = state.db.get_playlist_by_name_and_owner(&payload.playlist_name, &claims.sub).await
        .map_err(|e| ApiError::new(StatusCode::NOT_FOUND, &format!("Playlist not found: {}", e)))?;
    
    // Get target user
    let target_user = state.db.get_user_by_username(&payload.target_user).await
        .map_err(|e| ApiError::new(StatusCode::NOT_FOUND, &format!("User not found: {}", e)))?;
    
    // Revoke playlist share
    state.db.revoke_playlist_share(&playlist.id, &target_user.id).await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to revoke playlist share: {}", e)))?;
    
    Ok(Json(ApiResponse::no_data("Playlist share revoked")))
}
