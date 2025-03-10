use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug,FromRow, Serialize,Deserialize)]
pub struct Playlist {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}