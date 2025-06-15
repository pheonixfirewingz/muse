pub mod list;
pub mod pages;
pub mod r#static;
pub mod stream;
pub mod images;
pub mod login;

use axum::Router;
use std::sync::Arc;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(images::router())
}