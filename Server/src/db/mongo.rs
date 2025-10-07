use async_trait::async_trait;
use mongodb::{Client, Collection, bson::doc};
use uuid::Uuid;
use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

use crate::db::{Database, DbError};
use crate::db::models::{User, Artist, Song, Playlist, PlaylistShare};

#[derive(Debug, Serialize, Deserialize)]
struct MongoUser {
    #[serde(rename = "_id")]
    id: String,
    username: String,
    email: String,
    password_hash: String,
    is_admin: bool,
    created_at: i64,
}

impl From<MongoUser> for User {
    fn from(mongo_user: MongoUser) -> Self {
        let created_at = OffsetDateTime::from_unix_timestamp(mongo_user.created_at)
            .unwrap_or_else(|_| OffsetDateTime::now_utc());
        
        User {
            id: mongo_user.id,
            username: mongo_user.username,
            email: mongo_user.email,
            password_hash: mongo_user.password_hash,
            is_admin: mongo_user.is_admin,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MongoArtist {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    cover_image_path: Option<String>,
    created_at: i64,
}

impl From<MongoArtist> for Artist {
    fn from(mongo_artist: MongoArtist) -> Self {
        let created_at = OffsetDateTime::from_unix_timestamp(mongo_artist.created_at)
            .unwrap_or_else(|_| OffsetDateTime::now_utc());
        
        Artist {
            id: mongo_artist.id,
            name: mongo_artist.name,
            cover_image_path: mongo_artist.cover_image_path,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MongoSong {
    #[serde(rename = "_id")]
    id: String,
    title: String,
    artist_id: String,
    artist_name: String,
    album: Option<String>,
    duration: Option<i32>,
    file_path: String,
    cover_image_path: Option<String>,
    created_at: i64,
}

impl From<MongoSong> for Song {
    fn from(mongo_song: MongoSong) -> Self {
        let created_at = OffsetDateTime::from_unix_timestamp(mongo_song.created_at)
            .unwrap_or_else(|_| OffsetDateTime::now_utc());
        
        Song {
            id: mongo_song.id,
            title: mongo_song.title,
            artist_id: mongo_song.artist_id,
            artist_name: mongo_song.artist_name,
            album: mongo_song.album,
            duration: mongo_song.duration,
            file_path: mongo_song.file_path,
            cover_image_path: mongo_song.cover_image_path,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MongoPlaylist {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    owner_id: String,
    owner_username: String,
    is_public: bool,
    created_at: i64,
}

impl From<MongoPlaylist> for Playlist {
    fn from(mongo_playlist: MongoPlaylist) -> Self {
        let created_at = OffsetDateTime::from_unix_timestamp(mongo_playlist.created_at)
            .unwrap_or_else(|_| OffsetDateTime::now_utc());
        
        Playlist {
            id: mongo_playlist.id,
            name: mongo_playlist.name,
            owner_id: mongo_playlist.owner_id,
            owner_username: mongo_playlist.owner_username,
            is_public: mongo_playlist.is_public,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MongoPlaylistSong {
    playlist_id: String,
    song_id: String,
    added_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MongoPlaylistShare {
    #[serde(rename = "_id")]
    id: String,
    playlist_id: String,
    shared_with_user_id: String,
    shared_by_user_id: String,
    shared_at: i64,
}

impl From<MongoPlaylistShare> for PlaylistShare {
    fn from(mongo_share: MongoPlaylistShare) -> Self {
        let shared_at = OffsetDateTime::from_unix_timestamp(mongo_share.shared_at)
            .unwrap_or_else(|_| OffsetDateTime::now_utc());
        
        PlaylistShare {
            id: mongo_share.id,
            playlist_id: mongo_share.playlist_id,
            shared_with_user_id: mongo_share.shared_with_user_id,
            shared_by_user_id: mongo_share.shared_by_user_id,
            shared_at,
        }
    }
}

pub struct MongoDatabase {
    users_collection: Collection<MongoUser>,
    artists_collection: Collection<MongoArtist>,
    songs_collection: Collection<MongoSong>,
    playlists_collection: Collection<MongoPlaylist>,
    playlist_songs_collection: Collection<MongoPlaylistSong>,
    playlist_shares_collection: Collection<MongoPlaylistShare>,
}

impl MongoDatabase {
    pub async fn new(connection_string: &str) -> Result<Self, DbError> {
        let client = Client::with_uri_str(connection_string)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to connect to MongoDB: {}", e)))?;
        
        // Default to "muse" database
        let database = client.database("muse");
        let users_collection = database.collection::<MongoUser>("users");
        let artists_collection = database.collection::<MongoArtist>("artists");
        let songs_collection = database.collection::<MongoSong>("songs");
        let playlists_collection = database.collection::<MongoPlaylist>("playlists");
        let playlist_songs_collection = database.collection::<MongoPlaylistSong>("playlist_songs");
        let playlist_shares_collection = database.collection::<MongoPlaylistShare>("playlist_shares");
        
        Ok(Self { 
            users_collection,
            artists_collection,
            songs_collection,
            playlists_collection,
            playlist_songs_collection,
            playlist_shares_collection,
        })
    }
}

#[async_trait]
impl Database for MongoDatabase {
    async fn initialize(&self) -> Result<(), DbError> {
        // Create unique indices for username and email
        use mongodb::IndexModel;
        use mongodb::options::IndexOptions;
        
        let username_index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        
        let email_index = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        
        self.users_collection
            .create_index(username_index)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create username index: {}", e)))?;
        
        self.users_collection
            .create_index(email_index)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create email index: {}", e)))?;
        
        // Create unique index for artist names
        let artist_name_index = IndexModel::builder()
            .keys(doc! { "name": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        
        self.artists_collection
            .create_index(artist_name_index)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create artist name index: {}", e)))?;
        
        // Create index for song artist_id
        let song_artist_index = IndexModel::builder()
            .keys(doc! { "artist_id": 1 })
            .build();
        
        self.songs_collection
            .create_index(song_artist_index)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to create song artist index: {}", e)))?;
        
        Ok(())
    }
    
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
        let created_at_timestamp = created_at.unix_timestamp();
        
        let mongo_user = MongoUser {
            id: id.clone(),
            username: username.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            is_admin: false,
            created_at: created_at_timestamp,
        };
        
        self.users_collection
            .insert_one(&mongo_user)
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
        let filter = doc! { "username": username };
        
        let mongo_user = self.users_collection
            .find_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
            .ok_or(DbError::UserNotFound)?;
        
        Ok(mongo_user.into())
    }
    
    async fn get_user_by_email(&self, email: &str) -> Result<User, DbError> {
        let filter = doc! { "email": email };
        
        let mongo_user = self.users_collection
            .find_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
            .ok_or(DbError::UserNotFound)?;
        
        Ok(mongo_user.into())
    }
    
    async fn get_user_by_id(&self, id: &str) -> Result<User, DbError> {
        let filter = doc! { "_id": id };
        
        let mongo_user = self.users_collection
            .find_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
            .ok_or(DbError::UserNotFound)?;
        
        Ok(mongo_user.into())
    }
    
    async fn update_user_admin_status(&self, id: &str, is_admin: bool) -> Result<(), DbError> {
        let filter = doc! { "_id": id };
        let update = doc! { "$set": { "is_admin": is_admin } };
        
        self.users_collection
            .update_one(filter, update)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update user: {}", e)))?;
        
        Ok(())
    }
    
    async fn username_exists(&self, username: &str) -> Result<bool, DbError> {
        let filter = doc! { "username": username };
        
        let count = self.users_collection
            .count_documents(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count > 0)
    }
    
    async fn email_exists(&self, email: &str) -> Result<bool, DbError> {
        let filter = doc! { "email": email };
        
        let count = self.users_collection
            .count_documents(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count > 0)
    }
    
    async fn get_all_users(&self, offset: usize, limit: usize) -> Result<Vec<User>, DbError> {
        use mongodb::options::FindOptions;
        
        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(offset as u64)
            .limit(limit as i64)
            .build();
        
        let mut cursor = self.users_collection
            .find(doc! {})
            .with_options(options)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut users = Vec::new();
        while cursor.advance().await
            .map_err(|e| DbError::DatabaseError(format!("Failed to iterate cursor: {}", e)))? {
            let mongo_user = cursor.deserialize_current()
                .map_err(|e| DbError::DatabaseError(format!("Failed to deserialize user: {}", e)))?;
            users.push(mongo_user.into());
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
        
        let filter = doc! { "username": username };
        let update = doc! { "$set": { "email": new_email } };
        
        let result = self.users_collection
            .update_one(filter, update)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update user: {}", e)))?;
        
        if result.matched_count == 0 {
            return Err(DbError::UserNotFound);
        }
        
        Ok(())
    }
    
    async fn update_username(&self, user_id: &str, new_username: &str) -> Result<(), DbError> {
        // Check if new username already exists
        if self.username_exists(new_username).await? {
            return Err(DbError::UserAlreadyExists);
        }
        
        let filter = doc! { "id": user_id };
        let update = doc! { "$set": { "username": new_username } };
        
        let result = self.users_collection
            .update_one(filter, update)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update username: {}", e)))?;
        
        if result.matched_count == 0 {
            return Err(DbError::UserNotFound);
        }
        
        Ok(())
    }
    
    async fn update_user_password(&self, user_id: &str, new_password_hash: &str) -> Result<(), DbError> {
        let filter = doc! { "id": user_id };
        let update = doc! { "$set": { "password_hash": new_password_hash } };
        
        let result = self.users_collection
            .update_one(filter, update)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update password: {}", e)))?;
        
        if result.matched_count == 0 {
            return Err(DbError::UserNotFound);
        }
        
        Ok(())
    }
    
    async fn delete_user_by_username(&self, username: &str) -> Result<(), DbError> {
        let filter = doc! { "username": username };

        let result = self.users_collection
            .delete_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to delete user: {}", e)))?;

        if result.deleted_count == 0 {
            return Err(DbError::UserNotFound);
        }

        Ok(())
    }
    
    async fn delete_user_by_id(&self, user_id: &str) -> Result<(), DbError> {
        let filter = doc! { "id": user_id };

        let result = self.users_collection
            .delete_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to delete user: {}", e)))?;

        if result.deleted_count == 0 {
            return Err(DbError::UserNotFound);
        }

        Ok(())
    }
    
    async fn get_total_users(&self) -> Result<usize, DbError> {
        let count = self.users_collection
            .count_documents(doc! {})
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
        let created_at_timestamp = created_at.unix_timestamp();
        
        let mongo_artist = MongoArtist {
            id: id.clone(),
            name: name.to_string(),
            cover_image_path: None,
            created_at: created_at_timestamp,
        };
        
        self.artists_collection
            .insert_one(&mongo_artist)
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
        let filter = doc! { "_id": id };
        
        let mongo_artist = self.artists_collection
            .find_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
            .ok_or(DbError::DatabaseError("Artist not found".to_string()))?;
        
        Ok(mongo_artist.into())
    }
    
    async fn get_artist_by_name(&self, name: &str) -> Result<Artist, DbError> {
        let filter = doc! { "name": name };
        
        let mongo_artist = self.artists_collection
            .find_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
            .ok_or(DbError::DatabaseError("Artist not found".to_string()))?;
        
        Ok(mongo_artist.into())
    }
    
    async fn get_artists(&self, offset: usize, limit: usize) -> Result<Vec<Artist>, DbError> {
        use mongodb::options::FindOptions;
        
        let options = FindOptions::builder()
            .sort(doc! { "name": 1 })
            .skip(offset as u64)
            .limit(limit as i64)
            .build();
        
        let mut cursor = self.artists_collection
            .find(doc! {})
            .with_options(options)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut artists = Vec::new();
        while cursor.advance().await
            .map_err(|e| DbError::DatabaseError(format!("Failed to iterate cursor: {}", e)))? {
            let mongo_artist = cursor.deserialize_current()
                .map_err(|e| DbError::DatabaseError(format!("Failed to deserialize artist: {}", e)))?;
            artists.push(mongo_artist.into());
        }
        
        Ok(artists)
    }
    
    async fn get_total_artists(&self) -> Result<usize, DbError> {
        let count = self.artists_collection
            .count_documents(doc! {})
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count as usize)
    }
    
    async fn update_artist_cover(&self, id: &str, cover_path: &str) -> Result<(), DbError> {
        let filter = doc! { "_id": id };
        let update = doc! { "$set": { "cover_image_path": cover_path } };
        
        let result = self.artists_collection
            .update_one(filter, update)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update artist: {}", e)))?;
        
        if result.matched_count == 0 {
            return Err(DbError::DatabaseError("Artist not found".to_string()));
        }
        
        Ok(())
    }
    
    async fn artist_exists(&self, name: &str) -> Result<bool, DbError> {
        let filter = doc! { "name": name };
        
        let count = self.artists_collection
            .count_documents(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count > 0)
    }
    
    // Song operations
    async fn create_song(&self, title: &str, artist_id: &str, file_path: &str) -> Result<Song, DbError> {
        let artist = self.get_artist_by_id(artist_id).await?;
        
        let id = Uuid::new_v4().to_string();
        let created_at = OffsetDateTime::now_utc();
        let created_at_timestamp = created_at.unix_timestamp();
        
        let mongo_song = MongoSong {
            id: id.clone(),
            title: title.to_string(),
            artist_id: artist_id.to_string(),
            artist_name: artist.name.clone(),
            album: None,
            duration: None,
            file_path: file_path.to_string(),
            cover_image_path: None,
            created_at: created_at_timestamp,
        };
        
        self.songs_collection
            .insert_one(&mongo_song)
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
        let filter = doc! { "_id": id };
        
        let mongo_song = self.songs_collection
            .find_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
            .ok_or(DbError::DatabaseError("Song not found".to_string()))?;
        
        Ok(mongo_song.into())
    }
    
    async fn get_songs_by_artist(&self, artist_id: &str) -> Result<Vec<Song>, DbError> {
        use mongodb::options::FindOptions;
        
        let filter = doc! { "artist_id": artist_id };
        let options = FindOptions::builder()
            .sort(doc! { "title": 1 })
            .build();
        
        let mut cursor = self.songs_collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut songs = Vec::new();
        while cursor.advance().await
            .map_err(|e| DbError::DatabaseError(format!("Failed to iterate cursor: {}", e)))? {
            let mongo_song = cursor.deserialize_current()
                .map_err(|e| DbError::DatabaseError(format!("Failed to deserialize song: {}", e)))?;
            songs.push(mongo_song.into());
        }
        
        Ok(songs)
    }
    
    async fn get_songs(&self, offset: usize, limit: usize) -> Result<Vec<Song>, DbError> {
        use mongodb::options::FindOptions;
        
        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(offset as u64)
            .limit(limit as i64)
            .build();
        
        let mut cursor = self.songs_collection
            .find(doc! {})
            .with_options(options)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut songs = Vec::new();
        while cursor.advance().await
            .map_err(|e| DbError::DatabaseError(format!("Failed to iterate cursor: {}", e)))? {
            let mongo_song = cursor.deserialize_current()
                .map_err(|e| DbError::DatabaseError(format!("Failed to deserialize song: {}", e)))?;
            songs.push(mongo_song.into());
        }
        
        Ok(songs)
    }
    
    async fn get_total_songs(&self) -> Result<usize, DbError> {
        let count = self.songs_collection
            .count_documents(doc! {})
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count as usize)
    }
    
    async fn update_song_metadata(&self, id: &str, album: Option<&str>, duration: Option<i32>, cover_path: Option<&str>) -> Result<(), DbError> {
        let filter = doc! { "_id": id };
        let mut updates = doc! {};
        
        if let Some(album_val) = album {
            updates.insert("album", album_val);
        }
        if let Some(duration_val) = duration {
            updates.insert("duration", duration_val);
        }
        if let Some(cover_val) = cover_path {
            updates.insert("cover_image_path", cover_val);
        }
        
        let update = doc! { "$set": updates };
        
        let result = self.songs_collection
            .update_one(filter, update)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update song: {}", e)))?;
        
        if result.matched_count == 0 {
            return Err(DbError::DatabaseError("Song not found".to_string()));
        }
        
        Ok(())
    }
    
    async fn search_songs(&self, query: &str, offset: usize, limit: usize) -> Result<Vec<Song>, DbError> {
        use mongodb::options::FindOptions;
        use mongodb::bson::Regex;
        
        let regex = Regex {
            pattern: query.to_string(),
            options: "i".to_string(), // case-insensitive
        };
        
        let filter = doc! {
            "$or": [
                { "title": { "$regex": regex.clone() } },
                { "artist_name": { "$regex": regex } }
            ]
        };
        
        let options = FindOptions::builder()
            .sort(doc! { "title": 1 })
            .skip(offset as u64)
            .limit(limit as i64)
            .build();
        
        let mut cursor = self.songs_collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut songs = Vec::new();
        while cursor.advance().await
            .map_err(|e| DbError::DatabaseError(format!("Failed to iterate cursor: {}", e)))? {
            let mongo_song = cursor.deserialize_current()
                .map_err(|e| DbError::DatabaseError(format!("Failed to deserialize song: {}", e)))?;
            songs.push(mongo_song.into());
        }
        
        Ok(songs)
    }
    
    async fn delete_song_by_id(&self, id: &str) -> Result<(), DbError> {
        let filter = doc! { "_id": id };
        
        let result = self.songs_collection
            .delete_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to delete song: {}", e)))?;
        
        if result.deleted_count == 0 {
            return Err(DbError::DatabaseError("Song not found".to_string()));
        }
        
        Ok(())
    }
    
    // Playlist operations
    async fn create_playlist(&self, name: &str, owner_id: &str, is_public: bool) -> Result<Playlist, DbError> {
        let owner = self.get_user_by_id(owner_id).await?;
        
        let id = Uuid::new_v4().to_string();
        let created_at = OffsetDateTime::now_utc();
        let created_at_timestamp = created_at.unix_timestamp();
        
        let mongo_playlist = MongoPlaylist {
            id: id.clone(),
            name: name.to_string(),
            owner_id: owner_id.to_string(),
            owner_username: owner.username.clone(),
            is_public,
            created_at: created_at_timestamp,
        };
        
        self.playlists_collection
            .insert_one(&mongo_playlist)
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
        let filter = doc! { "_id": id };
        
        let mongo_playlist = self.playlists_collection
            .find_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
            .ok_or(DbError::DatabaseError("Playlist not found".to_string()))?;
        
        Ok(mongo_playlist.into())
    }
    
    async fn get_playlist_by_name_and_owner(&self, name: &str, owner_id: &str) -> Result<Playlist, DbError> {
        let filter = doc! { "name": name, "owner_id": owner_id };
        
        let mongo_playlist = self.playlists_collection
            .find_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?
            .ok_or(DbError::DatabaseError("Playlist not found".to_string()))?;
        
        Ok(mongo_playlist.into())
    }
    
    async fn get_user_playlists(&self, user_id: &str, offset: usize, limit: usize) -> Result<Vec<Playlist>, DbError> {
        use mongodb::options::FindOptions;
        
        let filter = doc! { "owner_id": user_id };
        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(offset as u64)
            .limit(limit as i64)
            .build();
        
        let mut cursor = self.playlists_collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut playlists = Vec::new();
        while cursor.advance().await
            .map_err(|e| DbError::DatabaseError(format!("Failed to iterate cursor: {}", e)))? {
            let mongo_playlist = cursor.deserialize_current()
                .map_err(|e| DbError::DatabaseError(format!("Failed to deserialize playlist: {}", e)))?;
            playlists.push(mongo_playlist.into());
        }
        
        Ok(playlists)
    }
    
    async fn get_public_playlists(&self, offset: usize, limit: usize) -> Result<Vec<Playlist>, DbError> {
        use mongodb::options::FindOptions;
        
        let filter = doc! { "is_public": true };
        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(offset as u64)
            .limit(limit as i64)
            .build();
        
        let mut cursor = self.playlists_collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut playlists = Vec::new();
        while cursor.advance().await
            .map_err(|e| DbError::DatabaseError(format!("Failed to iterate cursor: {}", e)))? {
            let mongo_playlist = cursor.deserialize_current()
                .map_err(|e| DbError::DatabaseError(format!("Failed to deserialize playlist: {}", e)))?;
            playlists.push(mongo_playlist.into());
        }
        
        Ok(playlists)
    }
    
    async fn get_shared_playlists(&self, user_id: &str) -> Result<Vec<(Playlist, PlaylistShare)>, DbError> {
        let filter = doc! { "shared_with_user_id": user_id };
        
        let mut cursor = self.playlist_shares_collection
            .find(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut results = Vec::new();
        while cursor.advance().await
            .map_err(|e| DbError::DatabaseError(format!("Failed to iterate cursor: {}", e)))? {
            let mongo_share: MongoPlaylistShare = cursor.deserialize_current()
                .map_err(|e| DbError::DatabaseError(format!("Failed to deserialize share: {}", e)))?;
            
            let playlist = self.get_playlist_by_id(&mongo_share.playlist_id).await?;
            let share: PlaylistShare = mongo_share.into();
            
            results.push((playlist, share));
        }
        
        Ok(results)
    }
    
    async fn delete_playlist(&self, playlist_id: &str, owner_id: &str) -> Result<(), DbError> {
        let filter = doc! { "_id": playlist_id, "owner_id": owner_id };
        
        let result = self.playlists_collection
            .delete_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to delete playlist: {}", e)))?;
        
        if result.deleted_count == 0 {
            return Err(DbError::DatabaseError("Playlist not found or unauthorized".to_string()));
        }
        
        // Also delete associated playlist songs and shares
        let playlist_filter = doc! { "playlist_id": playlist_id };
        let _ = self.playlist_songs_collection.delete_many(playlist_filter.clone()).await;
        let _ = self.playlist_shares_collection.delete_many(playlist_filter).await;
        
        Ok(())
    }
    
    async fn add_song_to_playlist(&self, playlist_id: &str, song_id: &str) -> Result<(), DbError> {
        // Verify song and playlist exist
        let _ = self.get_song_by_id(song_id).await?;
        let _ = self.get_playlist_by_id(playlist_id).await?;
        
        let added_at = OffsetDateTime::now_utc();
        let added_at_timestamp = added_at.unix_timestamp();
        
        let mongo_playlist_song = MongoPlaylistSong {
            playlist_id: playlist_id.to_string(),
            song_id: song_id.to_string(),
            added_at: added_at_timestamp,
        };
        
        // Try to insert, ignore if already exists
        let _ = self.playlist_songs_collection
            .insert_one(&mongo_playlist_song)
            .await;
        
        Ok(())
    }
    
    async fn remove_song_from_playlist(&self, playlist_id: &str, song_id: &str) -> Result<(), DbError> {
        let filter = doc! { "playlist_id": playlist_id, "song_id": song_id };
        
        let result = self.playlist_songs_collection
            .delete_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to remove song from playlist: {}", e)))?;
        
        if result.deleted_count == 0 {
            return Err(DbError::DatabaseError("Song not in playlist".to_string()));
        }
        
        Ok(())
    }
    
    async fn get_playlist_songs(&self, playlist_id: &str) -> Result<Vec<Song>, DbError> {
        use mongodb::options::FindOptions;
        
        let filter = doc! { "playlist_id": playlist_id };
        let options = FindOptions::builder()
            .sort(doc! { "added_at": 1 })
            .build();
        
        let mut cursor = self.playlist_songs_collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        let mut songs = Vec::new();
        while cursor.advance().await
            .map_err(|e| DbError::DatabaseError(format!("Failed to iterate cursor: {}", e)))? {
            let playlist_song: MongoPlaylistSong = cursor.deserialize_current()
                .map_err(|e| DbError::DatabaseError(format!("Failed to deserialize playlist song: {}", e)))?;
            
            if let Ok(song) = self.get_song_by_id(&playlist_song.song_id).await {
                songs.push(song);
            }
        }
        
        Ok(songs)
    }
    
    async fn is_song_in_playlist(&self, playlist_id: &str, song_id: &str) -> Result<bool, DbError> {
        let filter = doc! { "playlist_id": playlist_id, "song_id": song_id };
        
        let count = self.playlist_songs_collection
            .count_documents(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;
        
        Ok(count > 0)
    }
    
    async fn share_playlist(&self, playlist_id: &str, shared_with_user_id: &str, shared_by_user_id: &str) -> Result<PlaylistShare, DbError> {
        // Verify playlist and users exist
        let _ = self.get_playlist_by_id(playlist_id).await?;
        let _ = self.get_user_by_id(shared_with_user_id).await?;
        let _ = self.get_user_by_id(shared_by_user_id).await?;
        
        let id = Uuid::new_v4().to_string();
        let shared_at = OffsetDateTime::now_utc();
        let shared_at_timestamp = shared_at.unix_timestamp();
        
        let mongo_share = MongoPlaylistShare {
            id: id.clone(),
            playlist_id: playlist_id.to_string(),
            shared_with_user_id: shared_with_user_id.to_string(),
            shared_by_user_id: shared_by_user_id.to_string(),
            shared_at: shared_at_timestamp,
        };
        
        // Delete existing share if any
        let filter = doc! { "playlist_id": playlist_id, "shared_with_user_id": shared_with_user_id };
        let _ = self.playlist_shares_collection.delete_one(filter).await;
        
        // Insert new share
        self.playlist_shares_collection
            .insert_one(&mongo_share)
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
        let filter = doc! { "playlist_id": playlist_id, "shared_with_user_id": shared_with_user_id };
        
        let result = self.playlist_shares_collection
            .delete_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to revoke playlist share: {}", e)))?;
        
        if result.deleted_count == 0 {
            return Err(DbError::DatabaseError("Playlist share not found".to_string()));
        }
        
        Ok(())
    }
    
    async fn is_playlist_shared_with_user(&self, playlist_id: &str, user_id: &str) -> Result<bool, DbError> {
        let filter = doc! { "playlist_id": playlist_id, "shared_with_user_id": user_id };

        let count = self.playlist_shares_collection
            .count_documents(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;

        Ok(count > 0)
    }
    
    // Admin playlist operations
    async fn get_all_playlists(&self, offset: usize, limit: usize) -> Result<Vec<Playlist>, DbError> {
        use mongodb::options::FindOptions;
        
        let options = FindOptions::builder()
            .skip(offset as u64)
            .limit(limit as i64)
            .sort(doc! { "created_at": -1 })
            .build();

        let mut cursor = self.playlists_collection
            .find(doc! {})
            .with_options(options)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;

        let mut playlists = Vec::new();
        while cursor.advance().await.map_err(|e| DbError::DatabaseError(format!("Cursor error: {}", e)))? {
            let mongo_playlist: MongoPlaylist = cursor.deserialize_current()
                .map_err(|e| DbError::DatabaseError(format!("Failed to deserialize playlist: {}", e)))?;
            playlists.push(mongo_playlist.into());
        }

        Ok(playlists)
    }
    
    async fn get_total_playlists(&self) -> Result<usize, DbError> {
        let count = self.playlists_collection
            .count_documents(doc! {})
            .await
            .map_err(|e| DbError::DatabaseError(format!("Database query failed: {}", e)))?;

        Ok(count as usize)
    }
    
    async fn update_playlist_name(&self, playlist_id: &str, new_name: &str) -> Result<(), DbError> {
        let filter = doc! { "id": playlist_id };
        let update = doc! { "$set": { "name": new_name } };

        let result = self.playlists_collection
            .update_one(filter, update)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update playlist name: {}", e)))?;

        if result.matched_count == 0 {
            return Err(DbError::DatabaseError("Playlist not found".to_string()));
        }

        Ok(())
    }
    
    async fn update_playlist_visibility(&self, playlist_id: &str, is_public: bool) -> Result<(), DbError> {
        let filter = doc! { "id": playlist_id };
        let update = doc! { "$set": { "is_public": is_public } };

        let result = self.playlists_collection
            .update_one(filter, update)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to update playlist visibility: {}", e)))?;

        if result.matched_count == 0 {
            return Err(DbError::DatabaseError("Playlist not found".to_string()));
        }

        Ok(())
    }
    
    async fn delete_playlist_by_id(&self, playlist_id: &str) -> Result<(), DbError> {
        let filter = doc! { "id": playlist_id };

        let result = self.playlists_collection
            .delete_one(filter)
            .await
            .map_err(|e| DbError::DatabaseError(format!("Failed to delete playlist: {}", e)))?;

        if result.deleted_count == 0 {
            return Err(DbError::DatabaseError("Playlist not found".to_string()));
        }

        Ok(())
    }
}
