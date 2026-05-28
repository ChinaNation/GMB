//! 中文注释:公民电子护照记录、绑定状态机、查询接口 DTO,
//! 含 CPMS 档案码验真结果和 wuminapp 扫码签名绑定结果。

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum CitizenStatus {
    Normal,
    Abnormal,
}

// ── 公民身份记录（新模型）──────────────────────────────────────────────

/// 公民电子护照记录。
///
/// 以自增 ID 为主键，wallet_pubkey / archive_no / sfid_code 各自唯一（非空时）。
/// 绑定完成的判定只看 SFID 本地记录是否同时拥有档案、钱包和身份 ID。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CitizenRecord {
    pub(crate) id: u64,
    pub(crate) wallet_pubkey: Option<String>,
    /// SS58 地址（prefix=2027），方便展示和搜索。
    #[serde(default)]
    pub(crate) wallet_address: Option<String>,
    pub(crate) archive_no: Option<String>,
    pub(crate) sfid_code: Option<String>,
    /// CPMS 档案码原始状态。中文注释：它不是绑定状态，也不是最终身份ID状态。
    pub(crate) archive_status: Option<CitizenStatus>,
    /// 电子护照生效日期，格式固定为 YYYY-MM-DD。
    pub(crate) archive_valid_from: Option<String>,
    /// 电子护照截止日期，格式固定为 YYYY-MM-DD。
    pub(crate) archive_valid_until: Option<String>,
    /// CPMS 档案状态更新时间，和 ARCHIVE 签名原文一致。
    #[serde(default)]
    pub(crate) status_updated_at: Option<i64>,
    pub(crate) sfid_signature: Option<String>,
    pub(crate) province_code: Option<String>,
    #[serde(default)]
    pub(crate) city_code: Option<String>,
    pub(crate) bound_at: Option<DateTime<Utc>>,
    pub(crate) bound_by: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

impl CitizenRecord {
    pub(crate) fn bind_status(&self) -> CitizenBindStatus {
        match (&self.wallet_pubkey, &self.archive_no, &self.sfid_code) {
            (Some(_), Some(_), Some(_)) => CitizenBindStatus::Bound,
            _ => CitizenBindStatus::Pending,
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
    /// 本地还没有完整电子护照绑定结果。
    Pending,
    /// SFID 已完成档案、钱包、身份 ID 三者绑定。
    Bound,
}

/// 绑定 challenge（wumin 签名验证）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CitizenBindChallenge {
    pub(crate) challenge_id: String,
    pub(crate) challenge_text: String,
    pub(crate) mode: String,
    pub(crate) citizen_id: Option<u64>,
    pub(crate) archive_no: String,
    pub(crate) wallet_address: String,
    pub(crate) wallet_pubkey: String,
    pub(crate) wallet_sig_alg: String,
    pub(crate) archive_status: CitizenStatus,
    pub(crate) archive_valid_from: String,
    pub(crate) archive_valid_until: String,
    pub(crate) status_updated_at: i64,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) expire_at: DateTime<Utc>,
    pub(crate) created_at: DateTime<Utc>,
}

// ── 公民电子护照绑定接口类型 ──

/// 绑定 challenge 返回。
#[derive(Serialize)]
pub(crate) struct CitizenBindChallengeOutput {
    pub(crate) challenge_id: String,
    pub(crate) challenge_text: String,
    pub(crate) mode: String,
    pub(crate) archive_no: String,
    pub(crate) wallet_address: String,
    pub(crate) wallet_pubkey: String,
    pub(crate) archive_status: CitizenStatus,
    pub(crate) valid_from: String,
    pub(crate) valid_until: String,
    pub(crate) status_updated_at: i64,
    /// WUMIN_QR_V1 签名请求 JSON（前端直接展示为二维码）。
    pub(crate) sign_request: String,
    pub(crate) expire_at: i64,
}

/// 绑定 challenge 请求。
#[derive(Deserialize)]
pub(crate) struct CitizenBindChallengeInput {
    /// create=新增身份ID；replace=只更换既有身份ID的钱包绑定，档案号和身份ID不可改变。
    pub(crate) mode: String,
    /// CPMS 出具的 SFID_CPMS_V1 / ARCHIVE 档案码 JSON 原文。
    pub(crate) archive_code_payload: String,
    /// replace 模式必填。
    pub(crate) citizen_id: Option<u64>,
}

