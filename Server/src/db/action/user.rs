use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::{session, user, DbPool};

#[derive(Serialize,Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String
}

#[derive(Deserialize)]
pub struct UpdateUserInfo {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[allow(unused)]
pub async fn get_user_info_from_session_id(pool: &DbPool, session_id:&Uuid) -> Option<UserInfo>{
    let user_id = match session::get_user_id_from_session_id(session_id, pool).await {
        Ok(user_id) => user_id,
        Err(_) => return None
    };

    let user = match user::get_user_by_uuid(pool,&user_id).await {
        Ok(user) => user,
        Err(_) => return None
    };

    Some(UserInfo { username: user.username.to_string(), email: user.email.to_string() })
}

pub async fn update_user_info_from_session_id(
    pool: &DbPool,
    session_id: &Uuid,
    update: UpdateUserInfo,
) -> Result<(), String> {
    let user_id = match session::get_user_id_from_session_id(session_id, pool).await {
        Ok(user_id) => user_id,
        Err(_) => return Err("Invalid session".to_string()),
    };

    // Only update fields that are Some
    if update.username.is_none() && update.email.is_none() {
        return Ok(());
    }

    if let Some(username) = update.username {
        if let Err(e) = user::update_username(pool, &user_id, &username).await {
            return Err(format!("Failed to update username: {}", e));
        }
    }
    if let Some(email) = update.email {
        if let Err(e) = user::update_email(pool, &user_id, &email).await {
            return Err(format!("Failed to update email: {}", e));
        }
    }
    Ok(())
}

pub async fn delete_user_by_uuid(pool: &DbPool, uuid: &Uuid) -> Result<(), String> {
    crate::db::user::delete_user_by_uuid(pool, uuid)
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_user_with_password(pool: &DbPool, session_token: &str, password: &str) -> Result<(), String> {
    use uuid::Uuid;
    let session_id = Uuid::parse_str(session_token).map_err(|_| "Invalid session token".to_string())?;
    // Get user id from session
    let user_id = match crate::db::session::get_user_id_from_session_id(&session_id, pool).await {
        Ok(id) => id,
        Err(_) => return Err("Invalid session".to_string()),
    };
    // Get user
    let user = match crate::db::user::get_user_by_uuid(pool, &user_id).await {
        Ok(u) => u,
        Err(_) => return Err("User not found".to_string()),
    };
    // Check password
    if !bcrypt::verify(password, user.get_password_hash()).unwrap_or(false) {
        return Err("Incorrect password".to_string());
    }
    // Delete user
    delete_user_by_uuid(pool, &user_id).await.map_err(|e| e.to_string())?;
    // Delete all sessions for this user
    session::delete_user_sessions(pool, user_id).await.map_err(|e| e.to_string())?;
    Ok(())
}