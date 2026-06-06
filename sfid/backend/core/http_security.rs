use axum::{
    extract::{ConnectInfo, Request, State},
    http::{header::HeaderName, HeaderMap, HeaderValue, Method, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use redis::Script;
use std::{
    net::{IpAddr, SocketAddr},
    sync::OnceLock,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::warn;
use uuid::Uuid;

use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};

use crate::*;

type Blake2b256 = Blake2b<U32>;

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
    let actor_hash = hex::encode(Blake2b256::digest(actor.as_bytes()));
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
    let db_ok = state
        .db
        .with_client(|conn| {
            conn.query_one("SELECT 1", &[])
                .map(|_| ())
                .map_err(|e| format!("health query failed: {e}"))
        })
        .is_ok();
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: HealthData {
            service: "sfid-backend",
            status: if db_ok { "UP" } else { "DEGRADED" },
            checked_at: Utc::now().timestamp(),
        },
    })
}

// 中文注释:历史 attestor_public_key endpoint(`GET /api/v1/attestor/public-key`)
// 0 caller,2026-05-01 chain/ 重构一并下架。链端验证 SFID 凭证用的公钥已经
// 通过链上 SfidSystem::SfidMainAccount storage 维护(创世写入 + 链上 rotate
// extrinsic 维护),不需要再走 HTTP pull。

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

#[allow(dead_code)]
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
    let Some(expected) = optional_env("SFID_PUBLIC_SEARCH_TOKEN") else {
        return Err(api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            1004,
            "public search auth not configured",
        ));
    };
    if !constant_time_eq(incoming.as_str(), expected.as_str()) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1008,
            "public search auth invalid",
        ));
    }
    Ok(())
}

// 中文注释:历史 require_chain_auth + 整套 chain HMAC 鉴权(x-chain-token /
// x-chain-request-id / x-chain-nonce / x-chain-timestamp / x-chain-signature)
// 与已下架的 /api/v1/chain/* + /api/v1/vote/verify dead routes 配套使用,
// 2026-05-01 一并下架。chain pull 端点(duoqian_info / joint_vote /
// citizen_vote)的安全模型是"返回签名凭证只对请求者 account_pubkey 有效",
// 不需要请求侧 HMAC。

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

pub(crate) fn chain_header_value(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
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
