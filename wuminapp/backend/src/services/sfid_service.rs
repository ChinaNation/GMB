use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use reqwest::{Client, StatusCode};
use serde_json::Value;

use crate::{errors::ApiError, models::SfidPubkeyPushData};

const DEFAULT_SFID_PUBKEY_PUSH_URL: &str = "http://127.0.0.1:9777/api/v1/auth/wuminapp/pubkey";

pub async fn push_pubkey_to_sfid(pubkey_hex: &str) -> Result<SfidPubkeyPushData, ApiError> {
    let normalized = normalize_pubkey_hex(pubkey_hex)?;
    let push_url =
        env::var("SFID_PUBKEY_PUSH_URL").unwrap_or_else(|_| DEFAULT_SFID_PUBKEY_PUSH_URL.to_string());

    let client = Client::new();
    let response = client
        .post(push_url)
        .json(&serde_json::json!({ "pubkey_hex": normalized }))
        .send()
        .await
        .map_err(|_| ApiError::new(5101, "sfid request failed"))?;

    if response.status() == StatusCode::NOT_FOUND {
        return Err(ApiError::new(5102, "sfid endpoint not found"));
    }
    if !response.status().is_success() {
        return Err(ApiError::new(5101, "sfid http status failed"));
    }

    let payload = response
        .json::<Value>()
        .await
        .map_err(|_| ApiError::new(5103, "sfid response parse failed"))?;

    if payload.get("code").and_then(Value::as_i64).is_some_and(|code| code != 0) {
        return Err(ApiError::new(5104, "sfid rejected pubkey"));
    }

    Ok(SfidPubkeyPushData {
        accepted: true,
        pushed_at: now_secs(),
    })
}

fn normalize_pubkey_hex(input: &str) -> Result<String, ApiError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ApiError::new(1001, "missing pubkey_hex"));
    }
    let body = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if body.len() != 64 || !body.as_bytes().iter().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::new(1001, "invalid pubkey_hex"));
    }
    Ok(format!("0x{}", body.to_lowercase()))
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}
