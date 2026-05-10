use serde_json::Value;
use std::time::Duration;

use crate::shared::{constants::RPC_RESPONSE_LIMIT_SMALL, rpc};

use super::codec;
use super::subject_id;
use super::types::{kind_label, org_label, status_label, AdminSubjectState};
use crate::governance::storage_keys;

const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(
        method,
        params,
        RPC_REQUEST_TIMEOUT,
        RPC_RESPONSE_LIMIT_SMALL,
    )
}

/// 构造 `AdminsChange::Subjects` 的 StorageMap key。
pub fn admin_subjects_key(subject_id: &[u8; 48]) -> String {
    let pallet_hash = storage_keys::twox_128(b"AdminsChange");
    let storage_hash = storage_keys::twox_128(b"Subjects");
    let blake2_hash = storage_keys::blake2b_128(subject_id);

    let mut key = Vec::with_capacity(16 + 16 + 16 + 48);
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    key.extend_from_slice(&blake2_hash);
    key.extend_from_slice(subject_id);
    format!("0x{}", hex::encode(key))
}

pub fn fetch_admin_subject_by_sfid_number(
    sfid_number: &str,
) -> Result<Option<AdminSubjectState>, String> {
    let subject_id = subject_id::subject_id_from_builtin_sfid(sfid_number)?;
    fetch_admin_subject(&subject_id, Some(sfid_number.to_string()))
}

pub fn fetch_admin_subject(
    subject_id: &[u8; 48],
    sfid_number: Option<String>,
) -> Result<Option<AdminSubjectState>, String> {
    let storage_key = admin_subjects_key(subject_id);
    let result = rpc_post(
        "state_getStorage",
        Value::Array(vec![Value::String(storage_key)]),
    )?;
    let Value::String(hex_data) = result else {
        if result.is_null() {
            return Ok(None);
        }
        return Err("state_getStorage 返回格式无效".to_string());
    };

    let data = decode_hex_storage(&hex_data)?;
    let decoded = codec::decode_admin_subject(&data)?;
    Ok(Some(AdminSubjectState {
        subject_id_hex: hex::encode(subject_id),
        sfid_number,
        org: decoded.org,
        org_label: org_label(decoded.org).to_string(),
        kind: decoded.kind,
        kind_label: kind_label(decoded.kind).to_string(),
        admins: decoded.admins,
        threshold: decoded.threshold,
        creator_hex: decoded.creator_hex,
        created_at: decoded.created_at,
        updated_at: decoded.updated_at,
        status: decoded.status,
        status_label: status_label(decoded.status).to_string(),
    }))
}

pub fn fetch_admins_by_sfid_number(sfid_number: &str) -> Result<Vec<String>, String> {
    Ok(fetch_admin_subject_by_sfid_number(sfid_number)?
        .map(|state| state.admins)
        .unwrap_or_default())
}

fn decode_hex_storage(hex_str: &str) -> Result<Vec<u8>, String> {
    let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(clean).map_err(|e| format!("hex 解码失败: {e}"))
}
