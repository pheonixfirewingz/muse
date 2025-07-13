use crate::api::io_util::ApiError;
use crate::db::schema::{artist, artist_song_association, song};
use crate::db::util::sql_share::SQLResult;
use crate::db::DbPool;

#[derive(Clone)]
pub struct SongInfo{
    pub song_name: String,
    pub artist_name: String,
}
pub async fn get_info(pool: &DbPool, ascending: bool) -> SQLResult<Vec<SongInfo>>{
    let mut song_list: Vec<SongInfo> = Vec::new();
    if let Some(artist_list) = artist::get(pool, ascending).await {
        for artist in artist_list {
            let songs = artist_song_association::get_song_names_by_artist(pool, &artist.uuid, ascending).await.unwrap_or(Vec::new());

            for song_name in songs {
                let song_objs = song::get_by_name(pool, &song_name).await?;
                for song in song_objs {
                    if artist_song_association::dose_song_belong_to_artist(pool, &artist.uuid, song.get_id()).await? {
                        song_list.push(SongInfo { song_name:song.name.clone(), artist_name:artist.name.clone()});
                    }
                }
            }
        }
        Ok(song_list)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

pub async fn get_info_by_artist(pool: &DbPool,artist_name:&String, ascending: bool) -> SQLResult<Vec<SongInfo>> {
    let mut song_list: Vec<SongInfo> = Vec::new();
    let artist = artist::get_by_name(pool, artist_name).await?;
    let songs = artist_song_association::get_song_names_by_artist(pool, &artist.uuid, ascending).await.unwrap_or(Vec::new());

    for song_name in songs {
        let song_objs = song::get_by_name(pool, &song_name).await?;
        for song in song_objs {
            if artist_song_association::dose_song_belong_to_artist(pool, &artist.uuid, song.get_id()).await? {
                song_list.push(SongInfo { song_name:song.name.clone(), artist_name:artist.name.clone()});
            }
        }
    }
    Ok(song_list)
}

pub async fn get_count(pool: &DbPool)-> Result<usize, ApiError> {
    match song::get_count(&pool).await {
        Ok(songs_data) => Ok(songs_data),
        Err(_) => Err(ApiError::InternalServerError("".to_string()))
    }
}

pub async fn get_file_path(pool: &DbPool,song_name: &String, artist_name: &String, preferred_formats: Option<&[&str]>) -> SQLResult<(String, String)> {
    let songs = song::get_by_name(pool,song_name).await?;
    let artist = artist::get_by_name(pool,artist_name).await?;

    if let Some(formats) = preferred_formats {
        for &fmt in formats {
            for song in &songs {
                if song.format == fmt && artist_song_association::dose_song_belong_to_artist(pool,&artist.uuid,song.get_id()).await? {
                    return Ok((song.file_path.clone(), song.format.clone()));
                }
            }
        }
    }

    for song in songs {
        if artist_song_association::dose_song_belong_to_artist(pool,&artist.uuid,song.get_id()).await? {
            return Ok((song.file_path.clone(), song.format.clone()));
        }
    }
    Err(sqlx::Error::InvalidArgument(format!("song {song_name} not found by {artist_name}").to_string()))
}