use serde::Serialize;
use sqlx::FromRow;
use crate::db::DbPool;
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub is_admin: bool,
    pub username: String,
    pub password_hash: String
}

impl User {
    pub fn new(is_admin: bool, username: String, password_hash: String) -> Self {
        Self { id: Uuid::new_v4().to_string(), is_admin, username, password_hash }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn get_password_hash(&self) -> String {
        self.password_hash.clone()
    }

    pub fn get_is_admin(&self) -> bool {
        self.is_admin
    }
}

pub async fn create_users_table_if_not_exists(pool: &DbPool) {
    let result = sqlx::query("CREATE TABLE IF NOT EXISTS users (
        id VARCHAR(36) PRIMARY KEY,
        is_admin BOOLEAN NOT NULL,
        username VARCHAR(255) NOT NULL,
        password_hash VARCHAR(255) NOT NULL
    )").execute(pool).await;

    match result {
        Ok(_) => println!("Users table is ready."),
        Err(e) => println!("Failed to create users table: {}", e),
    }
}

pub async fn add_user(pool: &DbPool, username: String, password_hash: String, is_admin: bool) -> Result<User, sqlx::Error> {
    let user = User::new(is_admin,username,password_hash);

    sqlx::query("INSERT INTO users (id, is_admin, username, password_hash) VALUES (?, ?, ?, ?)")
        .bind(&user.id)
        .bind(user.is_admin)
        .bind(&user.username)
        .bind(&user.password_hash)
        .execute(pool)
        .await?;

    Ok(user)
}

pub async fn get_user_by_username(pool: &DbPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_by_id(pool: &DbPool, id: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn validate_login(pool: &DbPool, username: &str, password_hash: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ? AND password_hash = ?")
        .bind(username)
        .bind(password_hash)
        .fetch_optional(pool)
        .await
}

pub async fn is_admin(pool: &DbPool, user_id: &str) -> Result<bool, sqlx::Error> {
    let user = get_user_by_id(pool, user_id).await?;
    Ok(user.map_or(false, |u| u.is_admin))
}