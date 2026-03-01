use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    errors::ApiError,
    models::{TxStatusData, TxSubmitData, TxSubmitRequest},
};

const STATUS_PENDING: &str = "pending";
const STATUS_CONFIRMED: &str = "confirmed";
const STATUS_FAILED: &str = "failed";
const FAIL_REASON_SIMULATED: &str = "simulated chain rejection";
const FAIL_REASON_NOT_FOUND: &str = "tx not found";
const PENDING_CONFIRM_SECS: i64 = 8;
const PENDING_FAIL_SECS: i64 = 4;

#[derive(Clone, Copy)]
struct TxRuntimeState {
    created_at: i64,
    fail_after_pending: bool,
}

static TX_STATE: OnceLock<Mutex<HashMap<String, TxRuntimeState>>> = OnceLock::new();

fn tx_state() -> &'static Mutex<HashMap<String, TxRuntimeState>> {
    TX_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn submit_tx(req: TxSubmitRequest) -> Result<TxSubmitData, ApiError> {
    validate_submit_request(&req)?;
    let tx_hash = generate_tx_hash(&req);

    let state = TxRuntimeState {
        created_at: now_secs(),
        fail_after_pending: should_fail_later(&req),
    };
    if let Ok(mut map) = tx_state().lock() {
        map.insert(tx_hash.clone(), state);
    }

    Ok(TxSubmitData {
        tx_hash: Some(tx_hash),
        status: STATUS_PENDING,
        failure_reason: None,
    })
}

pub fn get_tx_status(tx_hash: &str) -> Result<TxStatusData, ApiError> {
    let state = match tx_state().lock() {
        Ok(map) => map.get(tx_hash).copied(),
        Err(_) => None,
    };

    let Some(state) = state else {
        return Err(ApiError::new(3004, FAIL_REASON_NOT_FOUND));
    };

    let now = now_secs();
    let elapsed = now.saturating_sub(state.created_at);

    if state.fail_after_pending {
        if elapsed < PENDING_FAIL_SECS {
            return Ok(TxStatusData {
                tx_hash: tx_hash.to_string(),
                status: STATUS_PENDING,
                failure_reason: None,
                updated_at: now,
            });
        }
        return Ok(TxStatusData {
            tx_hash: tx_hash.to_string(),
            status: STATUS_FAILED,
            failure_reason: Some(FAIL_REASON_SIMULATED),
            updated_at: now,
        });
    }

    if elapsed < PENDING_CONFIRM_SECS {
        return Ok(TxStatusData {
            tx_hash: tx_hash.to_string(),
            status: STATUS_PENDING,
            failure_reason: None,
            updated_at: now,
        });
    }

    Ok(TxStatusData {
        tx_hash: tx_hash.to_string(),
        status: STATUS_CONFIRMED,
        failure_reason: None,
        updated_at: now,
    })
}

fn validate_submit_request(req: &TxSubmitRequest) -> Result<(), ApiError> {
    if req.from_address.trim().is_empty()
        || req.to_address.trim().is_empty()
        || req.symbol.trim().is_empty()
        || req.nonce.trim().is_empty()
    {
        return Err(ApiError::new(1001, "missing required tx fields"));
    }

    if !req.pubkey_hex.starts_with("0x")
        || !req.signature_hex.starts_with("0x")
        || req.sign_message.trim().is_empty()
    {
        return Err(ApiError::new(1001, "invalid signature payload"));
    }

    if req.amount <= 0.0 || !req.amount.is_finite() {
        return Err(ApiError::new(1001, "invalid amount"));
    }

    if req.signed_at <= 0 {
        return Err(ApiError::new(1001, "invalid signed_at"));
    }

    Ok(())
}

fn generate_tx_hash(req: &TxSubmitRequest) -> String {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();

    let mut hasher = DefaultHasher::new();
    req.from_address.hash(&mut hasher);
    req.to_address.hash(&mut hasher);
    req.symbol.hash(&mut hasher);
    req.nonce.hash(&mut hasher);
    req.sign_message.hash(&mut hasher);
    req.signature_hex.hash(&mut hasher);
    req.signed_at.hash(&mut hasher);
    req.amount.to_bits().hash(&mut hasher);
    let digest = hasher.finish();

    format!("0x{now_nanos:x}{digest:x}")
}

fn should_fail_later(req: &TxSubmitRequest) -> bool {
    req.to_address.to_ascii_lowercase().contains("fail")
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}
