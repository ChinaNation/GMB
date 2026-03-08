use axum::{
    extract::{ConnectInfo, Request, State},
    http::{header::HeaderName, HeaderMap, HeaderValue, Method, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use redis::Script;
use reqwest::Url;
use serde::Serialize;
use std::{
    net::{IpAddr, SocketAddr},
    sync::OnceLock,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::warn;
use uuid::Uuid;

use crate::key_admins::chain_proof::build_public_key_output;
use crate::*;

static TRUSTED_PROXY_IPS: OnceLock<Vec<IpAddr>> = OnceLock::new();
static RATE_LIMIT_SCRIPT: OnceLock<Script> = OnceLock::new();
const RATE_LIMIT_WINDOW_MS: i64 = 60_000;

fn rate_limit_script() -> &'static Script {
    RATE_LIMIT_SCRIPT.get_or_init(|| {
        Script::new(
            r#"
local now_ms = tonumber(ARGV[1])
local window_ms = tonumber(ARGV[2])
local limit = tonumber(ARGV[3])
local member = ARGV[4]

redis.call('ZREMRANGEBYSCORE', KEYS[1], '-inf', now_ms - window_ms)
local count = redis.call('ZCARD', KEYS[1])
if count >= limit then
  redis.call('PEXPIRE', KEYS[1], window_ms)
  return 0
end
redis.call('ZADD', KEYS[1], now_ms, member)
redis.call('PEXPIRE', KEYS[1], window_ms)
return 1
"#,
        )
    })
}

async fn consume_rate_limit_slot_redis(
    state: &AppState,
    actor: &str,
    limit_per_min: usize,
    now_ms: i64,
) -> Result<bool, String> {
    if limit_per_min == 0 {
        return Ok(false);
    }
    let actor_hash = blake3::hash(actor.as_bytes()).to_hex().to_string();
    let key = format!("sfid:rate_limit:{actor_hash}");
    let member = format!("{now_ms}:{}", Uuid::new_v4().simple());
    let mut conn = state
        .rate_limit_redis
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| format!("redis connection failed: {e}"))?;
    let allowed: i32 = rate_limit_script()
        .key(key)
        .arg(now_ms)
        .arg(RATE_LIMIT_WINDOW_MS)
        .arg(limit_per_min as i64)
        .arg(member)
        .invoke_async(&mut conn)
        .await
        .map_err(|e| format!("redis rate-limit eval failed: {e}"))?;
    Ok(allowed == 1)
}

pub(crate) async fn global_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: middleware::Next,
) -> Response {
    let now = Utc::now();
    let limit_per_min = std::env::var("SFID_RATE_LIMIT_PER_MIN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(120);
    let actor = actor_ip_from_request(&request)
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let now_ms = now.timestamp_millis();
    let allowed =
        match consume_rate_limit_slot_redis(&state, actor.as_str(), limit_per_min, now_ms).await {
            Ok(v) => v,
            Err(err) => {
                warn!(error = %err, actor = %actor, "rate limiter unavailable");
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "rate limiter unavailable",
                );
            }
        };
    if !allowed {
        return api_error(StatusCode::TOO_MANY_REQUESTS, 1029, "rate limit exceeded");
    }

    next.run(request).await
}

pub(crate) fn required_env(key: &str) -> String {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => panic!("{key} is required and must be non-empty"),
    }
}

pub(crate) fn optional_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub(crate) fn runtime_meta_cipher_key() -> String {
    required_env("SFID_RUNTIME_META_KEY")
}

