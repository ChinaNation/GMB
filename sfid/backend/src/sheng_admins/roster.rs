//! 中文注释:省管理员 3-tier 名册(roster)操作 service —— **已被 phase4/7 取代,无活跃调用方**。
//!
//! ADR-008 决议(2026-05-01):add_sheng_admin_backup / remove_sheng_admin_backup
//! 必须由当前 main 公钥签名授权,链上 ShengAdmins[Province][Slot] storage 持久化。
//!
//! Phase 4 起真实推链入口转移到 `crate::chain::sheng_admin::{add_backup, remove_backup}`,
//! Phase 7 已切真。本文件保留作为 Phase 3 历史路径的占位,等 Phase 7 收尾时统一删除。
//!
//! 名册当前真相来源:
//! - main:`crate::sfid::province::PROVINCES[*].pubkey` 常量
//! - backup_1 / backup_2:`fetch_backup_admins` 占位返回 [None, None]

#![allow(dead_code)]

use crate::sfid::province::{
    fetch_backup_admins, province_admins_for, ProvinceAdmins, Slot,
};

#[derive(Debug)]
pub(crate) enum RosterError {
    UnknownProvince,
    SlotInvalidForOp,
    AlreadyOccupied(Slot),
    NotOccupied(Slot),
    ChainMockUnavailable,
}

impl std::fmt::Display for RosterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RosterError::UnknownProvince => write!(f, "unknown province"),
            RosterError::SlotInvalidForOp => write!(f, "slot invalid for this operation"),
            RosterError::AlreadyOccupied(s) => {
                write!(f, "slot {} already occupied", s.as_str())
            }
            RosterError::NotOccupied(s) => write!(f, "slot {} not occupied", s.as_str()),
            RosterError::ChainMockUnavailable => write!(f, "legacy chain push placeholder unavailable"),
        }
    }
}

impl std::error::Error for RosterError {}

/// 拉取本省 3-tier 名册(main 来自常量,backup 当前 mock 为 None)。
pub(crate) fn read_roster(province: &str) -> Result<ProvinceAdmins, RosterError> {
    province_admins_for(province).ok_or(RosterError::UnknownProvince)
}

/// 中文注释:Phase 3 历史路径 —— 推链注册 backup 公钥。
/// 已被 `crate::chain::sheng_admin::add_backup` 取代,无活跃调用方。
pub(crate) async fn add_backup(
    province: &str,
    slot: Slot,
    new_backup: [u8; 32],
) -> Result<(), RosterError> {
    if matches!(slot, Slot::Main) {
        return Err(RosterError::SlotInvalidForOp);
    }
    let current = read_roster(province)?;
    let occupied = match slot {
        Slot::Backup1 => current.backup_1.is_some(),
        Slot::Backup2 => current.backup_2.is_some(),
        Slot::Main => true,
    };
    if occupied {
        return Err(RosterError::AlreadyOccupied(slot));
    }
    push_chain_placeholder(&format!(
        "add_sheng_admin_backup province={province} slot={} new_backup=0x{}",
        slot.as_str(),
        hex::encode(new_backup)
    ))
    .await
}

/// 中文注释:Phase 3 历史路径 —— 推链注销 backup 公钥。
/// 已被 `crate::chain::sheng_admin::remove_backup` 取代,无活跃调用方。
pub(crate) async fn remove_backup(province: &str, slot: Slot) -> Result<(), RosterError> {
    if matches!(slot, Slot::Main) {
        return Err(RosterError::SlotInvalidForOp);
    }
    let current = read_roster(province)?;
    let occupied = match slot {
        Slot::Backup1 => current.backup_1.is_some(),
        Slot::Backup2 => current.backup_2.is_some(),
        Slot::Main => true,
    };
    if !occupied {
        return Err(RosterError::NotOccupied(slot));
    }
    push_chain_placeholder(&format!(
        "remove_sheng_admin_backup province={province} slot={}",
        slot.as_str()
    ))
    .await
}

/// Phase 3 历史路径占位。真实推链已由 `crate::chain::sheng_admin::{add_backup, remove_backup}`
/// 在 Phase 7 切真,无活跃调用方;保留仅为暂不破坏 module 边界,后续清理。
async fn push_chain_placeholder(name: &str) -> Result<(), RosterError> {
    tracing::warn!("legacy roster placeholder invoked for {name}; real path is chain::sheng_admin");
    let _ = fetch_backup_admins; // suppress dead_code warning of imported helper
    Ok(())
}
