use crate::db::util::sql_share::SQLResult;
use crate::db::DbPool;
use crate::{db, fetch_all_rows, fetch_scalar, run_command};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Song {
    pub uuid: Uuid,
    pub name: String,
    pub file_path: String,
    pub format: String,
}
impl Song {
    pub fn new(name: String, file_path: String, format: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: name.trim().to_string(),
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
            file_path TEXT NOT NULL,
            format TEXT NOT NULL
        )"#)?;
    Ok(())
}

pub async fn add(pool: &DbPool, song: &Song) -> SQLResult<()> {
    run_command!(pool,r#"INSERT INTO songs (uuid, name, file_path, format) VALUES (?, ?, ?, ?)"#,
        &song.uuid, &song.name, &song.file_path, &song.format)?;
    Ok(())
}

/*pub async fn delete(pool: &DbPool, id: &str) -> SQLResult<()> {
    run_command!(pool,"DELETE FROM songs WHERE uuid = ?")?;
    Ok(())
}*/

pub async fn has_table_got_name_by_artist(pool: &DbPool, artist_name: &String,name: &String, format:&String) -> SQLResult<bool> {
    let artist_name = artist_name.trim();
    let name = name.trim();
    let artist = db::schema::artist::get_by_name(pool,&artist_name.to_string()).await?;
    let songs = get_by_name(pool, &name.to_string(),format).await?;
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



pub async fn get_by_name(pool: &DbPool, name: &String, format:&String) -> SQLResult<Vec<Song>> {
    let name = name.trim();
    Ok(fetch_all_rows!(pool,Song,r#"SELECT uuid, name, file_path, format FROM songs WHERE name = ? AND format = ?"#, name,format)?)
}

pub async fn get_count(pool: &DbPool) -> SQLResult<usize> {
    Ok(fetch_scalar!(pool,i64,r#"SELECT count(*) FROM songs WHERE format = 'aac'"#)? as usize)
}

pub async fn get_paginated(pool: &DbPool, start: usize, end: usize) -> SQLResult<Vec<Song>> {
    let limit = (end as i64) - (start as i64);
    let offset = start as i64;
    Ok(fetch_all_rows!(pool, Song, r#"SELECT uuid, name, file_path, format FROM songs WHERE format = 'aac' ORDER BY name ASC LIMIT ? OFFSET ?"#, limit, offset)?)
}
pub async fn fuzzy_search_by_name(pool: &DbPool, query: &str) -> SQLResult<Vec<Song>> {
    //TODO - we don't protect from sql injection here we need to add it
    let pattern = format!("%{}%", query.trim());
    Ok(fetch_all_rows!(pool, Song, r#"SELECT uuid, name, file_path, format FROM songs WHERE format = 'aac' AND name LIKE ? COLLATE NOCASE ORDER BY name ASC"#, pattern)?)
}