pub(crate) fn build_cors_layer() -> CorsLayer {
    let env_mode = optional_env("SFID_ENV")
        .or_else(|| optional_env("ENV"))
        .unwrap_or_else(|| "dev".to_string())
        .to_ascii_lowercase();
    let is_prod = env_mode == "prod" || env_mode == "production";
    let allow_any_in_prod = env_flag_enabled("SFID_ALLOW_CORS_ANY_IN_PROD");
    let allow_all = std::env::var("SFID_CORS_ALLOWED_ORIGINS")
        .ok()
        .map(|v| v.trim().to_string())
        .is_some_and(|v| v == "*");
    if allow_all {
        if is_prod && !allow_any_in_prod {
            panic!("SFID_CORS_ALLOWED_ORIGINS='*' is forbidden in production");
        }
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(vec![
                HeaderName::from_static("authorization"),
                HeaderName::from_static("content-type"),
                HeaderName::from_static("x-request-id"),
                HeaderName::from_static("x-chain-token"),
                HeaderName::from_static("x-chain-request-id"),
                HeaderName::from_static("x-chain-nonce"),
                HeaderName::from_static("x-chain-timestamp"),
                HeaderName::from_static("x-chain-signature"),
                HeaderName::from_static("x-wallet-pubkey"),
                HeaderName::from_static("x-wallet-signature"),
                HeaderName::from_static("x-wallet-signature-message"),
            ]);
    }

    let configured = std::env::var("SFID_CORS_ALLOWED_ORIGINS")
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
            HeaderValue::from_static("http://127.0.0.1:5179"),
            HeaderValue::from_static("http://localhost:5179"),
            HeaderValue::from_static("http://127.0.0.1:5173"),
            HeaderValue::from_static("http://localhost:5173"),
        ]
    } else {
        configured
    };
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-chain-token"),
            HeaderName::from_static("x-chain-request-id"),
            HeaderName::from_static("x-chain-nonce"),
            HeaderName::from_static("x-chain-timestamp"),
            HeaderName::from_static("x-chain-signature"),
            HeaderName::from_static("x-wallet-pubkey"),
            HeaderName::from_static("x-wallet-signature"),
            HeaderName::from_static("x-wallet-signature-message"),
        ])
}

pub(crate) async fn root() -> impl IntoResponse {
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "sfid backend is running",
    })
}

pub(crate) async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let store = match state.store.read() {
        Ok(guard) => guard,
        Err(err) => {
            warn!("store read failed in /api/v1/health: {}", err);
            return Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: HealthData {
                    service: "sfid-backend",
                    status: "DEGRADED",
                    checked_at: Utc::now().timestamp(),
                },
            });
        }
    };
    let _ = latency_p95_p99_ms(&store.metrics.chain_latency_samples);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: HealthData {
            service: "sfid-backend",
            status: "UP",
            checked_at: Utc::now().timestamp(),
        },
    })
}

pub(crate) async fn attestor_public_key(State(state): State<AppState>) -> impl IntoResponse {
    let public_key_hex = match state.public_key_hex.read() {
        Ok(v) => v.clone(),
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "public key unavailable",
            )
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: build_public_key_output(
            &state.key_id,
            &state.key_version,
            &state.key_alg,
            &public_key_hex,
        ),
    })
    .into_response()
}

pub(crate) fn constant_time_eq(left: &str, right: &str) -> bool {
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

pub(crate) fn require_public_search_auth(
    headers: &HeaderMap,
) -> Result<(), axum::response::Response> {
    let incoming = headers
        .get("x-public-search-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim()
        .to_string();
    if incoming.is_empty() {
        return Err(api_error(
            StatusCode::UNAUTHORIZED,
            1002,
            "public search auth required",
        ));
    }
    let expected = required_env("SFID_PUBLIC_SEARCH_TOKEN");
    if !constant_time_eq(incoming.as_str(), expected.as_str()) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1008,
            "public search auth invalid",
        ));
    }
    Ok(())
}

pub(crate) fn require_chain_auth(headers: &HeaderMap) -> Result<(), axum::response::Response> {
    let incoming = headers
        .get("x-chain-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim()
        .to_string();
    if incoming.is_empty() {
        return Err(api_error(
            StatusCode::UNAUTHORIZED,
            1002,
            "chain auth required",
        ));
    }
    let expected = required_env("SFID_CHAIN_TOKEN");
    if !constant_time_eq(incoming.as_str(), expected.as_str()) {
        return Err(api_error(StatusCode::FORBIDDEN, 1008, "chain auth invalid"));
    }
    Ok(())
}

pub(crate) fn env_flag_enabled(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| {
            let value = v.trim();
            value.eq_ignore_ascii_case("1")
                || value.eq_ignore_ascii_case("true")
                || value.eq_ignore_ascii_case("yes")
                || value.eq_ignore_ascii_case("on")
        })
        .unwrap_or(false)
}

pub(crate) fn parse_csv_env_set(key: &str) -> Vec<String> {
    std::env::var(key)
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|v| v.to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn callback_allowed_hosts() -> Vec<String> {
    parse_csv_env_set("SFID_CALLBACK_ALLOWED_HOSTS")
}

fn trusted_proxy_ips() -> &'static [IpAddr] {
    TRUSTED_PROXY_IPS
        .get_or_init(|| {
            parse_csv_env_set("SFID_TRUST_PROXY_IPS")
                .into_iter()
                .filter_map(|raw| raw.parse::<IpAddr>().ok())
                .collect::<Vec<_>>()
        })
        .as_slice()
}

