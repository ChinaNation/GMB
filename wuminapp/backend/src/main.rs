use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tracing::info;

#[derive(Clone)]
struct AppState {
    service: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    code: u32,
    message: &'static str,
    data: T,
}

#[derive(Serialize)]
struct HealthData {
    service: &'static str,
    version: &'static str,
    status: &'static str,
}

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

    let app = Router::new()
        .route("/", get(root))
        .route("/api/v1/health", get(health))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8787));
    info!("wuminapp-backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind server listener");
    axum::serve(listener, app).await.expect("run axum server");
}

async fn root() -> impl IntoResponse {
    Json(ApiResponse {
        code: 0,
        message: "ok",
        data: "wuminapp backend is running",
    })
}

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(ApiResponse {
        code: 0,
        message: "ok",
        data: HealthData {
            service: state.service,
            version: state.version,
            status: "UP",
        },
    })
}
