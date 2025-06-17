use crate::db::schema::sql_share::SQLResult;
use crate::db::DbPool;
use crate::{fetch_scalar, run_command};
use arrayvec::ArrayString;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub uuid: Uuid,
    pub username: ArrayString<22>,
    pub email: ArrayString<254>,
    pub password_hash: ArrayString<60>,
}
impl User {
    pub fn new(username: &str, email: &str, password_hash: &str) -> Self {
        User {
            uuid: Uuid::new_v4(),
            username: ArrayString::from(username).expect("name too long"),
            email: ArrayString::from(email).expect("email too long"),
            password_hash: ArrayString::from(password_hash).expect("hash too long"),
        }
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    pub fn get_password_hash(&self) -> &str {
        &self.password_hash
    }
}

pub async fn create_user_table_if_not_exists(pool: &DbPool) -> SQLResult<()> {
    run_command!(pool, r#"CREATE TABLE IF NOT EXISTS users (
            uuid BLOB PRIMARY KEY NOT NULL,
            username VARCHAR(22) NOT NULL UNIQUE,
            email VARCHAR(254) NOT NULL UNIQUE,
            password_hash VARCHAR(60) NOT NULL
        )"#)?;
    Ok(())
}

pub async fn create_user_if_not_exists(pool: &DbPool, user: &User) -> SQLResult<bool> {
    if fetch_scalar!(pool, bool, r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)"#, user.get_username())? {
        return Ok(false);
    }

    if fetch_scalar!(pool, bool, r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = ?)"#, user.get_email())? {
        return Ok(false);
    }

    run_command!(pool,
        r#"INSERT INTO users (uuid, username, email, password_hash) VALUES (?, ?, ?, ?)"#,
        user.get_uuid(),user.get_username(),user.get_email(),user.get_password_hash())?;
    Ok(true)
}

/*
pub async fn delete_user_by_uuid(pool: &DbPool, uuid: Uuid) -> SQLResult<()> {
    run_command!(pool,r#"DELETE FROM users WHERE uuid = ?"#,uuid)?;
    Ok(())
}
 */

pub async fn is_valid_user(pool: &DbPool, email: &Option<&str>, username: &Option<&str>, password: &str) -> SQLResult<bool> {
    let query = match (email, username) {
        (Some(email), Some(username)) => {
            fetch_scalar!(pool,String,r#"SELECT password_hash FROM users WHERE email = ? AND name = ?"#,email,username)?
        }
        (Some(email), _) => {
            fetch_scalar!(pool,String,r#"SELECT password_hash FROM users WHERE email = ?"#,email)?
        }
        (None, Some(username)) => {
            fetch_scalar!(pool,String,r#"SELECT password_hash FROM users WHERE username = ?"#,username)?
        }
        (None, None) => {
            return Err(sqlx::Error::InvalidArgument("an email or username mush be provided to check".to_string()));
        }
    };
    Ok(bcrypt::verify(password, &query).unwrap_or(false))
}
pub async fn get_user_uuid_by_username(pool: &DbPool, username: &String) -> SQLResult<Uuid> {
    let bytes = fetch_scalar!(pool, Vec<u8>, r#"SELECT uuid FROM users WHERE username = ?"#, username)?;
    let uuid = Uuid::from_slice(&bytes).map_err(|e| sqlx::Error::ColumnDecode {
        index: "uuid".into(),
        source: Box::new(e),
    })?;
    Ok(uuid)
}