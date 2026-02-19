mod app_state;
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

    let state = Arc::new(AppState {
        service: "wuminapp-backend",
        version: env!("CARGO_PKG_VERSION"),
    });

    let app = routes::build_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8787));
    info!("wuminapp-backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind server listener");
    axum::serve(listener, app).await.expect("run axum server");
}
