use serde_json::json;

use crate::governance::signing::{self as gov_signing, VoteSignRequestResult, VoteSubmitResult};

use super::call_data::build_admin_set_change_call_data;
use super::subject_id;
use super::types::AdminSubjectState;
use super::validation::validate_admin_set_change;

pub fn build_admin_set_change_sign_request(
    state: &AdminSubjectState,
    pubkey_hex: &str,
    new_admins: &[String],
) -> Result<VoteSignRequestResult, String> {
    let normalized = validate_admin_set_change(state, pubkey_hex, new_admins)?;
    let pubkey_clean = subject_id::normalize_pubkey_hex(pubkey_hex)?;
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;
    let subject_id = subject_id::subject_id_from_hex(&state.subject_id_hex)?;
    let call_data = build_admin_set_change_call_data(state.org, &subject_id, &normalized)?;
    let summary = format!(
        "{} 管理员更换：{} 人 -> {} 人",
        state.kind_label,
        state.admins.len(),
        normalized.len()
    );
    let fields = json!({
        "org": state.org_label,
        "subjectId": format!("0x{}", state.subject_id_hex),
        "oldAdminCount": state.admins.len(),
        "newAdminCount": normalized.len(),
        "threshold": state.threshold,
        "newAdmins": normalized.iter().map(|item| format!("0x{item}")).collect::<Vec<_>>(),
    });

    gov_signing::build_sign_request_from_call_data(
        &pubkey_clean,
        &pubkey_bytes,
        &call_data,
        "propose_admin_set_change",
        &summary,
        &fields,
    )
}

pub fn submit_admin_set_change(
    state: &AdminSubjectState,
    request_id: &str,
    expected_pubkey_hex: &str,
    expected_payload_hash: &str,
    new_admins: &[String],
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: &str,
) -> Result<VoteSubmitResult, String> {
    let normalized = validate_admin_set_change(state, expected_pubkey_hex, new_admins)?;
    let subject_id = subject_id::subject_id_from_hex(&state.subject_id_hex)?;
    let call_data = build_admin_set_change_call_data(state.org, &subject_id, &normalized)?;
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
