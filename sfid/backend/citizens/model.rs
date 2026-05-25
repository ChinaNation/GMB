//! 中文注释:公民身份记录、绑定状态机、绑定/解绑/查询接口 DTO,
//! 含 wuminapp 投票账户对接 + 现场扫码绑定/状态 QR 载荷。

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum CitizenStatus {
    Normal,
    Abnormal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PendingBindScan {
    pub(crate) qr_id: String,
    pub(crate) archive_no: String,
    pub(crate) site_sfid: String,
    pub(crate) status: CitizenStatus,
    pub(crate) expire_at: i64,
    pub(crate) scanned_at: DateTime<Utc>,
}

// ── 公民身份记录（新模型）──────────────────────────────────────────────

/// 公民身份记录。
///
/// 以自增 ID 为主键，account_pubkey / archive_no / sfid_code 各自唯一（非空时）。
/// 四种状态：
/// - Pending：只有 pubkey（用户推送了钱包，未到现场）
/// - Bindable：pubkey + archive_no + 签名通过，待管理员推链
/// - Bound：chain_confirmed = true，链上已确认
/// - Unlinked：解绑后，archive_no + sfid_code 保留，pubkey 已清除
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CitizenRecord {
    pub(crate) id: u64,
    pub(crate) account_pubkey: Option<String>,
    /// SS58 地址（prefix=2027），方便展示和搜索。
    #[serde(default)]
    pub(crate) account_address: Option<String>,
    pub(crate) archive_no: Option<String>,
    pub(crate) sfid_code: Option<String>,
    /// CPMS 档案码原始状态。中文注释：它不是绑定状态，也不是最终身份ID状态。
    pub(crate) archive_status: Option<CitizenStatus>,
    /// 电子护照生效日期，格式固定为 YYYY-MM-DD。
    pub(crate) archive_valid_from: Option<String>,
    /// 电子护照截止日期，格式固定为 YYYY-MM-DD。
    pub(crate) archive_valid_until: Option<String>,
    pub(crate) sfid_signature: Option<String>,
    pub(crate) province_code: Option<String>,
    #[serde(default)]
    pub(crate) city_code: Option<String>,
    /// 链上绑定是否已确认（bind_sfid extrinsic InBestBlock）。
    #[serde(default)]
    pub(crate) chain_confirmed: bool,
    pub(crate) bound_at: Option<DateTime<Utc>>,
    pub(crate) bound_by: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

impl CitizenRecord {
    pub(crate) fn status(&self) -> CitizenBindStatus {
        match (&self.account_pubkey, &self.archive_no, self.chain_confirmed) {
            (Some(_), Some(_), true) => CitizenBindStatus::Bound,
            (Some(_), Some(_), false) => CitizenBindStatus::Bindable,
            (Some(_), None, _) => CitizenBindStatus::Pending,
            (None, Some(_), _) => CitizenBindStatus::Unlinked,
            (None, None, _) => CitizenBindStatus::Pending,
        }
    }

    pub(crate) fn computed_identity_status(&self) -> CitizenStatus {
        self.computed_identity_status_on_date(Utc::now().date_naive())
    }

    pub(crate) fn computed_identity_status_on_date(&self, today: NaiveDate) -> CitizenStatus {
        if self.archive_status != Some(CitizenStatus::Normal) {
            return CitizenStatus::Abnormal;
        }
        let Some(valid_from) = parse_archive_date(self.archive_valid_from.as_deref()) else {
            return CitizenStatus::Abnormal;
        };
        let Some(valid_until) = parse_archive_date(self.archive_valid_until.as_deref()) else {
            return CitizenStatus::Abnormal;
        };
        if valid_from <= today && today <= valid_until {
            CitizenStatus::Normal
        } else {
            CitizenStatus::Abnormal
        }
    }
}

fn parse_archive_date(value: Option<&str>) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value?.trim(), "%Y-%m-%d").ok()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum CitizenBindStatus {
    /// 有 pubkey，无 archive_no（用户推送了钱包，未到现场）。
    Pending,
    /// 有 pubkey + archive_no + 签名通过，待管理员推链。
    Bindable,
    /// chain_confirmed = true，链上已确认。
    Bound,
    /// 解绑后：有 archive_no + sfid_code，无 pubkey。
    Unlinked,
}

/// 绑定 challenge（公钥签名验证）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CitizenBindChallenge {
    pub(crate) challenge_id: String,
    pub(crate) challenge_text: String,
    pub(crate) account_address: String,
    pub(crate) account_pubkey: String,
    pub(crate) expire_at: DateTime<Utc>,
    pub(crate) created_at: DateTime<Utc>,
}

