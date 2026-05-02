use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};

use crate::*;

pub(crate) async fn admin_cpms_status_scan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CpmsStatusScanInput>,
) -> impl IntoResponse {
    crate::citizens::status::admin_cpms_status_scan(State(state), headers, Json(input)).await
}
