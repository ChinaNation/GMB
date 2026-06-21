use serde_json::json;

use crate::governance::signing::{self as gov_signing, VoteSignRequestResult, VoteSubmitResult};

use super::account_id;
use super::call_data::build_admin_set_change_call_data;
use super::types::{qr_org_display_value, AdminAccountState};
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
    let call_data = build_admin_set_change_call_data(state.org, &account_id, &normalized)?;
    let summary = format!(
        "{} 管理员更换：{} 人 -> {} 人",
        state.kind_label,
        state.admins.len(),
        normalized.len()
    );
    // display.fields 必须和 wumin PayloadDecoder 对 propose_admin_set_change
    // 解出的字段逐项一致：org / account / admins。
    let fields = json!([
        {
            "key": "org",
            "label": "组织类型",
            "value": qr_org_display_value(state.org),
        },
        {
            "key": "account",
            "label": "管理员账户",
            "value": format!("0x{}", state.account_hex),
        },
        {
            "key": "admins",
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
    let call_data = build_admin_set_change_call_data(state.org, &account_id, &normalized)?;
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