// ── 公民身份绑定接口类型 ──

/// 绑定/解绑 challenge 返回。
#[derive(Serialize)]
pub(crate) struct CitizenBindChallengeOutput {
    pub(crate) challenge_id: String,
    pub(crate) challenge_text: String,
    /// WUMIN_QR_V1 签名请求 JSON（前端直接展示为二维码）。
    pub(crate) sign_request: String,
    pub(crate) expire_at: i64,
}

/// 绑定/解绑 challenge 请求。
#[derive(Deserialize)]
pub(crate) struct CitizenBindChallengeInput {
    /// 用户 SS58 地址；SFID 生成 sign_request 前必须先锁定本次签名账户。
    pub(crate) user_address: String,
}

/// 绑定请求（两种模式）。
#[derive(Deserialize)]
pub(crate) struct CitizenBindInput {
    /// "bind_archive"（全新绑定）或 "bind_pubkey"（重新绑定公钥）
    pub(crate) mode: String,
    /// 用户 SS58 地址（从 WUMIN_QR_V1 二维码获取）
    pub(crate) user_address: String,
    /// 档案码内容（mode=bind_archive 时必填）
    pub(crate) archive_code_payload: Option<String>,
    /// 记录 ID（mode=bind_pubkey 时必填）
    pub(crate) citizen_id: Option<u64>,
    /// challenge ID
    pub(crate) challenge_id: String,
    /// WUMIN_QR_V1 签名结果（hex）
    pub(crate) signature: String,
}

/// 绑定返回。
#[derive(Serialize)]
pub(crate) struct CitizenBindOutput {
    pub(crate) id: u64,
    pub(crate) account_pubkey: Option<String>,
    pub(crate) account_address: Option<String>,
    pub(crate) archive_no: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) province_code: Option<String>,
    pub(crate) city_code: Option<String>,
    pub(crate) status: CitizenBindStatus,
}

/// 解绑请求（需要公钥签名确认）。
#[derive(Deserialize)]
pub(crate) struct CitizenUnbindInput {
    pub(crate) citizen_id: u64,
    pub(crate) challenge_id: String,
    pub(crate) signature: String,
}

