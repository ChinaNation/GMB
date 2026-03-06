pub mod admins;
pub mod chain_binding;
pub mod health;
pub mod tx;
pub mod wallet;

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header::HeaderName, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::app_state::AppState;

fn required_env(key: &str) -> String {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => panic!("{key} is required and must be non-empty"),
    }
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    let l = left.as_bytes();
    let r = right.as_bytes();
    let max_len = l.len().max(r.len());
    let mut diff = l.len() ^ r.len();
    for i in 0..max_len {
        let lb = l.get(i).copied().unwrap_or(0);
        let rb = r.get(i).copied().unwrap_or(0);
        diff |= usize::from(lb ^ rb);
    }
    diff == 0
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get("authorization")?.to_str().ok()?.trim();
    let token = auth.strip_prefix("Bearer ")?;
    if token.trim().is_empty() {
        return None;
    }
    Some(token.trim().to_string())
}

fn api_auth_error(message: &'static str) -> axum::response::Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "code": 1002,
            "message": message,
            "data": serde_json::Value::Null
        })),
    )
        .into_response()
}

async fn require_api_auth(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> axum::response::Response {
    let incoming = bearer_token(&headers)
        .or_else(|| {
            headers
                .get("x-api-token")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_default();
    if incoming.is_empty() {
        return api_auth_error("api auth required");
    }
    let expected = required_env("WUMINAPP_API_TOKEN");
    if !constant_time_eq(incoming.as_str(), expected.as_str()) {
        return api_auth_error("api auth invalid");
    }
    next.run(request).await
}

fn build_cors_layer() -> CorsLayer {
    let allow_all = std::env::var("WUMINAPP_CORS_ALLOWED_ORIGINS")
        .ok()
        .map(|v| v.trim().to_string())
        .is_some_and(|v| v == "*");
    if allow_all {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(vec![
                HeaderName::from_static("content-type"),
                HeaderName::from_static("authorization"),
                HeaderName::from_static("x-api-token"),
            ]);
    }

    let configured = std::env::var("WUMINAPP_CORS_ALLOWED_ORIGINS")
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .filter(|v| *v != "*")
                .filter_map(|v| HeaderValue::from_str(v).ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let origins = if configured.is_empty() {
        vec![
            HeaderValue::from_static("http://127.0.0.1:3000"),
            HeaderValue::from_static("http://localhost:3000"),
        ]
    } else {
        configured
    };
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(vec![
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-api-token"),
        ])
}

pub fn build_router(state: Arc<AppState>) -> Router {
    let protected = Router::new()
        .merge(admins::router())
        .merge(chain_binding::router())
        .merge(tx::router())
        .merge(wallet::router())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_auth,
        ));

    Router::new()
        .merge(health::router())
        .merge(protected)
        .layer(build_cors_layer())
        .with_state(state)
}
