use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::ApiError;

pub async fn connect(database_url: &str) -> Result<PgPool, ApiError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn health_check(pool: &PgPool) -> Result<(), ApiError> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}
