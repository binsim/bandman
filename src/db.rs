//! Database pool and migration helpers (SSR only).

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Shared Postgres connection pool.
pub type DbPool = PgPool;

/// Connect to Postgres using `DATABASE_URL` and run migrations.
pub async fn connect_and_migrate() -> Result<DbPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").map_err(|_| {
        sqlx::Error::Configuration("DATABASE_URL environment variable is not set".into())
    })?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
