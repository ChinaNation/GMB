//! 中文注释:公民电子护照记录、绑定状态机、查询接口 DTO。

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum CitizenStatus {
    Normal,
    Revoked,
}

// ── 公民身份记录（新模型）──────────────────────────────────────────────

/// 公民电子护照记录。
///
/// 以自增 ID 为主键，wallet_pubkey / cid_number 各自唯一（非空时）。
/// 绑定完成的判定只看 CID 本地记录是否同时拥有钱包和身份 ID。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CitizenRecord {
    pub(crate) id: u64,
    pub(crate) wallet_pubkey: Option<String>,
    /// SS58 地址（prefix=2027），方便展示和搜索。
    #[serde(default)]
    pub(crate) wallet_address: Option<String>,
    pub(crate) cid_number: Option<String>,
    /// 公民状态。中文注释：它不是绑定状态，也不是最终身份ID状态。
    pub(crate) citizen_status: Option<CitizenStatus>,
    /// 选举资格。
    #[serde(default)]
    pub(crate) voting_eligible: bool,
    /// 护照有效期生效日期，格式固定为 YYYY-MM-DD。
    pub(crate) archive_valid_from: Option<String>,
    /// 护照有效期截止日期，格式固定为 YYYY-MM-DD。
    pub(crate) archive_valid_until: Option<String>,
    /// 公民状态更新时间。
    #[serde(default)]
    pub(crate) status_updated_at: Option<i64>,
    pub(crate) province_code: Option<String>,
    #[serde(default)]
    pub(crate) city_code: Option<String>,
    #[serde(default)]
    pub(crate) residence_province_code: Option<String>,
    #[serde(default)]
    pub(crate) residence_city_code: Option<String>,
    #[serde(default)]
    pub(crate) residence_town_code: Option<String>,
    #[serde(default)]
    pub(crate) birth_province_code: Option<String>,
    #[serde(default)]
    pub(crate) birth_city_code: Option<String>,
    #[serde(default)]
    pub(crate) birth_town_code: Option<String>,
    #[serde(default)]
    pub(crate) election_scope_level: Option<String>,
    pub(crate) bound_at: Option<DateTime<Utc>>,
    pub(crate) bound_by: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

impl CitizenRecord {
    pub(crate) fn bind_status(&self) -> CitizenBindStatus {
        // 绑定完成=同时有钱包与身份ID(护照身份即cid_number)。
        match (&self.wallet_pubkey, &self.cid_number) {
            (Some(_), Some(_)) => CitizenBindStatus::Bound,
            _ => CitizenBindStatus::Pending,
        }
    }

    pub(crate) fn computed_identity_status(&self) -> CitizenStatus {
        self.computed_identity_status_on_date(Utc::now().date_naive())
    }

    pub(crate) fn computed_identity_status_on_date(&self, today: NaiveDate) -> CitizenStatus {
        if self.citizen_status != Some(CitizenStatus::Normal) {
            return CitizenStatus::Revoked;
        }
        let Some(valid_from) = parse_archive_date(self.archive_valid_from.as_deref()) else {
            return CitizenStatus::Revoked;
        };
        let Some(valid_until) = parse_archive_date(self.archive_valid_until.as_deref()) else {
            return CitizenStatus::Revoked;
        };
        if valid_from <= today && today <= valid_until {
            CitizenStatus::Normal
        } else {
            CitizenStatus::Revoked
        }
    }

