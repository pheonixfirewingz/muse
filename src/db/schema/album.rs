use crate::db::DbPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow};
use uuid::Uuid;

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct Album {
    id: String,
    artist_id: String,
    title: String,
    year: String,
}

impl Album {
    /// Construct a new `Album` with the given fields.
    ///
    /// `id` is the ID of the album, `artist_id` is the ID of the artist who
    /// created the album, `title` is the title of the album, `year` is the year
    /// the album was released.
    pub fn new(id: Uuid, artist_id: Uuid, title: String, year: DateTime<Utc>) -> Self {
        Self { id: id.to_string(), artist_id: artist_id.to_string(), title, year: year.to_rfc3339() }
    }
    /// Returns the ID associated with this album as a `Uuid`.
    pub fn get_id(&self) -> Uuid {
        Uuid::parse_str(&self.id).unwrap()
    }
    /// Returns the artist ID associated with this album as a `Uuid`.
    pub fn get_artist_id(&self) -> Uuid {
        Uuid::parse_str(&self.artist_id).unwrap()
    }
    /// Returns a clone of the `title` field of the `Album` struct.
    pub fn get_title(&self) -> String {
        self.title.clone()
    }
    /// Returns the year associated with this album as a `DateTime<Utc>`.
    pub fn get_year(&self) -> Option<DateTime<Utc>> {
        let year = DateTime::parse_from_rfc3339(&self.year);
        match year {
            Ok(year) => Some(year.to_utc()),
            Err(_) => {
                println!("failed to parse year from album");
                None
            }
        }
    }
}

/// Fetch all albums from the database.
///
/// This function queries the database, and returns `None` if any error
/// occurs while doing so. Otherwise, it returns a `Some` containing a
/// `Vec` of `Album` values, which are the albums in the database.
///
/// # Errors
///
/// This function errors if the database query fails for any reason.
pub async fn get_albums(pool: &DbPool) -> Option<Vec<Album>> {
    let result = query_as::<_, Album>("SELECT id, artist_id, title, year FROM albums")
        .fetch_all(pool).await;
    match result {
        Ok(albums) => Some(albums),
        Err(e) => {
            println!("{}", e.to_string());
            None
        }
    }
}


/// Fetch all albums by a specific artist from the database.
///
/// This function queries the database for albums associated with the given
/// `artist_id`. It returns `None` if any error occurs while querying.
/// Otherwise, it returns a `Some` containing a `Vec` of `Album` values,
/// representing the albums created by the specified artist.
///
/// # Errors
///
/// This function will print an error message and return `None` if the
/// database query fails for any reason.
///
/// # Arguments
///
pub async fn get_albums_by_artist(pool: &DbPool, artist_id: Uuid) -> Option<Vec<Album>> {
    let result = query_as::<_, Album>(
        "SELECT id, artist_id, title, year FROM albums WHERE artist_id = ?"
    ).bind(artist_id.to_string()).fetch_all(pool).await;
    match result {
        Ok(albums) => Some(albums),
        Err(e) => {
            println!("{}", e.to_string());
            None
        },
    }
}


/// Insert a new album into the database.
///
/// This function inserts a new album record into the `albums` table with the
/// provided `artist_id`, `title`, and `year`. The `year` is converted to an
/// ISO 8601 string format before being stored. A new UUID is generated for
/// the album as its `id`.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `artist_id` - The UUID of the artist associated with the album.
/// * `title` - The title of the album.
/// * `year` - The release year of the album as a `DateTime<Utc>`.
///
/// # Errors
///
/// This function will print an error message if the database insertion fails.
pub async fn insert_album(pool: &DbPool, artist_id: Uuid, title: &str, year: DateTime<Utc>) {
    let result = query("INSERT INTO albums (id, artist_id, title, year) VALUES (?, ?, ?, ?)")
        .bind(Uuid::new_v4().to_string()).bind(artist_id.to_string()).bind(title.to_string()).bind(year.to_rfc3339()).execute(pool).await;
    match result {
        Ok(_) => (),
        Err(e) => println!("Failed to insert album: {}", e),
    }
}


