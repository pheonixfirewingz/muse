use once_cell::sync::Lazy;
use crate::db::schema::sql_share::SQLResult;
use crate::db::{schema, DbPool};
use serde::Serialize;
use tokio::sync::Mutex;
use tracing::{error, info};
use crate::db::schema::{artist, artist_song_association, song};
use crate::db::schema::artist::Artist;
use crate::db::schema::song::Song;

#[derive(Debug,Serialize)]
pub struct SongInfo{
    song_name: String,
    artist_name: String,
}

impl SongInfo{
    pub fn new(song_name: String, artist_name: String) -> Self{
        Self{song_name, artist_name}
    }
    
    pub fn get_artist_name(&self) -> &String{
        &self.artist_name
    }
}

#[derive(Debug,Serialize)]
pub struct ArtistInfo{
    artist_name: String,
}

impl ArtistInfo{
    pub fn new(artist_name: String) -> Self{
        Self{artist_name}
    }
}

pub async fn get_db_song_info(pool: &DbPool, ascending: bool) -> SQLResult<Vec<SongInfo>>{
    let mut song_list: Vec<SongInfo> = Vec::new();
    if let Some(artist_list) = artist::get_artists(pool, ascending).await {
        for artist in artist_list {
            let songs = schema::song::get_song_names_by_artist(pool, *artist.get_id(), ascending).await?;
            for song in songs {
                song_list.push(SongInfo::new(song,artist.name.clone()))
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
        let song = Song::new(song_name,None,song_path.clone());
        song::add_song(pool,&song).await?;
        let artist = artist::get_artist_by_name(pool,&artist_name).await?;
        artist_song_association::add_artist_song_association(pool,artist.get_id(),song.get_id()).await?;
        Ok(true)
    } else {
        info!("the database has a song named {} by artist {}", song_name, artist_name);
        Ok(false)
    }
}

pub async fn get_song_file_path(pool: &DbPool,song_name: &String, artist_name: &String) -> SQLResult<String> {
    let songs = song::get_songs_by_name(pool,song_name).await?;
    let artist = artist::get_artist_by_name(pool,artist_name).await?;
    for song in songs {
        if artist_song_association::dose_song_belong_to_artist(pool,artist.get_id(),song.get_id()).await? {
            return Ok(song.file_path.clone());
        }
    }
    Err(sqlx::Error::InvalidArgument(format!("song {song_name} not found by {artist_name}").to_string()))
}