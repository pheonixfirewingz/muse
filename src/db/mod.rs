pub mod schema;
use crate::db::schema::album::{create_album_songs_table_if_not_exists, create_albums_table_if_not_exists};
use crate::db::schema::artist::create_artists_table_if_not_exists;
use crate::db::schema::song::create_songs_table_if_not_exists;
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
    let name = "test.db";
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
    create_album_songs_table_if_not_exists(&pool).await;
    pool
}