#[derive(Deserialize)]
pub(crate) struct CitizensQuery {
    pub(crate) keyword: Option<String>,
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PublicIdentitySearchQuery {
    pub(crate) archive_no: Option<String>,
    pub(crate) identity_code: Option<String>,
    pub(crate) account_pubkey: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct PublicIdentitySearchOutput {
    pub(crate) found: bool,
    pub(crate) archive_no: Option<String>,
    pub(crate) identity_code: Option<String>,
    pub(crate) account_pubkey: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct CitizenRow {
    pub(crate) id: u64,
    pub(crate) account_pubkey: Option<String>,
    pub(crate) account_address: Option<String>,
    pub(crate) archive_no: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) province_code: Option<String>,
    pub(crate) city_code: Option<String>,
    pub(crate) archive_status: Option<CitizenStatus>,
    pub(crate) identity_status: CitizenStatus,
    pub(crate) valid_from: Option<String>,
    pub(crate) valid_until: Option<String>,
    pub(crate) status: CitizenBindStatus,
}

// ── wuminapp 投票账户接口类型 ──

/// wuminapp 推送投票账户请求。
#[derive(Deserialize)]
pub(crate) struct VoteAccountRegisterInput {
    pub(crate) address: String,
    pub(crate) pubkey: String,
    pub(crate) signature: String,
    pub(crate) sign_message: String,
}

/// wuminapp 查询投票账户状态。
#[derive(Deserialize)]
pub(crate) struct VoteAccountStatusQuery {
    pub(crate) address: String,
}

#[derive(Serialize)]
pub(crate) struct VoteAccountStatusOutput {
    pub(crate) status: String,
    pub(crate) address: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) identity_status: Option<CitizenStatus>,
    pub(crate) valid_from: Option<String>,
    pub(crate) valid_until: Option<String>,
}

/// 管理员推链请求（绑定/解绑共用）。
#[derive(Deserialize)]
pub(crate) struct CitizenPushChainInput {
    pub(crate) citizen_id: u64,
}

#[derive(Serialize)]
pub(crate) struct CitizenPushChainOutput {
    pub(crate) tx_hash: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct BindScanInput {
    pub(crate) qr_payload: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct CitizenQrPayload {
    pub(crate) ver: String,
    pub(crate) issuer_id: String,
    pub(crate) site_sfid: String,
    pub(crate) archive_no: String,
    pub(crate) issued_at: i64,
    pub(crate) expire_at: i64,
    pub(crate) qr_id: String,
    pub(crate) sig_alg: String,
    pub(crate) status: CitizenStatus,
    pub(crate) signature: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct CitizenStatusQrPayload {
    pub(crate) ver: String,
    pub(crate) issuer_id: String,
    pub(crate) site_sfid: String,
    pub(crate) archive_no: String,
    pub(crate) status: CitizenStatus,
    pub(crate) issued_at: i64,
    pub(crate) expire_at: i64,
    pub(crate) qr_id: String,
    pub(crate) sig_alg: String,
    pub(crate) signature: String,
}

#[derive(Serialize)]
#[allow(dead_code)]
pub(crate) struct BindScanOutput {
    pub(crate) site_sfid: String,
    pub(crate) archive_no: String,
    pub(crate) qr_id: String,
    pub(crate) status: CitizenStatus,
    pub(crate) issued_at: i64,
    pub(crate) expire_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn vote_account_status_output_keeps_bind_status_and_identity_status_separate() {
        let output = VoteAccountStatusOutput {
            status: "bound".to_string(),
            address: Some("5F-test".to_string()),
            sfid_code: Some("1234567890".to_string()),
            identity_status: Some(CitizenStatus::Normal),
            valid_from: Some("2026-05-24".to_string()),
            valid_until: Some("2036-05-23".to_string()),
        };

        let value = serde_json::to_value(output).expect("serialize status output");
        assert_eq!(value["status"], "bound");
        assert_eq!(value["sfid_code"], "1234567890");
        assert_eq!(value["identity_status"], "NORMAL");
        assert_eq!(value["valid_from"], "2026-05-24");
        assert_eq!(value["valid_until"], "2036-05-23");
        assert!(value.get("status_updated_at").is_none());
    }

    #[test]
    fn citizen_record_computes_identity_status_from_archive_status_and_validity() {
        let record = CitizenRecord {
            id: 1,
            account_pubkey: Some("0xabc".to_string()),
            account_address: Some("5F-test".to_string()),
            archive_no: Some("ARCHIVE-1".to_string()),
            sfid_code: Some("1234567890".to_string()),
            archive_status: Some(CitizenStatus::Normal),
            archive_valid_from: Some("2026-05-24".to_string()),
            archive_valid_until: Some("2036-05-23".to_string()),
            sfid_signature: None,
            province_code: Some("GD".to_string()),
            city_code: Some("4401".to_string()),
            chain_confirmed: true,
            bound_at: None,
            bound_by: None,
            created_at: Utc::now(),
        };

        assert_eq!(
            record.computed_identity_status_on_date(NaiveDate::from_ymd_opt(2026, 5, 24).unwrap()),
            CitizenStatus::Normal
        );
        assert_eq!(
            record.computed_identity_status_on_date(NaiveDate::from_ymd_opt(2036, 5, 23).unwrap()),
            CitizenStatus::Normal
        );
        assert_eq!(
            record.computed_identity_status_on_date(NaiveDate::from_ymd_opt(2036, 5, 24).unwrap()),
            CitizenStatus::Abnormal
        );
    }
}
