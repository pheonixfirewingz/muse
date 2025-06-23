use axum::extract::Query;
use crate::{AppState, db, web};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use minijinja::context;
use std::sync::Arc;
use tracing::{debug, error};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

#[derive(Deserialize)]
pub struct PageData {
    start: Option<usize>,
    end: Option<usize>,
    artist_name: Option<String>,
    song_name: Option<String>,
    album: Option<String>,
    username: Option<String>,
}

#[derive(Serialize)]
struct ArtistPageData {
    name: String,
    uri_name: String
}

#[derive(Serialize)]
struct SongPageData {
    song_name: String,
    artist_name: String,
    song_uri: String,
    artist_uri: String,
}



pub async fn hander(
    Query(query): Query<PageData>,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    cookies: Cookies
) -> Result<Html<String>, (StatusCode, String)> {
    if path == ".well-known/appspecific/com.chrome.devtools.json" {
        return Err((StatusCode::NOT_FOUND, "not found".to_string()));
    }
    debug!("Handling page request for: {}", path);
    let db = &state.db;
    let env = &state.env.clone();
    let mut name = if path.starts_with("playlist/") {
        path.to_string()
    } else {
        match path.clone().split('/').last() {
        Some(n) => n.to_string(),
        None => {
            error!("Invalid path format: {}", path);
            return Err((StatusCode::BAD_REQUEST, "Invalid path format".to_string()));
        }
    }};

     name = match name.split('.').next() {
        Some(n) => n.to_string(),
        None => {
            error!("Invalid file name format: {}", name);
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid file name format".to_string(),
            ));
        }
    };
    match name.as_str() {
        "app" => {
            Ok(Html(include_str!("../../statics/index.html").to_string()))
        }
        "home" => {
            debug!("Rendering home template");
            let template = state.env.get_template("home.jinja").map_err(|e| {
                error!("Failed to get home template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Template error".to_string(),
                )
            })?;

            let rendered = template.render(context! {}).map_err(|e| {
                error!("Failed to render home template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Rendering error".to_string(),
                )
            })?;

            Ok(Html(rendered))
        }
        "artists" => {
            debug!("Rendering artists template");
            let artists = match db::actions::get_db_artist_info(&state.db, true).await {
                Ok(artists) => artists,
                Err(_) => {
                    error!("No artists found in database");
                    return Err((StatusCode::NOT_FOUND, "No artists found".to_string()));
                }
            };
            
            let mut artist_data: Vec<ArtistPageData> = Vec::new();
            
            for artist in artists {
                artist_data.push( ArtistPageData { name: artist.get_name().to_string(), uri_name:urlencoding::encode(artist.get_name()).to_string() })
            }

            let template = state.env.get_template("artists.jinja").map_err(|e| {
                error!("Failed to get artists template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Template error".to_string(),
                )
            })?;

            let rendered = template.render(context! { artist_data }).map_err(|e| {
                error!("Failed to render artists template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Rendering error".to_string(),
                )
            })?;
            Ok(Html(rendered))
        }
        "songs" => {
            debug!("Rendering songs template");
            let mut songs_data = match db::actions::get_db_song_info(&state.db, true).await {
                Ok(songs_data) => songs_data,
                Err(_) => {
                    error!("No songs found in database");
                    return Err((StatusCode::NOT_FOUND, "No songs found".to_string()));
                }
            };
            let mut start_list: usize = 0;
            let mut end_list: usize = songs_data.len();
            match (&query.start, &query.end) {
                (Some(start), Some(end)) => {
                    start_list = *start;
                    end_list = *end;
                    if *start <= *end && *end <= songs_data.len() {
                        songs_data = songs_data[start_list..end_list].to_vec();
                    } else {
                       return Err((StatusCode::BAD_REQUEST, "bad index request".to_string()));
                    }
                },
                _ => {}
            }

            let mut songs: Vec<SongPageData> = Vec::new();
            for song in songs_data {
                songs.push(SongPageData {
                    song_name:song.get_song_name().to_string(),
                    song_uri:urlencoding::encode(song.get_song_name()).to_string(),
                    artist_name:song.get_artist_name().to_string(),
                    artist_uri:urlencoding::encode(song.get_artist_name()).to_string()
                });
            }


            let template = state.env.get_template("songs.jinja").map_err(|e| {
                error!("Failed to get songs template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Template error".to_string(),
                )
            })?;

            let rendered = template.render(context! { songs, start_list, end_list }).map_err(|e| {
                error!("Failed to render songs template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Rendering error".to_string(),
                )
            })?;

            Ok(Html(rendered))
        }
        "list" => {
            let template = env.get_template("lists.jinja").unwrap();

            match (query.artist_name.clone(), query.song_name.clone(), query.album.clone()) {
                (Some(artist_name), None, None) => {
                    let songs_data = match db::actions::get_db_song_info(db, true).await {
                        Ok(songs) => songs
                            .into_iter()
                            .filter(|song| song.get_artist_name().matches(&artist_name).count() > 0)
                            .collect::<Vec<_>>(),
                        Err(_) => {
                            return Err((StatusCode::NOT_FOUND, "Could not find songs".to_string()));
                        }
                    };

                    let mut songs: Vec<SongPageData> = Vec::new();
                    for song in songs_data {
                        songs.push(SongPageData {
                            song_name:song.get_song_name().to_string(),
                            song_uri:urlencoding::encode(song.get_song_name()).to_string(),
                            artist_name:song.get_artist_name().to_string(),
                            artist_uri:urlencoding::encode(song.get_artist_name()).to_string()
                        });
                    }

                    let rendered = template.render(context! { songs }).unwrap();
                    Ok(Html(rendered))
                }
                _ => Err((
                    StatusCode::BAD_REQUEST,
                    "Invalid query parameters. Allowed: artist+song_name, song_name alone, artist alone, or album alone.".to_string()
                )),
            }
        },
        "playlists" => {
            debug!("Rendering playlists template");
            let session_id = match web::get_session_id_from_cookies(&cookies) {
                Ok(session_id) => session_id,
                _ => return Err((StatusCode::BAD_REQUEST, "bad session id".to_string())),
            };
            let my_playlists = db::actions::get_db_user_playlists_info(&state.db, &session_id).await.unwrap_or_else(|_| Vec::new());
            let public_playlists = db::actions::get_db_public_playlists_info(&state.db).await.unwrap_or_else(|_| Vec::new());
            let template = state.env.get_template("playlists.jinja").map_err(|e| {
                error!("Failed to get playlists template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Template error".to_string(),
                )
            })?;
            let rendered = template.render(context! { my_playlists, public_playlists }).map_err(|e| {
                error!("Failed to render playlists template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Rendering error".to_string(),
                )
            })?;
            Ok(Html(rendered))
        }
        _ if name.starts_with("playlist/") => {
            let playlist_public = name.strip_prefix("playlist/").unwrap_or("");
            if playlist_public.starts_with("private/") {
                let playlist_name = playlist_public.strip_prefix("private/").unwrap_or("");
                if playlist_name.is_empty() {
                    return Err((StatusCode::BAD_REQUEST, "No playlist name provided".to_string()));
                }

                let session_id = web::get_session_id_from_cookies(&cookies).ok();

                let result = if let Some(username) = &query.username {
                    // Public request may or may not have a session.
                    db::actions::get_playlist_details_by_name(&state.db, playlist_name, session_id.as_ref(), Some(username.as_str())).await
                } else {
                    // Private request requires a session.
                    if let Some(sid) = &session_id {
                        db::actions::get_playlist_details_by_name(&state.db, playlist_name, Some(sid), None).await
                    } else {
                        return Err((StatusCode::UNAUTHORIZED, "Authentication required for this playlist.".to_string()));
                    }
                };

                match result {
                    Ok((playlist, songs)) => {
                        let template = state.env.get_template("playlist_details.jinja").map_err(|e| {
                            error!("Failed to get playlist_details template: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Template error".to_string())
                        })?;
                        let rendered = template.render(context! { playlist, songs }).map_err(|e| {
                            error!("Failed to render playlist_details template: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Rendering error".to_string())
                        })?;
                        Ok(Html(rendered))
                    }
                    Err(_) => Err((StatusCode::NOT_FOUND, "Playlist not found".to_string())),
                }
            } else if playlist_public.starts_with("public/") {
                let playlist_name = playlist_public.strip_prefix("public/").unwrap_or("").to_string();
                if playlist_name.is_empty() {
                    return Err((StatusCode::BAD_REQUEST, "No playlist name provided".to_string()));
                }
                let playlists = db::actions::get_db_public_playlists_info(&state.db)
                    .await.map_err(|_| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;
                
                let playlist = match playlists.iter().find(|&p| p.name == playlist_name) {
                    Some(playlist) => playlist,
                    None => return Err((StatusCode::NOT_FOUND, "Playlist not found".to_string())),
                };

                let result = db::actions::get_playlist_details_by_name(&state.db, &playlist.name, None, Some(&playlist.username)).await;
                match result {
                    Ok((playlist, songs)) => {
                        let template = state.env.get_template("playlist_details.jinja").map_err(|e| {
                            error!("Failed to get playlist_details template: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Template error".to_string())
                        })?;
                        let rendered = template.render(context! { playlist, songs }).map_err(|e| {
                            error!("Failed to render playlist_details template: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Rendering error".to_string())
                        })?;
                        Ok(Html(rendered))
                    }
                    Err(_) => Err((StatusCode::NOT_FOUND, "Playlist not found".to_string())),
                }
            } else {
                Err((StatusCode::NOT_FOUND, "Page not found".to_string()))
            }
        }
        _ => {
            error!("Page not found: {}", name);
            Err((StatusCode::NOT_FOUND, "Page not found".to_string()))
        }
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
