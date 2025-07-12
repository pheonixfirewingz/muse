use time::OffsetDateTime;
use tracing::{debug, error};
use uuid::Uuid;
use crate::db::DbPool;
use crate::db::util::sql_share::SQLResult;
use crate::{run_command, fetch_optional_row, fetch_all_rows, fetch_scalar};

#[derive(sqlx::FromRow)]
pub struct Playlist {
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub public: bool,
}

impl Playlist {
    pub fn new(name: String, user_uuid: Uuid, public: bool) -> Self {
        Playlist {
            uuid: Uuid::new_v4(),
            user_uuid,
            name,
            created_at: OffsetDateTime::now_utc(),
            public,
        }
    }
}

pub async fn create_table_if_not_exists(pool: &DbPool) -> SQLResult<()> {
    run_command!(
        pool,
        r#"CREATE TABLE IF NOT EXISTS playlists (
            uuid BLOB PRIMARY KEY NOT NULL,
            user_uuid BLOB NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            public BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (user_uuid) REFERENCES users(uuid) ON DELETE CASCADE
        )"#
    )
        .map_err(|e| {
            error!("Failed to create playlists table: {:?}", e);
            e
        })?;
    Ok(())
}

pub async fn create(pool: &DbPool, playlist: &Playlist) -> SQLResult<()> {
    debug!("Creating playlist: {}", playlist.name);

    run_command!(
        pool,
        r#"INSERT INTO playlists (uuid, user_uuid, name, created_at, public)
           VALUES (?, ?, ?, ?, ?)"#,
        playlist.uuid.as_bytes().as_slice(),
        playlist.user_uuid.as_bytes().as_slice(),
        &playlist.name,
        playlist.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
        playlist.public,
    )
        .map_err(|e| {
            error!("Failed to create playlist: {:?}", e);
            e
        })?;

    debug!("Successfully created playlist: {}", playlist.name);
    Ok(())
}

pub async fn get_by_name(pool: &DbPool, name: &str, user_uuid: &Uuid) -> SQLResult<Option<Playlist>> {
    debug!("Fetching playlist by name: {} for user: {}", name, user_uuid);

    let playlist = fetch_optional_row!(
        pool,
        Playlist,
        r#"SELECT uuid, user_uuid, name, created_at, public FROM playlists
           WHERE name = ? AND user_uuid = ?"#,
        name,
        user_uuid.as_bytes().as_slice()
    )
        .map_err(|e| {
            error!("Failed to fetch playlist by name '{}' for user {}: {:?}", name, user_uuid, e);
            e
        })?;

    if playlist.is_some() {
        debug!("Found playlist with name: {} for user: {}", name, user_uuid);
    } else {
        debug!("No playlist found with name: {} for user: {}", name, user_uuid);
    }

    Ok(playlist)
}

pub async fn get_by_uuid(pool: &DbPool, uuid: &Uuid) -> SQLResult<Option<Playlist>> {
    debug!("Fetching playlist by UUID: {}", uuid);

    let playlist = fetch_optional_row!(
        pool,
        Playlist,
        r#"SELECT uuid, user_uuid, name, created_at, public FROM playlists
           WHERE uuid = ?"#,
        uuid.as_bytes().as_slice()
    )
        .map_err(|e| {
            error!("Failed to fetch playlist by UUID {}: {:?}", uuid, e);
            e
        })?;

    if playlist.is_some() {
        debug!("Found playlist with UUID: {}", uuid);
    } else {
        debug!("No playlist found with UUID: {}", uuid);
    }

    Ok(playlist)
}

pub async fn get_by_user(pool: &DbPool, user_uuid: &Uuid, start: usize, end: usize) -> SQLResult<Vec<Playlist>> {
    let limit = (end as i64) - (start as i64);
    let offset = start as i64;
    let playlists = fetch_all_rows!(
        pool,
        Playlist,
        r#"SELECT uuid, user_uuid, name, created_at, public
           FROM playlists
           WHERE user_uuid = ?
           ORDER BY created_at DESC
           LIMIT ? OFFSET ?"#,
        user_uuid.as_bytes().as_slice(),
        limit,
        offset
    )
    .map_err(|e| {
        error!("Failed to fetch paginated playlists for user {}: {:?}", user_uuid, e);
        e
    })?;
    Ok(playlists)
}


pub async fn get_public(pool: &DbPool, start: usize, end: usize) -> SQLResult<Vec<Playlist>> {
    let limit = (end as i64) - (start as i64);
    let offset = start as i64;
    let playlists = fetch_all_rows!(
        pool,
        Playlist,
        r#"SELECT uuid, user_uuid, name, created_at, public
           FROM playlists
           WHERE public = 1
           ORDER BY created_at DESC
           LIMIT ? OFFSET ?"#,
        limit,
        offset
    )
    .map_err(|e| {
        error!("Failed to fetch paginated public playlists: {:?}", e);
        e
    })?;
    Ok(playlists)
}


pub async fn get_public_count(pool: &DbPool) -> SQLResult<i64> {
    debug!("Getting public playlist count");
    let count = fetch_scalar!(
        pool,
        i64,
        r#"SELECT COUNT(*) FROM playlists WHERE public = 1"#
    )
    .map_err(|e| {
        error!("Failed to get public playlist count: {:?}", e);
        e
    })?;
    debug!("There are {} public playlists", count);
    Ok(count)
}

