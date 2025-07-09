use uuid::Uuid;
use crate::db::DbPool;
use crate::db::util::sql_share::SQLResult;
use crate::{fetch_scalar, fetch_all_rows, run_command};
use crate::db::schema::song::Song;

pub async fn create_table_if_not_exists(pool: &DbPool) -> SQLResult<()> {
    run_command!(pool,
        "CREATE TABLE IF NOT EXISTS playlists_songs (
            playlist_uuid BLOB NOT NULL,
            song_uuid BLOB NOT NULL,
            added_at TEXT NOT NULL,
            position INTEGER,
            PRIMARY KEY (playlist_uuid, song_uuid),
            FOREIGN KEY (playlist_uuid) REFERENCES playlists(uuid) ON DELETE CASCADE,
            FOREIGN KEY (song_uuid) REFERENCES songs(uuid) ON DELETE CASCADE
        )"
    )?;
    Ok(())
}

pub async fn add_song_to_playlist(pool: &DbPool, playlist_uuid: &Uuid, song_uuid: &Uuid) -> SQLResult<()> {
    let added_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    
    // Get the next position for this playlist
    let position = get_next_position(pool, playlist_uuid).await?;
    
    run_command!(pool,
        "INSERT INTO playlists_songs (playlist_uuid, song_uuid, added_at, position) VALUES (?, ?, ?, ?)",
        playlist_uuid, song_uuid, added_at, position
    )?;
    Ok(())
}

pub async fn remove_song_from_playlist(pool: &DbPool, playlist_uuid: &Uuid, song_uuid: &Uuid) -> SQLResult<bool> {
    let result = run_command!(pool,
        "DELETE FROM playlists_songs WHERE playlist_uuid = ? AND song_uuid = ?",
        playlist_uuid, song_uuid
    )?;
    
    Ok(result.rows_affected() > 0)
}

pub async fn is_song_in_playlist(pool: &DbPool, playlist_uuid: &Uuid, song_uuid: &Uuid) -> SQLResult<bool> {
    fetch_scalar!(pool, bool,
        "SELECT EXISTS(SELECT 1 FROM playlists_songs WHERE playlist_uuid = ? AND song_uuid = ?)",
        playlist_uuid, song_uuid
    )
}

pub async fn get_songs_in_playlist(pool: &DbPool, playlist_uuid: &Uuid) -> SQLResult<Vec<Song>> {
    fetch_all_rows!(pool, Song,
        r#"SELECT s.uuid, s.name, s.description, s.file_path, s.format 
           FROM songs s
           INNER JOIN playlists_songs ps ON s.uuid = ps.song_uuid
           WHERE ps.playlist_uuid = ?
           ORDER BY ps.position ASC"#,
        playlist_uuid
    )
}

pub async fn reorder_song_in_playlist(pool: &DbPool, playlist_uuid: &Uuid, song_uuid: &Uuid, new_position: i32) -> SQLResult<bool> {
    let result = run_command!(pool,
        "UPDATE playlists_songs SET position = ? WHERE playlist_uuid = ? AND song_uuid = ?",
        new_position, playlist_uuid, song_uuid
    )?;
    
    Ok(result.rows_affected() > 0)
}

async fn get_next_position(pool: &DbPool, playlist_uuid: &Uuid) -> SQLResult<i32> {
    let max_position: Option<i32> = fetch_scalar!(pool, Option<i32>,
        "SELECT MAX(position) FROM playlists_songs WHERE playlist_uuid = ?",
        playlist_uuid
    )?;
    
    Ok(max_position.unwrap_or(0) + 1)
}