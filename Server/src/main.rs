mod db;
mod login;
mod util;
mod api;

use std::env;
use std::process::exit;
use crate::db::DbPool;
use axum::Router;
use std::sync::Arc;
use async_recursion::async_recursion;
use id3::{Tag, TagLike};
use tokio::fs;
use tower_http::compression::CompressionLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use dotenvy::dotenv;
use tower_http::cors::{CorsLayer};

fn setup_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| env::var("LOG_LEVEL").unwrap_or("info".into()).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

struct AppState {
    db: DbPool
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    setup_logging();
    let _ = util::cache::init_cache().await;
    let app_state = Arc::new(AppState { db: db::init_db().await });
    // Scan and register music
    scan_and_register_songs(&app_state.db, "runtime/music").await;

    let cors = CorsLayer::very_permissive();
    
    
    // Top-level app with global compression
    let app = Router::new()
        .merge(api::router())
        .with_state(app_state.clone())
        .layer(CompressionLayer::new())
        .layer(cors);

    // Start server
    let listener = tokio::net::TcpListener::bind(env::var("SERVER_BIND").unwrap_or_else(|_| {
        error!("SERVER_BIND must be set for muse to run");
        exit(0)
    })).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

//this is only here as I have songs formated in this way it is not the norm
fn split_at_first_backslash(s: &str) -> &str {
    match s.find('\\') {
        Some(pos) => &s[..pos],
        None => s,
    }
}

pub async fn scan_and_register_songs(pool: &DbPool, file_path: &str) {
    let mut new_songs_registered: usize = 0;
    scan_and_register_id3_files(file_path, 0, pool, &mut new_songs_registered).await;
    info!("DB: {} new songs registered", new_songs_registered);
}

#[async_recursion]
async fn scan_and_register_id3_files(path: &str, depth: u8, db: & DbPool, new_songs_registered: &mut usize) {
    info!("ID3 SCANNING: {}",path);
    if depth > 3 {
        return;
    }
    let mut entries = match fs::read_dir(path).await {
        Ok(e) => e,
        Err(_) => return,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();

        if let Ok(metadata) = entry.metadata().await {
            if metadata.is_dir() {
                scan_and_register_id3_files(&path.to_str().unwrap(), depth + 1, db,new_songs_registered).await;
            } else if metadata.is_file() {
                let id3_data = Tag::read_from_path(&path);
                if let Ok(tag) = id3_data {
                    info!("ID3: TILE: {}, ARTIST: {}", tag.title().unwrap_or("!BROKEN!"), tag.artist().unwrap_or("!BROKEN!"));
                    if let (Some(song_name),Some(artist_name)) = (tag.title(),tag.artist()) {
                        let artist_name = split_at_first_backslash(artist_name);
                        // Detect format from file extension
                        let format = path.extension().and_then(|s| s.to_str()).unwrap_or("mp3").to_lowercase();
                        match db::action::register_song(db,song_name.to_string(),artist_name.to_string(),&path.to_str().unwrap().to_string()).await {
                            Ok(true) => {
                                info!("ID3: Registered song: {} - {} [{}]",song_name,artist_name,format);
                                *new_songs_registered += 1;
                            },
                            Ok(false) => {
                              info!("ID3: Song already registered: {} - {} [{}]",song_name,artist_name,format);
                            },
                            Err(e) => {
                                error!("ID3: Failed to register song: {:?}",e);
                                ()
                            },
                        }
                    } else {
                        let o = path.to_str().unwrap();
                        warn!("ID3: file rejected: no title or artist -> {o}");
                    }
                }
            }
        }
    }
}