use crate::api::io_util::ApiError;
use crate::db::action;
use crate::AppState;
use axum::extract::{Query, State};
use axum::{body::Body, http::{header, Response, StatusCode}};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::body::Bytes;
use tokio_util::io::ReaderStream;
use tracing::error;

#[derive(Serialize, Deserialize)]
pub struct StreamQuery {
    pub artist: String,
    pub name: String,
    pub format : Option<String>,
}

pub async fn song(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<StreamQuery>,
) -> Result<Response<Body>, ApiError> {
    if !action::is_valid_user(&state.db, auth.token()).await? {
        return Err(ApiError::Unauthorized);
    }

    let format = params.format.clone().unwrap_or("mp3".to_string());

    let file_path = match action::song::get_file_path(
        &state.db,
        &params.name,
        &params.artist,
        Some(&[format.as_str()])
    ).await {
        Ok(file_path) => file_path,
        Err(e) => {
            error!("Internal song fetch error: {:?}", e);
            return Err(ApiError::InternalServerError("Music could not be streamed.".to_string()));
        },
    };

    let true_format = file_path.1.as_str();

    let file = match tokio::fs::File::open(&file_path.0).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open file: {:?}", e);
            return Err(ApiError::InternalServerError("Music could not be streamed.".to_string()));
        }
    };

    let body_stream = ReaderStream::new(file)
        .map(|result| -> Result<Bytes, std::io::Error> {
            match result {
                Ok(chunk) => Ok(Bytes::from(chunk)),
                Err(e) => {
                    error!("File read error: {:?}", e);
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "Stream error"))
                },
            }
        });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, match true_format {
            "mp3" => "audio/mpeg",
            "m4a" | "aac" | "mp4" => "audio/mp4",
            _ => "application/octet-stream",
        })
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::from_stream(body_stream))
        .map_err(|e| {
            error!("Response build error: {:?}", e);
            ApiError::InternalServerError("Music could not be streamed.".to_string())
        })?)
}