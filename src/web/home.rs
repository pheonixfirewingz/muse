use axum::response::{Html, IntoResponse};
/// Serves the home page.
///
/// This asynchronous handler returns the main HTML content for the home page
/// by including the `index.html` file from the `runtime` directory and 
/// sending it as an HTML response. This function is part of the web server
/// implementation and is used to handle requests to the root URL.
pub async fn handler() -> impl IntoResponse {
    Html(include_str!("../../runtime/index.html")).into_response()
}