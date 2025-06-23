use time::OffsetDateTime;
use tracing::{error, info, instrument};
use uuid::Uuid;
use crate::db::DbPool;
use crate::db::schema::sql_share::SQLResult;
use crate::{run_command, fetch_optional_row, fetch_all_rows, fetch_scalar};

#[derive(Debug, sqlx::FromRow)]
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

#[instrument(skip(pool))]
pub async fn create_table_if_not_exists(pool: &DbPool) -> SQLResult<()> {
    info!("Creating playlists table if not exists");
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
    info!("playlists table ready");
    Ok(())
}

#[instrument(skip(pool))]
pub async fn create_playlist(pool: &DbPool, playlist: &Playlist) -> SQLResult<()> {
    info!("Creating playlist: {}", playlist.name);

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

    info!("Successfully created playlist: {}", playlist.name);
    Ok(())
}

#[instrument(skip(pool))]
pub async fn get_playlist_by_name(pool: &DbPool, name: &str, user_uuid: &Uuid) -> SQLResult<Option<Playlist>> {
    info!("Fetching playlist by name: {} for user: {}", name, user_uuid);

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
        info!("Found playlist with name: {} for user: {}", name, user_uuid);
    } else {
        info!("No playlist found with name: {} for user: {}", name, user_uuid);
    }

    Ok(playlist)
}

#[instrument(skip(pool))]
pub async fn get_playlist_by_uuid(pool: &DbPool, uuid: &Uuid) -> SQLResult<Option<Playlist>> {
    info!("Fetching playlist by UUID: {}", uuid);

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
        info!("Found playlist with UUID: {}", uuid);
    } else {
        info!("No playlist found with UUID: {}", uuid);
    }

    Ok(playlist)
}

#[instrument(skip(pool))]
pub async fn get_playlists_by_user(pool: &DbPool, user_uuid: &Uuid) -> SQLResult<Vec<Playlist>> {
    info!("Fetching playlists for user: {}", user_uuid);

    let playlists = fetch_all_rows!(
        pool,
        Playlist,
        r#"SELECT uuid, user_uuid, name, created_at, public
           FROM playlists
           WHERE user_uuid = ?
           ORDER BY created_at DESC"#,
        user_uuid.as_bytes().as_slice()
    )
        .map_err(|e| {
            error!("Failed to fetch playlists for user {}: {:?}", user_uuid, e);
            e
        })?;

    info!("Found {} playlists for user: {}", playlists.len(), user_uuid);
    Ok(playlists)
}

#[instrument(skip(pool))]
pub async fn update_playlist_name(pool: &DbPool, uuid: &Uuid, new_name: &str) -> SQLResult<bool> {
    info!("Updating playlist name for UUID: {}", uuid);

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
        info!("Successfully updated playlist name for UUID: {}", uuid);
    } else {
        info!("No playlist found to update with UUID: {}", uuid);
    }

    Ok(updated)
}

#[instrument(skip(pool))]
pub async fn delete_playlist(pool: &DbPool, uuid: &Uuid) -> SQLResult<bool> {
    info!("Deleting playlist with UUID: {}", uuid);

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
        info!("Successfully deleted playlist with UUID: {}", uuid);
    } else {
        info!("No playlist found to delete with UUID: {}", uuid);
    }

    Ok(deleted)
}

#[instrument(skip(pool))]
pub async fn delete_playlists_by_user(pool: &DbPool, user_uuid: &Uuid) -> SQLResult<u64> {
    info!("Deleting all playlists for user: {}", user_uuid);

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
    info!("Deleted {} playlists for user: {}", deleted_count, user_uuid);

    Ok(deleted_count)
}

#[instrument(skip(pool))]
pub async fn playlist_exists(pool: &DbPool, uuid: &Uuid) -> SQLResult<bool> {
    info!("Checking if playlist exists with UUID: {}", uuid);

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

    info!("Playlist with UUID {} exists: {}", uuid, exists);
    Ok(exists)
}

#[instrument(skip(pool))]
pub async fn playlist_name_exists_for_user(pool: &DbPool, name: &str, user_uuid: &Uuid) -> SQLResult<bool> {
    info!("Checking if playlist name '{}' exists for user: {}", name, user_uuid);

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

    info!("Playlist name '{}' exists for user {}: {}", name, user_uuid, exists);
    Ok(exists)
}

#[instrument(skip(pool))]
pub async fn get_playlist_count_by_user(pool: &DbPool, user_uuid: &Uuid) -> SQLResult<i64> {
    info!("Getting playlist count for user: {}", user_uuid);

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

    info!("User {} has {} playlists", user_uuid, count);
    Ok(count)
}

#[instrument(skip(pool))]
pub async fn get_public_playlists(pool: &DbPool) -> SQLResult<Vec<Playlist>> {
    info!("Fetching public playlists");

    let playlists = fetch_all_rows!(
        pool,
        Playlist,
        r#"SELECT uuid, user_uuid, name, created_at, public
           FROM playlists
           WHERE public = 1
           ORDER BY created_at DESC"#
    )
        .map_err(|e| {
            error!("Failed to fetch public playlists: {:?}", e);
            e
        })?;

    info!("Found {} public playlists", playlists.len());
    Ok(playlists)
}

#[instrument(skip(pool))]
pub async fn toggle_playlist_visibility_by_name(pool: &DbPool, name: &str, user_uuid: &Uuid) -> SQLResult<bool> {
    info!("Attempting to toggle visibility for playlist '{}' by user {}", name, user_uuid);

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
                info!("Successfully toggled visibility for playlist '{}' to {}", name, new_value);
                Ok(true)
            } else {
                info!("Playlist '{}' found, but update affected no rows", name);
                Ok(false)
            }
        }
        None => {
            info!("No playlist named '{}' found for user {}", name, user_uuid);
            Ok(false)
        }
    }
}