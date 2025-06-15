use tracing::error;
use crate::db::DbPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    id: Option<String>,
    name: String,
    email: String,
    password_hash: String
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserInfo {
    name: String,
    email: String,
}

impl User {
    pub fn new(name:String,email:String,password_hash:String) -> User {
        User {
            id: None, name, email, password_hash
        }
    }

    pub fn get_id(&self) -> &Option<String> {
        &self.id
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_email(&self) -> &String {
        &self.email
    }

    pub fn get_password_hash(&self) -> &String {
        &self.password_hash
    }
}

pub async fn create_user_table_if_not_exists(pool: &DbPool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL DEFAULT (
                lower(
                    hex(randomblob(4)) || '-' ||
                    hex(randomblob(2)) || '-' ||
                    '4' || substr(hex(randomblob(2)), 2) || '-' ||
                    substr('89ab', abs(random() % 4) + 1, 1) || substr(hex(randomblob(2)), 2) || '-' ||
                    hex(randomblob(6))
                )
            ),
            name VARCHAR(21) NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash VARCHAR(60) NOT NULL
        )"
    )
        .execute(pool)
        .await
        .unwrap();
}

pub async fn insert_user(pool: &DbPool, user: &User) {
    sqlx::query(
        "INSERT INTO users (name, email, password_hash) VALUES ($1, $2, $3)"
    ).bind(user.get_name()).bind(user.get_email())
    .bind(user.get_password_hash()).execute(pool).await.unwrap();
}

pub async fn is_valid_user(pool: &DbPool, email: &Option<&str>, name: &Option<&str>, password: &str) -> bool {
    let query = match (email, name) {
        (Some(email), _) => {
            sqlx::query_scalar::<_, String>(
                "SELECT password_hash FROM users WHERE email = $1"
            )
                .bind(email)
        },
        (None, Some(name)) => {
            sqlx::query_scalar::<_, String>(
                "SELECT password_hash FROM users WHERE name = $1"
            )
                .bind(name)
        },
        (None, None) => {
            return false;
        }
    };

    match query.fetch_optional(pool).await {
        Ok(Some(stored_hash)) => bcrypt::verify(password, &stored_hash).unwrap_or(false),
        _ => false
    }
}

pub async fn check_if_username_is_taken(pool: &DbPool, username: &str) -> bool {
    let query = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE name = $1)"
    ).bind(username);

    query.fetch_one(pool).await.unwrap_or_else(|_| false)
}

pub async fn check_if_email_is_taken(pool: &DbPool, email: &str) -> bool {
    let query = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
    ).bind(email);

    query.fetch_one(pool).await.unwrap_or_else(|_| false)
}

pub async fn get_user_by_id(pool: &DbPool, id: &str) -> Option<UserInfo> {
    let query = sqlx::query_as::<_, UserInfo>(
        "SELECT name, email FROM users WHERE id = $1"
    ).bind(id);

    match query.fetch_one(pool).await {
        Ok(user) => Some(user),
        Err(_) => None
    }

}

pub async fn get_user_id_by_email(pool: &DbPool, email: &str) -> Option<String> {
    let query = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM users WHERE email = $1"
    ).bind(email);
    match query.fetch_one(pool).await {
        Ok(id) => Some(id.0),
        Err(_) => None
    }
}

pub async fn get_user_id_by_username(pool: &DbPool, username: &str) -> Option<String> {
    let query = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM users WHERE name = $1"
    ).bind(username);
    match query.fetch_one(pool).await {
        Ok(id) => Some(id.0),
        Err(e) => {
            error!("{}", e.to_string());
            None
        }
    }
}

pub async fn delete_user_by_id(pool: &DbPool, id: &str) {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}

