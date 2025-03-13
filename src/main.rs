mod db;
pub mod util;
mod web;

use axum::routing::get;
use axum::Router;
use minijinja::Environment;
#[cfg(debug_assertions)]
use minijinja_autoreload::AutoReloader;
use std::sync::Arc;
use axum::response::Redirect;
struct AppState {
    #[cfg(debug_assertions)]
    env: AutoReloader,
    #[cfg(not(debug_assertions))]
    env: Environment<'static>,
    db: db::DbPool,
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    let env = AutoReloader::new(|notifier| {
        let template_path = "runtime/templates"; // Path to the templates directory
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader(template_path));
        notifier.watch_path(template_path, true);
        Ok(env)
    });
    #[cfg(not(debug_assertions))]
    let mut env = Environment::new();
    #[cfg(not(debug_assertions))]
    {
        env.add_template("base.jinja", include_str!("../runtime/templates/base.jinja"))
            .unwrap();
        env.add_template("songs.jinja", include_str!("../runtime/templates/songs.jinja"))
            .unwrap();
        env.add_template("home.jinja", include_str!("../runtime/templates/home.jinja"))
            .unwrap();
    }

    // Initialize the application state with the reloader and database pool
    let app_state = Arc::new(AppState { env, db: db::init_db().await });
    // register songs
    util::scan_and_register_songs(&app_state.db, "runtime/music").await;
    // define routes
    let app = Router::new()
        .route("/manifest.json", get(|| async { Redirect::permanent("/assets/manifest.json") }))
        .route("/robots.txt", get(|| async { Redirect::permanent("/assets/robots.txt") }))
        .route("/sitemap.xml", get(|| async { Redirect::permanent("/assets/sitemap.xml") }))
        .route("/", get(web::home::handler))
        .route("/stream", get(web::stream::handler))
        .route("/pages/{file}", get(web::pages::hander))
        .route("/assets/{*file}", get(web::r#static::handler))
        .with_state(app_state);
    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service()).await.unwrap();
}