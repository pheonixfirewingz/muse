use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Html;
use minijinja::context;
use serde::Deserialize;
use crate::{db, AppState};
use crate::db::schema::artist::Artist;

#[derive(Deserialize)]
pub struct SearchQuery {
    artist: Option<String>,
    song_name: Option<String>,
    album: Option<String>,
}

pub async fn handler(Query(params): Query<SearchQuery>, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let has_artist = params.artist.is_some();
    let has_song_name = params.song_name.is_some();
    let has_album = params.album.is_some();
    #[cfg(debug_assertions)]
    let env = state.env.acquire_env().unwrap().clone();
    #[cfg(not(debug_assertions))]
    let env = &state.env.clone();
    let template = env.get_template("lists.jinja").unwrap();
    // Validate allowed query combinations
    if (has_artist && has_song_name && !has_album) || // artist + song_name
        (has_song_name && !has_artist && !has_album) || // song_name alone
        (has_artist && !has_song_name && !has_album) || // artist alone
        (has_album && !has_artist && !has_song_name)    // album alone
    {
        let response = match (params.artist, params.song_name, params.album) {
            (Some(artist), Some(song), None) => {
                let songs = db::get_song_by_song_name_and_artist_name(&state.db, &song, &artist).await;
                match songs {
                    Some(songs) => {
                        let songs = songs.clean_for_web_view();
                        let songs = vec![songs];
                        let rendered = template.render(context! {
                            songs
                        }).unwrap();
                        Html(rendered)
                    },
                    None => Html("No songs found.".to_string())
                }
            },
            (Some(artist), None, None) => {
                let artist = Artist::new_auto_id(artist);
                let songs = db::get_songs_by_artist_name(&state.db, &artist.name).await;
                match songs {
                    Some(songs) => {
                        let songs = songs.into_iter().map(|a| a.clean_for_web_view()).collect::<Vec<db::schema::song::Song>>();
                        let rendered = template.render(context! {
                            songs
                        }).unwrap();
                        Html(rendered)
                    },
                    None => Html("No songs found.".to_string())
                }
            },
            (None, Some(_song), None) => {
                Html("path not implemented".to_string())
            },
            (None, None, Some(_album)) => {
                Html("path not implemented".to_string())
            },
            _ => unreachable!(), // Already filtered invalid cases
        };
        return response.into_response();
    }
    // Invalid query combination
    (StatusCode::BAD_REQUEST, Body::from("Invalid query parameters. Allowed: artist+song_name, song_name alone, artist alone, or album alone.")).into_response()
}