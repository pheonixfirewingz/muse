use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use crate::util::cache::{load_cache, store_cache, CacheEntry, CacheError};
use crate::util::cache::bson_ext::BsonFormat;
use reqwest::Client;
use crate::util::cache::img_ext::ImageFormat;

mod music_brainz;
mod spotify;

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct ArtistData {
    pub name: String,
    pub picture_url: String,
    pub genres: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct SongData {
    pub name: String,
    pub artists: Vec<String>,
    pub album_name: String,
    pub album_art_url: String,
    pub album_type: String,
}

/// Helper to download image bytes from a URL
async fn download_image_bytes(url: &str) -> Result<Vec<u8>, CacheError> {
    let client = Client::new();
    let resp = client.get(url).send().await.map_err(|e| CacheError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to fetch image: {}", e))))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(CacheError::InvalidFormat(format!("Image download failed with status: {}", status)));
    }
    let bytes = resp.bytes().await.map_err(|e| CacheError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to read image bytes: {}", e))))?;
    Ok(bytes.to_vec())
}

pub async fn get_artist_data(artist_name: &str) -> Result<Option<ArtistData>, CacheError> {
    match load_cache::<ArtistData, BsonFormat>(artist_name, "artist/data").await? {
        Some(CacheEntry::Hit { data, .. }) => {
            debug!("Artist cache hit (found) {}", artist_name);
            return Ok(Some(data));
        }
        Some(CacheEntry::Miss) | None => {
            debug!("Artist cache miss {}", artist_name);
        }
    }

    fetch_fresh_artist_data(artist_name).await
}

async fn fetch_fresh_artist_data(artist_name: &str) -> Result<Option<ArtistData>, CacheError> {
    if spotify::is_spotify_enabled() {
        match spotify::get_artist_data(artist_name).await {
            Ok(Some(data)) => {
                let cache_entry = CacheEntry::new_hit(data.clone(), 1024);
                store_cache::<ArtistData, BsonFormat>(&cache_entry, artist_name, "artist/data").await?;
                Ok(Some(data))
            }
            Ok(None) => get_music_brainz_artist(artist_name).await,
            Err(err) => Err(CacheError::Unsupported(err)),
        }
    } else {
        get_music_brainz_artist(artist_name).await
    }
}

pub async fn get_song_data(song_name: &str, artist_name: Option<&str>) -> Result<Option<SongData>, CacheError> {
    let cache_key = if let Some(artist) = artist_name {
        format!("{}___{}", song_name, artist)
    } else {
        song_name.to_string()
    };

    match load_cache::<SongData, BsonFormat>(&cache_key, "song/data").await? {
        Some(CacheEntry::Hit { data, .. }) => {
            debug!("Song cache hit (found) {}:{}", song_name, artist_name.unwrap_or("none"));
            return Ok(Some(data));
        }
        Some(CacheEntry::Miss) | None => {
            debug!("Song cache miss {}:{}", song_name, artist_name.unwrap_or("none"));
        }
    }

    fetch_fresh_song_data(song_name, &artist_name, &cache_key).await
}

async fn fetch_fresh_song_data(song_name: &str, artist_name: &Option<&str>, cache_key: &str) -> Result<Option<SongData>, CacheError> {
    if spotify::is_spotify_enabled() {
        match spotify::get_song_data(song_name, artist_name).await {
            Ok(Some(data)) => {
                let cache_entry = CacheEntry::new_hit(data.clone(), 1024);
                store_cache::<SongData, BsonFormat>(&cache_entry, cache_key, "song/data").await?;
                Ok(Some(data))
            }
            Ok(None) => get_music_brainz_song(song_name, artist_name, cache_key).await,
            Err(err) => Err(CacheError::Unsupported(err)),
        }
    } else {
        get_music_brainz_song(song_name, artist_name, cache_key).await
    }
}

async fn get_music_brainz_song(song_name: &str, artist_name: &Option<&str>, cache_key: &str) -> Result<Option<SongData>, CacheError> {
    match music_brainz::get_song_data(song_name, artist_name).await {
        Ok(Some(data)) => {
            let cache_entry = CacheEntry::new_hit(data.clone(), 1024);
            store_cache::<SongData, BsonFormat>(&cache_entry, cache_key, "song/data").await?;
            Ok(Some(data))
        }
        Ok(None) => {
            debug!("MusicBrainz song not found - not caching negative result {}:{}", song_name, artist_name.unwrap_or("none"));
            Ok(None)
        }
        Err(err) => Err(CacheError::Unsupported(err)),
    }
}

async fn get_music_brainz_artist(artist_name: &str) -> Result<Option<ArtistData>, CacheError> {
    match music_brainz::get_artist_data(artist_name).await {
        Ok(Some(data)) => {
            let cache_entry = CacheEntry::new_hit(data.clone(), 1024);
            store_cache::<ArtistData, BsonFormat>(&cache_entry, artist_name, "artist/data").await?;
            Ok(Some(data))
        }
        Ok(None) => {
            debug!("MusicBrainz artist not found - not caching negative result {}", artist_name);
            Ok(None)
        }
        Err(err) => Err(CacheError::Unsupported(err)),
    }
}

/// Fetch and cache artist image bytes, returning the bytes if successful
pub async fn fetch_and_cache_artist_image(artist_name: &str) -> Result<Option<Vec<u8>>, CacheError> {
    // 1. Try cache
    if let Some(CacheEntry::Hit { data, .. }) = load_cache::<Vec<u8>, ImageFormat>(artist_name, "artist/images").await? {
        return Ok(Some(data));
    }
    // 2. Get artist data for image URL
    let artist_data = match get_artist_data(artist_name).await? {
        Some(data) => data,
        None => return Ok(None),
    };
    if artist_data.picture_url.is_empty() {
        return Ok(None);
    }
    // 3. Download image
    let image_bytes = match download_image_bytes(&artist_data.picture_url).await {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!("Failed to download artist image: {}", e);
            return Ok(None);
        }
    };
    // 4. Store in cache
    let cache_entry = CacheEntry::new_hit(image_bytes.clone(), image_bytes.len() as u64);
    if let Err(e) = store_cache::<Vec<u8>, ImageFormat>(&cache_entry, artist_name, "artist/images").await {
        warn!("Failed to cache artist image: {}", e);
    }
    // 5. Return bytes
    Ok(Some(image_bytes))
}

/// Fetch and cache song image bytes, returning the bytes if successful
pub async fn fetch_and_cache_song_image(artist_name: &str, song_name: &str) -> Result<Option<Vec<u8>>, CacheError> {
    let cache_key = format!("{}___{}", song_name, artist_name);
    // 1. Try cache
    if let Some(CacheEntry::Hit { data, .. }) = load_cache::<Vec<u8>, ImageFormat>(&cache_key, "song/images").await? {
        return Ok(Some(data));
    }
    // 2. Get song data for image URL
    let song_data = match get_song_data(song_name, Some(artist_name)).await? {
        Some(data) => data,
        None => return Ok(None),
    };
    if song_data.album_art_url.is_empty() {
        return Ok(None);
    }
    // 3. Download image
    let image_bytes = match download_image_bytes(&song_data.album_art_url).await {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!("Failed to download song image: {}", e);
            return Ok(None);
        }
    };
    // 4. Store in cache
    let cache_entry = CacheEntry::new_hit(image_bytes.clone(), image_bytes.len() as u64);
    if let Err(e) = store_cache::<Vec<u8>, ImageFormat>(&cache_entry, &cache_key, "song/images").await {
        warn!("Failed to cache song image: {}", e);
    }
    // 5. Return bytes
    Ok(Some(image_bytes))
}
