pub mod error;
pub mod middleware;
pub mod router;
pub mod routes;

use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
}

pub async fn build_app(static_dir: Option<PathBuf>) -> Result<axum::Router, Box<dyn std::error::Error>> {
    let data_dir = db::migrate::data_dir_from_env();
    let pool = db::init_pool(&data_dir).await?;
    db::run_migrations(&pool).await?;
    db::settings::ensure_settings(&pool).await?;
    db::auth::bootstrap_auth(&pool).await?;
    let state = AppState { pool };
    Ok(router::app(state, static_dir))
}
