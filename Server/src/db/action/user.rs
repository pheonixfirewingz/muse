use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::{session, user, DbPool};

#[derive(Serialize,Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String
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