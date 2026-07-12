use sqlx::SqlitePool;

pub struct TestDb {
    pub pool: SqlitePool,
}

pub async fn test_pool() -> TestDb {
    let pool = crate::migrate::init_pool_memory().await.expect("pool");
    crate::migrate::run_migrations(&pool).await.expect("migrate");
    crate::settings::ensure_settings(&pool).await.expect("settings");
    crate::auth::set_credentials(&pool, "test", "test")
        .await
        .expect("auth");
    TestDb { pool }
}
