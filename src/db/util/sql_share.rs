pub type SQLResult<T> = Result<T, sqlx::Error>;

/// Macro to fetch one row and deserialize it into a struct of type `$ty`.
///
/// Usage:
/// ```rust
/// let user: User = fetch_one_row!(pool, User, "SELECT * FROM users WHERE id = ?", 42).await?;
/// ```
///
/// This expands roughly to:
/// ```rust
/// sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
///     .bind(42)
///     .fetch_one(&pool)
///     .await
/// ```
#[macro_export]
macro_rules! fetch_one_row {
    // Accept zero or more binds after the query string
    ($pool:expr, $ty:ty, $query:expr $(, $bind:expr )* $(,)?) => {{
        let q = sqlx::query_as::<_, $ty>($query)
            $(.bind($bind))*
            ;
        q.fetch_one($pool).await
    }};
}

/// Fetch a single row of type `$ty` from the database using the given query and bindings,
/// returning `Ok(Some(row))` if found, `Ok(None)` if not found, or an `Err` on failure.
///
/// # Parameters
/// - `$pool`: a reference to the database pool (e.g., `&DbPool`)
/// - `$ty`: the expected Rust type that implements `FromRow`
/// - `$query`: the SQL query string (should return at most one row)
/// - `$bind`: any number of optional bind parameters (in order)
///
/// # Example
/// ```ignore
/// let song: Option<Song> = fetch_optional_row!(
///     &pool, Song, "SELECT * FROM songs WHERE uuid = ?", song_uuid
/// )?;
/// ```
#[macro_export]
macro_rules! fetch_optional_row {
    ($pool:expr, $ty:ty, $query:expr $(, $bind:expr )* $(,)?) => {{
        let q = sqlx::query_as::<_, $ty>($query)
            $(.bind($bind))*
            ;
        match q.fetch_optional($pool).await {
            Ok(opt) => Ok(opt),
            Err(e) => Err(e),
        }
    }};
}

/// Macro to fetch a single scalar value of type `$ty` from the database.
///
/// # Example
///
/// ```rust
/// let exists: bool = fetch_scalar!(pool, bool, "SELECT EXISTS(SELECT 1 FROM users WHERE name = ?)", username).await?;
/// ```
///
/// Expands roughly to:
///
/// ```rust
/// sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE name = ?)")
///     .bind(username)
///     .fetch_one(&pool)
///     .await
/// ```
#[macro_export]
macro_rules! fetch_scalar {
    ($pool:expr, $ty:ty, $query:expr $(, $bind:expr )* $(,)?) => {{
        sqlx::query_scalar::<_, $ty>($query)
            $(.bind($bind))*
            .fetch_one($pool)
            .await
    }};
}

/// Macro to fetch all rows and deserialize into a Vec of `$ty`.
///
/// Usage:
/// ```rust
/// let users: Vec<User> = fetch_all_rows!(pool, User, "SELECT * FROM users WHERE active = ?", true).await?;
/// ```
///
/// Expands roughly to:
/// ```rust
/// sqlx::query_as::<_, User>("SELECT * FROM users WHERE active = ?")
///     .bind(true)
///     .fetch_all(&pool)
///     .await
/// ```
#[macro_export]
macro_rules! fetch_all_rows {
    ($pool:expr, $ty:ty, $query:expr $(, $bind:expr )* $(,)?) => {{
        let q = sqlx::query_as::<_, $ty>($query)
            $(.bind($bind))*
            ;
        q.fetch_all($pool).await
    }};
}

/// Macro to fetch a list of scalar values of type `$ty` from the database.
///
/// # Example
///
/// ```rust
/// let usernames: Vec<String> = fetch_all_scalar!(
///     pool,
///     String,
///     "SELECT name FROM users WHERE active = ?",
///     true
/// ).await?;
/// ```
///
/// Expands roughly to:
///
/// ```rust
/// sqlx::query_scalar::<_, String>("SELECT name FROM users WHERE active = ?")
///     .bind(true)
///     .fetch_all(&pool)
///     .await
/// ```
#[macro_export]
macro_rules! fetch_all_scalar {
    ($pool:expr, $ty:ty, $query:expr $(, $bind:expr )* $(,)?) => {{
        sqlx::query_scalar::<_, $ty>($query)
            $(.bind($bind))*
            .fetch_all($pool)
            .await
    }};
}

/// Macro to execute a SQL command that does not return rows (e.g. `INSERT`, `UPDATE`, `DELETE`, `CREATE TABLE`).
///
/// # Parameters
/// - `$executor`: The database executor or connection (e.g., a connection pool).
/// - `$sql`: The SQL command string with `?` placeholders for parameters.
/// - `$bind`: Zero or more values to bind to the SQL command.
///
/// # Returns
/// A `Future` resolving to `Result<sqlx::postgres::PgQueryResult, sqlx::Error>` (or the appropriate backend result),
/// representing the outcome of the command execution.
///
/// # Example
/// ```rust
/// run_command!(pool, "DELETE FROM sessions WHERE expires_at < ?", some_expiration_time).await?;
/// ```
///
/// # Expansion
/// Expands roughly to:
/// ```rust
/// sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
///     .bind(some_expiration_time)
///     .execute(&pool)
///     .await
/// ```
#[macro_export]
macro_rules! run_command {
    ($executor:expr, $sql:expr $(, $bind:expr )* $(,)?) => {{
        let query = sqlx::query($sql)
            $(.bind($bind))*
            ;
        query.execute($executor).await
    }};
}