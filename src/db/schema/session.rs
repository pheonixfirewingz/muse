use crate::db::DbPool;
use time::OffsetDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Session {
    id: Option<String>,
    user_id: String,
    expires_at: OffsetDateTime,
}

impl Session {
    pub fn new(user_id: String, expires_at: OffsetDateTime) -> Session {
        Session {
            id: None,
            user_id,
            expires_at,
        }
    }

    pub fn get_id(&self) -> &Option<String> {
        &self.id
    }

    pub fn get_user_id(&self) -> &String {
        &self.user_id
    }

    pub fn get_expires_at(&self) -> &OffsetDateTime {
        &self.expires_at
    }
}

pub async fn create_sessions_table_if_not_exists(pool: &DbPool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY NOT NULL DEFAULT (
                lower(
                    hex(randomblob(4)) || '-' ||
                    hex(randomblob(2)) || '-' ||
                    '4' || substr(hex(randomblob(2)), 2) || '-' ||
                    substr('89ab', abs(random() % 4) + 1, 1) || substr(hex(randomblob(2)), 2) || '-' ||
                    hex(randomblob(6))
                )
            ),
            user_id TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )"
    )
    .execute(pool)
    .await
    .unwrap();
}

pub async fn create_session(pool: &DbPool, user_id: &str) -> String {
    // First cleanup expired sessions
    cleanup_expired_sessions(pool).await;

    let expires_at = OffsetDateTime::now_utc() + time::Duration::hours(24);
    let session = Session::new(user_id.to_string(), expires_at);
    
    // Convert OffsetDateTime to ISO 8601 / RFC 3339 format
    let expires_at_str = session.get_expires_at().to_string();

    let result = sqlx::query_as::<_, (String,)>(
        "INSERT INTO sessions (user_id, expires_at) VALUES (?, ?) RETURNING id"
    )
    .bind(session.get_user_id())
    .bind(expires_at_str)
    .fetch_one(pool)
    .await
    .unwrap();

    result.0
}

pub async fn validate_session(pool: &DbPool, session_id: &str) -> Option<String> {
    // First cleanup expired sessions
    cleanup_expired_sessions(pool).await;

    let query = sqlx::query_as::<_, (String,)>(
        "SELECT user_id FROM sessions 
         WHERE id = ? AND expires_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now')"
    )
    .bind(session_id);

    match query.fetch_one(pool).await {
        Ok(result) => Some(result.0),
        Err(_) => None
    }
}

pub async fn delete_session(pool: &DbPool, session_id: &str) {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn delete_user_sessions(pool: &DbPool, user_id: &str) {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn cleanup_expired_sessions(pool: &DbPool) {
    sqlx::query("DELETE FROM sessions WHERE expires_at < strftime('%Y-%m-%dT%H:%M:%SZ', 'now')")
        .execute(pool)
        .await
        .unwrap();
}