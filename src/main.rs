mod db;
pub mod util;
mod web;

use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use minijinja::{context, Environment};
use minijinja_autoreload::AutoReloader;

struct AppState {
    env:  AutoReloader,
    db: db::DbPool,
}

#[tokio::main]
async fn main() {
    let reloader = AutoReloader::new(|notifier| {
        let template_path = "templates"; // Path to the templates directory
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader(template_path));
        notifier.watch_path(template_path, true);
        Ok(env)
    });

    // Initialize the application state with the reloader and database pool
    let app_state = Arc::new(AppState { env: reloader, db: db::init_db().await });

    // register songs
    util::scan_and_register_songs(&app_state.db, "music").await;

    // define routes
    let app = Router::new()
        .route("/", get(handler_home))
        .route("/songs", get(handler_songs))
        .route("/song/{name}", get(handler_song))
        .route("/assets/{file}", get(web::static_files::static_handler))
        .route("/cover-art", get(web::song_images::get_cover_art))
        .with_state(app_state);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler_song(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Html<String>, StatusCode> {
    let env = state.env.acquire_env().unwrap();
    let template = env.get_template("song.jinja").unwrap();
    let rendered = template
        .render(context! {
            title => format!("Song - {}", name),
            song => db::schema::song::get_song_by_name(&state.db, &name).await
        })
        .unwrap();
    Ok(Html(rendered))
}

async fn handler_songs(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let env = state.env.acquire_env().unwrap();
    let template = env.get_template("songs.jinja").unwrap();
    let rendered = template
        .render(context! {
            title => "Home",
            songs => db::schema::song::get_songs(&state.db).await
        })
        .unwrap();
    Ok(Html(rendered))
}

async fn handler_home(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let env = state.env.acquire_env().unwrap();
    let template = env.get_template("index.jinja").unwrap();
    let rendered = template
        .render(context! {
            title => "Home",
            albums => db::schema::album::get_albums(&state.db).await
        })
        .unwrap();
    Ok(Html(rendered))
}
