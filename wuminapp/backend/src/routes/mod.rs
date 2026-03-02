pub mod health;
pub mod tx;
pub mod wallet;

use std::sync::Arc;

use axum::Router;

use crate::app_state::AppState;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(health::router())
        .merge(tx::router())
        .merge(wallet::router())
        .with_state(state)
}