/// Associate a list of songs with an album in the `album_songs` table.
///
/// This function loops through the provided `song_ids` and inserts a new record
/// into the `album_songs` table for each song, with the `album_id` being the
/// `album_id` provided. The function will print an error message to standard
/// output if the insertion fails for any reason.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `album_id` - The UUID of the album with which the songs should be associated.
/// * `song_ids` - A vector of UUIDs of the songs to associate with the album.
pub async fn associate_songs_with_album(pool: &DbPool, album_id: Uuid, song_ids: Vec<Uuid>) {
    for id in song_ids {
        let result = query("INSERT INTO album_songs (album_id, id) VALUES (?, ?)")
            .bind(album_id.to_string()).bind(id.to_string()).execute(pool).await;
        match result {
            Ok(_) => (),
            Err(e) => println!("Failed to associate song {} with album {}: {}", id, album_id, e),
        }
    }
}


/// Associates a single song with an album in the `album_songs` table.
///
/// This function inserts a new record into the `album_songs` table with the
/// `album_id` and `id` provided. The function will print an error message
/// to standard output if the insertion fails for any reason.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `album_id` - The UUID of the album with which the song should be associated.
/// * `id` - The UUID of the song to associate with the album.
///
/// # Errors
///
/// This function prints an error message if the database operation fails.
pub async fn associate_song_with_album(pool: &DbPool, album_id: Uuid, id: Uuid) {
    let result = query("INSERT INTO album_songs (album_id, id) VALUES (?, ?)")
        .bind(album_id.to_string()).bind(id.to_string()).execute(pool).await;
    match result {
        Ok(_) => (),
        Err(e) => println!("Failed to associate song to album: {}", e),
    }
}

/// Deletes a specific song from an album in the `album_songs` table.
///
/// This function removes the entry corresponding to the given `album_id` and `id`
/// from the `album_songs` table in the database. If the deletion fails for any reason,
/// an error message is printed to standard output.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `album_id` - The UUID of the album from which the song should be deleted.
/// * `id` - The UUID of the song to delete from the album.
///
/// # Errors
///
/// This function prints an error message if the database operation fails.
pub async fn delete_song_from_album(pool: &DbPool, album_id: Uuid, id: Uuid) {
    let result = query("DELETE FROM album_songs WHERE album_id = ? AND id = ?")
        .bind(album_id.to_string()).bind(id.to_string()).execute(pool).await;
    match result {
        Ok(_) => (),
        Err(e) => println!("Failed to delete song {} from album {}: {}", id, album_id, e),
    }
}

/// Delete an album and all associated songs from the database.
///
/// This function first deletes the album with the specified `album_id` from
/// the `albums` table. Then, it deletes all records from the `album_songs`
/// table that have the same `album_id`.
///
/// # Errors
///
/// This function will print an error message to standard output if either the
/// album deletion or the album song deletion fails for any reason.
pub async fn delete_album(pool: &DbPool, album_id: Uuid) {
    let result = query("DELETE FROM albums WHERE id = ?")
        .bind(album_id.to_string())
        .execute(pool)
        .await;
    match result {
        Ok(_) => (),
        Err(e) => println!("Failed to delete album {}: {}", album_id, e),
    }

    let result = query("DELETE FROM album_songs WHERE album_id = ?")
        .bind(album_id.to_string()).execute(pool).await;
    match result {
        Ok(_) => (),
        Err(e) => println!("Failed to delete album songs for {}: {}", album_id, e),
    }
}

/// Retrieve all songs associated with an album.
///
/// This function queries the database for all songs associated with the album
/// specified by `album_id`. It returns a `Vec` of `Uuid` values, which are the
/// IDs of the songs associated with the album.
///
/// # Errors
///
/// This function will print an error message to standard output if the database
/// query fails for any reason. In this case, it will return an empty `Vec`.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `album_id` - The UUID of the album for which to retrieve associated songs.
pub async fn get_songs_for_album(pool: &DbPool, album_id: Uuid) -> Vec<Uuid> {
    let result = query_as::<_, (String,)>("SELECT id FROM album_songs WHERE album_id = ?")
        .bind(album_id.to_string()).fetch_all(pool).await;
    match result {
        Ok(songs) => songs.into_iter().map(|(id,)| Uuid::parse_str(&id).unwrap()).collect(),
        Err(e) => {
            println!("{}", e.to_string());
            vec![]
        }
    }
}