pub async fn update_name(pool: &DbPool, uuid: &Uuid, new_name: &str) -> SQLResult<bool> {
    debug!("Updating playlist name for UUID: {}", uuid);

    let result = run_command!(
        pool,
        r#"UPDATE playlists SET name = ? WHERE uuid = ?"#,
        new_name,
        uuid.as_bytes().as_slice()
    )
        .map_err(|e| {
            error!("Failed to update playlist name for UUID {}: {:?}", uuid, e);
            e
        })?;

    let updated = result.rows_affected() > 0;
    if updated {
        debug!("Successfully updated playlist name for UUID: {}", uuid);
    } else {
        debug!("No playlist found to update with UUID: {}", uuid);
    }

    Ok(updated)
}

pub async fn delete(pool: &DbPool, uuid: &Uuid) -> SQLResult<bool> {
    debug!("Deleting playlist with UUID: {}", uuid);

    let result = run_command!(
        pool,
        r#"DELETE FROM playlists WHERE uuid = ?"#,
        uuid.as_bytes().as_slice()
    )
        .map_err(|e| {
            error!("Failed to delete playlist with UUID {}: {:?}", uuid, e);
            e
        })?;

    let deleted = result.rows_affected() > 0;
    if deleted {
        debug!("Successfully deleted playlist with UUID: {}", uuid);
    } else {
        debug!("No playlist found to delete with UUID: {}", uuid);
    }

    Ok(deleted)
}



pub async fn deletes_by_user(pool: &DbPool, user_uuid: &Uuid) -> SQLResult<u64> {
    debug!("Deleting all playlists for user: {}", user_uuid);

    let result = run_command!(
        pool,
        r#"DELETE FROM playlists WHERE user_uuid = ?"#,
        user_uuid.as_bytes().as_slice()
    )
        .map_err(|e| {
            error!("Failed to delete playlists for user {}: {:?}", user_uuid, e);
            e
        })?;

    let deleted_count = result.rows_affected();
    debug!("Deleted {} playlists for user: {}", deleted_count, user_uuid);

    Ok(deleted_count)
}


pub async fn playlist_exists(pool: &DbPool, uuid: &Uuid) -> SQLResult<bool> {
    debug!("Checking if playlist exists with UUID: {}", uuid);

    let exists = fetch_scalar!(
        pool,
        bool,
        r#"SELECT EXISTS(SELECT 1 FROM playlists WHERE uuid = ?)"#,
        uuid.as_bytes().as_slice()
    )
        .map_err(|e| {
            error!("Failed to check playlist existence for UUID {}: {:?}", uuid, e);
            e
        })?;

    debug!("Playlist with UUID {} exists: {}", uuid, exists);
    Ok(exists)
}

pub async fn playlist_name_exists_for_user(pool: &DbPool, name: &str, user_uuid: &Uuid) -> SQLResult<bool> {
    debug!("Checking if playlist name '{}' exists for user: {}", name, user_uuid);

    let exists = fetch_scalar!(
        pool,
        bool,
        r#"SELECT EXISTS(SELECT 1 FROM playlists WHERE name = ? AND user_uuid = ?)"#,
        name,
        user_uuid.as_bytes().as_slice()
    )
        .map_err(|e| {
            error!("Failed to check playlist name existence for user {}: {:?}", user_uuid, e);
            e
        })?;

    debug!("Playlist name '{}' exists for user {}: {}", name, user_uuid, exists);
    Ok(exists)
}

pub async fn get_count_by_user(pool: &DbPool, user_uuid: &Uuid) -> SQLResult<i64> {
    debug!("Getting playlist count for user: {}", user_uuid);

    let count = fetch_scalar!(
        pool,
        i64,
        r#"SELECT COUNT(*) FROM playlists WHERE user_uuid = ?"#,
        user_uuid.as_bytes().as_slice()
    )
        .map_err(|e| {
            error!("Failed to get playlist count for user {}: {:?}", user_uuid, e);
            e
        })?;

    debug!("User {} has {} playlists", user_uuid, count);
    Ok(count)
}

pub async fn toggle_visibility_by_name(pool: &DbPool, name: &str, user_uuid: &Uuid) -> SQLResult<bool> {
    debug!("Attempting to toggle visibility for playlist '{}' by user {}", name, user_uuid);

    // Fetch the playlist by name and user ID
    let playlist = fetch_optional_row!(
        pool,
        Playlist,
        r#"SELECT uuid, user_uuid, name, created_at, public
           FROM playlists
           WHERE name = ? AND user_uuid = ?"#,
        name,
        user_uuid.as_bytes().as_slice()
    ).map_err(|e| {
        error!("Error fetching playlist '{}': {:?}", name, e);
        e
    })?;

    match playlist {
        Some(p) => {
            let new_value = !p.public;

            let result = run_command!(
                pool,
                r#"UPDATE playlists SET public = ? WHERE uuid = ? AND user_uuid = ?"#,
                new_value,
                p.uuid.as_bytes().as_slice(),
                user_uuid.as_bytes().as_slice()
            ).map_err(|e| {
                error!("Failed to update visibility for playlist '{}': {:?}", name, e);
                e
            })?;

            if result.rows_affected() > 0 {
                debug!("Successfully toggled visibility for playlist '{}' to {}", name, new_value);
                Ok(true)
            } else {
                debug!("Playlist '{}' found, but update affected no rows", name);
                Ok(false)
            }
        }
        None => {
            debug!("No playlist named '{}' found for user {}", name, user_uuid);
            Ok(false)
        }
    }
}