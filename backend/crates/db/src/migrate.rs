use std::path::{Path, PathBuf};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

pub async fn init_pool_memory() -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new().in_memory(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    Ok(pool)
}

pub async fn init_pool(data_dir: &Path) -> Result<SqlitePool, sqlx::Error> {
    std::fs::create_dir_all(data_dir).ok();
    let db_path = data_dir.join("dept_tracker.db");
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

async fn run_sql_script(pool: &SqlitePool, sql: &str) -> Result<(), sqlx::Error> {
    for statement in sql.split(';') {
        let stmt = statement.trim();
        if !stmt.is_empty() {
            sqlx::query(stmt).execute(pool).await?;
        }
    }
    Ok(())
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    run_sql_script(pool, include_str!("../../../migrations/001_initial.sql")).await?;
    // Idempotent ALTERs for existing databases
    let _ = run_sql_script(pool, include_str!("../../../migrations/002_tilgung_auth.sql")).await;
    let _ = run_sql_script(
        pool,
        include_str!("../../../migrations/003_first_payment_date.sql"),
    )
    .await;
    Ok(())
}

pub fn data_dir_from_env() -> PathBuf {
    std::env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data"))
}
