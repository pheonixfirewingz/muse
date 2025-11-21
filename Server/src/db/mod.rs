pub mod models;
pub mod sqlite;
pub mod postgres;
pub mod mongo;

use crate::db::models::{Artist, Playlist, PlaylistShare, Song, User};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("User already exists")]
    UserAlreadyExists,
    
    #[error("Invalid credentials")]
    #[allow(dead_code)]
    InvalidCredentials,
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Database abstraction trait for user operations
#[async_trait]
pub trait Database: Send + Sync {
    /// Create a new user
    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User, DbError>;
    
    /// Get user by username
    async fn get_user_by_username(&self, username: &str) -> Result<User, DbError>;
    
    /// Get user by email
    async fn get_user_by_email(&self, email: &str) -> Result<User, DbError>;
    
    /// Get user by ID
    async fn get_user_by_id(&self, id: &str) -> Result<User, DbError>;
    
    /// Update user's admin status
    async fn update_user_admin_status(&self, id: &str, is_admin: bool) -> Result<(), DbError>;
    
    /// Check if username exists
    async fn username_exists(&self, username: &str) -> Result<bool, DbError>;
    
    /// Check if email exists
    async fn email_exists(&self, email: &str) -> Result<bool, DbError>;
    
    /// Initialize database (create tables/collections if needed)
    async fn initialize(&self) -> Result<(), DbError>;
    
    // Admin operations
    /// Get all users with pagination
    async fn get_all_users(&self, offset: usize, limit: usize) -> Result<Vec<User>, DbError>;
    
    /// Update user email
    async fn update_user_email(&self, username: &str, new_email: &str) -> Result<(), DbError>;
    
    /// Update username
    async fn update_username(&self, user_id: &str, new_username: &str) -> Result<(), DbError>;
    
    /// Update user password hash
    async fn update_user_password(&self, user_id: &str, new_password_hash: &str) -> Result<(), DbError>;
    
    /// Delete user by username
    async fn delete_user_by_username(&self, username: &str) -> Result<(), DbError>;
    
    /// Delete user by ID
    async fn delete_user_by_id(&self, user_id: &str) -> Result<(), DbError>;
    
    /// Get total user count
    #[allow(dead_code)]
    async fn get_total_users(&self) -> Result<usize, DbError>;
    
    // Artist operations
    /// Create a new artist
    async fn create_artist(&self, name: &str) -> Result<Artist, DbError>;
    
    /// Get artist by ID
    async fn get_artist_by_id(&self, id: &str) -> Result<Artist, DbError>;
    
    /// Get artist by name
    async fn get_artist_by_name(&self, name: &str) -> Result<Artist, DbError>;
    
    /// Get all artists with pagination
    async fn get_artists(&self, offset: usize, limit: usize) -> Result<Vec<Artist>, DbError>;
    
    /// Get total artist count
    async fn get_total_artists(&self) -> Result<usize, DbError>;
    
    /// Update artist cover image path
    #[allow(dead_code)]
    async fn update_artist_cover(&self, id: &str, cover_path: &str) -> Result<(), DbError>;
    
    /// Check if artist exists by name
    async fn artist_exists(&self, name: &str) -> Result<bool, DbError>;
    
    // Song operations
    /// Create a new song
    async fn create_song(&self, title: &str, artist_id: &str, file_path: &str) -> Result<Song, DbError>;
    
    /// Get song by ID
    async fn get_song_by_id(&self, id: &str) -> Result<Song, DbError>;
    
    /// Get songs by artist ID
    async fn get_songs_by_artist(&self, artist_id: &str) -> Result<Vec<Song>, DbError>;
    
    /// Get all songs with pagination
    async fn get_songs(&self, offset: usize, limit: usize) -> Result<Vec<Song>, DbError>;
    
    /// Get total song count
    async fn get_total_songs(&self) -> Result<usize, DbError>;
    
    /// Update song metadata
    async fn update_song_metadata(&self, id: &str, album: Option<&str>, duration: Option<i32>, cover_path: Option<&str>) -> Result<(), DbError>;
    
    /// Delete a song by ID
    async fn delete_song_by_id(&self, id: &str) -> Result<(), DbError>;
    
    // Playlist operations
    /// Create a new playlist
    async fn create_playlist(&self, name: &str, owner_id: &str, is_public: bool) -> Result<Playlist, DbError>;
    
