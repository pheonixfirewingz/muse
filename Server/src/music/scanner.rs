use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use lofty::prelude::*;
use lofty::probe::Probe;

use crate::db::{Database, DbError};

const COVER_CACHE_DIR: &str = "runtime/cache/covers";

pub struct MusicScanner {
    db: Arc<dyn Database>,
    music_dir: PathBuf,
    use_spotify: bool,
}

impl MusicScanner {
    pub fn new(db: Arc<dyn Database>, music_dir: impl Into<PathBuf>) -> Self {
        Self {
            db,
            music_dir: music_dir.into(),
            use_spotify: std::env::var("USE_SPOTIFY_API")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        }
    }

    /// Scan the music directory and register all audio files
    pub async fn scan_and_register(&self) -> Result<ScanResult, ScanError> {
        tracing::info!("Starting music directory scan: {:?}", self.music_dir);
        
        // Check if directory exists
        if !self.music_dir.exists() {
            return Err(ScanError::DirectoryNotFound(self.music_dir.clone()));
        }

        let mut result = ScanResult {
            total_files: 0,
            registered: 0,
            skipped: 0,
            updated: 0,
            removed: 0,
            errors: 0,
        };

        // Step 1: Clean up removed songs
        result.removed = self.cleanup_removed_songs().await?;

        // Step 2: Read directory entries
        let mut entries = match fs::read_dir(&self.music_dir).await {
            Ok(entries) => entries,
            Err(e) => return Err(ScanError::IoError(e)),
        };

        // Step 3: Process each file
        while let Some(entry) = entries.next_entry().await.map_err(ScanError::IoError)? {
            let path = entry.path();
            
            // Skip if not a file
            if !path.is_file() {
                continue;
            }

            result.total_files += 1;

            // Check if it's an audio file by extension
            if !is_audio_file(&path) {
                result.skipped += 1;
                continue;
            }

            // Register or update the song
            match self.register_or_update_song(&path).await {
                Ok(SongAction::Registered) => {
                    result.registered += 1;
                    tracing::info!("Registered song: {:?}", path.file_name());
                }
                Ok(SongAction::Updated) => {
                    result.updated += 1;
                    tracing::info!("Updated song: {:?}", path.file_name());
                }
                Ok(SongAction::Skipped) => {
                    result.skipped += 1;
                    tracing::debug!("Skipped song (already exists): {:?}", path.file_name());
                }
                Err(e) => {
                    result.errors += 1;
                    tracing::error!("Failed to register {:?}: {}", path.file_name(), e);
                }
            }
        }

        tracing::info!("Scan complete: {:?}", result);
        Ok(result)
    }

    /// Clean up songs whose files no longer exist
    async fn cleanup_removed_songs(&self) -> Result<usize, ScanError> {
        let all_songs = self.db.get_songs(0, 10000).await
            .map_err(ScanError::DatabaseError)?;
        
        let mut removed_count = 0;
        
        for song in all_songs {
            let file_path = PathBuf::from(&song.file_path);
            
            // Check if file still exists
            if !file_path.exists() {
                tracing::info!("Removing song with missing file: {} ({})", song.title, song.file_path);
                
                if let Err(e) = self.db.delete_song_by_id(&song.id).await {
                    tracing::error!("Failed to remove song {}: {}", song.id, e);
                } else {
                    removed_count += 1;
                }
            }
        }
        
        if removed_count > 0 {
            tracing::info!("Removed {} songs with missing files", removed_count);
        }
        
        Ok(removed_count)
    }

