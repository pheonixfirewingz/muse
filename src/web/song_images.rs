use axum::{
    extract::Query
    ,
    response::Json

    ,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

#[derive(Deserialize)]
pub struct SongQuery {
    song: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    cover_url: Option<String>,
}

pub async fn get_cover_art(Query(params): Query<SongQuery>) -> Json<ApiResponse> {
    // return empty json for now
    Json(ApiResponse { cover_url: None })
}