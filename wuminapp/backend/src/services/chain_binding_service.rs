use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use reqwest::Client;
use serde_json::Value;
use sqlx::PgPool;

use crate::{errors::ApiError, models::ChainBindRequestData};

pub async fn request_chain_bind(
    db: &PgPool,
    account_pubkey: &str,
) -> Result<ChainBindRequestData, ApiError> {
    let normalized = normalize_pubkey_hex(account_pubkey)?;
    let gateway_url = env::var("WUMINAPP_CHAIN_BIND_REQUEST_URL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or(ApiError::new(
            5201,
            "chain bind gateway url missing (WUMINAPP_CHAIN_BIND_REQUEST_URL)",
        ))?;

    let client = Client::new();
    let mut request = client
        .post(gateway_url)
        .json(&serde_json::json!({ "account_pubkey": normalized }));

    if let Some(token) = env::var("WUMINAPP_CHAIN_GATEWAY_TOKEN")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
    {
        request = request.bearer_auth(token);
    }

    let response = request
        .send()
        .await
        .map_err(|_| ApiError::new(5202, "chain bind gateway request failed"))?;
    if !response.status().is_success() {
        persist_chain_bind_request(db, &normalized, false, Some(5203)).await;
        return Err(ApiError::new(5203, "chain bind gateway http status failed"));
    }

    let payload = response
        .json::<Value>()
        .await
        .map_err(|_| ApiError::new(5204, "chain bind gateway response parse failed"))?;
    if payload
        .get("code")
        .and_then(Value::as_i64)
        .is_some_and(|v| v != 0)
    {
        persist_chain_bind_request(db, &normalized, false, Some(5205)).await;
        return Err(ApiError::new(5205, "chain bind gateway rejected"));
    }

    persist_chain_bind_request(db, &normalized, true, Some(0)).await;
    Ok(ChainBindRequestData {
        accepted: true,
        requested_at: now_secs(),
    })
}

async fn persist_chain_bind_request(
    db: &PgPool,
    account_pubkey: &str,
    accepted: bool,
    result_code: Option<i32>,
) {
    let now = now_secs();
    let _ = sqlx::query(
        "INSERT INTO chain_bind_requests \
         (account_pubkey, accepted, result_code, requested_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(account_pubkey)
    .bind(accepted)
    .bind(result_code)
    .bind(now)
    .bind(now)
    .execute(db)
    .await;
}

fn normalize_pubkey_hex(input: &str) -> Result<String, ApiError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ApiError::new(1001, "missing account_pubkey"));
    }
    let body = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if body.len() != 64 || !body.as_bytes().iter().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::new(1001, "invalid account_pubkey"));
    }
    Ok(format!("0x{}", body.to_lowercase()))
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}
