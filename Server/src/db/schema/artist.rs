use crate::db::util::sql_share::SQLResult;
use crate::db::DbPool;
use crate::{fetch_all_rows, fetch_optional_row, fetch_scalar, run_command};
use sqlx::FromRow;
use tracing::error;
use uuid::Uuid;

#[derive(FromRow)]
pub struct Artist {
    pub uuid: Uuid,
    pub name: String
}

pub async fn create_table_if_not_exists(pool: &DbPool) -> SQLResult<()> {
    run_command!(pool,r#"CREATE TABLE IF NOT EXISTS artists (uuid BLOB PRIMARY KEY NOT NULL, name TEXT NOT NULL)"#)?;
    Ok(())
}

pub async fn add(pool: &DbPool, artist_name:String) -> SQLResult<()> {
    let artist = Artist {uuid:Uuid::new_v4(),name:artist_name};
    run_command!(pool,"INSERT INTO artists (uuid, name) VALUES (?, ?)",&artist.uuid,&artist.name)?;
    Ok(())
}

pub async fn has_table_got_name(pool: &DbPool, artist_name: &String) -> SQLResult<bool> {
    let artist_name = artist_name.trim();
    fetch_scalar!(pool,bool,
        r#"SELECT EXISTS(SELECT 1 FROM artists WHERE name = ? COLLATE NOCASE)"#,artist_name)
}

pub async fn get_by_name(pool: &DbPool, artist_name: &String) -> SQLResult<Artist> {
    let artist_name = artist_name.trim();
    match fetch_optional_row!(pool,Artist,"SELECT uuid, name FROM artists WHERE name = ?",artist_name)? {
        Some(data) => Ok(data),
        None => Err(sqlx::Error::InvalidArgument(format!("could not find Artist by {artist_name}")))
    }
}

pub async fn get(pool: &DbPool, ascending: bool) -> Option<Vec<Artist>> {
    let result;
    if ascending {
        result = fetch_all_rows!(pool,Artist,"SELECT uuid, name FROM artists ORDER BY name ASC")
    } else {
        result = fetch_all_rows!(pool,Artist, "SELECT uuid, name FROM artists ORDER BY name DESC")
    }

    match result {
        Ok(artists) => Some(artists),
        Err(e) => {
            error!("{:?}",e);
            None
        }
    }
}

pub async fn get_by_uuid(pool: &DbPool, artist_uuid: &Uuid) -> SQLResult<Artist> {
    match fetch_optional_row!(pool, Artist, "SELECT uuid, name FROM artists WHERE uuid = ?", artist_uuid)? {
        Some(data) => Ok(data),
        None => Err(sqlx::Error::InvalidArgument(format!("could not find Artist by uuid {artist_uuid}")))
    }
}

pub async fn get_count(pool: &DbPool) -> SQLResult<usize> {
    Ok(fetch_scalar!(pool,i64,"SELECT count(*) FROM artists")? as usize)
}