fn peer_ip_from_request(request: &Request) -> Option<IpAddr> {
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip())
}

fn actor_ip_from_request(request: &Request) -> Option<String> {
    let trusted_ips = trusted_proxy_ips();
    let peer_ip = peer_ip_from_request(request);
    if let Some(peer) = peer_ip {
        if trusted_ips.iter().any(|ip| *ip == peer) {
            return actor_ip_from_headers(request.headers()).or_else(|| Some(peer.to_string()));
        }
        return Some(peer.to_string());
    }
    actor_ip_from_headers(request.headers())
}

pub(crate) fn host_matches_rule(host: &str, rule: &str) -> bool {
    if let Some(suffix) = rule.strip_prefix("*.") {
        return host.ends_with(&format!(".{suffix}"));
    }
    if let Some(suffix) = rule.strip_prefix('.') {
        return host.ends_with(&format!(".{suffix}"));
    }
    host == rule
}

pub(crate) fn is_blocked_callback_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_unspecified()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
                || v6.is_multicast()
        }
    }
}

pub(crate) fn validate_bind_callback_url(url: &str) -> Result<(), String> {
    let parsed = Url::parse(url).map_err(|_| "callback_url is not a valid URL".to_string())?;
    let insecure_http_allowed = env_flag_enabled("SFID_ALLOW_INSECURE_CALLBACK_HTTP");
    match parsed.scheme() {
        "https" => {}
        "http" if insecure_http_allowed => {}
        "http" => {
            return Err(
                "callback_url must use https (set SFID_ALLOW_INSECURE_CALLBACK_HTTP=true only for local dev)"
                    .to_string(),
            )
        }
        _ => return Err("callback_url scheme must be http or https".to_string()),
    }

    let Some(host) = parsed.host_str() else {
        return Err("callback_url host is required".to_string());
    };
    let host_lower = host.to_ascii_lowercase();
    if host_lower == "localhost" || host_lower.ends_with(".localhost") {
        return Err("callback_url localhost is not allowed".to_string());
    }
    if let Ok(ip) = host_lower.parse::<IpAddr>() {
        if is_blocked_callback_ip(ip) {
            return Err("callback_url private/local IP literals are not allowed".to_string());
        }
    }

    let allowlist = callback_allowed_hosts();
    let env_mode = optional_env("SFID_ENV")
        .or_else(|| optional_env("ENV"))
        .unwrap_or_else(|| "dev".to_string())
        .to_ascii_lowercase();
    let is_prod = env_mode == "prod" || env_mode == "production";
    if allowlist.is_empty()
        && is_prod
        && !env_flag_enabled("SFID_ALLOW_OPEN_CALLBACK_TARGETS_IN_PROD")
    {
        return Err(
            "SFID_CALLBACK_ALLOWED_HOSTS is required in production (or set SFID_ALLOW_OPEN_CALLBACK_TARGETS_IN_PROD=true explicitly)"
                .to_string(),
        );
    }
    if !allowlist.is_empty()
        && !allowlist
            .iter()
            .any(|rule| host_matches_rule(host_lower.as_str(), rule.as_str()))
    {
        return Err("callback_url host is not in SFID_CALLBACK_ALLOWED_HOSTS".to_string());
    }

    Ok(())
}

pub(crate) fn chain_header_value(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub(crate) fn chain_request_signing_secret() -> Result<String, axum::response::Response> {
    let secret = std::env::var("SFID_CHAIN_SIGNING_SECRET")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "SFID_CHAIN_SIGNING_SECRET must be configured",
            )
        })?;
    if secret.len() < 32 {
        return Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "SFID_CHAIN_SIGNING_SECRET must be at least 32 chars",
        ));
    }
    Ok(secret)
}

pub(crate) fn chain_signature_payload(
    route_key: &str,
    request_id: &str,
    nonce: &str,
    timestamp: i64,
    fingerprint: &str,
) -> String {
    format!(
        "route={route_key}\nrequest_id={request_id}\nnonce={nonce}\ntimestamp={timestamp}\nfingerprint={fingerprint}"
    )
}

