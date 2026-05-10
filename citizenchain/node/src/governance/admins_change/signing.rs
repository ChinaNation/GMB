use serde_json::json;

use crate::governance::signing::{self as gov_signing, VoteSignRequestResult, VoteSubmitResult};

use super::call_data::build_admin_set_change_call_data;
use super::subject_id;
use super::types::{qr_org_display_value, AdminSubjectState};
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
    // display.fields 必须和 wumin PayloadDecoder 对 propose_admin_set_change
    // 解出的字段逐项一致：org / subject / new_admins。
    let fields = json!([
        {
            "key": "org",
            "label": "组织类型",
            "value": qr_org_display_value(state.org),
        },
        {
            "key": "subject",
            "label": "管理员主体",
            "value": format!("0x{}", state.subject_id_hex),
        },
        {
            "key": "new_admins",
            "label": "新管理员",
            "value": normalized
                .iter()
                .map(|admin| format!("0x{admin}"))
                .collect::<Vec<_>>()
                .join(","),
        }
    ]);

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
