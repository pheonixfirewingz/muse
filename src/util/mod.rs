pub mod async_sync;
use std::path::Path;
use crate::db::DbPool;
use crate::db::schema::artist::{create_artist_entry, get_artist_by_name, Artist};
use crate::db::schema::song::{create_song_entry, get_song_by_name, Song};

async fn check_and_register_song(pool: &DbPool, file_path: &str) -> bool {
    let path = Path::new(file_path);
    let song_name = match path.file_stem().and_then(|n| n.to_str()) {
        Some(name) => name.to_string(),
        None => {
            println!("Could not extract song name from path: {}", file_path);
            return false;
        }
    };

    let artist = match path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()) {
        Some(name) => name.to_string(),
        None => {
            println!("Could not extract artist from path: {}", file_path);
            return false;
        }
    };

    // if the artist name has any non-alphanumeric characters or whitespace but '_' is allowed, reject it
    if artist.chars().any(|c| !c.is_alphanumeric() && c != '_') {
        println!("Artist name '{}' contains non-alphanumeric characters or whitespace. only [a-zA-Z0-9_] are allowed", artist);
        return false;
    }
    
    if get_artist_by_name(pool, &artist).await.is_none() {
        // Artist isn't found, register it
        let new_artist = Artist::new_auto_id(artist.clone());
        create_artist_entry(pool, &new_artist).await;
    }

    let description = Some(format!("Song by {}", artist));

    // Check if the song already exists
    if let Some(_) = get_song_by_name(pool, &song_name).await {
        return true;
    }
    
    // if the song name has any non-alphanumeric characters or whitespace but '_' is allowed, reject it
    if song_name.chars().any(|c| !c.is_alphanumeric() && c != '_') {
        println!("Song name '{}' contains non-alphanumeric characters or whitespace. only [a-zA-Z0-9_] are allowed", song_name);
        return false;
    }

    // Song not found, register it
    let new_song = Song::new_auto_id(song_name.clone(), description, file_path.to_string());

    // Create the entry in the database
    create_song_entry(pool, &new_song).await;

    println!("New song registered: '{}' by '{}'", song_name, artist);
    true
}

pub async fn scan_and_register_songs(pool: &DbPool, dir_path: &str) -> usize {
    let mut registered_count = 0;
    let music_dir = std::path::Path::new(dir_path);

    // Check if directory exists
    if !music_dir.is_dir() {
        println!("Music directory does not exist: {}", dir_path);
        return registered_count;
    }

    // Read all entries in the music directory (artist directories)
    if let Ok(artists) = std::fs::read_dir(music_dir) {
        for artist_result in artists {
            if let Ok(artist_entry) = artist_result {
                let artist_path = artist_entry.path();

                // Skip if not a directory
                if !artist_path.is_dir() {
                    continue;
                }

                // Read all entries in the artist directory (song files)
                if let Ok(songs) = std::fs::read_dir(&artist_path) {
                    for song_result in songs {
                        if let Ok(song_entry) = song_result {
                            let song_path = song_entry.path();

                            // Skip if not an MP3 file
                            if !song_path.is_file() || song_path.extension().and_then(|e| e.to_str()) != Some("mp3") {
                                continue;
                            }

                            // Register the song
                            if check_and_register_song(pool, song_path.to_str().unwrap_or("")).await {
                                registered_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Registered {} songs in total", registered_count);
    registered_count
}