use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};

use crate::{
    app_state::AppState,
    models::{AdminCatalogData, ApiResponse},
    services::admin_catalog_service,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/admins/catalog", get(admin_catalog))
}

async fn admin_catalog(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    match admin_catalog_service::fetch_admin_catalog().await {
        Ok(data) => Json(ApiResponse {
            code: 0,
            message: "ok",
            data,
        }),
        Err(err) => Json(ApiResponse {
            code: err.code,
            message: err.message,
            data: AdminCatalogData {
                source: "chain",
                updated_at: 0,
                institution_count: 0,
                admin_count: 0,
                entries: Vec::new(),
            },
        }),
    }
}
