mod db;
pub mod util;
mod web;
pub mod debug;
mod login;

use crate::db::schema::session::validate_session;
use axum::body::Body;
use axum::response::Html;
use axum::routing::get;
use axum::{extract::State, http::{Request, StatusCode}, middleware::Next, response::{IntoResponse, Redirect, Response}, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use db::schema;
use minijinja::Environment;
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use tower_http::compression::CompressionLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    env: Environment<'static>,
    db: db::DbPool,
}

async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    cookies: CookieJar,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    let public_paths = [
        "/",
        "/login",
        "/register",
        "/login/submit",
        "/register/submit",
        "/logout",
        "/robots.txt",
        "/sitemap.xml",
        "/manifest.json",
    ];

    let is_public = public_paths.contains(&path) || path.starts_with("/assets/");

    let session_id = cookies
        .get("session_id")
        .map(|cookie| cookie.value().to_string());

    // If the path is public...
    if is_public {
        if let Some(session_id) = session_id {
            if validate_session(&state.db, &session_id).await.is_some() {
                // For login and register, redirect if already authenticated
                if path == "/login" || path == "/register" {
                    return Ok(Redirect::to("/app").into_response());
                }
            }
        }
        return Ok(next.run(request).await);
    }

    // Non-public route, enforce auth
    match session_id {
        Some(session_id) => {
            if let Some(_user_id) = validate_session(&state.db, &session_id).await {
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


fn setup_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() {
    setup_logging();
    // Rest of your main function...
    // Initialize image cache
    schema::music_brainz::init_cache();
    
    let mut env = Environment::new();
    env.add_template("base.jinja", include_str!("../statics/templates/base.jinja")).unwrap();
    env.add_template("songs.jinja", include_str!("../statics/templates/songs.jinja")).unwrap();
    env.add_template("artists.jinja", include_str!("../statics/templates/artists.jinja")).unwrap();
    env.add_template("home.jinja", include_str!("../statics/templates/home.jinja")).unwrap();
    env.add_template("lists.jinja", include_str!("../statics/templates/lists.jinja")).unwrap();

    // Create a shared application state
    let app_state = Arc::new(AppState { env, db: db::init_db().await });

    // Scan and register music
    util::scan_and_register_songs(&app_state.db, "runtime/music").await;
    // Top-level app with global compression
    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("login") }))
        .route("/app", get(Html(include_str!("../statics/index.html"))))
        .route("/manifest.json", get(|| async { Redirect::permanent("/assets/manifest.json")}))
        .route("/robots.txt", get(|| async { Redirect::permanent("/assets/robots.txt") }))
        .route("/sitemap.xml", get(|| async { Redirect::permanent("/assets/sitemap.xml")}))
        .route("/stream", get(web::stream::handler))
        .route("/list", get(web::list::handler))
        .route("/pages/{file}", get(web::pages::hander))
        .route("/assets/{*file}", get(web::r#static::handler))
        .merge(web::login::router())
        .merge(web::router())
        .with_state(app_state.clone())
        .layer(CompressionLayer::new())
        .layer(CookieManagerLayer::new())
        .layer(axum::middleware::from_fn_with_state(
            app_state,
            auth_middleware,
        ));

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service()).await.unwrap();
}