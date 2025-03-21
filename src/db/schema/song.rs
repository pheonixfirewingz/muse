use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct Song {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub file_path: String
}

impl Song {
    /// Construct a new `Song` with the given fields.
    ///
    /// `id` is a unique identifier for the song, `name` is the name of the song,
    /// `description` is a description of the song, and `file_path` is the path
    /// to the song's file. The `id` is stored as a string, and the `name` and
    /// `file_path` are required, while the `description` is optional.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for the song.
    /// * `name` - The name of the song.
    /// * `description` - The description of the song. This is optional.
    /// * `file_path` - The path to the song's file.
    pub fn new(id: Uuid, name: String, description: Option<String>, file_path: String) -> Self {
        Self { id: id.to_string(), name: name.to_lowercase().trim().to_string(), description, file_path }
    }

    /// Construct a new `Song` with an automatically generated ID.
    ///
    /// This function creates a new `Song` instance with a unique identifier
    /// generated automatically using a UUID. The `name` and `file_path` are
    /// required fields, while `description` is optional.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the song.
    /// * `description` - The description of the song. This is optional.
    /// * `file_path` - The path to the song's file.
    pub fn new_auto_id(name: String, description: Option<String>, file_path: String) -> Self {
        Self::new(Uuid::new_v4(), name, description, file_path)
    }

    /// Returns the unique identifier of the song as a `Uuid`.
    ///
    /// This function parses the `id` field of the `Song` struct, which is stored
    /// as a string, into a `Uuid` and returns it. The function will panic if the
    /// `id` cannot be parsed as a valid `Uuid`.
    pub fn get_id(&self) -> Uuid {
        Uuid::parse_str(&self.id).unwrap()
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
        let description = if self.description.is_some() {
            let description = self.description.clone().unwrap().replace('_', " ").split_whitespace()
                .map(|word| {
                    let mut c = word.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>().join(" ");
            Some(description)
        } else {
            self.description.clone()
        };
        Self {
            id: self.id.clone(),
            name,
            description,
            file_path: self.file_path.clone()
        }
    }
}

/// Creates the `songs` table in the database if it does not already exist.
///
/// The table is created with the specified schema, including columns for
/// `id`, `name`, `description`, and `file_path`. The `id` is the primary key,
/// and it, along with `name` and `file_path`, are required fields.
///
/// If the table already exists, this function does nothing.
///
/// # Errors
///
/// If the table creation fails for any reason, this function will print an
/// error message to standard output.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
pub async fn create_songs_table_if_not_exists(pool: &DbPool) {
    let result = sqlx::query("CREATE TABLE IF NOT EXISTS songs (
        id VARCHAR(36) PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        description VARCHAR(255),
        file_path VARCHAR(255) NOT NULL
    )").execute(pool).await;

    match result {
        Ok(_) => println!("Songs table is ready."),
        Err(e) => println!("Failed to create songs table: {}", e),
    }
}

pub async fn create_song_entry(pool: &DbPool, song: &Song) {
    let result = sqlx::query("INSERT INTO songs (
                   id,
                   name,
                   description,
                   file_path
        ) VALUES (?, ?, ?, ?)")
        .bind(song.id.clone()).bind(song.name.clone()).bind(song.description.clone())
        .bind(song.file_path.clone()).execute(pool).await;

    match result {
        Ok(_) => (),
        Err(e) => println!("Failed to create song entry: {}", e),
    }
}

/// Retrieve all songs from the `songs` table in the database.
///
/// This function queries the `songs` table and returns a vector of `Song`
/// instances representing all the songs in the database. It returns `None`
/// if the query fails for any reason, otherwise it returns `Some` containing
/// a `Vec<Song>`.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
///
/// # Returns
///
/// An `Option<Vec<Song>>` containing the list of all songs if the query is
/// successful, or `None` if the query fails.
pub async fn get_songs(pool: &DbPool) -> Option<Vec<Song>>{
    let result = sqlx::query_as("SELECT id, name, description, file_path FROM songs").fetch_all(pool).await;

    match result {
        Ok(songs) => Some(songs),
        Err(e) => {
            println!("{}", e.to_string());
            None
        },
    }
}

/// Retrieve a song by its name from the `songs` table in the database.
///
/// This function queries the `songs` table for a song with the specified `name`.
/// It returns an `Option<Song>`, where `Some(Song)` is returned if a song
/// with the given name is found, and `None` is returned if no such song
/// exists or if the query fails for any reason.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `name` - The name of the song to retrieve.
///
/// # Returns
///
/// An `Option<Song>` containing the `Song` with the given `name` if found,
/// or `None` if the query fails or no song with the specified name exists.
pub async fn get_song_by_name(pool: &DbPool, name: &String) -> Option<Song> {
    let result = sqlx::query_as("SELECT id, name, description, file_path FROM songs WHERE name = ?")
        .bind(name.to_lowercase()).fetch_one(pool).await;

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
        .bind(format!("%{}%", name.to_lowercase())).fetch_all(pool).await;

    match result {
        Ok(songs) => Some(songs),
        Err(e) => {
            println!("{}", e.to_string());
            None
        },
    }
}



/// Deletes a song from the `songs` table in the database.
///
/// This function deletes the song with the specified `song_id` from the `songs`
/// table. If the deletion fails for any reason, an error message is printed
/// to standard output.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `song_id` - The UUID of the song to delete.
///
/// # Errors
///
/// This function prints an error message if the database operation fails.
pub async fn delete_song(pool: &DbPool, song_id: Uuid) {
    let result = sqlx::query("DELETE FROM songs WHERE id = ?")
        .bind(song_id.to_string()).execute(pool).await;

    match result {
        Ok(_) => (),
        Err(e) => println!("Failed to delete song {}: {}", song_id, e),
    }
}