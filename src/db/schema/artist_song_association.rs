use uuid::Uuid;
use crate::db::DbPool;
use crate::db::schema::sql_share::SQLResult;
use crate::{fetch_scalar, run_command};

pub async fn create_table_if_not_exists(pool: &DbPool) -> SQLResult<()> {
    run_command!(pool,
        "CREATE TABLE IF NOT EXISTS artists_songs (
            artist_uuid BLOB NOT NULL,
            song_uuid BLOB NOT NULL,
            PRIMARY KEY (artist_uuid, song_uuid),
            FOREIGN KEY (artist_uuid) REFERENCES artists(uuid) ON DELETE CASCADE,
            FOREIGN KEY (song_uuid) REFERENCES songs(uuid) ON DELETE CASCADE
        )"
    )?;
    Ok(())
}

pub async fn add_artist_song_association(pool: &DbPool, artist_uuid: &Uuid, song_uuid: &Uuid) -> SQLResult<()> {
    run_command!(pool,"INSERT INTO artists_songs (artist_uuid, song_uuid) VALUES (?, ?)",artist_uuid,song_uuid)?;
    Ok(())
}

pub async fn dose_song_belong_to_artist(pool: &DbPool, artist_uuid: &Uuid, song_uuid: &Uuid) -> SQLResult<bool> {
    fetch_scalar!(pool,bool,r#"
        SELECT EXISTS( SELECT 1 FROM artists_songs WHERE song_uuid = ? AND artist_uuid = ?)"#,
        song_uuid, artist_uuid
    )
}