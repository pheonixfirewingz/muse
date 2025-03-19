use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;
use crate::db::DbPool;
use crate::db::schema::song::{get_song_by_name, Song};

#[derive(FromRow, Serialize)]
pub struct Artist {
    pub id: String,
    pub name: String
}

impl Artist {
    /// Construct a new `Artist` with the given fields.
    ///
    /// This function creates a new `Artist` instance with the given `id` and `name`.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for the artist.
    /// * `name` - The name of the artist.
    ///
    pub fn new(id: String, name: String) -> Self {
        Self { id, name:name.to_lowercase().trim().to_string().replace(" ", "_") }
    }
    
    /// Construct a new `Artist` with a unique identifier generated automatically.
    ///
    /// This function creates a new `Artist` instance with a unique identifier
    /// generated automatically using a UUID, and the given `name`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the artist.
    ///
    pub fn new_auto_id(name: String) -> Self {
        Self::new(Uuid::new_v4().to_string(), name)
    }
    
    /// Returns the unique identifier of the artist as a `Uuid`.
    ///
    /// This function parses the `id` field of the `Artist` struct, which is stored
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
            }).collect::<Vec<_>>().join(" ");
        Self {
            id: self.id.clone(),
            name
        }
    }
}

/// Creates the `artists` table in the database if it does not already exist.
///
/// The table is created with two columns: `id` (a 36-character string that is
/// the primary key of the table) and `name` (a 255-character string that is
/// required).
///
/// If the table already exists, this function does nothing.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
pub async fn create_artists_table_if_not_exists(pool: &DbPool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS artists (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(255) NOT NULL
        )"
    ).execute(pool).await.unwrap();
}

pub async fn add_artist_song_association(pool: &DbPool, artist_id: &Uuid, song_id: &Uuid) {
    sqlx::query(
        "INSERT INTO artists_songs (artist_id, song_id) VALUES ($1, $2)"
    ).bind(artist_id.to_string()).bind(song_id.to_string()).execute(pool).await.unwrap();
}

/// Creates the `artists_songs` table in the database if it does not already exist.
///
/// The table is created with two columns: `artist_id` and `song_id`, both of
/// which are 36-character strings and are required. The table also enforces a
/// composite primary key of `artist_id` and `song_id`, and foreign key
/// constraints to the `artists` and `songs` tables.
///
/// If the table already exists, this function does nothing.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
pub async fn create_artists_songs_table_if_not_exists(pool: &DbPool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS artists_songs (
            artist_id VARCHAR(36) NOT NULL,
            song_id VARCHAR(36) NOT NULL,
            PRIMARY KEY (artist_id, song_id),
            FOREIGN KEY (artist_id) REFERENCES artists(id),
            FOREIGN KEY (song_id) REFERENCES songs(id)
        )"
    ).execute(pool).await.unwrap();
}

/// Checks if an artist has a song with the given name.
///
/// This function takes the name of an artist and the name of a song, and
/// checks if the artist has a song with that name in the database.
///
/// It first queries the `songs` table to get the ID of the song with the
/// given name, and then queries the `artists` table to get the ID of the
/// artist with the given name. It then checks if the artist ID and song ID
/// are in the `artists_songs` table, which is a many-to-many join table
/// between the `artists` and `songs` tables. If the IDs are in the table,
/// the function returns `true`; otherwise, it returns `false`.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `artist` - The name of the artist.
/// * `song_name` - The name of the song.
///
/// # Returns
///
/// A boolean indicating whether the artist has a song with the given name.
pub async fn dose_artist_have_a_song_by_name(pool: &DbPool, artist: &String, song_name: &String) -> bool {
    //get song id from 'songs' table
    let song = get_song_by_name(pool, &song_name).await;
    //get artist id from 'artists' table
    let artist = get_artist_by_name(pool, &artist).await;
    // Check if both song and artist were found
    match (song, artist) {
        (Some(s), Some(a)) => {
            // Check if there's an association between the artist and song
            let result = sqlx::query(
                "SELECT 1 FROM artists_songs WHERE artist_id = ? AND song_id = ?"
            ).bind(a.id).bind(s.id).fetch_optional(pool).await;

            match result {
                Ok(option) => option.is_some(),
                Err(_) => false,
            }
        },
        _ => false, // Either the song or artist wasn't found
    }
}

pub async fn get_songs_by_artist_name(pool: &DbPool, artist_name: &String) -> Option<Song> {
    let artist = get_artist_by_name(pool, artist_name).await;
    match artist {
        Some(a) => {
            //get song id from 'songs' table via 'artists_songs' table
            let result = sqlx::query_as::<_,Song>(
                "SELECT s.* FROM artists_songs as aso INNER JOIN songs as s ON aso.song_id = s.id WHERE aso.artist_id = ?"
            ).bind(a.id).fetch_one(pool).await;
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

/// Inserts a new artist into the `artists` table in the database.
///
/// This function adds a new record to the `artists` table with the provided
/// `id` and `name` of the `artist`. The insertion is performed asynchronously,
/// and the function will panic if the operation fails.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `artist` - A reference to the `Artist` instance containing the `id`
///   and `name` to be inserted.
pub async fn create_artist_entry(pool: &DbPool, artist: &Artist) {
    sqlx::query(
        "INSERT INTO artists (id, name) VALUES (?, ?)"
    ).bind(&artist.id).bind(&artist.name).execute(pool).await.unwrap();
}

/// Retrieves all artists from the `artists` table in the database.
///
/// This function queries the database, and returns `None` if any error
/// occurs while doing so. Otherwise, it returns a `Some` containing a
/// `Vec` of `Artist` values, which are the artists in the database.
///
/// # Errors
///
/// This function errors if the database query fails for any reason.
pub async fn get_artists(pool: &DbPool) -> Option<Vec<Artist>> {
    let result = sqlx::query_as(
        "SELECT id, name FROM artists"
    ).fetch_all(pool).await;
    
    match result {
        Ok(artists) => Some(artists),
        Err(e) => {
            println!("{}", e.to_string());
            None
        },
    }
}

/// Retrieves an artist from the `artists` table in the database by its name.
///
/// This function queries the `artists` table for an artist with the specified
/// `artist_name`. It returns `None` if any error occurs while doing so.
/// Otherwise, it returns a `Some` containing the `Artist` instance with the
/// matching `name`.
///
/// # Errors
///
/// This function errors if the database query fails for any reason.
pub async fn get_artist_by_name(pool: &DbPool, artist_name: &String) -> Option<Artist> {
    let result = sqlx::query_as(
        "SELECT id, name FROM artists WHERE name = ?"
    ).bind(artist_name).fetch_one(pool).await;
    
    match result {
        Ok(artist) => Some(artist),
        Err(e) => {
            println!("{}", e.to_string());
            None
        },
    }
}

/// Deletes an artist from the `artists` table in the database by its ID.
///
/// This function deletes the artist with the specified `artist_id` from the
/// `artists` table. If the deletion fails for any reason, an error message is
/// printed to standard output.
///
/// # Errors
///
/// This function prints an error message if the database operation fails.
pub async fn delete_artist_by_id(pool: &DbPool, artist_id: String) {
    sqlx::query(
        "DELETE FROM artists WHERE id = ?"
    ).bind(artist_id).execute(pool).await.unwrap();
}