/// 绑定请求。
#[derive(Deserialize)]
pub(crate) struct CitizenBindInput {
    /// challenge ID
    pub(crate) challenge_id: String,
    /// WUMIN_QR_V1 sign_response.pubkey。
    pub(crate) pubkey: String,
    /// WUMIN_QR_V1 sign_response.signature。
    pub(crate) signature: String,
    /// WUMIN_QR_V1 sign_response.payload_hash。
    pub(crate) payload_hash: String,
}

/// 绑定返回。
#[derive(Serialize)]
pub(crate) struct CitizenBindOutput {
    pub(crate) id: u64,
    pub(crate) wallet_pubkey: Option<String>,
    pub(crate) wallet_address: Option<String>,
    pub(crate) archive_no: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) archive_status: Option<CitizenStatus>,
    pub(crate) identity_status: CitizenStatus,
    pub(crate) valid_from: Option<String>,
    pub(crate) valid_until: Option<String>,
    pub(crate) status_updated_at: Option<i64>,
    pub(crate) province_code: Option<String>,
    pub(crate) city_code: Option<String>,
    pub(crate) bind_status: CitizenBindStatus,
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
    pub(crate) wallet_pubkey: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct PublicIdentitySearchOutput {
    pub(crate) found: bool,
    pub(crate) archive_no: Option<String>,
    pub(crate) identity_code: Option<String>,
    pub(crate) wallet_pubkey: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct CitizenRow {
    pub(crate) id: u64,
    pub(crate) wallet_pubkey: Option<String>,
    pub(crate) wallet_address: Option<String>,
    pub(crate) archive_no: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) province_code: Option<String>,
    pub(crate) city_code: Option<String>,
    pub(crate) archive_status: Option<CitizenStatus>,
    pub(crate) identity_status: CitizenStatus,
    pub(crate) valid_from: Option<String>,
    pub(crate) valid_until: Option<String>,
    pub(crate) status_updated_at: Option<i64>,
    pub(crate) bind_status: CitizenBindStatus,
}

// ── wuminapp 电子护照状态接口类型 ──

/// wuminapp 查询电子护照状态。
#[derive(Deserialize)]
pub(crate) struct MyIdStatusQuery {
    pub(crate) wallet_address: String,
}

#[derive(Serialize)]
pub(crate) struct MyIdStatusOutput {
    pub(crate) bind_status: String,
    pub(crate) wallet_address: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) identity_status: Option<CitizenStatus>,
    pub(crate) valid_from: Option<String>,
    pub(crate) valid_until: Option<String>,
    pub(crate) status_updated_at: Option<i64>,
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
    fn myid_status_output_keeps_bind_status_and_identity_status_separate() {
        let output = MyIdStatusOutput {
            bind_status: "bound".to_string(),
            wallet_address: Some("5F-test".to_string()),
            sfid_code: Some("1234567890".to_string()),
            identity_status: Some(CitizenStatus::Normal),
            valid_from: Some("2026-05-24".to_string()),
            valid_until: Some("2036-05-23".to_string()),
            status_updated_at: Some(1_779_580_800),
        };

        let value = serde_json::to_value(output).expect("serialize status output");
        assert_eq!(value["bind_status"], "bound");
        assert_eq!(value["sfid_code"], "1234567890");
        assert_eq!(value["identity_status"], "NORMAL");
        assert_eq!(value["valid_from"], "2026-05-24");
        assert_eq!(value["valid_until"], "2036-05-23");
        assert_eq!(value["status_updated_at"], 1_779_580_800);
    }

    #[test]
    fn citizen_record_computes_identity_status_from_archive_status_and_validity() {
        let record = CitizenRecord {
            id: 1,
            wallet_pubkey: Some("0xabc".to_string()),
            wallet_address: Some("5F-test".to_string()),
            archive_no: Some("ARCHIVE-1".to_string()),
            sfid_code: Some("1234567890".to_string()),
            archive_status: Some(CitizenStatus::Normal),
            archive_valid_from: Some("2026-05-24".to_string()),
            archive_valid_until: Some("2036-05-23".to_string()),
            status_updated_at: Some(1_779_580_800),
            sfid_signature: None,
            province_code: Some("GD".to_string()),
            city_code: Some("4401".to_string()),
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