    pub(crate) fn computed_vote_status(&self) -> CitizenStatus {
        if self.voting_eligible && self.citizen_status == Some(CitizenStatus::Normal) {
            CitizenStatus::Normal
        } else {
            CitizenStatus::Revoked
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
    /// CID 已完成钱包、身份 ID 两者绑定。
    Bound,
}

#[derive(Deserialize)]
pub(crate) struct CitizensQuery {
    pub(crate) keyword: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) page_size: Option<usize>,
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PublicIdentitySearchQuery {
    pub(crate) identity_code: Option<String>,
    pub(crate) wallet_pubkey: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct PublicIdentitySearchOutput {
    pub(crate) found: bool,
    pub(crate) identity_code: Option<String>,
    pub(crate) wallet_pubkey: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct CitizenRow {
    pub(crate) id: u64,
    pub(crate) wallet_pubkey: Option<String>,
    pub(crate) wallet_address: Option<String>,
    pub(crate) cid_number: Option<String>,
    pub(crate) citizen_status: Option<CitizenStatus>,
    pub(crate) voting_eligible: bool,
    pub(crate) vote_status: CitizenStatus,
    pub(crate) identity_status: CitizenStatus,
    pub(crate) valid_from: Option<String>,
    pub(crate) valid_until: Option<String>,
    pub(crate) status_updated_at: Option<i64>,
    pub(crate) residence_province_code: Option<String>,
    pub(crate) residence_city_code: Option<String>,
    pub(crate) residence_town_code: Option<String>,
    pub(crate) residence_province_name: Option<String>,
    pub(crate) residence_city_name: Option<String>,
    pub(crate) residence_town_name: Option<String>,
    pub(crate) birth_province_code: Option<String>,
    pub(crate) birth_city_code: Option<String>,
    pub(crate) birth_town_code: Option<String>,
    pub(crate) birth_province_name: Option<String>,
    pub(crate) birth_city_name: Option<String>,
    pub(crate) birth_town_name: Option<String>,
    pub(crate) election_scope_level: Option<String>,
    pub(crate) bind_status: CitizenBindStatus,
}

// ── CitizenApp 电子护照状态接口类型 ──

/// CitizenApp 查询电子护照状态。
#[derive(Deserialize)]
pub(crate) struct MyIdStatusQuery {
    pub(crate) wallet_address: String,
}

#[derive(Serialize)]
pub(crate) struct MyIdStatusOutput {
    pub(crate) bind_status: String,
    pub(crate) wallet_address: Option<String>,
    pub(crate) cid_number: Option<String>,
    pub(crate) citizen_status: Option<CitizenStatus>,
    pub(crate) voting_eligible: Option<bool>,
    pub(crate) vote_status: Option<CitizenStatus>,
    pub(crate) identity_status: Option<CitizenStatus>,
    pub(crate) valid_from: Option<String>,
    pub(crate) valid_until: Option<String>,
    pub(crate) status_updated_at: Option<i64>,
    pub(crate) residence_province_code: Option<String>,
    pub(crate) residence_city_code: Option<String>,
    pub(crate) residence_town_code: Option<String>,
    pub(crate) birth_province_code: Option<String>,
    pub(crate) birth_city_code: Option<String>,
    pub(crate) birth_town_code: Option<String>,
    pub(crate) election_scope_level: Option<String>,
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
            cid_number: Some("1234567890".to_string()),
            citizen_status: Some(CitizenStatus::Normal),
            voting_eligible: Some(true),
            vote_status: Some(CitizenStatus::Normal),
            identity_status: Some(CitizenStatus::Normal),
            valid_from: Some("2026-05-24".to_string()),
            valid_until: Some("2036-05-23".to_string()),
            status_updated_at: Some(1_779_580_800),
            residence_province_code: Some("GD".to_string()),
            residence_city_code: Some("001".to_string()),
            residence_town_code: None,
            birth_province_code: Some("GD".to_string()),
            birth_city_code: Some("001".to_string()),
            birth_town_code: None,
            election_scope_level: Some("CITY".to_string()),
        };

        let value = serde_json::to_value(output).expect("serialize status output");
        assert_eq!(value["bind_status"], "bound");
        assert_eq!(value["cid_number"], "1234567890");
        assert_eq!(value["citizen_status"], "NORMAL");
        assert_eq!(value["voting_eligible"], true);
        assert_eq!(value["vote_status"], "NORMAL");
        assert_eq!(value["identity_status"], "NORMAL");
        assert_eq!(value["valid_from"], "2026-05-24");
        assert_eq!(value["valid_until"], "2036-05-23");
        assert_eq!(value["status_updated_at"], 1_779_580_800);
        assert_eq!(value["residence_province_code"], "GD");
        assert_eq!(value["birth_city_code"], "001");
        assert_eq!(value["election_scope_level"], "CITY");
    }

    #[test]
    fn citizen_record_computes_identity_status_from_citizen_status_and_validity() {
        let record = CitizenRecord {
            id: 1,
            wallet_pubkey: Some("0xabc".to_string()),
            wallet_address: Some("5F-test".to_string()),
            cid_number: Some("1234567890".to_string()),
            citizen_status: Some(CitizenStatus::Normal),
            voting_eligible: true,
            archive_valid_from: Some("2026-05-24".to_string()),
            archive_valid_until: Some("2036-05-23".to_string()),
            status_updated_at: Some(1_779_580_800),
            province_code: Some("GD".to_string()),
            city_code: Some("4401".to_string()),
            residence_province_code: Some("GD".to_string()),
            residence_city_code: Some("4401".to_string()),
            residence_town_code: Some("HD".to_string()),
            birth_province_code: Some("GD".to_string()),
            birth_city_code: Some("4401".to_string()),
            birth_town_code: Some("HD".to_string()),
            election_scope_level: Some("TOWN".to_string()),
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
            CitizenStatus::Revoked
        );
        assert_eq!(record.computed_vote_status(), CitizenStatus::Normal);
    }
}
