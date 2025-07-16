use crate::api::io_util::ApiError;
use crate::db::schema::{artist, artist_song_association, song};
use crate::db::util::sql_share::SQLResult;
use crate::db::DbPool;

#[derive(Clone)]
pub struct SongInfo{
    pub song_name: String,
    pub artist_name: String,
}

pub async fn get_info_by_artist(pool: &DbPool,artist_name:&String, ascending: bool) -> SQLResult<Vec<SongInfo>> {
    let mut song_list: Vec<SongInfo> = Vec::new();
    let artist = artist::get_by_name(pool, artist_name).await?;
    let songs = artist_song_association::get_song_names_by_artist(pool, &artist.uuid,&"aac".to_string(), ascending).await.unwrap_or(Vec::new());

    for song_name in songs {
        let song_objs = song::get_by_name(pool, &song_name,&"aac".to_string()).await?;
        for song in song_objs {
            if artist_song_association::dose_song_belong_to_artist(pool, &artist.uuid, song.get_id()).await? {
                song_list.push(SongInfo { song_name:song.name.clone(), artist_name:artist.name.clone()});
            }
        }
    }
    Ok(song_list)
}

pub async fn get_info(pool: &DbPool, start: usize, end: usize) -> SQLResult<Vec<SongInfo>> {
    let songs = song::get_paginated(pool, start, end).await?;
    let mut song_list: Vec<SongInfo> = Vec::new();
    for song in songs {
        // Find artist for each song
        if let Some(artist_uuid) = artist_song_association::get_artist_uuid_by_song_uuid(pool, &song.uuid).await? {
            let artist = artist::get_by_uuid(pool, &artist_uuid).await?;
            song_list.push(SongInfo { song_name: song.name, artist_name: artist.name });
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
    let songs = song::get_by_name(pool,song_name,&"aac".to_string()).await?;
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

// Fuzzy search for songs by name and return SongInfo with artist name
pub async fn fuzzy_search(pool: &DbPool, query: &str) -> SQLResult<Vec<SongInfo>> {
    let songs = song::fuzzy_search_by_name(pool, query).await?;
    let mut song_list: Vec<SongInfo> = Vec::new();
    for song in songs {
        if let Some(artist_uuid) = artist_song_association::get_artist_uuid_by_song_uuid(pool, &song.uuid).await? {
            let artist = artist::get_by_uuid(pool, &artist_uuid).await?;
            song_list.push(SongInfo { song_name: song.name, artist_name: artist.name });
        }
    }
    Ok(song_list)
}