use crate::{db, AppState};
use axum::extract::Query;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CacheQueryType {
    Image,
}

#[derive(Deserialize)]
struct CacheQuery {
    artist_name: Option<String>,
    song_name: Option<String>,
    info_type: CacheQueryType
}

pub async fn handler(Query(params): Query<CacheQuery>) -> Json<Value> {
    match (&params.artist_name, &params.song_name, &params.info_type) {
        (Some(artist_name), None, CacheQueryType::Image) => {
            match db::thirdparty::get_artist_image_url(artist_name).await {
                Ok(Some(image_url)) => Json(json!({
                    "success": true,
                    "image_url": image_url,
                })),
                _=> Json(json!({
                    "success": false,
                    "error": "Image not found"
                }))
            }
        },
        (Some(artist_name), Some(song_name), CacheQueryType::Image) => {
            match db::thirdparty::get_song_image_url(artist_name,song_name).await {
                Ok(Some(image_url)) => Json(json!({
                    "success": true,
                    "image_url": image_url,
                })),
                _=> Json(json!({
                    "success": false,
                    "error": "Image not found"
                }))
            }
        },
        _ => Json(json!({
            "success": false,
            "error": "request invalid"
        })),
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/cache",get(handler))
}