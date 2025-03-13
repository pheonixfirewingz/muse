pub mod schema;
use crate::db::schema::album::{create_album_songs_table_if_not_exists, create_albums_table_if_not_exists};
use crate::db::schema::artist::{create_artists_songs_table_if_not_exists, create_artists_table_if_not_exists, get_artist_by_name};
use crate::db::schema::song::{create_songs_table_if_not_exists, get_songs_by_name};
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

pub type DbPool = Pool<Sqlite>;
/// Initialize database pool.
///
/// This function reads database url from environment variables, and use the
/// correct database driver to connect to the database. If the environment
/// variable is not set, it will panic.
///
/// The behavior of this function depends on the `debug_assertions` feature. If
/// `debug_assertions` is enabled, it will use the `DATABASE_URL_SQLITE`
/// environment variable and use the `Sqlite` driver. If `debug_assertions` is
/// not enabled, it will use the `DATABASE_URL_MARIADB` environment variable and
/// use the `MySql` driver.
///
/// The database pool is wrapped in `Arc` so it can be shared across the
/// application.
///
/// # Errors
///
/// This function will panic if the database connection failed.
pub async fn init_db() -> DbPool {
    dotenv().ok();
    #[cfg(debug_assertions)]
    let name = "runtime/test.db";
    #[cfg(not(debug_assertions))]
    let name = "runtime/muse.db";
    let path = std::path::Path::new(name);

    // Only create the file if it does not already exist.
    if !path.exists() {
        std::fs::File::create(&path).expect("Failed to create test database file");
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect(&format!("sqlite://{}", name))
        .await
        .expect("Failed to create database pool");

    create_albums_table_if_not_exists(&pool).await;
    create_songs_table_if_not_exists(&pool).await;
    create_artists_table_if_not_exists(&pool).await;
    create_artists_songs_table_if_not_exists(&pool).await;
    create_album_songs_table_if_not_exists(&pool).await;
    pool
}

pub async fn get_song_path_by_song_name_and_artist_name(pool: &DbPool, song_name: &str, artist: &str) -> Option<String> {
    // Get artist by name
    let artist_result = get_artist_by_name(pool, &artist.to_string()).await;

    match artist_result {
        Some(artist) => {
            // Get all songs with the given name
            let songs_result = get_songs_by_name(pool, &song_name.to_string()).await;

            match songs_result {
                Some(songs) => {
                    // For each song, check if it's associated with the artist
                    for song in songs {
                        let result = sqlx::query_scalar::<_, String>(
                            "SELECT s.file_path FROM artists_songs as aso 
                             INNER JOIN songs as s ON aso.song_id = s.id 
                             WHERE aso.artist_id = ? AND s.id = ?")
                            .bind(&artist.id)
                            .bind(&song.id)
                            .fetch_optional(pool)
                            .await;

                        // If we found a path, return it
                        return match result {
                            Ok(Some(path)) => Some(path),
                            Err(err) => {
                                println!("{}", err.to_string());
                                None
                            }
                            _ => None
                        }
                    }
                    // No matching song found for this artist
                    None
                },
                None => None, // No songs with this name were found
            }
        },
        None => None, // Artist wasn't found
    }
}