    /// Get playlist by ID
    async fn get_playlist_by_id(&self, id: &str) -> Result<Playlist, DbError>;
    
    /// Get playlist by name and owner
    async fn get_playlist_by_name_and_owner(&self, name: &str, owner_id: &str) -> Result<Playlist, DbError>;
    
    /// Get user's private playlists with pagination
    async fn get_user_playlists(&self, user_id: &str, offset: usize, limit: usize) -> Result<Vec<Playlist>, DbError>;
    
    /// Get public playlists with pagination
    async fn get_public_playlists(&self, offset: usize, limit: usize) -> Result<Vec<Playlist>, DbError>;
    
    /// Get playlists shared with a user
    async fn get_shared_playlists(&self, user_id: &str) -> Result<Vec<(Playlist, PlaylistShare)>, DbError>;
    
    /// Delete a playlist
    async fn delete_playlist(&self, playlist_id: &str, owner_id: &str) -> Result<(), DbError>;
    
    /// Add a song to a playlist
    async fn add_song_to_playlist(&self, playlist_id: &str, song_id: &str) -> Result<(), DbError>;
    
    /// Remove a song from a playlist
    async fn remove_song_from_playlist(&self, playlist_id: &str, song_id: &str) -> Result<(), DbError>;
    
    /// Get songs in a playlist
    async fn get_playlist_songs(&self, playlist_id: &str) -> Result<Vec<Song>, DbError>;
    
    /// Check if a song is in a playlist
    #[allow(dead_code)]
    async fn is_song_in_playlist(&self, playlist_id: &str, song_id: &str) -> Result<bool, DbError>;
    
    /// Share a playlist with a user
    async fn share_playlist(&self, playlist_id: &str, shared_with_user_id: &str, shared_by_user_id: &str) -> Result<PlaylistShare, DbError>;
    
    /// Revoke playlist share
    async fn revoke_playlist_share(&self, playlist_id: &str, shared_with_user_id: &str) -> Result<(), DbError>;
    
    /// Check if a playlist is shared with a user
    #[allow(dead_code)]
    async fn is_playlist_shared_with_user(&self, playlist_id: &str, user_id: &str) -> Result<bool, DbError>;
    
    // Admin playlist operations
    /// Get all playlists with pagination (admin)
    async fn get_all_playlists(&self, offset: usize, limit: usize) -> Result<Vec<Playlist>, DbError>;
    
    /// Get total playlist count (admin)
    #[allow(dead_code)]
    async fn get_total_playlists(&self) -> Result<usize, DbError>;
    
    /// Update playlist name (admin)
    async fn update_playlist_name(&self, playlist_id: &str, new_name: &str) -> Result<(), DbError>;
    
    /// Update playlist visibility (admin)
    async fn update_playlist_visibility(&self, playlist_id: &str, is_public: bool) -> Result<(), DbError>;
    
    /// Delete playlist by ID (admin - bypasses owner check)
    async fn delete_playlist_by_id(&self, playlist_id: &str) -> Result<(), DbError>;
}

/// Database backend type
#[derive(Debug, Clone)]
pub enum DbBackend {
    SQLite,
    Postgres,
    MongoDB,
}

impl DbBackend {
    pub fn from_string(s: &str) -> Result<Self, DbError> {
        match s.to_lowercase().as_str() {
            "sqlite" => Ok(DbBackend::SQLite),
            "postgres" | "postgresql" => Ok(DbBackend::Postgres),
            "mongo" | "mongodb" => Ok(DbBackend::MongoDB),
            _ => Err(DbError::ConfigError(format!("Unknown database backend: {}", s))),
        }
    }
}

/// Create database instance based on configuration
pub async fn create_database(backend: DbBackend, connection_string: &str) -> Result<Arc<dyn Database>, DbError> {
    let db: Arc<dyn Database> = match backend {
        DbBackend::SQLite => {
            let sqlite_db = sqlite::SqliteDatabase::new(connection_string).await?;
            Arc::new(sqlite_db)
        },
        DbBackend::Postgres => {
            let postgres_db = postgres::PostgresDatabase::new(connection_string).await?;
            Arc::new(postgres_db)
        },
        DbBackend::MongoDB => {
            let mongo_db = mongo::MongoDatabase::new(connection_string).await?;
            Arc::new(mongo_db)
        },
    };
    
    // Initialize the database (create tables/collections)
    db.initialize().await?;
    
    Ok(db)
}
