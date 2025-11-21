use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::models::{Artist, Playlist, PlaylistShare, Song, User};
use crate::db::{Database, DbError};

pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub async fn new(database_url: &str) -> Result<Self, DbError> {
        // Extract the file path from the database URL
        // Format is typically: sqlite:path/to/file.db
        if let Some(file_path) = database_url.strip_prefix("sqlite:") {
            // Create parent directories if they don't exist
            if let Some(parent) = std::path::Path::new(file_path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DbError::DatabaseError(format!("Failed to create database directory: {}", e)))?;
            }
        }
        
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to connect to SQLite: {}", e)))?;
        
        Ok(Self { pool })
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User, DbError> {
        // Check if user already exists
        if self.username_exists(username).await? {
            return Err(DbError::UserAlreadyExists);
        }
        
        if self.email_exists(email).await? {
            return Err(DbError::UserAlreadyExists);
        }
        
        let id = Uuid::new_v4().to_string();
        let created_at = OffsetDateTime::now_utc();
        let created_at_str = created_at.unix_timestamp().to_string();
        
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, is_admin, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(0) // is_admin = false
        .bind(&created_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to create user: {}", e)))?;
        
        Ok(User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            is_admin: false,
            created_at,
        })
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, DbError> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, is_admin, created_at FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or(DbError::UserNotFound)?;
        
        let timestamp: i64 = row.get("created_at");
        let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
        
        Ok(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            is_admin: row.get::<i32, _>("is_admin") != 0,
            created_at,
        })
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User, DbError> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, is_admin, created_at FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or(DbError::UserNotFound)?;
        
        let timestamp: i64 = row.get("created_at");
        let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
        
        Ok(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            is_admin: row.get::<i32, _>("is_admin") != 0,
            created_at,
        })
    }

    async fn get_user_by_id(&self, id: &str) -> Result<User, DbError> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, is_admin, created_at FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or(DbError::UserNotFound)?;
        
        let timestamp: i64 = row.get("created_at");
        let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
        
        Ok(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            is_admin: row.get::<i32, _>("is_admin") != 0,
            created_at,
        })
    }

    async fn update_user_admin_status(&self, id: &str, is_admin: bool) -> Result<(), DbError> {
        let is_admin_int = if is_admin { 1 } else { 0 };
        
        sqlx::query("UPDATE users SET is_admin = ? WHERE id = ?")
            .bind(is_admin_int)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update user: {}", e)))?;
        
        Ok(())
    }

    async fn username_exists(&self, username: &str) -> Result<bool, DbError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = ?")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count > 0)
    }

    async fn email_exists(&self, email: &str) -> Result<bool, DbError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count > 0)
    }

    async fn initialize(&self) -> Result<(), DbError> {
        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                is_admin INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to create users table: {}", e)))?;
        
        // Create artists table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS artists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                cover_image_path TEXT,
                created_at TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to create artists table: {}", e)))?;
        
        // Create songs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS songs (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist_id TEXT NOT NULL,
                artist_name TEXT NOT NULL,
                album TEXT,
                duration INTEGER,
                file_path TEXT NOT NULL,
                cover_image_path TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to create songs table: {}", e)))?;
        
        // Create indices for faster lookups
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create index: {}", e)))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create index: {}", e)))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_artists_name ON artists(name)")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create index: {}", e)))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_songs_artist_id ON songs(artist_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create index: {}", e)))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title)")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create index: {}", e)))?;
        
        // Create playlists table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                owner_id TEXT NOT NULL,
                owner_username TEXT NOT NULL,
                is_public INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to create playlists table: {}", e)))?;
        
        // Create playlist_songs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playlist_songs (
                playlist_id TEXT NOT NULL,
                song_id TEXT NOT NULL,
                added_at TEXT NOT NULL,
                PRIMARY KEY (playlist_id, song_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to create playlist_songs table: {}", e)))?;
        
        // Create playlist_shares table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playlist_shares (
                id TEXT PRIMARY KEY,
                playlist_id TEXT NOT NULL,
                shared_with_user_id TEXT NOT NULL,
                shared_by_user_id TEXT NOT NULL,
                shared_at TEXT NOT NULL,
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (shared_with_user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (shared_by_user_id) REFERENCES users(id) ON DELETE CASCADE,
                UNIQUE(playlist_id, shared_with_user_id)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to create playlist_shares table: {}", e)))?;
        
        // Create indices for playlists
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_playlists_owner_id ON playlists(owner_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create index: {}", e)))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_playlists_is_public ON playlists(is_public)")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create index: {}", e)))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_playlist_shares_shared_with ON playlist_shares(shared_with_user_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create index: {}", e)))?;
        
        Ok(())
    }
    
    async fn get_all_users(&self, offset: usize, limit: usize) -> Result<Vec<User>, DbError> {
        let rows = sqlx::query(
            "SELECT id, username, email, password_hash, is_admin, created_at FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut users = Vec::new();
        for row in rows {
            let timestamp: i64 = row.get("created_at");
            let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
            
            users.push(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                is_admin: row.get::<i32, _>("is_admin") != 0,
                created_at,
            });
        }
        
        Ok(users)
    }
    
    async fn update_user_email(&self, username: &str, new_email: &str) -> Result<(), DbError> {
        // Check if new email already exists for another user
        if self.email_exists(new_email).await? {
            let existing_user = self.get_user_by_email(new_email).await?;
            if existing_user.username != username {
                return Err(DbError::UserAlreadyExists);
            }
        }
        
        let result = sqlx::query("UPDATE users SET email = ? WHERE username = ?")
            .bind(new_email)
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update user: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::UserNotFound);
        }
        
        Ok(())
    }
    
    async fn update_username(&self, user_id: &str, new_username: &str) -> Result<(), DbError> {
        // Check if new username already exists
        if self.username_exists(new_username).await? {
            return Err(DbError::UserAlreadyExists);
        }
        
        let result = sqlx::query("UPDATE users SET username = ? WHERE id = ?")
            .bind(new_username)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update username: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::UserNotFound);
        }
        
        Ok(())
    }
    
    async fn update_user_password(&self, user_id: &str, new_password_hash: &str) -> Result<(), DbError> {
        let result = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
            .bind(new_password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update password: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::UserNotFound);
        }
        
        Ok(())
    }
    
    async fn delete_user_by_username(&self, username: &str) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM users WHERE username = ?")
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to delete user: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::UserNotFound);
        }
        
        Ok(())
    }
    
    async fn delete_user_by_id(&self, user_id: &str) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to delete user: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::UserNotFound);
        }
        
        Ok(())
    }
    
    async fn get_total_users(&self) -> Result<usize, DbError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count as usize)
    }
    
    // Artist operations
    async fn create_artist(&self, name: &str) -> Result<Artist, DbError> {
        if self.artist_exists(name).await? {
            return Err(DbError::DatabaseError("Artist already exists".to_string()));
        }
        
        let id = Uuid::new_v4().to_string();
        let created_at = OffsetDateTime::now_utc();
        let created_at_str = created_at.unix_timestamp().to_string();
        
        sqlx::query("INSERT INTO artists (id, name, created_at) VALUES (?, ?, ?)")
            .bind(&id)
            .bind(name)
            .bind(&created_at_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create artist: {}", e)))?;
        
        Ok(Artist {
            id,
            name: name.to_string(),
            cover_image_path: None,
            created_at,
        })
    }
    
    async fn get_artist_by_id(&self, id: &str) -> Result<Artist, DbError> {
        let row = sqlx::query("SELECT id, name, cover_image_path, created_at FROM artists WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
            .ok_or(DbError::DatabaseError("Artist not found".to_string()))?;
        
        let timestamp: i64 = row.get("created_at");
        let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
        
        Ok(Artist {
            id: row.get("id"),
            name: row.get("name"),
            cover_image_path: row.get("cover_image_path"),
            created_at,
        })
    }
    
    async fn get_artist_by_name(&self, name: &str) -> Result<Artist, DbError> {
        let row = sqlx::query("SELECT id, name, cover_image_path, created_at FROM artists WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
            .ok_or(DbError::DatabaseError("Artist not found".to_string()))?;
        
        let timestamp: i64 = row.get("created_at");
        let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
        
        Ok(Artist {
            id: row.get("id"),
            name: row.get("name"),
            cover_image_path: row.get("cover_image_path"),
            created_at,
        })
    }
    
    async fn get_artists(&self, offset: usize, limit: usize) -> Result<Vec<Artist>, DbError> {
        let rows = sqlx::query(
            "SELECT id, name, cover_image_path, created_at FROM artists ORDER BY name ASC LIMIT ? OFFSET ?"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut artists = Vec::new();
        for row in rows {
            let timestamp: i64 = row.get("created_at");
            let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
            
            artists.push(Artist {
                id: row.get("id"),
                name: row.get("name"),
                cover_image_path: row.get("cover_image_path"),
                created_at,
            });
        }
        
        Ok(artists)
    }
    
    async fn get_total_artists(&self) -> Result<usize, DbError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM artists")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count as usize)
    }
    
    async fn update_artist_cover(&self, id: &str, cover_path: &str) -> Result<(), DbError> {
        let result = sqlx::query("UPDATE artists SET cover_image_path = ? WHERE id = ?")
            .bind(cover_path)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update artist: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::DatabaseError("Artist not found".to_string()));
        }
        
        Ok(())
    }
    
    async fn artist_exists(&self, name: &str) -> Result<bool, DbError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM artists WHERE name = ?")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count > 0)
    }
    
    // Song operations
    async fn create_song(&self, title: &str, artist_id: &str, file_path: &str) -> Result<Song, DbError> {
        // Get artist to get the artist name
        let artist = self.get_artist_by_id(artist_id).await?;
        
        let id = Uuid::new_v4().to_string();
        let created_at = OffsetDateTime::now_utc();
        let created_at_str = created_at.unix_timestamp().to_string();
        
        sqlx::query(
            "INSERT INTO songs (id, title, artist_id, artist_name, file_path, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(title)
        .bind(artist_id)
        .bind(&artist.name)
        .bind(file_path)
        .bind(&created_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to create song: {}", e)))?;
        
        Ok(Song {
            id,
            title: title.to_string(),
            artist_id: artist_id.to_string(),
            artist_name: artist.name,
            album: None,
            duration: None,
            file_path: file_path.to_string(),
            cover_image_path: None,
            created_at,
        })
    }
    
    async fn get_song_by_id(&self, id: &str) -> Result<Song, DbError> {
        let row = sqlx::query(
            "SELECT id, title, artist_id, artist_name, album, duration, file_path, cover_image_path, created_at FROM songs WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or(DbError::DatabaseError("Song not found".to_string()))?;
        
        let timestamp: i64 = row.get("created_at");
        let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
        
        Ok(Song {
            id: row.get("id"),
            title: row.get("title"),
            artist_id: row.get("artist_id"),
            artist_name: row.get("artist_name"),
            album: row.get("album"),
            duration: row.get("duration"),
            file_path: row.get("file_path"),
            cover_image_path: row.get("cover_image_path"),
            created_at,
        })
    }
    
    async fn get_songs_by_artist(&self, artist_id: &str) -> Result<Vec<Song>, DbError> {
        let rows = sqlx::query(
            "SELECT id, title, artist_id, artist_name, album, duration, file_path, cover_image_path, created_at FROM songs WHERE artist_id = ? ORDER BY title ASC"
        )
        .bind(artist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut songs = Vec::new();
        for row in rows {
            let timestamp: i64 = row.get("created_at");
            let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
            
            songs.push(Song {
                id: row.get("id"),
                title: row.get("title"),
                artist_id: row.get("artist_id"),
                artist_name: row.get("artist_name"),
                album: row.get("album"),
                duration: row.get("duration"),
                file_path: row.get("file_path"),
                cover_image_path: row.get("cover_image_path"),
                created_at,
            });
        }
        
        Ok(songs)
    }
    
    async fn get_songs(&self, offset: usize, limit: usize) -> Result<Vec<Song>, DbError> {
        let rows = sqlx::query(
            "SELECT id, title, artist_id, artist_name, album, duration, file_path, cover_image_path, created_at FROM songs ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut songs = Vec::new();
        for row in rows {
            let timestamp: i64 = row.get("created_at");
            let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
            
            songs.push(Song {
                id: row.get("id"),
                title: row.get("title"),
                artist_id: row.get("artist_id"),
                artist_name: row.get("artist_name"),
                album: row.get("album"),
                duration: row.get("duration"),
                file_path: row.get("file_path"),
                cover_image_path: row.get("cover_image_path"),
                created_at,
            });
        }
        
        Ok(songs)
    }
    
    async fn get_total_songs(&self) -> Result<usize, DbError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM songs")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count as usize)
    }
    
    async fn update_song_metadata(&self, id: &str, album: Option<&str>, duration: Option<i32>, cover_path: Option<&str>) -> Result<(), DbError> {
        let result = sqlx::query(
            "UPDATE songs SET album = ?, duration = ?, cover_image_path = ? WHERE id = ?"
        )
        .bind(album)
        .bind(duration)
        .bind(cover_path)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to update song: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::DatabaseError("Song not found".to_string()));
        }
        
        Ok(())
    }
   
    async fn delete_song_by_id(&self, id: &str) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM songs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to delete song: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::DatabaseError("Song not found".to_string()));
        }
        
        Ok(())
    }
    
    // Playlist operations
    async fn create_playlist(&self, name: &str, owner_id: &str, is_public: bool) -> Result<Playlist, DbError> {
        // Get owner username
        let owner = self.get_user_by_id(owner_id).await?;
        
        let id = Uuid::new_v4().to_string();
        let created_at = OffsetDateTime::now_utc();
        let created_at_str = created_at.unix_timestamp().to_string();
        let is_public_int = if is_public { 1 } else { 0 };
        
        sqlx::query(
            "INSERT INTO playlists (id, name, owner_id, owner_username, is_public, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(name)
        .bind(owner_id)
        .bind(&owner.username)
        .bind(is_public_int)
        .bind(&created_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to create playlist: {}", e)))?;
        
        Ok(Playlist {
            id,
            name: name.to_string(),
            owner_id: owner_id.to_string(),
            owner_username: owner.username,
            is_public,
            created_at,
        })
    }
    
    async fn get_playlist_by_id(&self, id: &str) -> Result<Playlist, DbError> {
        let row = sqlx::query(
            "SELECT id, name, owner_id, owner_username, is_public, created_at FROM playlists WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or(DbError::DatabaseError("Playlist not found".to_string()))?;
        
        let timestamp: i64 = row.get("created_at");
        let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
        
        Ok(Playlist {
            id: row.get("id"),
            name: row.get("name"),
            owner_id: row.get("owner_id"),
            owner_username: row.get("owner_username"),
            is_public: row.get::<i32, _>("is_public") != 0,
            created_at,
        })
    }
    
    async fn get_playlist_by_name_and_owner(&self, name: &str, owner_id: &str) -> Result<Playlist, DbError> {
        let row = sqlx::query(
            "SELECT id, name, owner_id, owner_username, is_public, created_at FROM playlists WHERE name = ? AND owner_id = ?"
        )
        .bind(name)
        .bind(owner_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or(DbError::DatabaseError("Playlist not found".to_string()))?;
        
        let timestamp: i64 = row.get("created_at");
        let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
        
        Ok(Playlist {
            id: row.get("id"),
            name: row.get("name"),
            owner_id: row.get("owner_id"),
            owner_username: row.get("owner_username"),
            is_public: row.get::<i32, _>("is_public") != 0,
            created_at,
        })
    }
    
    async fn get_user_playlists(&self, user_id: &str, offset: usize, limit: usize) -> Result<Vec<Playlist>, DbError> {
        let rows = sqlx::query(
            "SELECT id, name, owner_id, owner_username, is_public, created_at FROM playlists WHERE owner_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(user_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut playlists = Vec::new();
        for row in rows {
            let timestamp: i64 = row.get("created_at");
            let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
            
            playlists.push(Playlist {
                id: row.get("id"),
                name: row.get("name"),
                owner_id: row.get("owner_id"),
                owner_username: row.get("owner_username"),
                is_public: row.get::<i32, _>("is_public") != 0,
                created_at,
            });
        }
        
        Ok(playlists)
    }
    
    async fn get_public_playlists(&self, offset: usize, limit: usize) -> Result<Vec<Playlist>, DbError> {
        let rows = sqlx::query(
            "SELECT id, name, owner_id, owner_username, is_public, created_at FROM playlists WHERE is_public = 1 ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut playlists = Vec::new();
        for row in rows {
            let timestamp: i64 = row.get("created_at");
            let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
            
            playlists.push(Playlist {
                id: row.get("id"),
                name: row.get("name"),
                owner_id: row.get("owner_id"),
                owner_username: row.get("owner_username"),
                is_public: row.get::<i32, _>("is_public") != 0,
                created_at,
            });
        }
        
        Ok(playlists)
    }
    
    async fn get_shared_playlists(&self, user_id: &str) -> Result<Vec<(Playlist, PlaylistShare)>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                p.id, p.name, p.owner_id, p.owner_username, p.is_public, p.created_at,
                ps.id as share_id, ps.playlist_id, ps.shared_with_user_id, ps.shared_by_user_id, ps.shared_at
            FROM playlists p
            INNER JOIN playlist_shares ps ON p.id = ps.playlist_id
            WHERE ps.shared_with_user_id = ?
            ORDER BY ps.shared_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut results = Vec::new();
        for row in rows {
            let playlist_timestamp: i64 = row.get("created_at");
            let playlist_created_at = OffsetDateTime::from_unix_timestamp(playlist_timestamp)
                .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
            
            let share_timestamp: i64 = row.get("shared_at");
            let shared_at = OffsetDateTime::from_unix_timestamp(share_timestamp)
                .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
            
            let playlist = Playlist {
                id: row.get("id"),
                name: row.get("name"),
                owner_id: row.get("owner_id"),
                owner_username: row.get("owner_username"),
                is_public: row.get::<i32, _>("is_public") != 0,
                created_at: playlist_created_at,
            };
            
            let share = PlaylistShare {
                id: row.get("share_id"),
                playlist_id: row.get("playlist_id"),
                shared_with_user_id: row.get("shared_with_user_id"),
                shared_by_user_id: row.get("shared_by_user_id"),
                shared_at,
            };
            
            results.push((playlist, share));
        }
        
        Ok(results)
    }
    
    async fn delete_playlist(&self, playlist_id: &str, owner_id: &str) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM playlists WHERE id = ? AND owner_id = ?")
            .bind(playlist_id)
            .bind(owner_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to delete playlist: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::DatabaseError("Playlist not found or unauthorized".to_string()));
        }
        
        Ok(())
    }
    
    async fn add_song_to_playlist(&self, playlist_id: &str, song_id: &str) -> Result<(), DbError> {
        // Check if song exists
        let _ = self.get_song_by_id(song_id).await?;
        
        // Check if playlist exists
        let _ = self.get_playlist_by_id(playlist_id).await?;
        
        let added_at = OffsetDateTime::now_utc();
        let added_at_str = added_at.unix_timestamp().to_string();
        
        sqlx::query(
            "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id, added_at) VALUES (?, ?, ?)"
        )
        .bind(playlist_id)
        .bind(song_id)
        .bind(&added_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to add song to playlist: {}", e)))?;
        
        Ok(())
    }
    
    async fn remove_song_from_playlist(&self, playlist_id: &str, song_id: &str) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM playlist_songs WHERE playlist_id = ? AND song_id = ?")
            .bind(playlist_id)
            .bind(song_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to remove song from playlist: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::DatabaseError("Song not in playlist".to_string()));
        }
        
        Ok(())
    }
    
    async fn get_playlist_songs(&self, playlist_id: &str) -> Result<Vec<Song>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT s.id, s.title, s.artist_id, s.artist_name, s.album, s.duration, s.file_path, s.cover_image_path, s.created_at
            FROM songs s
            INNER JOIN playlist_songs ps ON s.id = ps.song_id
            WHERE ps.playlist_id = ?
            ORDER BY ps.added_at ASC
            "#
        )
        .bind(playlist_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut songs = Vec::new();
        for row in rows {
            let timestamp: i64 = row.get("created_at");
            let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
            
            songs.push(Song {
                id: row.get("id"),
                title: row.get("title"),
                artist_id: row.get("artist_id"),
                artist_name: row.get("artist_name"),
                album: row.get("album"),
                duration: row.get("duration"),
                file_path: row.get("file_path"),
                cover_image_path: row.get("cover_image_path"),
                created_at,
            });
        }
        
        Ok(songs)
    }
    
    async fn is_song_in_playlist(&self, playlist_id: &str, song_id: &str) -> Result<bool, DbError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = ? AND song_id = ?"
        )
        .bind(playlist_id)
        .bind(song_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count > 0)
    }
    
    async fn share_playlist(&self, playlist_id: &str, shared_with_user_id: &str, shared_by_user_id: &str) -> Result<PlaylistShare, DbError> {
        // Check if playlist exists
        let _ = self.get_playlist_by_id(playlist_id).await?;
        
        // Check if users exist
        let _ = self.get_user_by_id(shared_with_user_id).await?;
        let _ = self.get_user_by_id(shared_by_user_id).await?;
        
        let id = Uuid::new_v4().to_string();
        let shared_at = OffsetDateTime::now_utc();
        let shared_at_str = shared_at.unix_timestamp().to_string();
        
        sqlx::query(
            "INSERT OR REPLACE INTO playlist_shares (id, playlist_id, shared_with_user_id, shared_by_user_id, shared_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(playlist_id)
        .bind(shared_with_user_id)
        .bind(shared_by_user_id)
        .bind(&shared_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to share playlist: {}", e)))?;
        
        Ok(PlaylistShare {
            id,
            playlist_id: playlist_id.to_string(),
            shared_with_user_id: shared_with_user_id.to_string(),
            shared_by_user_id: shared_by_user_id.to_string(),
            shared_at,
        })
    }
    
    async fn revoke_playlist_share(&self, playlist_id: &str, shared_with_user_id: &str) -> Result<(), DbError> {
        let result = sqlx::query(
            "DELETE FROM playlist_shares WHERE playlist_id = ? AND shared_with_user_id = ?"
        )
        .bind(playlist_id)
        .bind(shared_with_user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Failed to revoke playlist share: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::DatabaseError("Playlist share not found".to_string()));
        }
        
        Ok(())
    }
    
    async fn is_playlist_shared_with_user(&self, playlist_id: &str, user_id: &str) -> Result<bool, DbError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM playlist_shares WHERE playlist_id = ? AND shared_with_user_id = ?"
        )
        .bind(playlist_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count > 0)
    }
    
    // Admin playlist operations
    async fn get_all_playlists(&self, offset: usize, limit: usize) -> Result<Vec<Playlist>, DbError> {
        let rows = sqlx::query(
            "SELECT id, name, owner_id, owner_username, is_public, created_at FROM playlists ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut playlists = Vec::new();
        for row in rows {
            let timestamp: i64 = row.get("created_at");
            let created_at = OffsetDateTime::from_unix_timestamp(timestamp)
                .map_err(|e| DbError::DatabaseError(format!("Invalid timestamp: {}", e)))?;
            
            playlists.push(Playlist {
                id: row.get("id"),
                name: row.get("name"),
                owner_id: row.get("owner_id"),
                owner_username: row.get("owner_username"),
                is_public: row.get::<i32, _>("is_public") != 0,
                created_at,
            });
        }
        
        Ok(playlists)
    }
    
    async fn get_total_playlists(&self) -> Result<usize, DbError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM playlists")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count as usize)
    }
    
    async fn update_playlist_name(&self, playlist_id: &str, new_name: &str) -> Result<(), DbError> {
        let result = sqlx::query("UPDATE playlists SET name = ? WHERE id = ?")
            .bind(new_name)
            .bind(playlist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update playlist name: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::DatabaseError("Playlist not found".to_string()));
        }
        
        Ok(())
    }
    
    async fn update_playlist_visibility(&self, playlist_id: &str, is_public: bool) -> Result<(), DbError> {
        let is_public_int = if is_public { 1 } else { 0 };
        
        let result = sqlx::query("UPDATE playlists SET is_public = ? WHERE id = ?")
            .bind(is_public_int)
            .bind(playlist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update playlist visibility: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::DatabaseError("Playlist not found".to_string()));
        }
        
        Ok(())
    }
    
    async fn delete_playlist_by_id(&self, playlist_id: &str) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(playlist_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to delete playlist: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(DbError::DatabaseError("Playlist not found".to_string()));
        }
        
        Ok(())
    }
}
