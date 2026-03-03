use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use reqwest::Client;
use serde_json::Value;

use crate::{errors::ApiError, models::ChainBindRequestData};

pub async fn request_chain_bind(account_pubkey: &str) -> Result<ChainBindRequestData, ApiError> {
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
        return Err(ApiError::new(5203, "chain bind gateway http status failed"));
    }

    let payload = response
        .json::<Value>()
        .await
        .map_err(|_| ApiError::new(5204, "chain bind gateway response parse failed"))?;
    if payload.get("code").and_then(Value::as_i64).is_some_and(|v| v != 0) {
        return Err(ApiError::new(5205, "chain bind gateway rejected"));
    }

    Ok(ChainBindRequestData {
        accepted: true,
        requested_at: now_secs(),
    })
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
