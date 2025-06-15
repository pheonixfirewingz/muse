use serde::Serialize;
use sqlx::FromRow;

use crate::db::DbPool;
use crate::db::schema::song::{get_song_by_name, Song};

#[derive(FromRow, Serialize)]
pub struct Artist {
    pub id: Option<String>,
    pub name: String,
}

impl Artist {
    /// Constructs a new Artist with no manually assigned ID.
    pub fn new(name: String) -> Self {
        Self {
            id: None,
            name: name.to_lowercase().trim().to_string().replace(" ", "_"),
        }
    }

    pub fn clean_for_web_view(&self) -> Self {
        let name = self.name.replace('_', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        Self {
            id: self.id.clone(),
            name,
        }
    }
}

pub async fn create_artists_table_if_not_exists(pool: &DbPool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS artists (
            id TEXT PRIMARY KEY NOT NULL DEFAULT (
                lower(
                    hex(randomblob(4)) || '-' ||
                    hex(randomblob(2)) || '-' ||
                    '4' || substr(hex(randomblob(2)), 2) || '-' ||
                    substr('89ab', abs(random() % 4) + 1, 1) || substr(hex(randomblob(2)), 2) || '-' ||
                    hex(randomblob(6))
                )
            ),
            name TEXT NOT NULL
        )"
    )
        .execute(pool)
        .await
        .unwrap();
}

pub async fn create_artists_songs_table_if_not_exists(pool: &DbPool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS artists_songs (
            artist_id TEXT NOT NULL,
            song_id TEXT NOT NULL,
            PRIMARY KEY (artist_id, song_id),
            FOREIGN KEY (artist_id) REFERENCES artists(id),
            FOREIGN KEY (song_id) REFERENCES songs(id)
        )"
    )
        .execute(pool)
        .await
        .unwrap();
}

pub async fn add_artist_song_association(pool: &DbPool, artist_id: &str, song_id: &str) {
    sqlx::query("INSERT INTO artists_songs (artist_id, song_id) VALUES (?, ?)")
        .bind(artist_id)
        .bind(song_id)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn dose_artist_have_a_song_by_name(pool: &DbPool, artist: &String, song_name: &String) -> bool {
    let song = get_song_by_name(pool, song_name).await;
    let artist = get_artist_by_name(pool, artist).await;

    match (song, artist) {
        (Some(s), Some(a)) => {
            let result = sqlx::query("SELECT 1 FROM artists_songs WHERE artist_id = ? AND song_id = ?")
                .bind(a.id)
                .bind(s.id)
                .fetch_optional(pool)
                .await;

            result.map(|opt| opt.is_some()).unwrap_or(false)
        }
        _ => false,
    }
}

pub async fn get_songs_by_artist_name(pool: &DbPool, name: &str) -> Option<Vec<Song>> {
    let artist = get_artist_by_name(pool, &name.to_string()).await;
    match artist {
        Some(a) => {
            //get song id from 'songs' table via 'artists_songs' table
            let result = sqlx::query_as::<_,Song>(
                "SELECT s.* FROM artists_songs as aso INNER JOIN songs as s ON aso.song_id = s.id WHERE aso.artist_id = ?"
            ).bind(a.id).fetch_all(pool).await;
            let result = match result {
                Ok(s) => Ok(s),
                Err(e) => Err(e),
            };
            match result {
                Ok(s) => Some(s),
                Err(e) => {
                    println!("{}", e.to_string());
                    None
                },
            }
        }
        None => None,
    }
}

pub async fn create_artist_entry(pool: &DbPool, artist: &Artist) {
    sqlx::query("INSERT INTO artists (name) VALUES (?)")
        .bind(&artist.name)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn get_artists(pool: &DbPool, acending: bool) -> Option<Vec<Artist>> {
    let result = sqlx::query_as(
        if acending { "SELECT id, name FROM artists ORDER BY name ASC"} 
        else { "SELECT id, name FROM artists ORDER BY name DESC" })
        .fetch_all(pool)
        .await;

    match result {
        Ok(artists) => Some(artists),
        Err(e) => {
            println!("{}", e);
            None
        }
    }
}

pub async fn get_artist_by_name(pool: &DbPool, artist_name: &String) -> Option<Artist> {
    let result = sqlx::query_as("SELECT id, name FROM artists WHERE name = ?")
        .bind(artist_name)
        .fetch_one(pool)
        .await;

    match result {
        Ok(artist) => Some(artist),
        Err(e) => {
            println!("{}", e);
            None
        }
    }
}

pub async fn delete_artist_by_id(pool: &DbPool, artist_id: &str) {
    sqlx::query("DELETE FROM artists WHERE id = ?")
        .bind(artist_id)
        .execute(pool)
        .await
        .unwrap();
}
