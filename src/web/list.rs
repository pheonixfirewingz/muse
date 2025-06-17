use crate::{db, AppState};
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::response::IntoResponse;
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct SearchQuery {
    artist_name: Option<String>,
    song_name: Option<String>,
    album: Option<String>,
}

pub async fn handler(
    Query(params): Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let db = &state.db;
    let env = &state.env.clone();
    let template = env.get_template("lists.jinja").unwrap();

    match (params.artist_name.clone(), params.song_name.clone(), params.album.clone()) {
        (Some(artist_name), None, None) => {
            let songs_info = match db::actions::get_db_song_info(db, true).await {
                Ok(songs) => songs
                    .into_iter()
                    .filter(|song| song.get_artist_name().matches(&artist_name).count() > 0)
                    .collect::<Vec<_>>(),
                Err(_) => {
                    return (StatusCode::NOT_FOUND, Body::from("Could not find songs")).into_response();
                }
            };
            let rendered = template.render(context! { songs_info }).unwrap();
            Html(rendered).into_response()
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Body::from("Invalid query parameters. Allowed: artist+song_name, song_name alone, artist alone, or album alone."),
        )
            .into_response(),
    }
}
/*let has_artist = params.artist.is_some();
    let has_song_name = params.song_name.is_some();
    let has_album = params.album.is_some();
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
                let songs = db::actions::get_songs_by_artist_and_song(&state.db, &song, &artist).await;
                match songs {
                    Ok(_songs) => {
                        /*let songs = songs.clean_for_web_view();
                        let songs = vec![songs];
                        let rendered = template.render(context! {
                            songs
                        }).unwrap();
                        Html(rendered)*/
                        Html("NO IMPLEMENTED.".to_string())
                    },
                    Err(_) => Html("No songs found.".to_string())
                }
            },
            (Some(_artist), None, None) => {
                /*let artist = Artist::new(artist);
                let songs = artist::get_songs_by_artist_name(&state.db, &artist.name).await;
                match songs {
                    Some(songs) => {
                        let rendered = template.render(context! {
                            songs
                        }).unwrap();
                        Html(rendered)
                    },
                    None => Html("No songs found.".to_string())
                }*/
                Html("NO IMPLEMENTED.".to_string())
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
}*/
