//! 跨模块共享 DTO（≈ 前端 `common/types.ts`）。
//!
//! 脱离 crate 根后，字段对子模块不再默认可见，故统一标 `pub(crate)`。

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct AdminUser {
    pub(crate) user_id: String,
    pub(crate) admin_account: String,
    pub(crate) admin_display_name: String,
    pub(crate) user_group: String,
    pub(crate) immutable: bool,
    pub(crate) managed_key_id: Option<String>,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Archive {
    pub(crate) archive_id: String,
    pub(crate) archive_no: String,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) last_name: String,
    pub(crate) first_name: String,
    pub(crate) birth_date: String,
    pub(crate) gender_code: String,
    pub(crate) height_cm: Option<f32>,
    pub(crate) passport_no: String,
    pub(crate) town_code: String,
    pub(crate) address_unit_id: String,
    pub(crate) address_unit_name_snapshot: String,
    pub(crate) address_detail: String,
    pub(crate) address_full_snapshot: String,
    pub(crate) birth_province_code: String,
    pub(crate) birth_city_code: String,
    pub(crate) birth_town_code: String,
    pub(crate) election_scope_level: String,
    pub(crate) status: String,
    pub(crate) citizen_status: String,
    pub(crate) voting_eligible: bool,
    pub(crate) valid_from: String,
    pub(crate) valid_until: String,
    pub(crate) citizen_status_updated_at: i64,
    pub(crate) wallet_address: Option<String>,
    pub(crate) wallet_pubkey: Option<String>,
    pub(crate) wallet_sig_alg: String,
    pub(crate) wallet_bound_at: Option<i64>,
    pub(crate) wallet_bound_by: Option<String>,
    pub(crate) archive_qr_payload: String,
    pub(crate) deleted_at: Option<i64>,
    pub(crate) deleted_by: Option<String>,
    pub(crate) delete_reason: Option<String>,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}
