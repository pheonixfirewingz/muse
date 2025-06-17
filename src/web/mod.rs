pub mod list;
pub mod pages;
pub mod r#static;
pub mod stream;
pub mod images;
pub mod login;

use std::env;
use crate::db::session::validate_session;
use crate::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::Router;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use std::sync::Arc;
use std::time::Instant;
use tower_cookies::cookie::time::Duration;
use uuid::Uuid;

pub struct CacheEntry {
    valid: bool,
    expires_at: Instant,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(images::router())
}

const PUBLIC_PATHS: [&str; 7] = [
"/",
"/login",
"/register",
"/logout",
"/robots.txt",
"/sitemap.xml",
"/manifest.json",
];

const API_PATHS: [&str; 3] = [
"/stream", 
"/login/submit", 
"/register/submit"];

const CACHE_TTL: Duration = Duration::minutes(10);

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    cookies: CookieJar,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    let is_public = PUBLIC_PATHS.contains(&path) || path.starts_with("/assets/");
    // Skip auth and origin checks on public routes completely
    if is_public {
        return Ok(next.run(request).await);
    }
    // Only check origin on these API endpoints
    let origin = request.headers().get("origin").and_then(|v| v.to_str().ok());
    let referer = request.headers().get("referer").and_then(|v| v.to_str().ok());
    // Define allowed origin(s)
    let allowed_origin = env::var("WEBSITE_URL").unwrap_or_else(|_| {
        //TODO: add a way to define if this is http or https
        "http://".to_owned() + &env::var("SERVER_BIND").expect("SERVER_BIND must be set")
    });
    if API_PATHS.contains(&path) {
        let source_valid = match (origin, referer) {
            (Some(origin), _) if origin.starts_with(&allowed_origin) => true,
            (_, Some(referer)) if referer.starts_with(&allowed_origin) => true,
            _ => false,
        };

        if !source_valid {
            tracing::warn!(
                "Blocked request to '{}' — origin: {:?}, referer: {:?}, user-agent: {}",
                path,
                origin,
                referer,
                request.headers().get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("")
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let session_id = cookies.get("session_id")
        .and_then(|cookie| Uuid::parse_str(cookie.value()).ok());
    
    async fn check_session(session_id: Uuid, state: &AppState, ttl: Duration) -> bool {
        if let Some(entry) = state.auth_cache.get(&session_id) {
            if !entry.is_expired() {
                return entry.valid;
            }
        }

        let valid = validate_session(&state.db, session_id).await.unwrap_or(false);

        state.auth_cache.insert(
            session_id,
            CacheEntry {
                valid,
                expires_at: Instant::now() + ttl,
            },
        );

        valid
    }

    match session_id {
        Some(session_id) => {
            let valid = check_session(session_id, &state, CACHE_TTL).await;
            if valid {
                Ok(next.run(request).await)
            } else {
                let mut expired = Cookie::new("session_id", "");
                expired.set_path("/");
                expired.make_removal();
                let _ = cookies.remove(expired);
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}