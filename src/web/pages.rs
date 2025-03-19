use crate::{db, AppState};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use minijinja::context;
use std::sync::Arc;

pub async fn hander(State(state): State<Arc<AppState>>, axum::extract::Path(name): axum::extract::Path<String>) -> Result<Html<String>, StatusCode> {
    let name = name.clone().split('/').last().unwrap().to_string();
    let name = name.split('.').next().unwrap().to_string();
    match name.as_str() {
        "home" => {
            #[cfg(debug_assertions)]
            let env = state.env.acquire_env().unwrap();
            #[cfg(not(debug_assertions))]
            let env = &state.env;
            let template = env.get_template("home.jinja").unwrap();
            let rendered = template.render(context! {
            }).unwrap();
            Ok(Html(rendered))
        }
        "artists" => {
            let mut artists = db::schema::artist::get_artists(&state.db).await.unwrap_or_else(|| Vec::<db::schema::artist::Artist>::new());
            artists = artists.into_iter().map(|a| a.clean_for_web_view()).collect::<Vec<db::schema::artist::Artist>>();
            #[cfg(debug_assertions)]
            let env = state.env.acquire_env().unwrap();
            #[cfg(not(debug_assertions))]
            let env = &state.env;
            let template = env.get_template("artists.jinja").unwrap();
            let rendered = template.render(context! {
                artists
            }).unwrap();
            Ok(Html(rendered))
        }
        "songs" => {
            let mut songs = db::schema::song::get_songs(&state.db).await.unwrap_or_else(|| Vec::<db::schema::song::Song>::new());
            songs = songs.into_iter().map(|s| s.clean_for_web_view()).collect::<Vec<db::schema::song::Song>>();
            #[cfg(debug_assertions)]
            let env = state.env.acquire_env().unwrap();
            #[cfg(not(debug_assertions))]
            let env = &state.env;
            let template = env.get_template("songs.jinja").unwrap();
            let rendered = template.render(context! {
                songs
            }).unwrap();
            Ok(Html(rendered))
        }
        _ => Err(StatusCode::NOT_FOUND)
    }
}