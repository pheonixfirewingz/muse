use std::borrow::Cow;
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "runtime/assets/"]
struct Assets;

/// Handler for static assets.
///
/// This handler serves assets from the `runtime/assets/` directory. The
/// `Content-Type` header is set based on the file extension, and the
/// `Cache-Control` header is set to allow the asset to be cached for a year.
///
/// If the requested file does not exist, this handler returns a 404 error.
pub async fn handler(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    match Assets::get(path) {
        Some(content) => {
            // Determine the content type based on file extension
            let content_type = match path.split('.').last() {
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("png") => "image/png",
                Some("webp") => "image/webp",
                Some("ico") => "image/x-icon",
                _ => "application/octet-stream",
            };
            // Handle different content types (binary vs utf8)
            let body = match content.data {
                Cow::Borrowed(bytes) => Body::from(bytes.to_vec()),
                Cow::Owned(bytes) => Body::from(bytes),
            };
            // Build the response
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                // Add cache control for static assets
                .header(header::CACHE_CONTROL, "public, max-age=31536000")
                .body(body)
                .unwrap()
        },
        None => {
            // File not found
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("404 Not Found: File not found in static directory"))
                .unwrap()
        },
    }
}
