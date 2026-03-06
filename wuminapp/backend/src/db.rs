use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::errors::ApiError;

pub async fn connect(database_url: &str) -> Result<PgPool, ApiError> {
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(|_| ApiError::new(5401, "postgres connect failed"))
}

pub async fn migrate(pool: &PgPool) -> Result<(), ApiError> {
    sqlx::migrate!("./db/migrations")
        .run(pool)
        .await
        .map_err(|_| ApiError::new(5402, "postgres migration failed"))
}
