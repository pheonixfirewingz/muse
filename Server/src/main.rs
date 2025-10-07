mod api;
mod db;
mod auth;
mod music;

use std::sync::Arc;

use crate::api::auth::AppState;
use crate::auth::{JwtService, PasswordService};
use crate::db::{create_database, DbBackend};
use crate::music::MusicScanner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Initialize tracing with LOG_LEVEL from .env
    let log_level = std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();
    
    let level = match log_level.as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => {
            eprintln!("Invalid LOG_LEVEL '{}', defaulting to 'info'", log_level);
            tracing::Level::INFO
        }
    };
    
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
    
    // Get configuration from environment
    let db_backend = std::env::var("DB_BACKEND")
        .unwrap_or_else(|_| "sqlite".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:runtime/cache/users.db".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change_this_to_a_secure_random_secret_key".to_string());
    let jwt_expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse::<i64>()
        .unwrap_or(24);
    let server_bind = std::env::var("SERVER_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    
    tracing::info!("Using database backend: {}", db_backend);
    tracing::info!("Database URL: {}", db_url);
    tracing::info!("Server will bind to: {}", server_bind);
    
    // Create database connection
    let backend = DbBackend::from_string(&db_backend)?;
    let db = create_database(backend, &db_url).await?;
    tracing::info!("Database initialized successfully");
    
    // Perform initial music library scan
    let music_dir = std::env::var("MUSIC_DIR")
        .unwrap_or_else(|_| "runtime/music".to_string());
    
    tracing::info!("Scanning music directory: {}", music_dir);
    let scanner = MusicScanner::new(db.clone(), &music_dir);
    
    match scanner.scan_and_register().await {
        Ok(result) => {
            tracing::info!(
                "Music scan complete - Total: {}, Registered: {}, Updated: {}, Skipped: {}, Removed: {}, Errors: {}",
                result.total_files,
                result.registered,
                result.updated,
                result.skipped,
                result.removed,
                result.errors
            );
        }
        Err(e) => {
            tracing::warn!("Music scan failed: {}. Server will continue without music library.", e);
        }
    }
    
    // Create services
    let jwt_service = Arc::new(JwtService::new(&jwt_secret, jwt_expiration_hours));
    let password_service = Arc::new(PasswordService::new());
    
    // Create application state
    let app_state = AppState {
        db: db.clone(),
        jwt_service: jwt_service.clone(),
        password_service,
    };
    
    // Create the main API router using the defined api module
    let app = api::create_router(app_state);
    
    // Start server
    let listener = tokio::net::TcpListener::bind(&server_bind).await?;
    tracing::info!("Server listening on {}", server_bind);
    tracing::info!("API routes available at http://{}/api/*", server_bind);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
