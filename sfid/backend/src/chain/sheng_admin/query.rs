//! 中文注释:链上 `ShengAdmins[Province][Slot]` 三槽 pull(phase45)。
//!
//! ADR-008:每省 main / backup_1 / backup_2 三个 admin 槽位,链上 storage
//! `ShengAdmins: DoubleMap<Province, Slot, Pubkey>`。
//!
//! ## phase45 mock 行为
//!
//! `fetch_roster` 直接读 `crate::sfid::province::PROVINCES` 常量取 main pubkey,
//! backup_1 / backup_2 一律返回 `None`。phase7 切真:走 subxt `storage().fetch()`
//! 真正读链。

#![allow(dead_code)]

use crate::sfid::province::{pubkey_from_hex, PROVINCES};

#[derive(Debug)]
pub(crate) enum RosterQueryError {
    UnknownProvince,
    PubkeyDecode,
}

impl std::fmt::Display for RosterQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RosterQueryError::UnknownProvince => write!(f, "unknown province"),
            RosterQueryError::PubkeyDecode => write!(f, "decode main pubkey hex failed"),
        }
    }
}

impl std::error::Error for RosterQueryError {}

/// 拉本省 3 槽公钥(slot 顺序固定:[main, backup_1, backup_2])。
///
/// mock 实现:`[Some(province.main_admin_pubkey), None, None]`。
///
/// `province` 入参接受省名(如 "安徽省")。
pub(crate) async fn fetch_roster(
    province: &str,
) -> Result<[Option<[u8; 32]>; 3], RosterQueryError> {
    let entry = PROVINCES
        .iter()
        .find(|p| p.name == province)
        .ok_or(RosterQueryError::UnknownProvince)?;
    let main = pubkey_from_hex(entry.pubkey).ok_or(RosterQueryError::PubkeyDecode)?;
    tracing::warn!(province = %province, "chain pull mocked, awaiting Step 2");
    Ok([Some(main), None, None])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetch_roster_returns_main_for_known_province() {
        let roster = fetch_roster("安徽省").await.expect("known province");
        assert!(roster[0].is_some(), "main slot must be Some");
        assert!(roster[1].is_none(), "backup_1 mock must be None");
        assert!(roster[2].is_none(), "backup_2 mock must be None");
    }

    #[tokio::test]
    async fn fetch_roster_rejects_unknown_province() {
        let err = fetch_roster("不存在省").await.unwrap_err();
        assert!(matches!(err, RosterQueryError::UnknownProvince));
    }
}
