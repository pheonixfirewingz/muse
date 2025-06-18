use serde::{Deserialize, Serialize};
use tower_cookies::cookie::time;
use tracing::debug;
use crate::db::thirdparty::cache::{load_cache, store_cache, CacheError, Cached};

mod music_brainz;
mod spotify;
pub mod cache;


#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct ArtistData {
    pub name: String,
    pub picture_url: String,
    pub genres: Vec<String>,
}


#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct SongData {
    pub name: String,
    pub artists: Vec<String>,
    pub album_name: String,
    pub album_art_url: String,
    pub album_type: String,
}


pub async fn get_artist_data(artist_name: &str) -> Result<Option<ArtistData>, CacheError> {
    match load_cache::<ArtistData>(artist_name,"artist/data").await? {
        Some(Cached::Found(data)) => {
            debug!(%artist_name, "Artist cache hit (found)");
            return Ok(Some(data));
        }
        Some(Cached::NotFound) => {
            debug!(%artist_name, "Artist cache hit (not found)");
            return Ok(None);
        }
        None => {
            debug!(%artist_name, "Artist cache miss");
        }
    }
    if spotify::is_spotify_enabled() {
        match spotify::get_artist_data(artist_name).await {
            Ok(Some(data)) => {
                store_cache(&Cached::Found(data.clone()), artist_name,"artist/data").await?;
                Ok(Some(data))
            }
            Ok(None) => {
                get_music_brains_artist(artist_name).await
            }
            Err(str) => Err(CacheError::Other(str))
        }
    } else { 
        get_music_brains_artist(artist_name).await
    }
}


pub async fn get_song_data(song_name: &str, artist_name: Option<&str>) -> Result<Option<SongData>, CacheError> {
    // Create a cache key that includes both song name and artist (if provided)
    let cache_key = if let Some(artist) = artist_name {
        format!("{}___{}", song_name, artist)
    } else {
        song_name.to_string()
    };

    match load_cache::<SongData>(&cache_key, "song/data").await? {
        Some(Cached::Found(data)) => {
            debug!(song_name, ?artist_name, "Song cache hit (found)");
            return Ok(Some(data));
        }
        Some(Cached::NotFound) => {
            debug!(song_name, ?artist_name, "Song cache hit (not found)");
            return Ok(None);
        }
        None => {
            debug!(song_name, ?artist_name, "Song cache miss");
        }
    }

    if spotify::is_spotify_enabled() {
        match spotify::get_song_data(song_name, artist_name).await {
            Ok(Some(data)) => {
                store_cache(&Cached::Found(data.clone()), &cache_key, "song/data").await?;
                Ok(Some(data))
            }
            Ok(None) => {
                get_music_brains_song(song_name, artist_name,&cache_key).await
            }
            Err(str) => Err(CacheError::Other(str))
        }
    } else {
        get_music_brains_song(song_name, artist_name,&cache_key).await
    }
}

#[warn(unused)]
pub async fn get_song_data_no_cache(song_name: &str, artist_name: Option<&str>) -> Result<Option<SongData>, CacheError> {
    if spotify::is_spotify_enabled() {
        match spotify::get_song_data(song_name, artist_name).await {
            Ok(data) => Ok(data),
            Err(str) => Err(CacheError::Other(str))
        }
    } else {
        match music_brainz::get_song_data(song_name, artist_name).await {
            Ok(data) => Ok(data),
            Err(str) => Err(CacheError::Other(str))
        }
    }
}

async fn get_music_brains_song(song_name: &str, artist_name: Option<&str>,cache_key:&String) -> Result<Option<SongData>, CacheError>{
    match music_brainz::get_song_data(song_name, artist_name).await {
        Ok(Some(data)) => {
            store_cache(&Cached::Found(data.clone()), cache_key, "song/data").await?;
            Ok(Some(data))
        }
        Ok(None) => {
            store_cache(&Cached::<SongData>::NotFound, cache_key, "song/data").await?;
            Ok(None)
        }
        Err(str) => Err(CacheError::Other(str))
    }
}

async fn get_music_brains_artist(artist_name: &str) -> Result<Option<ArtistData>, CacheError>{
    match music_brainz::get_artist_data(artist_name).await {
        Ok(Some(data)) => {
            store_cache(&Cached::Found(data.clone()), artist_name,"artist/data").await?;
            Ok(Some(data))
        }
        Ok(None) => {
            store_cache(&Cached::<ArtistData>::NotFound, artist_name,"artist/data").await?;
            Ok(None)
        }
        Err(str) => Err(CacheError::Other(str))
    }
}


pub async fn get_artist_image_url(artist_name: &str) -> Result<Option<String>, CacheError> {
    // Get artist data to find the image URL
    let artist_data = match get_artist_data(artist_name).await? {
        Some(data) => data,
        None => return Ok(None),
    };
    //TODO: we should cache this images as well
    Ok(Some(artist_data.picture_url.clone()))
}

pub async fn get_song_image_url(artist_name: &str,song_name:&str) -> Result<Option<String>, CacheError> {
    // Get artist data to find the image URL
    let artist_data = match get_song_data(song_name,Some(artist_name)).await? {
        Some(data) => data,
        None => return Ok(None),
    };
    //TODO: we should cache this images as well
    Ok(Some(artist_data.album_art_url.clone()))
}