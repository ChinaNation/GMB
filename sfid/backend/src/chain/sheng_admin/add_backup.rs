//! 中文注释:SFID → 链上 `add_sheng_admin_backup` 推 extrinsic(phase45 mock)。
//!
//! ADR-008:由当前 main 公钥签名授权,新增 backup_1 / backup_2 公钥到链上
//! `ShengAdmins[Province][Slot]` storage。
//!
//! ## extrinsic 入参(phase7 切真时使用)
//!
//! ```ignore
//! add_sheng_admin_backup(
//!     province: ProvinceCode,
//!     slot: Slot,            // Backup1 / Backup2
//!     new_pubkey: [u8; 32],
//!     sig_by_main: [u8; 64], // main 私钥对 (province, slot, new_pubkey, nonce) 的签名
//! )
//! ```
//!
//! ## phase45 行为
//!
//! 调用 `chain/client.rs::submit_immortal_paysno_mock` 返回 [`MockTxHash`],
//! 不真实推链。

#![allow(dead_code)]

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::chain::client::{submit_immortal_paysno_mock, ChainPushError, MockTxHash};
use crate::login::require_sheng_admin;
use crate::sfid::province::Slot;
use crate::AppState;

/// 业务层 service:封装"参数校验 + mock 推链"。
///
/// `slot` 必须是 Backup1 / Backup2(Main 槽不允许动)。
/// `new_pubkey` 是新加入的 backup admin 32 字节公钥。
pub(crate) async fn add_backup(
    province: &str,
    slot: Slot,
    new_pubkey: [u8; 32],
) -> Result<MockTxHash, ChainPushError> {
    if matches!(slot, Slot::Main) {
        return Err(ChainPushError::Other(
            "slot=MAIN cannot be modified via add_backup".to_string(),
        ));
    }
    tracing::info!(
        province = %province,
        slot = slot.as_str(),
        new_pubkey_hex = %format!("0x{}", hex::encode(new_pubkey)),
        "[chain push] add_sheng_admin_backup 即将提交"
    );
    submit_immortal_paysno_mock("add_sheng_admin_backup").await
}

// ─── HTTP handler ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct AddBackupInput {
    /// 槽位:"BACKUP_1" / "BACKUP_2"。
    pub(crate) slot: String,
    /// 新 backup 公钥,0x 小写 hex(32 字节)。
    pub(crate) new_pubkey: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AddBackupOutput {
    pub(crate) ok: bool,
    pub(crate) tx_hash: String,
}

/// `POST /api/v1/admin/sheng-admin/roster/add-backup`
///
/// session: 必须是 main 槽对应私钥登录(后续 phase7 真实推链会校验
/// session.admin_pubkey 等于本省链上 main 公钥;phase45 mock 只校验角色)。
pub(crate) async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AddBackupInput>,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let Some(province) = ctx.admin_province.clone() else {
        return crate::api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin province scope missing",
        );
    };
    let slot = match input.slot.as_str() {
        "BACKUP_1" => Slot::Backup1,
        "BACKUP_2" => Slot::Backup2,
        _ => {
            return crate::api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "slot must be BACKUP_1 or BACKUP_2",
            );
        }
    };
    let Some(new_pubkey) = crate::sfid::province::pubkey_from_hex(input.new_pubkey.as_str()) else {
        return crate::api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "new_pubkey must be 0x + 64 hex",
        );
    };
    match add_backup(province.as_str(), slot, new_pubkey).await {
        Ok(tx) => Json(AddBackupOutput {
            ok: true,
            tx_hash: tx.hex,
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "add_sheng_admin_backup mock submit failed");
            crate::api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                1502,
                "chain push mock failed",
            )
        }
    }
}
