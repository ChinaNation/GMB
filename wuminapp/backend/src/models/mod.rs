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
pub struct TxPrepareRequest {
    pub from_address: String,
    pub pubkey_hex: String,
    pub to_address: String,
    pub amount: f64,
    pub symbol: String,
}

#[derive(Serialize)]
pub struct TxPrepareData {
    pub prepared_id: String,
    pub signer_payload_hex: String,
    pub expires_at: i64,
}

#[derive(Deserialize)]
pub struct TxSubmitRequest {
    pub prepared_id: String,
    pub pubkey_hex: String,
    pub signature_hex: String,
}

#[derive(Serialize)]
pub struct TxSubmitData {
    pub tx_hash: Option<String>,
    pub status: String,
    pub failure_reason: Option<String>,
}

#[derive(Serialize)]
pub struct TxStatusData {
    pub tx_hash: String,
    pub status: String,
    pub failure_reason: Option<String>,
    pub updated_at: i64,
}

#[derive(Serialize)]
pub struct WalletBalanceData {
    pub account: String,
    pub balance: f64,
    pub symbol: &'static str,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct SfidPubkeyPushRequest {
    pub pubkey_hex: String,
}

#[derive(Serialize)]
pub struct SfidPubkeyPushData {
    pub accepted: bool,
    pub pushed_at: i64,
}