pub(crate) fn chain_signature_hex(secret: &str, payload: &str) -> String {
    let key_digest = blake3::hash(secret.as_bytes());
    let hash = blake3::keyed_hash(key_digest.as_bytes(), payload.as_bytes());
    hex::encode(hash.as_bytes())
}

pub(crate) fn constant_time_eq_hex(a: &str, b: &str) -> bool {
    let left = a.as_bytes();
    let right = b.as_bytes();
    let max_len = left.len().max(right.len());
    let mut diff = left.len() ^ right.len();
    for idx in 0..max_len {
        let lb = left.get(idx).copied().unwrap_or(0);
        let rb = right.get(idx).copied().unwrap_or(0);
        diff |= usize::from(lb ^ rb);
    }
    diff == 0
}

pub(crate) fn require_chain_signature(
    headers: &HeaderMap,
    route_key: &str,
    request_id: &str,
    nonce: &str,
    timestamp: i64,
    fingerprint: &str,
) -> Result<(), axum::response::Response> {
    let secret = chain_request_signing_secret()?;
    let Some(incoming_sig) = chain_header_value(headers, "x-chain-signature") else {
        return Err(api_error(
            StatusCode::UNAUTHORIZED,
            1020,
            "x-chain-signature is required",
        ));
    };
    let payload = chain_signature_payload(route_key, request_id, nonce, timestamp, fingerprint);
    let expected = chain_signature_hex(secret.as_str(), payload.as_str());
    let incoming_norm = incoming_sig.to_ascii_lowercase();
    if !constant_time_eq_hex(incoming_norm.as_str(), expected.as_str()) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1021,
            "chain signature invalid",
        ));
    }
    Ok(())
}

pub(crate) fn request_fingerprint<T: Serialize>(input: &T) -> Result<String, Response> {
    let payload = serde_json::to_vec(input).map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "fingerprint serialization failed",
        )
    })?;
    Ok(hex::encode(blake3::hash(&payload).as_bytes()))
}

pub(crate) fn cleanup_chain_auth_tracking(store: &mut Store, now: DateTime<Utc>) {
    store
        .chain_requests_by_key
        .retain(|_, record| record.received_at > now - Duration::hours(24));
    store
        .chain_nonce_seen
        .retain(|_, seen_at| *seen_at > now - Duration::hours(24));
}

fn chain_auth_cleanup_interval_seconds() -> i64 {
    std::env::var("SFID_CHAIN_AUTH_CLEANUP_INTERVAL_SECONDS")
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(60)
}

pub(crate) fn maybe_cleanup_chain_auth_tracking(store: &mut Store, now: DateTime<Utc>) {
    let interval = Duration::seconds(chain_auth_cleanup_interval_seconds());
    let should_cleanup = store
        .chain_auth_last_cleanup_at
        .map(|last| now - last >= interval)
        .unwrap_or(true);
    if should_cleanup {
        cleanup_chain_auth_tracking(store, now);
        store.chain_auth_last_cleanup_at = Some(now);
    }
}

pub(crate) fn parse_chain_request_auth(
    headers: &HeaderMap,
    route_key: &str,
    fingerprint: &str,
) -> Result<ChainRequestAuth, axum::response::Response> {
    require_chain_auth(headers)?;
    let Some(request_id) = chain_header_value(headers, "x-chain-request-id") else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1011,
            "x-chain-request-id is required",
        ));
    };
    let Some(nonce) = chain_header_value(headers, "x-chain-nonce") else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1012,
            "x-chain-nonce is required",
        ));
    };
    let Some(ts_text) = chain_header_value(headers, "x-chain-timestamp") else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1013,
            "x-chain-timestamp is required",
        ));
    };
    let Ok(ts) = ts_text.parse::<i64>() else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1014,
            "x-chain-timestamp must be unix timestamp seconds",
        ));
    };
    let now = Utc::now();
    if (now.timestamp() - ts).abs() > 300 {
        return Err(api_error(
            StatusCode::UNAUTHORIZED,
            1015,
            "chain request timestamp expired",
        ));
    }
    require_chain_signature(headers, route_key, &request_id, &nonce, ts, fingerprint)?;

    Ok(ChainRequestAuth {
        request_id,
        nonce,
        timestamp: ts,
    })
}

