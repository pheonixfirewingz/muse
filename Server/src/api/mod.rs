pub mod io_util;
pub mod login;
mod songs;
mod images;
mod artists;

use crate::AppState;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/register", post(login::register))
        .route("/api/login",post(login::login))
        .route("/api/songs",get(songs::get_songs))
        .route("/api/songs/total",get(songs::get_song_total))
        .route("/api/songs/cover",get(images::get_song_image))
        .route("/api/artists",get(artists::get_artists))
        .route("/api/artists/total",get(artists::get_artist_total))
        .route("/api/artists/cover",get(images::get_artist_image))
        .route("/api/artists/songs",get(artists::get_artist_songs))
}