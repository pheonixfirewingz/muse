use crate::{db, AppState};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use minijinja::context;
use std::sync::Arc;
use tracing::{error, info};

pub async fn hander(
    State(state): State<Arc<AppState>>, 
    axum::extract::Path(name): axum::extract::Path<String>
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
            return Err((StatusCode::BAD_REQUEST, "Invalid file name format".to_string()));
        }
    };

    match name.as_str() {
        "home" => {
            info!("Rendering home template");
            let env = &state.env;
            let template = env.get_template("home.jinja").map_err(|e| {
                error!("Failed to get home template: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Template error".to_string())
            })?;
            
            let rendered = template.render(context! {}).map_err(|e| {
                error!("Failed to render home template: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Rendering error".to_string())
            })?;
            
            Ok(Html(rendered))
        }
        "artists" => {
            info!("Rendering artists template");
            let artists = match db::schema::artist::get_artists(&state.db, true).await {
                Some(artists) => artists,
                None => {
                    error!("No artists found in database");
                    return Err((StatusCode::NOT_FOUND, "No artists found".to_string()));
                }
            };
            
            let artists = artists.into_iter()
                .map(|a| a.clean_for_web_view())
                .collect::<Vec<_>>();
            
            let env = &state.env;
            let template = env.get_template("artists.jinja").map_err(|e| {
                error!("Failed to get artists template: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Template error".to_string())
            })?;
            
            let rendered = template.render(context! { artists }).map_err(|e| {
                error!("Failed to render artists template: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Rendering error".to_string())
            })?;
            
            Ok(Html(rendered))
        }
        "songs" => {
            info!("Rendering songs template");
            let songs = match db::schema::song::get_songs(&state.db, true).await {
                Some(songs) => songs,
                None => {
                    error!("No songs found in database");
                    return Err((StatusCode::NOT_FOUND, "No songs found".to_string()));
                }
            };
            
            let songs = songs.into_iter()
                .map(|s| s.clean_for_web_view())
                .collect::<Vec<_>>();
            
            let env = &state.env;
            let template = env.get_template("songs.jinja").map_err(|e| {
                error!("Failed to get songs template: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Template error".to_string())
            })?;
            
            let rendered = template.render(context! { songs }).map_err(|e| {
                error!("Failed to render songs template: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Rendering error".to_string())
            })?;
            
            Ok(Html(rendered))
        }
        _ => {
            error!("Page not found: {}", name);
            Err((StatusCode::NOT_FOUND, "Page not found".to_string()))
        }
    }
}