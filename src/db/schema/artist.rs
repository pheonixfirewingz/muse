use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;
use crate::db::DbPool;

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
        Self { id, name }
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

/// Retrieves an artist from the `artists` table in the database by its ID.
///
/// This function queries the `artists` table for an artist with the specified
/// `artist_id`. It returns `None` if any error occurs while doing so.
/// Otherwise, it returns a `Some` containing the `Artist` instance with the
/// matching `id` and `name`.
///
/// # Errors
///
/// This function errors if the database query fails for any reason.
pub async fn get_artist_by_id(pool: &DbPool, artist_id: String) -> Option<Artist> {
    let result = sqlx::query_as(
        "SELECT id, name FROM artists WHERE id = ?"
    ).bind(artist_id).fetch_one(pool).await;
    
    match result {
        Ok(artist) => Some(artist),
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