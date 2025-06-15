use crate::db::DbPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Song {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub file_path: String,
}

impl Song {
    /// Construct a new `Song` with no manually assigned ID.
    ///
    /// The database will generate the UUID automatically.
    pub fn new(name: String, description: Option<String>, file_path: String) -> Self {
        Self {
            id: None,
            name: name.to_lowercase().trim().to_string(),
            description,
            file_path,
        }
    }

    pub fn clean_for_web_view(&self) -> Self {
        let name = self.name.clone().replace('_', " ").split_whitespace()
            .map(|word| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>().join(" ");
        let description = self.description.clone().map(|desc| {
            desc.replace('_', " ").split_whitespace()
                .map(|word| {
                    let mut c = word.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>().join(" ")
        });

        Self {
            id: self.id.clone(),
            name,
            description,
            file_path: self.file_path.clone(),
        }
    }
}

pub async fn create_songs_table_if_not_exists(pool: &DbPool) {
    let result = sqlx::query(
        "CREATE TABLE IF NOT EXISTS songs (
            id TEXT PRIMARY KEY NOT NULL DEFAULT (
                lower(
                    hex(randomblob(4)) || '-' ||
                    hex(randomblob(2)) || '-' ||
                    '4' || substr(hex(randomblob(2)), 2) || '-' ||
                    substr('89ab', abs(random() % 4) + 1, 1) || substr(hex(randomblob(2)), 2) || '-' ||
                    hex(randomblob(6))
                )
            ),
            name TEXT NOT NULL,
            description TEXT,
            file_path TEXT NOT NULL
        )"
    ).execute(pool).await;

    match result {
        Ok(_) => println!("Songs table is ready."),
        Err(e) => println!("Failed to create songs table: {}", e),
    }
}

pub async fn create_song_entry(pool: &DbPool, song: &Song) {
    let result = sqlx::query("INSERT INTO songs (
                   name,
                   description,
                   file_path
        ) VALUES (?, ?, ?)")
        .bind(&song.name)
        .bind(&song.description)
        .bind(&song.file_path)
        .execute(pool).await;

    match result {
        Ok(_) => (),
        Err(e) => println!("Failed to create song entry: {}", e),
    }
}

pub async fn get_songs(pool: &DbPool, ascending: bool) -> Option<Vec<Song>> {
    let result = sqlx::query_as(
        if ascending {
            "SELECT id, name, description, file_path FROM songs ORDER BY name ASC"
        } else {
            "SELECT id, name, description, file_path FROM songs ORDER BY name DESC"
        }
    ).fetch_all(pool).await;

    match result {
        Ok(songs) => Some(songs),
        Err(e) => {
            println!("{}", e.to_string());
            None
        },
    }
}

pub async fn get_song_by_name(pool: &DbPool, name: &String) -> Option<Song> {
    let result = sqlx::query_as("SELECT id, name, description, file_path FROM songs WHERE name = ?")
        .bind(name.to_lowercase())
        .fetch_one(pool).await;

    match result {
        Ok(song) => Some(song),
        Err(sqlx::error::Error::RowNotFound) => None,
        Err(e) => {
            println!("{}", e.to_string());
            None
        },
    }
}

pub async fn get_songs_by_name(pool: &DbPool, name: &String) -> Option<Vec<Song>> {
    let result = sqlx::query_as("SELECT id, name, description, file_path FROM songs WHERE name LIKE ?")
        .bind(format!("%{}%", name.to_lowercase()))
        .fetch_all(pool).await;

    match result {
        Ok(songs) => Some(songs),
        Err(e) => {
            println!("{}", e.to_string());
            None
        },
    }
}

pub async fn delete_song(pool: &DbPool, song_id: &str) {
    let result = sqlx::query("DELETE FROM songs WHERE id = ?")
        .bind(song_id)
        .execute(pool).await;

    match result {
        Ok(_) => (),
        Err(e) => println!("Failed to delete song {}: {}", song_id, e),
    }
}
