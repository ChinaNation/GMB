mod app_state;
mod db;
mod errors;
mod models;
mod routes;
mod services;

use crate::app_state::AppState;
use std::{net::SocketAddr, sync::Arc};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();

    match std::env::var("WUMINAPP_API_TOKEN") {
        Ok(v) if !v.trim().is_empty() => {}
        _ => panic!("WUMINAPP_API_TOKEN is required and must be non-empty"),
    }

    let database_url = match std::env::var("WUMINAPP_DATABASE_URL") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => panic!("WUMINAPP_DATABASE_URL is required and must be non-empty"),
    };
    let pool = db::connect(database_url.trim())
        .await
        .expect("connect postgres");
    db::migrate(&pool).await.expect("run postgres migrations");

    let state = Arc::new(AppState {
        service: "wuminapp-backend",
        version: env!("CARGO_PKG_VERSION"),
        db: pool,
    });

    let app = routes::build_router(state);

    let addr = std::env::var("BIND_ADDR")
        .ok()
        .and_then(|v| v.parse::<SocketAddr>().ok())
        .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 8787)));
    info!("wuminapp-backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind server listener");
    axum::serve(listener, app).await.expect("run axum server");
}
