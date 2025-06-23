use crate::db::schema::sql_share::SQLResult;
use crate::db::DbPool;
use crate::{db, fetch_all_rows, fetch_all_scalar, run_command};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Song {
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub file_path: String,
    pub format: String,
}
impl Song {
    pub fn new(name: String, description: Option<String>, file_path: String, format: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: name.trim().to_string(),
            description,
            file_path,
            format,
        }
    }
    
    pub fn get_id(&self) -> &Uuid {
        &self.uuid
    }
}

pub async fn create_table_if_not_exists(pool: &DbPool) -> SQLResult<()>{
    run_command!(pool,r#"CREATE TABLE IF NOT EXISTS songs (
            uuid BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            file_path TEXT NOT NULL,
            format TEXT NOT NULL
        )"#)?;
    Ok(())
}

pub async fn add_song(pool: &DbPool, song: &Song) -> SQLResult<()> {
    run_command!(pool,r#"INSERT INTO songs (uuid, name, description, file_path, format) VALUES (?, ?, ?, ?, ?)"#,
        &song.uuid, &song.name, &song.description, &song.file_path, &song.format)?;
    Ok(())
}

/*pub async fn delete_song(pool: &DbPool, song_id: &str) -> SQLResult<()> {
    run_command!(pool,"DELETE FROM songs WHERE uuid = ?")?;
    Ok(())
}*/

pub async fn has_table_got_song_name_by_artist(pool: &DbPool, artist_name: &String,song_name: &String) -> SQLResult<bool> {
    let artist_name = artist_name.trim();
    let song_name = song_name.trim();
    let artist = db::schema::artist::get_artist_by_name(pool,&artist_name.to_string()).await?;
    let songs = get_songs_by_name(pool,&song_name.to_string()).await?;
    if songs.is_empty() {
        return Ok(false);
    }
    for song in songs {
       let result = db::schema::artist_song_association::dose_song_belong_to_artist(pool,artist.get_id(),song.get_id()).await?;
        if result {
            return Ok(true);       
        }
    }
    Ok(false)
}

pub async fn get_song_names_by_artist(pool: &DbPool, artist_id: Uuid, ascending: bool) -> SQLResult<Vec<String>> {
    if ascending {
        fetch_all_scalar!(pool,String,
            r#"
            SELECT s.name FROM songs s
            INNER JOIN artists_songs a_s ON s.uuid = a_s.song_uuid
            WHERE a_s.artist_uuid = ?
            ORDER BY s.name ASC
            "#,
            artist_id)
    } else {
        fetch_all_scalar!(pool,String,
            r#"
            SELECT s.name FROM songs s
            INNER JOIN artists_songs a_s ON s.uuid = a_s.song_uuid
            WHERE a_s.artist_uuid = ?
            ORDER BY s.name DESC
            "#, artist_id)
    }
}

pub async fn get_songs_by_name(pool: &DbPool, name: &String) -> SQLResult<Vec<Song>> {
    let name = name.trim();
    Ok(fetch_all_rows!(pool,Song,r#"SELECT uuid, name, description, file_path, format FROM songs WHERE name = ?"#, name)?)
}

