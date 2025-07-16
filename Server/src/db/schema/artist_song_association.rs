use tracing::error;
use uuid::Uuid;
use crate::db::DbPool;
use crate::db::util::sql_share::SQLResult;
use crate::{fetch_all_scalar, fetch_scalar, run_command};

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

pub async fn get_song_names_by_artist(pool: &DbPool, artist_uuid: &Uuid,format:&String, ascending: bool) -> Option<Vec<String>> {
    let result;
    if ascending {
        result = fetch_all_scalar!(pool,String,
            r#"SELECT s.name FROM songs s INNER JOIN artists_songs a_s ON s.uuid = a_s.song_uuid
            WHERE a_s.artist_uuid = ? AND s.format = ? ORDER BY s.name ASC"#,
            artist_uuid,format)
    } else {
        result = fetch_all_scalar!(pool,String,
            r#"SELECT s.name FROM songs s INNER JOIN artists_songs a_s ON s.uuid = a_s.song_uuid
            WHERE a_s.artist_uuid = ? AND s.format = ? ORDER BY s.name  DESC"#, artist_uuid,format)
    }

    match result {
        Ok(result) => Some(result),
        Err(e) => {
            error!("{:?}",e);
            None
        }
    }
}

pub async fn dose_song_belong_to_artist(pool: &DbPool, artist_uuid: &Uuid, song_uuid: &Uuid) -> SQLResult<bool> {
    fetch_scalar!(pool,bool,r#"
        SELECT EXISTS( SELECT 1 FROM artists_songs WHERE song_uuid = ? AND artist_uuid = ?)"#,
        song_uuid, artist_uuid
    )
}

pub async fn get_artist_uuid_by_song_uuid(pool: &DbPool, song_uuid: &Uuid) -> SQLResult<Option<Uuid>> {
    fetch_scalar!(pool, Option<Uuid>,
        "SELECT artist_uuid FROM artists_songs WHERE song_uuid = ? LIMIT 1",
        song_uuid
    )
}