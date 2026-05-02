//! 中文注释:SFID → 链上 `remove_sheng_admin_backup` 推 extrinsic(phase45 mock)。
//!
//! ADR-008:由当前 main 公钥签名授权,从链上 `ShengAdmins[Province][Slot]`
//! storage 注销 backup_1 / backup_2 公钥(Main 槽不允许动)。
//!
//! ## extrinsic 入参(phase7 切真时使用)
//!
//! ```ignore
//! remove_sheng_admin_backup(
//!     province: ProvinceCode,
//!     slot: Slot,            // Backup1 / Backup2
//!     sig_by_main: [u8; 64], // main 私钥对 (province, slot, nonce) 的签名
//! )
//! ```
//!
//! ## phase45 行为
//!
//! 调用 `chain/client.rs::submit_immortal_paysno_mock`,不真实推链。

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
pub(crate) async fn remove_backup(
    province: &str,
    slot: Slot,
) -> Result<MockTxHash, ChainPushError> {
    if matches!(slot, Slot::Main) {
        return Err(ChainPushError::Other(
            "slot=MAIN cannot be modified via remove_backup".to_string(),
        ));
    }
    tracing::info!(
        province = %province,
        slot = slot.as_str(),
        "[chain push] remove_sheng_admin_backup 即将提交"
    );
    submit_immortal_paysno_mock("remove_sheng_admin_backup").await
}

// ─── HTTP handler ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct RemoveBackupInput {
    /// 槽位:"BACKUP_1" / "BACKUP_2"。
    pub(crate) slot: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RemoveBackupOutput {
    pub(crate) ok: bool,
    pub(crate) tx_hash: String,
}

/// `POST /api/v1/admin/sheng-admin/roster/remove-backup`
pub(crate) async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<RemoveBackupInput>,
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
    match remove_backup(province.as_str(), slot).await {
        Ok(tx) => Json(RemoveBackupOutput {
            ok: true,
            tx_hash: tx.hex,
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "remove_sheng_admin_backup mock submit failed");
            crate::api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                1502,
                "chain push mock failed",
            )
        }
    }
}
