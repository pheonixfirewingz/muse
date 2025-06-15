use std::path::Path;
use crate::db::DbPool;
use crate::db::schema::artist::{add_artist_song_association, create_artist_entry, dose_artist_have_a_song_by_name, get_artist_by_name, Artist};
use crate::db::schema::song::{create_song_entry, get_song_by_name, Song};
use tracing::{error, info, warn, debug};

async fn check_and_register_song(pool: &DbPool, file_path: &str) -> bool {
    debug!("Checking and registering song from path: {}", file_path);
    
    let path = Path::new(file_path);
    let song_name = match path.file_stem().and_then(|n| n.to_str()) {
        Some(name) => name.to_string(),
        None => {
            error!("Failed to extract song name from path: {}", file_path);
            return false;
        }
    };

    let artist = match path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()) {
        Some(name) => name.to_string(),
        None => {
            error!("Failed to extract artist from path: {}", file_path);
            return false;
        }
    };

    if artist.chars().any(|c| c.is_uppercase() || (!c.is_alphanumeric() && c != '_')) {
        warn!("Invalid artist name format '{}': contains uppercase letters or invalid characters", artist);
        return false;
    }

    if get_artist_by_name(pool, &artist).await.is_none() {
        info!("Registering new artist: {}", artist);
        let new_artist = Artist::new(artist.clone());
        create_artist_entry(pool, &new_artist).await;
    }

    let description = Some(format!("Song by {}", artist));
    if song_name.chars().any(|c| c.is_uppercase() || (!c.is_alphanumeric() && c != '_')) {
        warn!("Invalid song name format '{}': contains uppercase letters or invalid characters", song_name);
        return false;
    }
    
    if dose_artist_have_a_song_by_name(pool, &artist, &song_name).await {
        debug!("Song '{}' by '{}' already exists", song_name, artist);
        return false;
    }

    info!("Registering new song '{}' by '{}'", song_name, artist);
    let new_song = Song::new(song_name.clone(), description, file_path.to_string());
    create_song_entry(pool, &new_song).await;
    
    let artist_id = get_artist_by_name(pool, &artist).await.unwrap().id.unwrap();
    let song_id = get_song_by_name(pool, &song_name).await.unwrap().id.unwrap();
    add_artist_song_association(pool, &artist_id, &song_id).await;

    info!("Successfully registered song '{}' by '{}'", song_name, artist);
    true
}

pub async fn scan_and_register_songs(pool: &DbPool, dir_path: &str) -> usize {
    info!("Starting music directory scan: {}", dir_path);
    let mut registered_count = 0;
    let music_dir = std::path::Path::new(dir_path);

    if !music_dir.is_dir() {
        error!("Music directory does not exist: {}", dir_path);
        return registered_count;
    }

    match std::fs::read_dir(music_dir) {
        Ok(artists) => {
            for artist_result in artists {
                match artist_result {
                    Ok(artist_entry) => {
                        let artist_path = artist_entry.path();
                        if !artist_path.is_dir() {
                            debug!("Skipping non-directory entry: {:?}", artist_path);
                            continue;
                        }

                        debug!("Scanning artist directory: {:?}", artist_path);
                        match std::fs::read_dir(&artist_path) {
                            Ok(songs) => {
                                for song_result in songs {
                                    match song_result {
                                        Ok(song_entry) => {
                                            let song_path = song_entry.path();
                                            if !song_path.is_file() || song_path.extension().and_then(|e| e.to_str()) != Some("mp3") {
                                                debug!("Skipping non-MP3 file: {:?}", song_path);
                                                continue;
                                            }

                                            if let Some(path_str) = song_path.to_str() {
                                                if check_and_register_song(pool, path_str).await {
                                                    registered_count += 1;
                                                }
                                            } else {
                                                error!("Invalid path encoding: {:?}", song_path);
                                            }
                                        }
                                        Err(e) => error!("Error reading song entry: {}", e),
                                    }
                                }
                            }
                            Err(e) => error!("Error reading artist directory '{}': {}", artist_path.display(), e),
                        }
                    }
                    Err(e) => error!("Error reading artist entry: {}", e),
                }
            }
        }
        Err(e) => error!("Error reading music directory '{}': {}", dir_path, e),
    }

    info!("Scan completed. Registered {} songs in total", registered_count);
    registered_count
}