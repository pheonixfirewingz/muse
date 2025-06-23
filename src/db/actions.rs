use once_cell::sync::Lazy;
use crate::db::util::sql_share::SQLResult;
use crate::db::{session, user, DbPool};
use serde::Serialize;
use tokio::sync::Mutex;
use tracing::{error, info};
use uuid::Uuid;
use crate::db::schema::{artist, artist_song_association, song, playlist, playlist_song_association};
use crate::db::schema::artist::Artist;
use crate::db::schema::song::Song;
use crate::db::schema::playlist::Playlist;

#[derive(Debug,Serialize,Clone)]
pub struct SongInfo{
    song_name: String,
    artist_name: String,
    format: String,
}

impl SongInfo{
    pub fn new(song_name: String, artist_name: String, format: String) -> Self{
        Self{song_name, artist_name, format}
    }
    
    pub fn get_artist_name(&self) -> &String{
        &self.artist_name
    }
    
    pub fn get_song_name(&self) -> &String{
        &self.song_name
    }
}

#[derive(Debug,Serialize,Clone)]
pub struct ArtistInfo{
    artist_name: String,
}

#[derive(Debug,Serialize,Clone)]
pub struct PlaylistInfo {
    name: String
}

#[derive(Debug,Serialize,Clone)]
pub struct PublicPlaylistInfo {
    pub name: String,
    pub username: String,
}

#[derive(Debug,Serialize,Clone)]
pub struct PlaylistDetails {
    pub name: String,
    pub username: String,
}

#[derive(Debug,Serialize,Clone)]
pub struct PlaylistSongInfo {
    pub name: String,
    pub artist_name: String,
}

impl ArtistInfo {
    pub fn new(artist_name: String) -> Self{
        Self{artist_name}
    }

    pub fn get_name(&self) -> &String {
        &self.artist_name
    }
}

#[derive(Debug,Serialize,Clone)]
pub struct UserInfo {
    pub username: String,
    pub email: String
}

pub async fn get_user_info_from_session_id(pool: &DbPool, session_id:&Uuid) -> Option<UserInfo>{
    let user_id = match session::get_user_id_from_session_id(session_id, pool).await {
        Ok(user_id) => user_id,
        Err(_) => return None
    };
    
    let user = match user::get_user_by_uuid(pool,&user_id).await {
        Ok(user) => user,
        Err(_) => return None
    };
    
    Some(UserInfo { username: user.username.to_string(), email: user.email.to_string() })
}

pub async fn get_db_song_info(pool: &DbPool, ascending: bool) -> SQLResult<Vec<SongInfo>>{
    let mut song_list: Vec<SongInfo> = Vec::new();
    if let Some(artist_list) = artist::get_artists(pool, ascending).await {
        for artist in artist_list {
            let songs = song::get_song_names_by_artist(pool, *artist.get_id(), ascending).await?;
            for song_name in songs {
                // Fetch the song to get the format
                let song_objs = song::get_songs_by_name(pool, &song_name).await?;
                for song in song_objs {
                    if artist_song_association::dose_song_belong_to_artist(pool, artist.get_id(), song.get_id()).await? {
                        song_list.push(SongInfo::new(song.name.clone(), artist.name.clone(), song.format.clone()));
                    }
                }
            }
        }
        Ok(song_list)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

pub async fn get_db_artist_info(pool: &DbPool, ascending: bool) -> SQLResult<Vec<ArtistInfo>> {
    let mut list: Vec<ArtistInfo> = Vec::new();
    if let Some(artist_list) = artist::get_artists(pool, ascending).await {
        for artist in artist_list { 
            list.push(ArtistInfo::new(artist.name))
        }
        Ok(list)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

static REG_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
pub async fn register_song(pool: &DbPool, song_name: String, artist_name: String, song_path: &String) -> SQLResult<bool>{
    let _guard = REG_MUTEX.lock().await;
    if !artist::has_table_got_artist_name(pool,&artist_name).await? {
        let artist: Artist = Artist::new(&artist_name);
        match artist::add_artist(pool,&artist).await {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to add artist: {}", e);
            }
        }
    }
    let has_song = song::has_table_got_song_name_by_artist(pool,&artist_name,&song_name).await?;
    if !has_song {
        // Detect format from file extension
        let format = song_path.split('.').last().unwrap_or("mp3").to_lowercase();
        let song = Song::new(song_name,None,song_path.clone(),format);
        song::add_song(pool,&song).await?;
        let artist = artist::get_artist_by_name(pool,&artist_name).await?;
        artist_song_association::add_artist_song_association(pool,artist.get_id(),song.get_id()).await?;
        Ok(true)
    } else {
        info!("the database has a song named {} by artist {}", song_name, artist_name);
        Ok(false)
    }
}

pub async fn get_song_file_path(pool: &DbPool,song_name: &String, artist_name: &String, preferred_formats: Option<&[&str]>) -> SQLResult<(String, String)> {
    let songs = song::get_songs_by_name(pool,song_name).await?;
    let artist = artist::get_artist_by_name(pool,artist_name).await?;
    // Try to find the best format
    if let Some(formats) = preferred_formats {
        for &fmt in formats {
            for song in &songs {
                if song.format == fmt && artist_song_association::dose_song_belong_to_artist(pool,artist.get_id(),song.get_id()).await? {
                    return Ok((song.file_path.clone(), song.format.clone()));
                }
            }
        }
    }
    // Fallback: return any format
    for song in songs {
        if artist_song_association::dose_song_belong_to_artist(pool,artist.get_id(),song.get_id()).await? {
            return Ok((song.file_path.clone(), song.format.clone()));
        }
    }
    Err(sqlx::Error::InvalidArgument(format!("song {song_name} not found by {artist_name}").to_string()))
}

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
    let songs = crate::db::schema::playlist_song_association::get_songs_in_playlist(pool, playlist_uuid).await?;
    Ok(songs.into_iter().map(|song| PlaylistSongInfo {
        name: song.name,
        artist_name: song.description.unwrap_or_default(),
    }).collect())
}