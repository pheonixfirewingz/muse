use std::path::Path;
use std::str::FromStr;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tracing::{error, info};
use uuid::Uuid;
use crate::api::io_util::ApiError;
use crate::db::{session, DbPool};
use crate::db::schema::{artist as a, artist_song_association, song as s};
use crate::db::schema::song::Song;
use crate::db::util::sql_share::SQLResult;
use crate::util::transcoder;

pub mod playlist;
pub mod song;
pub mod artist;
pub mod user;

static REG_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
pub async fn register_song(pool: &DbPool, song_name: String, artist_name: String, song_path: &String) -> SQLResult<bool> {
    let _guard = REG_MUTEX.lock().await;

    if !a::has_table_got_name(pool, &artist_name).await? {
        if let Err(e) = a::add(pool, artist_name.clone()).await {
            error!("Failed to add artist: {}", e);
        }
    }

    let original_format = song_path.split('.').last().unwrap_or("mp3").to_lowercase();

    let has_song = s::has_table_got_name_by_artist(pool, &artist_name, &song_name,&original_format).await?;
    if !has_song {
        // Detect format from file extension
        if original_format != "aac" {
            let mut final_path = song_path.clone();
            let mut format_to_register = original_format.clone();
            // Transcode using your transcoder module
            let original_path = Path::new(song_path);
            let dir = original_path.parent().unwrap_or_else(|| Path::new("."));
            let file_stem = original_path.file_stem().and_then(|stem| stem.to_str()).unwrap_or(&song_name);
            let transcoded_path = dir.join(format!("{}.aac", file_stem));

            // Check if file already exists in target format
            if transcoded_path.exists() {
                final_path = transcoded_path.to_str().unwrap().to_string();
                format_to_register = "aac".to_string();
            } else {
                match transcoder::transcode_to_aac(song_path, transcoded_path.to_str().unwrap()).await {
                    Ok(_) => {
                        final_path = transcoded_path.to_str().unwrap().to_string();
                        format_to_register = "aac".to_string();
                    }
                    Err(e) => {
                        error!("Transcoding failed: {:?}", e);
                        return Ok(false);
                    }
                }
            }
            let song = Song::new(song_name.clone(), final_path, format_to_register);
            s::add(pool, &song).await?;
            let artist = a::get_by_name(pool, &artist_name).await?;
            artist_song_association::add_artist_song_association(pool, &artist.uuid, song.get_id()).await?;
        }
        let song = Song::new(song_name.clone(), song_path.clone(), original_format.clone());
        s::add(pool, &song).await?;
        let artist = a::get_by_name(pool, &artist_name).await?;
        artist_song_association::add_artist_song_association(pool, &artist.uuid, song.get_id()).await?;
        Ok(true)
    } else {
        info!("the database has a song named {} by artist {}", song_name, artist_name);
        Ok(false)
    }
}


pub async fn is_valid_user(pool: &DbPool, string_id:&str) -> Result<bool, ApiError> {
    let session_id = match Uuid::from_str(&string_id) {
        Ok(session_id) => session_id,
        Err(_) => return Err(ApiError::Unauthorized)
    };

    match session::validate_session(pool, session_id).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false)
    }
}