pub(crate) fn track_chain_request(
    store: &mut Store,
    route_key: &str,
    auth: &ChainRequestAuth,
    fingerprint: &str,
) -> Result<(), axum::response::Response> {
    let now = Utc::now();
    maybe_cleanup_chain_auth_tracking(store, now);
    let nonce_key = format!("{route_key}:{}", auth.nonce);
    if store.chain_nonce_seen.contains_key(&nonce_key) {
        store.metrics.chain_replay_rejects += 1;
        store.metrics.chain_request_failed_total += 1;
        return Err(api_error(
            StatusCode::CONFLICT,
            1016,
            "duplicate chain nonce (memory)",
        ));
    }
    let request_key = format!("{route_key}:{}", auth.request_id);
    if let Some(existing) = store.chain_requests_by_key.get(&request_key) {
        store.metrics.chain_replay_rejects += 1;
        store.metrics.chain_request_failed_total += 1;
        if existing.fingerprint == fingerprint {
            return Err(api_error(
                StatusCode::CONFLICT,
                1017,
                "duplicate chain request (memory)",
            ));
        }
        return Err(api_error(
            StatusCode::CONFLICT,
            1018,
            "chain request id conflict (memory)",
        ));
    }

    insert_bounded_map(
        &mut store.chain_nonce_seen,
        nonce_key,
        now,
        bounded_cache_limit("SFID_CHAIN_NONCE_CACHE_MAX", 50_000),
    );
    insert_bounded_map(
        &mut store.chain_requests_by_key,
        request_key,
        ChainRequestReceipt {
            route_key: route_key.to_string(),
            request_id: auth.request_id.clone(),
            nonce: auth.nonce.clone(),
            fingerprint: fingerprint.to_string(),
            received_at: now,
        },
        bounded_cache_limit("SFID_CHAIN_REQUEST_CACHE_MAX", 50_000),
    );
    Ok(())
}

pub(crate) fn rollback_chain_request_tracking(
    store: &mut Store,
    route_key: &str,
    auth: &ChainRequestAuth,
) {
    let nonce_key = format!("{route_key}:{}", auth.nonce);
    store.chain_nonce_seen.remove(&nonce_key);
    let request_key = format!("{route_key}:{}", auth.request_id);
    store.chain_requests_by_key.remove(&request_key);
}

#[cfg(test)]
pub(crate) fn require_chain_request(
    store: &mut Store,
    headers: &HeaderMap,
    route_key: &str,
    fingerprint: &str,
) -> Result<ChainRequestAuth, axum::response::Response> {
    store.metrics.chain_request_total += 1;
    let auth = match parse_chain_request_auth(headers, route_key, fingerprint) {
        Ok(v) => v,
        Err(resp) => {
            store.metrics.chain_auth_failures += 1;
            store.metrics.chain_request_failed_total += 1;
            return Err(resp);
        }
    };
    track_chain_request(store, route_key, &auth, fingerprint)?;
    Ok(auth)
}

pub(crate) fn record_chain_latency(store: &mut Store, started_at: DateTime<Utc>) {
    let elapsed_ms = (Utc::now() - started_at).num_milliseconds().max(0) as u32;
    let samples = &mut store.metrics.chain_latency_samples;
    samples.push(elapsed_ms);
    if samples.len() > 1024 {
        let drop_count = samples.len() - 1024;
        samples.drain(0..drop_count);
    }
}

pub(crate) fn latency_p95_p99_ms(samples: &[u32]) -> (u32, u32) {
    if samples.is_empty() {
        return (0, 0);
    }
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let len = ordered.len();
    let p95 = ordered[((len as f64 * 0.95).ceil() as usize).saturating_sub(1)];
    let p99 = ordered[((len as f64 * 0.99).ceil() as usize).saturating_sub(1)];
    (p95, p99)
}

pub(crate) fn actor_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    let forwarded = chain_header_value(headers, "x-forwarded-for");
    if let Some(ff) = forwarded {
        return ff
            .split(',')
            .map(|v| v.trim())
            .find(|candidate| candidate.parse::<IpAddr>().is_ok())
            .map(|v| v.to_string());
    }
    chain_header_value(headers, "x-real-ip").filter(|candidate| candidate.parse::<IpAddr>().is_ok())
}

pub(crate) fn request_id_from_headers(headers: &HeaderMap) -> Option<String> {
    chain_header_value(headers, "x-chain-request-id")
        .or_else(|| chain_header_value(headers, "x-request-id"))
}
