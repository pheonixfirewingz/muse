//! Muse Music Server API - Spec-compliant implementation
//! 
//! All endpoints match the API reference documentation.
//! Requires JWT authentication except for public endpoints.

pub mod response;
pub mod auth;
pub mod songs;
pub mod artists;
pub mod playlists;
pub mod users;
pub mod streaming;
pub mod admin;

use axum::{Router, routing::{get, post, put, delete}, middleware};
use tower_http::cors::{CorsLayer, Any};
use crate::api::auth::AppState;
use crate::auth::middleware::{require_auth, require_admin, AuthState};

/// Create the main API router with all endpoints
pub fn create_router(state: AppState) -> Router {
    // Create auth state for middleware
    let auth_state = AuthState {
        jwt_service: state.jwt_service.clone(),
    };
    
    Router::new()
        // Public health check
        .route("/api/health", get(auth::health_check))
        
        // Public auth routes
        .merge(public_auth_routes())
        
        // Protected routes (require authentication)
        .merge(protected_routes(auth_state.clone()))
        
        // Admin routes (require admin role)
        .merge(admin_routes_protected(auth_state.clone()))
        
        // Add CORS
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        
        // Add application state
        .with_state(state)
}

fn public_auth_routes() -> Router<AppState> {
    Router::new()
        .route("/api/register", post(auth::register))
        .route("/api/login", post(auth::login))
}

fn protected_routes(auth_state: AuthState) -> Router<AppState> {
    Router::new()
        // Auth routes
        .route("/api/logout", post(auth::logout))
        .route("/api/refresh", post(auth::refresh_token))
        
        // Song routes
        .nest("/api/songs", songs_routes())
        
        // Artist routes
        .nest("/api/artists", artists_routes())
        
        // Playlist routes
        .nest("/api/playlists", playlists_routes())
        
        // User routes
        .nest("/api/user", user_routes())
        
        // Streaming routes
        .nest("/api/stream", streaming_routes())
        
        // Apply authentication middleware to all these routes
        .route_layer(middleware::from_fn_with_state(auth_state, require_auth))
}

fn admin_routes_protected(auth_state: AuthState) -> Router<AppState> {
    Router::new()
        .nest("/api/admin", admin_routes())
        // Apply admin middleware
        .route_layer(middleware::from_fn_with_state(auth_state, require_admin))
}


fn songs_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(songs::get_songs))
        .route("/info", get(songs::get_song_info))
        .route("/cover", get(songs::get_song_cover))
}

fn artists_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(artists::get_artists))
        .route("/cover", get(artists::get_artist_cover))
        .route("/songs", get(artists::get_artist_songs))
}

fn playlists_routes() -> Router<AppState> {
    Router::new()
        .route("/private", get(playlists::get_private_playlists))
        .route("/public", get(playlists::get_public_playlists))
        .route("/shared", get(playlists::get_shared_playlists))
        .route("/", post(playlists::create_playlist))
        .route("/", delete(playlists::delete_playlist))
        .route("/song/add", post(playlists::add_song_to_playlist))
        .route("/song/remove", post(playlists::remove_song_from_playlist))
        .route("/share", post(playlists::share_playlist))
        .route("/share", delete(playlists::revoke_playlist_share))
}

fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(users::get_user_info))
        .route("/", put(users::update_user_info))
        .route("/password", put(users::change_password))
        .route("/reset", post(users::reset_password))
        .route("/delete", post(users::delete_account))
}

fn streaming_routes() -> Router<AppState> {
    Router::new().route("/", get(streaming::stream_song))
}

fn admin_routes() -> Router<AppState> {
    Router::new()
        // Admin users authenticate via the regular /login endpoint
        // Access is controlled by the is_admin flag in their JWT token
        .route("/users", get(admin::get_all_users))
        .route("/users/edit", put(admin::edit_user))
        .route("/users/delete", delete(admin::delete_user))
        .route("/songs/add", post(admin::add_song))
        .route("/songs/edit", put(admin::edit_song))
        .route("/songs/delete", delete(admin::delete_song))
        .route("/songs/scan", post(admin::scan_music_directory))
        .route("/playlists", get(admin::get_all_playlists))
        .route("/playlists/edit", put(admin::edit_playlist))
        .route("/playlists/delete", delete(admin::delete_playlist))
}