/*
pub async fn get_db_user_playlists_info(pool: &DbPool,session_id: &Uuid) -> SQLResult<Vec<PlaylistInfo>> {
    let user_uuid = session::get_user_id_from_session_id(session_id, pool).await?;
    let playlists = playlist::get_playlists_by_user(pool,&user_uuid).await?;
    let mut data:Vec<PlaylistInfo> = Vec::new();
    for playlist in playlists {
        data.push( PlaylistInfo {name: playlist.name.clone()});
    }
    Ok(data)
}

pub async fn create_playlist_for_user(pool: &DbPool, session_id: &Uuid, playlist_name: &String, public: bool) -> SQLResult<Uuid> {
    let user_uuid = session::get_user_id_from_session_id(session_id, pool).await?;

    // Check if playlist name already exists for this user
    let exists = playlist::playlist_name_exists_for_user(pool, playlist_name, &user_uuid).await?;
    if exists {
        return Err(sqlx::Error::InvalidArgument(format!("Playlist '{}' already exists", playlist_name)));
    }

    let new_playlist = Playlist::new(playlist_name.clone(), user_uuid, public);
    playlist::create_playlist(pool, &new_playlist).await?;

    info!("Created playlist '{}' for user {}", playlist_name, user_uuid);
    Ok(new_playlist.uuid)
}

pub async fn add_song_to_playlist(pool: &DbPool, session_id: &Uuid, playlist_name: &String, song_name: &String, artist_name: &String) -> SQLResult<bool> {
    let user_uuid = session::get_user_id_from_session_id(session_id, pool).await?;

    // Get the playlist
    let playlist = match playlist::get_playlist_by_name(pool, playlist_name, &user_uuid).await? {
        Some(p) => p,
        None => return Err(sqlx::Error::InvalidArgument(format!("Playlist '{}' not found", playlist_name)))
    };

    // Get the song
    let song = match get_song_by_name_and_artist(pool, song_name, artist_name).await? {
        Some(s) => s,
        None => return Err(sqlx::Error::InvalidArgument(format!("Song '{}' by '{}' not found", song_name, artist_name)))
    };

    // Check if song is already in playlist
    let already_in_playlist = playlist_song_association::is_song_in_playlist(pool, &playlist.uuid, &song.uuid).await?;
    if already_in_playlist {
        info!("Song '{}' is already in playlist '{}'", song_name, playlist_name);
        return Ok(false);
    }

    // Add song to playlist
    playlist_song_association::add_song_to_playlist(pool, &playlist.uuid, &song.uuid).await?;

    info!("Added song '{}' to playlist '{}'", song_name, playlist_name);
    Ok(true)
}

pub async fn create_playlist_and_add_song(pool: &DbPool, session_id: &Uuid, playlist_name: &String, song_name: &String, artist_name: &String, public: bool) -> SQLResult<Uuid> {
    // Create the playlist first
    let playlist_uuid = create_playlist_for_user(pool, session_id, playlist_name, public).await?;

    // Get the song
    let song = match get_song_by_name_and_artist(pool, song_name, artist_name).await? {
        Some(s) => s,
        None => return Err(sqlx::Error::InvalidArgument(format!("Song '{}' by '{}' not found", song_name, artist_name)))
    };

    // Add song to the newly created playlist
    playlist_song_association::add_song_to_playlist(pool, &playlist_uuid, &song.uuid).await?;

    info!("Created playlist '{}' and added song '{}'", playlist_name, song_name);
    Ok(playlist_uuid)
}

pub async fn delete_playlist(pool: &DbPool, playlist_name: &String, session_id: &Uuid) -> SQLResult<()> {
    let user_id = session::get_user_id_from_session_id(session_id, pool).await?;
    let playlist = match  playlist::get_playlist_by_name(pool, playlist_name,&user_id).await? {
        Some(playlist) => playlist,
        None => return Err(sqlx::Error::InvalidArgument(format!("Playlist '{}' not found", playlist_name)))
    };
    playlist::delete_playlist(pool, &playlist.uuid).await?;
    Ok(())
}

async fn get_song_by_name_and_artist(pool: &DbPool, song_name: &String, artist_name: &String) -> SQLResult<Option<Song>> {
    let songs = song::get_songs_by_name(pool, song_name).await?;

    for song in songs {
        let artist = artist::get_artist_by_name(pool, artist_name).await?;
        let belongs = artist_song_association::dose_song_belong_to_artist(pool, &artist.uuid, &song.uuid).await?;
        if belongs {
            return Ok(Some(song));
        }
    }

    Ok(None)
}

pub async fn get_db_public_playlists_info(pool: &DbPool) -> SQLResult<Vec<PublicPlaylistInfo>> {
    let playlists = playlist::get_public_playlists(pool).await?;
    let mut data: Vec<PublicPlaylistInfo> = Vec::new();
    for playlist in playlists {
        let username = user::get_username_by_uuid(pool, &playlist.user_uuid).await.unwrap_or_else(|_| "unknown".to_string());
        data.push(PublicPlaylistInfo {
            name: playlist.name.clone(),
            username,
        });
    }
    Ok(data)
}

pub async fn get_playlist_details_by_name(
    pool: &DbPool,
    playlist_name: &str,
    session_id: Option<&Uuid>,
    public_playlist_username: Option<&str>,
) -> SQLResult<(PlaylistDetails, Vec<PlaylistSongInfo>)> {
    let mut playlist_owner_username = None;
    let mut playlist_uuid = None;

    // If a session ID is provided, try to find a personal playlist first.
    if let Some(sid) = session_id {
        if let Ok(user_uuid) = session::get_user_id_from_session_id(sid, pool).await {
            if let Ok(Some(pl)) = playlist::get_playlist_by_name(pool, playlist_name, &user_uuid).await {
                if !pl.public || public_playlist_username.is_none() {
                    playlist_uuid = Some(pl.uuid);
                    playlist_owner_username = user::get_username_by_uuid(pool, &user_uuid).await.ok();
                }
            }
        }
    }

    // Fallback to public playlist if no personal one was found or required
    if playlist_uuid.is_none() {
        let all_public = get_db_public_playlists_info(pool).await.unwrap_or_default();
        let target_playlist = if let Some(uname) = public_playlist_username {
            // Filter by username if provided
            all_public.into_iter().find(|p| p.name == playlist_name && p.username == uname)
        } else {
            // Otherwise, just find by name (for cases where session lookup failed but it might be a public playlist)
            all_public.into_iter().find(|p| p.name == playlist_name)
        };

        if let Some(public_playlist) = target_playlist {
            // We need to find the user's UUID to find the playlist UUID
            if let Ok(user_uuid) = user::get_user_uuid_by_username(pool, &public_playlist.username).await {
                if let Ok(Some(pl)) = crate::db::schema::playlist::get_playlist_by_name(pool, &public_playlist.name, &user_uuid).await {
                    if pl.public {
                        playlist_uuid = Some(pl.uuid);
                        playlist_owner_username = Some(public_playlist.username);
                    }
                }
            }
        }
    }

    if let (Some(username), Some(uuid)) = (playlist_owner_username, playlist_uuid) {
        let songs = get_songs_in_playlist_info(pool, &uuid).await.unwrap_or_default();
        Ok((PlaylistDetails { name: playlist_name.to_string(), username }, songs))
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

pub async fn get_songs_in_playlist_info(pool: &DbPool, playlist_uuid: &Uuid) -> SQLResult<Vec<PlaylistSongInfo>> {
    let songs = match playlist_song_association::get_songs_in_playlist(pool, playlist_uuid).await {
        Ok(songs) => songs,
        Err(e) => {
            error!("{}",e.to_string());
            return Err(sqlx::Error::RowNotFound);
        }
    };
    let mut result = Vec::new();
    for song in songs {
        let artist_uuid = artist_song_association::get_artist_uuid_by_song_uuid(pool, &song.uuid).await?;
        let artist_name = if let Some(artist_uuid) = artist_uuid {
            match artist::get_artist_by_uuid(pool, &artist_uuid).await {
                Ok(artist) => artist.name,
                Err(_) => String::from("")
            }
        } else {
            String::from("")
        };
        result.push(PlaylistSongInfo {
            name: song.name,
            artist_name,
        });
    }
    Ok(result)
}

pub async fn reorder_songs_in_playlist(
    pool: &DbPool,
    session_id: &uuid::Uuid,
    playlist_name: &str,
    song_order: &[(String, String)],
) -> SQLResult<()> {
    let user_uuid = session::get_user_id_from_session_id(session_id, pool).await?;
    let playlist = match playlist::get_playlist_by_name(pool, &playlist_name.to_string(), &user_uuid).await? {
        Some(p) => p,
        None => return Err(sqlx::Error::InvalidArgument(format!("Playlist '{}' not found", playlist_name)))
    };
    for (position, (title, artist)) in song_order.iter().enumerate() {
        let song = match get_song_by_name_and_artist(pool, title, artist).await? {
            Some(s) => s,
            None => return Err(sqlx::Error::InvalidArgument(format!("Song '{}' by '{}' not found", title, artist)))
        };
        playlist_song_association::reorder_song_in_playlist(pool, &playlist.uuid, &song.uuid, position as i32).await?;
    }
    Ok(())
}*/

pub use song::fuzzy_search;