/// Creates the `albums` table in the database if it does not already exist.
///
/// If the table does not exist, this function will create it with the
/// specified schema. If the table already exists, this function does nothing.
///
/// # Errors
///
/// If the table creation fails for any reason, this function will print an
/// error message to standard output and do nothing else.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
pub async fn create_albums_table_if_not_exists(pool: &DbPool) {
    let result = query(
        "CREATE TABLE IF NOT EXISTS albums (
            id TEXT PRIMARY KEY,
            artist_id TEXT NOT NULL,
            title TEXT NOT NULL,
            year TEXT NOT NULL
        )"
    ).execute(pool).await;

    match result {
        Ok(_) => println!("Albums table is ready."),
        Err(e) => println!("Failed to create albums table: {}", e),
    }
}

/// Creates the album_songs table if it does not already exist.
///
/// The table is created with album_id and id as a composite primary key.
/// The table also has foreign key constraints to the album table and the 'songs' table,
/// with DELETE CASCADE. This means that if an album is deleted, all associated
/// songs are also deleted, and if a song is deleted, it is removed from all
/// associated albums.
///
/// If the table already exists, this function does nothing.
///
/// # Errors
///
/// If the table creation fails for any reason, this function will print an
/// error message and do nothing else.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
pub async fn create_album_songs_table_if_not_exists(pool: &DbPool) {
    let result = query(
        "CREATE TABLE IF NOT EXISTS album_songs (
            album_id TEXT NOT NULL,
            id TEXT NOT NULL,
            PRIMARY KEY (album_id, id),
            FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE,
            FOREIGN KEY (id) REFERENCES songs(id) ON DELETE CASCADE
        )"
    ).execute(pool).await;

    match result {
        Ok(_) => println!("Album_songs table is ready."),
        Err(e) => println!("Failed to create album_songs table: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::song::{create_song_entry, create_songs_table_if_not_exists, Song};
    use sqlx::sqlite::SqlitePoolOptions;
    use uuid::Uuid;

    async fn setup_test_db(name: &str) -> DbPool {
        let path = std::path::PathBuf::from(&name);
        std::fs::File::create(&path).expect("Failed to create test database file");

        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect(&format!("sqlite://{}", name)).await
            .expect("Failed to create test database pool");

        // Create tables
        create_albums_table_if_not_exists(&pool).await;
        create_songs_table_if_not_exists(&pool).await;
        create_album_songs_table_if_not_exists(&pool).await;

        pool
    }
    
    pub fn delete_test_db(name: &str) {
        let path = std::path::PathBuf::from(&name);
        std::fs::remove_file(&path).expect("Failed to delete test database file");
    }

    #[sqlx::test]
    async fn test_get_albums_empty() {
        let file_name = format!("test_dbs/{}-test.db", Uuid::new_v4());
        let pool = setup_test_db(&file_name).await;
        let albums = get_albums(&pool).await.unwrap();
        assert!(albums.is_empty(), "Expected empty albums list from empty database");
        delete_test_db(&file_name);
    }

    #[sqlx::test]
    async fn test_insert_and_get_album() {
        let file_name = format!("test_dbs/{}-test.db", Uuid::new_v4());
        let pool = setup_test_db(&file_name).await;
        let artist_id = Uuid::new_v4();
        let title = "Test Album";
        let year = Utc::now();

        insert_album(&pool, artist_id, title, year).await;

        let albums = get_albums(&pool).await.unwrap();

        assert_eq!(albums.len(), 1, "Expected one album after insertion");
        assert_eq!(albums[0].get_title(), title);
        assert_eq!(albums[0].get_artist_id(), artist_id);

        // Compare years with some tolerance for formatting differences
        let album_year = albums[0].get_year().unwrap();
        assert!(
            (album_year.timestamp() - year.timestamp()).abs() < 2,
            "Year should match within 2 seconds tolerance"
        );
        delete_test_db(&file_name);
    }

    #[sqlx::test]
    async fn test_get_albums_by_artist() {
        let file_name = format!("test_dbs/{}-test.db", Uuid::new_v4());
        let pool = setup_test_db(&file_name).await;
        let artist_id_1 = Uuid::new_v4();
        let artist_id_2 = Uuid::new_v4();

        // Insert albums for two different artists
        insert_album(&pool, artist_id_1, "Artist 1 Album 1", Utc::now()).await;
        insert_album(&pool, artist_id_1, "Artist 1 Album 2", Utc::now()).await;
        insert_album(&pool, artist_id_2, "Artist 2 Album", Utc::now()).await;

        // Test fetch for artist 1
        let albums = get_albums_by_artist(&pool, artist_id_1).await.unwrap();
        assert_eq!(albums.len(), 2, "Expected two albums for artist 1");

        // Test fetch for artist 2
        let albums = get_albums_by_artist(&pool, artist_id_2).await.unwrap();
        assert_eq!(albums.len(), 1, "Expected one album for artist 2");

        // Test fetch for non-existent artist
        let albums = get_albums_by_artist(&pool, Uuid::new_v4()).await.unwrap();
        assert!(albums.is_empty(), "Expected no albums for non-existent artist");
        delete_test_db(&file_name);
    }

    #[sqlx::test]
    fn test_album_getters() {
        let id = Uuid::new_v4();
        let artist_id = Uuid::new_v4();
        let title = "Test Album".to_string();
        let year = Utc::now();

        let album = Album::new(id, artist_id, title.clone(), year);

        assert_eq!(album.get_id(), id);
        assert_eq!(album.get_artist_id(), artist_id);
        assert_eq!(album.get_title(), title);

        let album_year = album.get_year().unwrap();
        assert!(
            (album_year.timestamp() - year.timestamp()).abs() < 2,
            "Year should match within 2 seconds tolerance"
        );
    }

    #[sqlx::test]
    async fn test_associate_songs_with_album() {
        let file_name = format!("test_dbs/{}-test.db", Uuid::new_v4());
        let pool = setup_test_db(&file_name).await;
        let artist_id = Uuid::new_v4();

        // Insert album
        let title = "Test Album with Songs";
        let year = Utc::now();
        insert_album(&pool, artist_id, title, year).await;

        // Get the album ID
        let albums = get_albums(&pool).await.unwrap();
        assert_eq!(albums.len(), 1);
        let album_id = albums[0].get_id();

        // Insert songs
        let song_1 = Song::new_auto_id("test song 1".to_string(),None,"".to_string());
            create_song_entry(&pool,&song_1).await;
        let song_2 = Song::new_auto_id("test song 2".to_string(),None,"".to_string());
            create_song_entry(&pool,&song_2).await;
        let song_id_1 = song_1.get_id();
        let song_id_2 = song_2.get_id();

        // Associate songs with album
        let song_ids = vec![song_id_1, song_id_2];
        associate_songs_with_album(&pool, album_id, song_ids.clone()).await;

        // Get songs for album
        let retrieved_song_ids = get_songs_for_album(&pool, album_id).await;

        assert_eq!(retrieved_song_ids.len(), 2, "Expected two songs associated with album");

        // Ensure all song IDs are in the retrieved list
        for id in song_ids {
            assert!(
                retrieved_song_ids.contains(&id),
                "Expected song {} to be associated with album",
                id
            );
        }
        delete_test_db(&file_name);
    }

    #[sqlx::test]
    async fn test_associate_song_with_album() {
        let file_name = format!("test_dbs/{}-test.db", Uuid::new_v4());
        let pool = setup_test_db(&file_name).await;
        let artist_id = Uuid::new_v4();

        // Insert album
        insert_album(&pool, artist_id, "Album for single song test", Utc::now()).await;

        // Get the album ID
        let albums = get_albums(&pool).await.unwrap();
        let album_id = albums[0].get_id();

        // Create song
        let song = Song::new_auto_id("test song".to_string(),None,"".to_string());

        // Insert song
        create_song_entry(&pool, &song).await;

        let id = song.get_id();

        // Associate single song with album
        associate_song_with_album(&pool, album_id, id).await;

        // Get songs for album
        let retrieved_song_ids = get_songs_for_album(&pool, album_id).await;

        assert_eq!(retrieved_song_ids.len(), 1, "Expected one song associated with album");
        assert_eq!(retrieved_song_ids[0], id, "Expected song ID to match");
        delete_test_db(&file_name);
    }

    #[sqlx::test]
    async fn test_delete_song_from_album() {
        let file_name = format!("test_dbs/{}-test.db", Uuid::new_v4());
        let pool = setup_test_db(&file_name).await;
        let artist_id = Uuid::new_v4();

        // Insert album
        insert_album(&pool, artist_id, "Album for deletion test", Utc::now()).await;

        // Get the album ID
        let albums = get_albums(&pool).await.unwrap();
        let album_id = albums[0].get_id();

        // create two songs
        let song_1 = Song::new_auto_id("test song 1".to_string(),None,"".to_string());
        let song_2 = Song::new_auto_id("test song 2".to_string(),None,"".to_string());
        let song_id_1 = song_1.get_id();
        let song_id_2 = song_2.get_id();

        create_song_entry(&pool,&song_1).await;
        create_song_entry(&pool,&song_2).await;

        // Associate songs with album
        associate_song_with_album(&pool, album_id, song_id_1).await;
        associate_song_with_album(&pool, album_id, song_id_2).await;

        // Verify both songs are associated
        let song_ids = get_songs_for_album(&pool, album_id).await;
        assert_eq!(song_ids.len(), 2, "Expected two songs before deletion");

        // Delete one song from album
        delete_song_from_album(&pool, album_id, song_id_1).await;

        // Verify only one song remains
        let remaining_song_ids = get_songs_for_album(&pool, album_id).await;
        assert_eq!(remaining_song_ids.len(), 1, "Expected one song after deletion");
        assert_eq!(remaining_song_ids[0], song_id_2, "Expected second song to remain");
        delete_test_db(&file_name);
    }

    #[sqlx::test]
    async fn test_delete_album() {
        let file_name = format!("test_dbs/{}-test.db", Uuid::new_v4());
        let pool = setup_test_db(&file_name).await;
        let artist_id = Uuid::new_v4();

        // Insert albums
         insert_album(&pool, artist_id, "Album to delete", Utc::now()).await;
         insert_album(&pool, artist_id, "Album to keep", Utc::now()).await;

        // Get album IDs
        let albums = get_albums(&pool).await.unwrap();
        assert_eq!(albums.len(), 2, "Expected two albums before deletion");

        let album_to_delete_id = albums[0].get_id();

        // Create song
        let song = Song::new_auto_id("test song".to_string(),None,"".to_string());
        let id = song.get_id();

        // Insert song and associate with album
        create_song_entry(&pool, &song).await;
        associate_song_with_album(&pool, album_to_delete_id, id).await;

        // Delete the album
        delete_album(&pool, album_to_delete_id).await;

        // Verify album is deleted
        let remaining_albums = get_albums(&pool).await.unwrap();
        assert_eq!(remaining_albums.len(), 1, "Expected one album after deletion");
        assert_ne!(remaining_albums[0].get_id(), album_to_delete_id, "Deleted album should not be present");

        // Verify album_songs entries are deleted
        let song_ids = get_songs_for_album(&pool, album_to_delete_id).await;
        assert!(song_ids.is_empty(), "Expected no songs associated with deleted album");
        delete_test_db(&file_name);
    }

    #[sqlx::test]
    async fn test_get_nonexistent_album_songs() {
        let file_name = format!("test_dbs/{}-test.db", Uuid::new_v4());
        let pool = setup_test_db(&file_name).await;
        let nonexistent_album_id = Uuid::new_v4();

        let song_ids = get_songs_for_album(&pool, nonexistent_album_id).await;
        assert!(song_ids.is_empty(), "Expected no songs for nonexistent album");
        delete_test_db(&file_name);
    }

    #[sqlx::test]
    fn test_invalid_year_format() {
        let id = Uuid::new_v4();
        let artist_id = Uuid::new_v4();
        let title = "Invalid Year Album".to_string();
        
        let album = Album {
            id: id.to_string(),
            artist_id: artist_id.to_string(),
            title,
            year: "invalid-date-format".to_string(),
        };
        
        assert!(album.get_year().is_none(), "Expected None for invalid year format");
    }

    #[sqlx::test]
    async fn test_create_tables() {
        let file_name = format!("test_dbs/{}-test.db", Uuid::new_v4());
        let pool = setup_test_db(&file_name).await;

        // Verify album table exists by inserting and querying
        let artist_id = Uuid::new_v4();
        insert_album(&pool, artist_id, "Test Table Creation", Utc::now()).await;

        let albums = get_albums(&pool).await.unwrap();
        assert_eq!(albums.len(), 1, "Expected one album after table creation");

        // create song
        let song = Song::new_auto_id("test song".to_string(),None,"".to_string());
        let id = song.get_id();
        create_song_entry(&pool, &song).await;

        // Verify album_songs table exists
        let album_id = albums[0].get_id();
        associate_song_with_album(&pool, album_id, id).await;

        let songs = get_songs_for_album(&pool, album_id).await;
        assert_eq!(songs.len(), 1, "Expected one song after association");
        delete_test_db(&file_name);
    }
}