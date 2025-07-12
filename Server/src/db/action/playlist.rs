use crate::api::io_util::ApiError;
use crate::db::schema::playlist;
use crate::db::util::sql_share::SQLResult;
use crate::db::{session, user, DbPool};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug,Serialize,Clone)]
pub struct PlaylistInfo {
    pub name: String,
    pub username: String,
}

pub async fn get_private_info(pool: &DbPool, session_id: &Uuid, start: usize, end: usize) -> SQLResult<Vec<PlaylistInfo>> {
    let user_id: Uuid = session::get_user_id_from_session_id(session_id, pool).await?;
    let playlists = playlist::get_by_user(pool, &user_id, start, end).await?;
    let username: String = user::get_username_by_uuid(pool, &user_id).await?;
    let mut data: Vec<PlaylistInfo> = Vec::new();
    for playlist in playlists {
        data.push(PlaylistInfo { name: playlist.name.clone(), username: username.clone() });
    }
    Ok(data)
}

pub async fn get_public_info(pool: &DbPool, start: usize, end: usize) -> SQLResult<Vec<PlaylistInfo>> {
    let playlists = playlist::get_public(pool, start, end).await?;
    let mut data: Vec<PlaylistInfo> = Vec::new();
    for playlist in playlists {
        data.push(PlaylistInfo { name: playlist.name.clone(), username: user::get_username_by_uuid(pool, &playlist.user_uuid).await? });
    }
    Ok(data)
}

pub async fn get_private_count(pool: &DbPool, session_id: &Uuid) -> Result<usize, ApiError> {
    let user_id = match session::get_user_id_from_session_id(session_id, pool).await {
        Ok(user_id) => user_id,
        Err(_) => return Err(ApiError::Unauthorized)
    };
    match playlist::get_count_by_user(pool, &user_id).await {
        Ok(count) => Ok(count as usize),
        Err(_) => Err(ApiError::InternalServerError("".to_string())),
    }
}

pub async fn get_public_count(pool: &DbPool) -> Result<usize, ApiError> {
    match playlist::get_public_count(pool).await {
        Ok(count) => Ok(count as usize),
        Err(_) => Err(ApiError::InternalServerError("".to_string())),
    }
}