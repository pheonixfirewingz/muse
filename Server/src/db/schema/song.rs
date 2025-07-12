use crate::db::util::sql_share::SQLResult;
use crate::db::DbPool;
use crate::{db, fetch_all_rows, fetch_scalar, run_command};
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

pub async fn add(pool: &DbPool, song: &Song) -> SQLResult<()> {
    run_command!(pool,r#"INSERT INTO songs (uuid, name, description, file_path, format) VALUES (?, ?, ?, ?, ?)"#,
        &song.uuid, &song.name, &song.description, &song.file_path, &song.format)?;
    Ok(())
}

/*pub async fn delete(pool: &DbPool, id: &str) -> SQLResult<()> {
    run_command!(pool,"DELETE FROM songs WHERE uuid = ?")?;
    Ok(())
}*/

pub async fn has_table_got_name_by_artist(pool: &DbPool, artist_name: &String,name: &String) -> SQLResult<bool> {
    let artist_name = artist_name.trim();
    let name = name.trim();
    let artist = db::schema::artist::get_by_name(pool,&artist_name.to_string()).await?;
    let songs = get_by_name(pool,&name.to_string()).await?;
    if songs.is_empty() {
        return Ok(false);
    }
    for song in songs {
       let result = db::schema::artist_song_association::dose_song_belong_to_artist(pool,&artist.uuid,song.get_id()).await?;
        if result {
            return Ok(true);
        }
    }
    Ok(false)
}



pub async fn get_by_name(pool: &DbPool, name: &String) -> SQLResult<Vec<Song>> {
    let name = name.trim();
    Ok(fetch_all_rows!(pool,Song,r#"SELECT uuid, name, description, file_path, format FROM songs WHERE name = ?"#, name)?)
}

pub async fn get_count(pool: &DbPool) -> SQLResult<usize> {
    Ok(fetch_scalar!(pool,i64,"SELECT count(*) FROM songs")? as usize)
}