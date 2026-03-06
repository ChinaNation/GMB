use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub service: &'static str,
    pub version: &'static str,
    pub db: PgPool,
}
