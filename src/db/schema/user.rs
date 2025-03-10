use serde::Serialize;
use sqlx::FromRow;
#[derive(FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub is_admin: bool,
    pub username: String,
    pub password_hash: String
}