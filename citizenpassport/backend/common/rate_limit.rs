//! 本机内存限流：用于内网高风险入口的误操作和脚本刷接口防护。

use std::{collections::HashMap, net::SocketAddr};

use axum::{
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use tokio::sync::Mutex;

use super::response::{err, ApiError};
use crate::AppState;

#[derive(Default)]
pub(crate) struct RateLimiter {
    buckets: Mutex<HashMap<String, RateBucket>>,
}

#[derive(Clone)]
struct RateBucket {
    window_start: i64,
    count: u32,
    last_seen: i64,
}

impl RateLimiter {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

pub(crate) async fn check(
    state: &AppState,
    client_addr: SocketAddr,
    headers: &HeaderMap,
    scope: &str,
    max_requests: u32,
    window_seconds: i64,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let now = Utc::now().timestamp();
    let source = source_key(client_addr, headers);
    let key = format!("{scope}:{source}");
    let mut buckets = state.rate_limiter.buckets.lock().await;

    // 中文注释：顺手清理长期未命中的桶，避免单进程内存随误扫 IP 无限增长。
    buckets.retain(|_, bucket| now - bucket.last_seen <= window_seconds * 4);

    let bucket = buckets.entry(key).or_insert(RateBucket {
        window_start: now,
        count: 0,
        last_seen: now,
    });
    if now - bucket.window_start >= window_seconds {
        bucket.window_start = now;
        bucket.count = 0;
    }
    bucket.last_seen = now;
    bucket.count = bucket.count.saturating_add(1);

    if bucket.count > max_requests {
        return Err(err(
            StatusCode::TOO_MANY_REQUESTS,
            1010,
            "too many requests",
        ));
    }
    Ok(())
}

fn source_key(client_addr: SocketAddr, _headers: &HeaderMap) -> String {
    // 中文注释：CPMS 默认无反向代理，直接用 TCP 来源 IP；如以后接入可信反代再统一解析代理头。
    client_addr.ip().to_string()
}
