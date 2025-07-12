use serde::Serialize;
use crate::api::io_util::ApiError;
use crate::db::DbPool;
use crate::db::schema::artist;
use crate::db::util::sql_share::SQLResult;


#[derive(Debug,Serialize,Clone)]
pub struct ArtistInfo{
    pub artist_name: String,
}

pub async fn get_info(pool: &DbPool, ascending: bool) -> SQLResult<Vec<ArtistInfo>> {
    let mut list: Vec<ArtistInfo> = Vec::new();
    if let Some(artist_list) = artist::get(pool, ascending).await {
        for artist in artist_list {
            list.push(ArtistInfo { artist_name:artist.name})
        }
        Ok(list)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

pub async fn get_count(pool: &DbPool)-> Result<usize, ApiError> {
    match artist::get_count(&pool).await {
        Ok(artist_data) => Ok(artist_data),
        Err(_) => Err(ApiError::InternalServerError("".to_string()))
    }
}