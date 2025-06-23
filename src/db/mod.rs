mod schema;
pub mod actions;
pub mod thirdparty;
mod util;

use bcrypt::hash;
pub use schema::session;
pub use schema::user;

use crate::db::schema::{artist, playlist};
use crate::db::schema::artist_song_association;
use crate::db::schema::playlist_song_association;
use crate::db::schema::song;
use util::sql_share::SQLResult;
use crate::fetch_scalar;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};
use tracing::instrument;

pub type DbPool = Pool<Sqlite>;
pub async fn init_db() -> DbPool {
    #[cfg(debug_assertions)]
    let name = "runtime/cache/test.db";
    #[cfg(not(debug_assertions))]
    let name = "runtime/cache/muse.db";
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
    let _ = song::create_table_if_not_exists(&pool).await;
    let _ = artist::create_table_if_not_exists(&pool).await;
    let _ = artist_song_association::create_table_if_not_exists(&pool).await;
    let _ = user::create_table_if_not_exists(&pool).await;
    let _ = session::create_table_if_not_exists(&pool).await;
    let _ = playlist::create_table_if_not_exists(&pool).await;
    let _ = playlist_song_association::create_table_if_not_exists(&pool).await;
    
    #[cfg(debug_assertions)]
    if is_sessions_table_empty(&pool).await.unwrap_or(false) {
        let password = "tuh6y6Q8N5q*tF4^vhx&@fPE8s";
        let hash = hash(&password, crate::web::login::BCRYPT_COST).unwrap();
        let user =  user::User::new("local_checks","127.0.0.1.imprecise369@passmail.net",&hash);
        let _ = user::create_user_if_not_exists(&pool,&user).await;
    }
    pool
}

#[cfg(debug_assertions)]
#[instrument(skip(pool))]
pub async fn is_sessions_table_empty(pool: &DbPool) -> SQLResult<bool> {
    let exists: i64 = fetch_scalar!(
        pool,
        i64,
        r#"SELECT EXISTS(SELECT 1 FROM users LIMIT 1)"#
    )?;
    Ok(exists == 0)
}