    /// Register a new song or update existing one
    async fn register_or_update_song(&self, path: &Path) -> Result<SongAction, ScanError> {
        // Extract metadata from the audio file
        let metadata = self.extract_metadata(path).await?;
        
        // Get or create the artist
        let artist = match self.db.get_artist_by_name(&metadata.artist).await {
            Ok(artist) => artist,
            Err(_) => {
                // Artist doesn't exist, try to create it
                match self.db.create_artist(&metadata.artist).await {
                    Ok(artist) => artist,
                    Err(_) => {
                        // If creation fails (e.g., concurrent creation), try to get "Unknown Artist"
                        self.db.get_artist_by_name("Unknown Artist").await
                            .map_err(ScanError::DatabaseError)?
                    }
                }
            }
        };

        // Convert path to string for storage
        let file_path = path
            .to_str()
            .ok_or_else(|| ScanError::InvalidFileName(path.to_path_buf()))?
            .to_string();

        // Check if a song with this title and artist already exists
        let existing_songs = self.db.get_songs_by_artist(&artist.id).await
            .map_err(ScanError::DatabaseError)?;
        
        // Find if there's a song with the same title (case-insensitive)
        if let Some(existing_song) = existing_songs.iter()
            .find(|s| s.title.eq_ignore_ascii_case(&metadata.title))
        {
            // Song exists - check if it's the same file or different format
            if existing_song.file_path == file_path {
                // Same file, check if we need to update metadata
                if metadata.album.is_some() && existing_song.album.is_none() ||
                   metadata.duration.is_some() && existing_song.duration.is_none() {
                    // Update metadata
                    self.db.update_song_metadata(
                        &existing_song.id,
                        metadata.album.as_deref(),
                        metadata.duration,
                        None
                    ).await.map_err(ScanError::DatabaseError)?;
                    
                    return Ok(SongAction::Updated);
                }
                return Ok(SongAction::Skipped);
            } else {
                // Different file path - this is a duplicate in different format
                // Skip registration to prevent duplicates
                tracing::warn!(
                    "Song '{}' by '{}' already exists with different file: {}. Skipping: {}",
                    metadata.title, metadata.artist, existing_song.file_path, file_path
                );
                return Ok(SongAction::Skipped);
            }
        }

        // Song doesn't exist, create it
        let song = self.db.create_song(&metadata.title, &artist.id, &file_path).await
            .map_err(ScanError::DatabaseError)?;

        // Download and cache cover image if available
        let cover_image_path = if let Some(cover_url) = &metadata.cover_url {
            match self.download_and_cache_cover(cover_url, &song.id).await {
                Ok(path) => {
                    tracing::info!("Downloaded cover for: {}", metadata.title);
                    Some(path)
                }
                Err(e) => {
                    tracing::warn!("Failed to download cover for {}: {}", metadata.title, e);
                    None
                }
            }
        } else {
            None
        };

        // Update with additional metadata if available
        if metadata.album.is_some() || metadata.duration.is_some() || cover_image_path.is_some() {
            self.db.update_song_metadata(
                &song.id,
                metadata.album.as_deref(),
                metadata.duration,
                cover_image_path.as_deref()
            ).await.map_err(ScanError::DatabaseError)?;
        }

        tracing::debug!("Created song: {} by {} (ID: {})", metadata.title, metadata.artist, song.id);
        Ok(SongAction::Registered)
    }

    /// Extract metadata from an audio file
    async fn extract_metadata(&self, path: &Path) -> Result<SongMetadata, ScanError> {
        // Try to extract metadata using lofty
        let tagged_file = Probe::open(path)
            .map_err(|e| ScanError::MetadataError(format!("Failed to open file: {}", e)))?
            .read()
            .map_err(|e| ScanError::MetadataError(format!("Failed to read metadata: {}", e)))?;

        let mut metadata = SongMetadata {
            title: String::new(),
            artist: String::from("Unknown Artist"),
            album: None,
            duration: None,
            cover_url: None,
        };

        // Extract duration
        let properties = tagged_file.properties();
        metadata.duration = Some(properties.duration().as_secs() as i32);

        // Try to get tags
        if let Some(tag) = tagged_file.primary_tag() {
            // Extract title
            metadata.title = tag.title()
                .map(|t| t.to_string())
                .unwrap_or_else(|| {
                    // Fallback to filename
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown Title")
                        .to_string()
                });

            // Extract artist
            if let Some(artist) = tag.artist() {
                metadata.artist = artist.to_string();
            }

            // Extract album
            metadata.album = tag.album().map(|a| a.to_string());
        } else {
            // No tags found, use filename as title
            metadata.title = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown Title")
                .to_string();
        }

        // If we still don't have artist info, try MusicBrainz (always enabled)
        if metadata.artist == "Unknown Artist" {
            if let Ok(enriched) = self.enrich_from_musicbrainz(&metadata.title).await {
                metadata.artist = enriched.artist;
                if metadata.album.is_none() {
                    metadata.album = enriched.album;
                }
                if metadata.cover_url.is_none() {
                    metadata.cover_url = enriched.cover_url;
                }
            }
        }

        // If Spotify is enabled and we still need more info, try Spotify
        if self.use_spotify && (metadata.artist == "Unknown Artist" || metadata.album.is_none()) {
            if let Ok(enriched) = self.enrich_from_spotify(&metadata.title, &metadata.artist).await {
                if metadata.artist == "Unknown Artist" {
                    metadata.artist = enriched.artist;
                }
                if metadata.album.is_none() {
                    metadata.album = enriched.album;
                }
                if metadata.cover_url.is_none() {
                    metadata.cover_url = enriched.cover_url;
                }
            }
        }

        Ok(metadata)
    }

