use tower_cookies::Cookies;
use uuid::Uuid;
use crate::web;
use crate::api::error::ApiError;
use crate::api::error::ApiError::{BadRequest, Unauthorized};

pub fn get_session_id_from_cookies(cookies: &Cookies) -> Result<Uuid, ApiError>
{
    match web::get_session_id_from_cookies(cookies) {
        Ok(uuid) => Ok(uuid),
        Err(Some(msg)) => Err(BadRequest(msg)),
        Err(None) => Err(Unauthorized),
    }
}