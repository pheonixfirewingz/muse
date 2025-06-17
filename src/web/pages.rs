use axum::extract::Query;
use crate::{AppState, db};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use minijinja::context;
use std::sync::Arc;
use tracing::{error, info};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PageData {
    start: Option<usize>,
    end: Option<usize>,
}

pub async fn hander(
    Query(_query): Query<PageData>,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    info!("Handling page request for: {}", name);

    let name = match name.clone().split('/').last() {
        Some(n) => n.to_string(),
        None => {
            error!("Invalid path format: {}", name);
            return Err((StatusCode::BAD_REQUEST, "Invalid path format".to_string()));
        }
    };

    let name = match name.split('.').next() {
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
        "home" => {
            info!("Rendering home template");
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
            info!("Rendering artists template");
            let artists = match db::actions::get_db_artist_info(&state.db, true).await {
                Ok(artists) => artists,
                Err(_) => {
                    error!("No artists found in database");
                    return Err((StatusCode::NOT_FOUND, "No artists found".to_string()));
                }
            };

            let template = state.env.get_template("artists.jinja").map_err(|e| {
                error!("Failed to get artists template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Template error".to_string(),
                )
            })?;

            let rendered = template.render(context! { artists }).map_err(|e| {
                error!("Failed to render artists template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Rendering error".to_string(),
                )
            })?;
            Ok(Html(rendered))
        }
        "songs" => {
            info!("Rendering songs template");
            let songs = match db::actions::get_db_song_info(&state.db, true).await {
                Ok(songs) => songs,
                Err(_) => {
                    error!("No songs found in database");
                    return Err((StatusCode::NOT_FOUND, "No songs found".to_string()));
                }
            };

            let template = state.env.get_template("songs.jinja").map_err(|e| {
                error!("Failed to get songs template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Template error".to_string(),
                )
            })?;

            let rendered = template.render(context! { songs }).map_err(|e| {
                error!("Failed to render songs template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Rendering error".to_string(),
                )
            })?;

            Ok(Html(rendered))
        }
        _ => {
            error!("Page not found: {}", name);
            Err((StatusCode::NOT_FOUND, "Page not found".to_string()))
        }
    }
}