    /// Download and cache cover image from URL
    async fn download_and_cache_cover(&self, cover_url: &str, song_id: &str) -> Result<String, ScanError> {
        // Ensure cache directory exists
        fs::create_dir_all(COVER_CACHE_DIR)
            .await
            .map_err(ScanError::IoError)?;

        // Download the image
        let client = reqwest::Client::new();
        let response = client.get(cover_url)
            .send()
            .await
            .map_err(|e| ScanError::MetadataError(format!("Failed to download cover: {}", e)))?;

        if !response.status().is_success() {
            return Err(ScanError::MetadataError(format!("Failed to download cover: HTTP {}", response.status())));
        }

        let image_bytes = response.bytes()
            .await
            .map_err(|e| ScanError::MetadataError(format!("Failed to read cover data: {}", e)))?;

        // Determine file extension from URL or content type
        let extension = if cover_url.contains(".jpg") || cover_url.contains(".jpeg") {
            "jpg"
        } else if cover_url.contains(".png") {
            "png"
        } else if cover_url.contains(".webp") {
            "webp"
        } else {
            "jpg" // default
        };

        // Save to cache with song ID as filename
        let cache_path = format!("{}/{}.{}", COVER_CACHE_DIR, song_id, extension);
        fs::write(&cache_path, &image_bytes)
            .await
            .map_err(ScanError::IoError)?;

        tracing::debug!("Downloaded and cached cover to: {}", cache_path);
        Ok(cache_path)
    }

