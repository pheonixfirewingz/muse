use time::{Duration, OffsetDateTime};
use crate::db::util::sql_share::SQLResult;
use crate::db::DbPool;
use uuid::Uuid;
use tracing::{instrument, error};
use crate::{fetch_one_row, run_command};

/// Represents a user session with uuid, user uuid, and expiration timestamp.
#[derive(Debug, sqlx::FromRow)]
pub struct Session {
    uuid: Uuid,
    user_uuid: Uuid,
    expires_at: OffsetDateTime,
}

impl Session {
    /// Create a new session with a fresh UUID and expiry time.
    pub fn new(user_uuid: Uuid, expires_at: OffsetDateTime) -> Session {
        Session {
            uuid: Uuid::new_v4(),
            user_uuid,
            expires_at,
        }
    }
}

/// Creates the `sessions` table if it does not exist and clears all existing sessions.
///
/// The table has columns:
/// - `uuid` (UUID string, PK),
/// - `user_uuid` (UUID string, FK referencing users),
/// - `expires_at` (timestamp string).
///
/// # Errors
/// Returns an SQL error if the creation query fails.
#[instrument(skip(pool))]
pub async fn create_table_if_not_exists(pool: &DbPool) -> SQLResult<()> {
    // Create table if it doesn't exist
    run_command!(
        pool,
        r#"CREATE TABLE IF NOT EXISTS sessions (
            uuid BLOB PRIMARY KEY NOT NULL,
            user_uuid BLOB NOT NULL,
            expires_at TEXT NOT NULL,
            FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
        )"#).map_err(|e| {
        error!("Failed to create sessions table: {:?}", e);
        e
    })?;

    // Clear all existing sessions on boot
    run_command!(pool, "DELETE FROM sessions")?;

    Ok(())
}

/// Inserts a new session for the given user uuid.
///
/// Automatically cleans up expired sessions beforehand.
/// Sets expiry to 24 hours from the current UTC time.
///
/// # Returns
/// The UUID of the created session.
///
/// # Errors
/// Returns SQL errors from insertion or cleanup failures?
#[instrument(skip(pool))]
pub async fn create_session(pool: &DbPool, user_uuid: &Uuid) -> SQLResult<Uuid> {
    cleanup_expired_sessions(pool).await?;
    let expires_at = OffsetDateTime::now_utc() + Duration::hours(24);
    let session = Session::new(*user_uuid, expires_at);
    run_command!(
        pool,
        r#"INSERT INTO sessions (uuid, user_uuid, expires_at) VALUES (?, ?, ?)"#,
        &session.uuid,&session.user_uuid,session.expires_at)
        .map_err(|e| {
            error!("Failed to create session: {:?}", e);
            e
        })?;
    Ok(session.uuid)
}

/// Validates if a session with given uuid exists and is not expired.
///
/// # Returns
/// `Ok(true)` if valid session exists, `Ok(false)` otherwise.
///
/// # Errors
/// SQL errors from the query execution.
#[instrument(skip(pool))]
pub async fn validate_session(pool: &DbPool, session_uuid: Uuid) -> SQLResult<bool> {
    cleanup_expired_sessions(pool).await?;
    #[derive(sqlx::FromRow)]
    struct Exists {  count: i64 }
    let now = OffsetDateTime::now_utc();
    let result = fetch_one_row!(pool,Exists,
        "SELECT COUNT(*) as count FROM sessions WHERE uuid = ? AND expires_at > ?",
        session_uuid,now)?;
    let valid = result.count > 0;
    Ok(valid)
}

/// Retrieves the user UUID associated with a given session ID, 
/// only if the session has not expired.
///
/// # Returns
///  a [`Result`] containing the user's UUID if the session is valid,
/// or a [`sqlx::Error`] if the session is not found or a query error occurs.
///
/// # Errors
/// Returns an error if:
/// - The session does not exist.
/// - The session has expired.
/// - A database error occurs.
pub async fn get_user_id_from_session_id(session_id: &Uuid, db: &DbPool) -> SQLResult<Uuid> {
    let now = OffsetDateTime::now_utc();
    let session = fetch_one_row!(db,Session,
        r#"SELECT uuid, user_uuid, expires_at FROM sessions
        WHERE uuid = $1 AND expires_at > $2"#,
        session_id, now)?;
    Ok(session.user_uuid)
}

/// Deletes a session by session uuid.
///
/// # Errors
/// SQL errors from deletion failure.
#[instrument(skip(pool))]
pub async fn delete_session(pool: &DbPool, session_uuid: Uuid) -> SQLResult<()> {
    run_command!(pool, "DELETE FROM sessions WHERE uuid = ?", session_uuid).map_err(|e| {
        error!("Failed to delete session: {:?}", e);
        e
    })?;
    Ok(())
}

/// Deletes all sessions for a given user uuid.
///
/// # Errors
/// SQL errors from deletion failure.
#[instrument(skip(pool))]
pub async fn delete_user_sessions(pool: &DbPool, user_uuid: Uuid) -> SQLResult<()> {
    run_command!(pool, "DELETE FROM sessions WHERE user_uuid = ?", user_uuid).map_err(|e| {
        error!("Failed to delete user sessions: {:?}", e);
        e
    })?;
    Ok(())
}

/// Cleans up expired sessions by deleting those with `expires_at` before the current time.
///
/// Called internally before creating a session.
///
/// # Errors
/// SQL errors from deletion failure.
#[instrument(skip(pool))]
async fn cleanup_expired_sessions(pool: &DbPool) -> SQLResult<()> {
    run_command!(pool,r#"DELETE FROM sessions WHERE expires_at < strftime('%Y-%m-%dT%H:%M:%SZ', 'now')"#)
        .map_err(|e| {
            error!("Failed to cleanup expired sessions: {:?}", e);
            e
        })?;
    Ok(())
}
