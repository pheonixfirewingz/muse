pub(crate) mod io_util;
pub(crate) mod login;
mod songs;
mod artists;
mod playlist;

mod stream;

use crate::AppState;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/register", post(login::register))
        .route("/api/login",post(login::login))
        .route("/api/stream", get(stream::song))
        .route("/api/songs",get(songs::get))
        .route("/api/songs/total",get(songs::get_total))
        .route("/api/songs/cover",get(songs::get_image))
        .route("/api/songs/search", get(songs::search))
        .route("/api/artists",get(artists::get))
        .route("/api/artists/total",get(artists::get_total))
        .route("/api/artists/cover",get(artists::get_image))
        .route("/api/artists/songs",get(artists::get_songs))
        .route("/api/playlists/private", get(playlist::get_private))
        .route("/api/playlists/public", get(playlist::get_public))
        .route("/api/playlists/private/total", get(playlist::get_private_total))
        .route("/api/playlists/public/total", get(playlist::get_public_total))
}