    /// Try to enrich metadata from MusicBrainz API
    async fn enrich_from_musicbrainz(&self, title: &str) -> Result<SongMetadata, ScanError> {
        tracing::debug!("Querying MusicBrainz for: {}", title);
        
        let client = reqwest::Client::builder()
            .user_agent("Muse-Server/0.1.0")
            .build()
            .map_err(|e| ScanError::MetadataError(format!("HTTP client error: {}", e)))?;

        let url = format!(
            "https://musicbrainz.org/ws/2/recording/?query={}&fmt=json&limit=1",
            urlencoding::encode(title)
        );

        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| ScanError::MetadataError(format!("MusicBrainz request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ScanError::MetadataError("MusicBrainz API returned error".to_string()));
        }

        let data: serde_json::Value = response.json()
            .await
            .map_err(|e| ScanError::MetadataError(format!("Failed to parse response: {}", e)))?;

        // Extract first recording if available
        if let Some(recordings) = data["recordings"].as_array() {
            if let Some(recording) = recordings.first() {
                let artist = recording["artist-credit"]
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|ac| ac["name"].as_str())
                    .unwrap_or("Unknown Artist")
                    .to_string();

                let album = recording["releases"]
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|rel| rel["title"].as_str())
                    .map(|s| s.to_string());

                // Try to get cover art from Cover Art Archive
                let cover_url = if let Some(releases) = recording["releases"].as_array() {
                    if let Some(release) = releases.first() {
                        if let Some(release_id) = release["id"].as_str() {
                            self.get_cover_art_from_musicbrainz(&client, release_id).await.ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                return Ok(SongMetadata {
                    title: title.to_string(),
                    artist,
                    album,
                    duration: None,
                    cover_url,
                });
            }
        }

        Err(ScanError::MetadataError("No results from MusicBrainz".to_string()))
    }

    /// Get cover art URL from Cover Art Archive
    async fn get_cover_art_from_musicbrainz(&self, client: &reqwest::Client, release_id: &str) -> Result<String, ScanError> {
        let cover_url = format!("https://coverartarchive.org/release/{}", release_id);
        
        let response = client.get(&cover_url)
            .send()
            .await
            .map_err(|e| ScanError::MetadataError(format!("Cover Art Archive request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ScanError::MetadataError("No cover art found".to_string()));
        }

        let data: serde_json::Value = response.json()
            .await
            .map_err(|e| ScanError::MetadataError(format!("Failed to parse cover art response: {}", e)))?;

        // Get the front cover image URL
        if let Some(images) = data["images"].as_array() {
            for image in images {
                if image["front"].as_bool().unwrap_or(false) {
                    if let Some(url) = image["image"].as_str() {
                        return Ok(url.to_string());
                    }
                }
            }
            // If no front cover, use first available image
            if let Some(first_image) = images.first() {
                if let Some(url) = first_image["image"].as_str() {
                    return Ok(url.to_string());
                }
            }
        }

        Err(ScanError::MetadataError("No cover art images available".to_string()))
    }

    /// Try to enrich metadata from Spotify API (requires SPOTIFY_CLIENT_ID and SPOTIFY_CLIENT_SECRET)
    async fn enrich_from_spotify(&self, title: &str, artist: &str) -> Result<SongMetadata, ScanError> {
        tracing::debug!("Querying Spotify for: {} by {}", title, artist);
        
        // Get Spotify credentials from environment
        let client_id = std::env::var("SPOTIFY_CLIENT_ID")
            .map_err(|_| ScanError::MetadataError("SPOTIFY_CLIENT_ID not set".to_string()))?;
        let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
            .map_err(|_| ScanError::MetadataError("SPOTIFY_CLIENT_SECRET not set".to_string()))?;

        // Get access token
        let client = reqwest::Client::new();
        let auth_response = client
            .post("https://accounts.spotify.com/api/token")
            .form(&[("grant_type", "client_credentials")])
            .basic_auth(client_id, Some(client_secret))
            .send()
            .await
            .map_err(|e| ScanError::MetadataError(format!("Spotify auth failed: {}", e)))?;

        if !auth_response.status().is_success() {
            return Err(ScanError::MetadataError("Spotify authentication failed".to_string()));
        }

        let auth_data: serde_json::Value = auth_response.json()
            .await
            .map_err(|e| ScanError::MetadataError(format!("Failed to parse Spotify auth response: {}", e)))?;

        let access_token = auth_data["access_token"]
            .as_str()
            .ok_or_else(|| ScanError::MetadataError("No access token in Spotify response".to_string()))?;

        // Search for track
        let query = if artist != "Unknown Artist" {
            format!("track:{} artist:{}", title, artist)
        } else {
            format!("track:{}", title)
        };

        let search_url = format!(
            "https://api.spotify.com/v1/search?q={}&type=track&limit=1",
            urlencoding::encode(&query)
        );

        let search_response = client
            .get(&search_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ScanError::MetadataError(format!("Spotify search failed: {}", e)))?;

        if !search_response.status().is_success() {
            return Err(ScanError::MetadataError("Spotify search API returned error".to_string()));
        }

        let data: serde_json::Value = search_response.json()
            .await
            .map_err(|e| ScanError::MetadataError(format!("Failed to parse Spotify response: {}", e)))?;

        // Extract first track if available
        if let Some(tracks) = data["tracks"]["items"].as_array() {
            if let Some(track) = tracks.first() {
                let spotify_artist = track["artists"]
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|a| a["name"].as_str())
                    .unwrap_or("Unknown Artist")
                    .to_string();

                let spotify_album = track["album"]["name"]
                    .as_str()
                    .map(|s| s.to_string());

                // Get album cover URL (largest available)
                let cover_url = track["album"]["images"]
                    .as_array()
                    .and_then(|images| images.first())
                    .and_then(|img| img["url"].as_str())
                    .map(|s| s.to_string());

                return Ok(SongMetadata {
                    title: title.to_string(),
                    artist: spotify_artist,
                    album: spotify_album,
                    duration: None,
                    cover_url,
                });
            }
        }

        Err(ScanError::MetadataError("No results from Spotify".to_string()))
    }
}

/// Check if a file is an audio file based on extension
fn is_audio_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac" | "opus" | "wma" | "alac"
        )
    } else {
        false
    }
}

#[derive(Debug, Clone)]
pub struct SongMetadata {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration: Option<i32>,
    pub cover_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub total_files: usize,
    pub registered: usize,
    pub updated: usize,
    pub skipped: usize,
    pub removed: usize,
    pub errors: usize,
}

enum SongAction {
    Registered,
    Updated,
    Skipped,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Music directory not found: {0}")]
    DirectoryNotFound(PathBuf),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbError),
    
    #[error("Invalid file name: {0}")]
    InvalidFileName(PathBuf),
    
    #[error("Metadata error: {0}")]
    MetadataError(String),
}
