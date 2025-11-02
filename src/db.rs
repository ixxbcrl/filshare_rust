use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;
use tracing::info;

pub type DbPool = Pool<Sqlite>;

pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
    info!("Initializing database connection...");

    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Run migrations
    info!("Running database migrations...");
    sqlx::query(include_str!("../migrations/001_create_files_table.sql"))
        .execute(&pool)
        .await?;

    sqlx::query(include_str!("../migrations/002_create_directories_table.sql"))
        .execute(&pool)
        .await?;

    info!("Database initialized successfully");
    Ok(pool)
}
