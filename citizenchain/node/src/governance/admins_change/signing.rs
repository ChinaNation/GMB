use crate::governance::signing::{self as gov_signing, VoteSignRequestResult, VoteSubmitResult};

use super::account_id;
use super::call_data::build_admin_set_change_call_data;
use super::types::AdminAccountState;
use super::validation::validate_admin_set_change;

pub fn build_admin_set_change_sign_request(
    state: &AdminAccountState,
    pubkey_hex: &str,
    admins: &[String],
) -> Result<VoteSignRequestResult, String> {
    let normalized = validate_admin_set_change(state, pubkey_hex, admins)?;
    let pubkey_clean = account_id::normalize_pubkey_hex(pubkey_hex)?;
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;
    let account_id = account_id::account_id_from_hex(&state.account_hex)?;
    let call_data =
        build_admin_set_change_call_data(&state.institution_code, &account_id, &normalized)?;
    gov_signing::build_sign_request_from_call_data(&pubkey_clean, &pubkey_bytes, &call_data)
}

pub fn submit_admin_set_change(
    state: &AdminAccountState,
    request_id: &str,
    expected_pubkey_hex: &str,
    expected_payload_hash: &str,
    admins: &[String],
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: &str,
) -> Result<VoteSubmitResult, String> {
    let normalized = validate_admin_set_change(state, expected_pubkey_hex, admins)?;
    let account_id = account_id::account_id_from_hex(&state.account_hex)?;
    let call_data =
        build_admin_set_change_call_data(&state.institution_code, &account_id, &normalized)?;
    gov_signing::verify_and_submit(
        request_id,
        expected_pubkey_hex,
        expected_payload_hash,
        &call_data,
        sign_nonce,
        sign_block_number,
        response_json,
    )
}
