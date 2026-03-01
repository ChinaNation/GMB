use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: u32,
    pub message: &'static str,
    pub data: T,
}

#[derive(Serialize)]
pub struct HealthData {
    pub service: &'static str,
    pub version: &'static str,
    pub status: &'static str,
}

#[derive(Deserialize)]
pub struct TxSubmitRequest {
    pub from_address: String,
    pub pubkey_hex: String,
    pub to_address: String,
    pub amount: f64,
    pub symbol: String,
    pub nonce: String,
    pub signed_at: i64,
    pub sign_message: String,
    pub signature_hex: String,
}

#[derive(Serialize)]
pub struct TxSubmitData {
    pub tx_hash: Option<String>,
    pub status: &'static str,
    pub failure_reason: Option<&'static str>,
}

#[derive(Serialize)]
pub struct TxStatusData {
    pub tx_hash: String,
    pub status: &'static str,
    pub failure_reason: Option<&'static str>,
    pub updated_at: i64,
}
