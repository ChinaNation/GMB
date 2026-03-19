//! 全局活跃提案数量限制。
//!
//! 每个机构（InstitutionPalletId）同时最多允许 `MAX_ACTIVE_PROPOSALS` 个活跃提案，
//! 不区分提案类型（转账、销毁、换管理员等），由投票引擎统一管控。
//!
//! 使用方式：
//! - 创建提案时调用 `try_add_active_proposal`
//! - 提案完成/清理时调用 `remove_active_proposal`
//! - App 端查询活跃数调用 `active_proposal_count`

use crate::pallet::{self, Config, Error};
use crate::InstitutionPalletId;
use frame_support::pallet_prelude::*;

/// 每个机构最多同时存在的活跃提案数。
pub const MAX_ACTIVE_PROPOSALS: u32 = 10;

/// 尝试为机构新增一个活跃提案。
/// 成功返回 Ok(())，达到上限返回 Err。
pub fn try_add_active_proposal<T: Config>(
    institution: InstitutionPalletId,
    proposal_id: u64,
) -> DispatchResult {
    pallet::ActiveProposalsByInstitution::<T>::try_mutate(institution, |ids| {
        ensure!(
            (ids.len() as u32) < MAX_ACTIVE_PROPOSALS,
            Error::<T>::ActiveProposalLimitReached
        );
        ids.try_push(proposal_id)
            .map_err(|_| Error::<T>::ActiveProposalLimitReached)?;
        Ok(())
    })
}

/// 从机构的活跃提案列表中移除指定提案。
pub fn remove_active_proposal<T: Config>(
    institution: InstitutionPalletId,
    proposal_id: u64,
) {
    pallet::ActiveProposalsByInstitution::<T>::mutate(institution, |ids| {
        ids.retain(|&id| id != proposal_id);
    });
}

/// 查询机构当前活跃提案数量。
pub fn active_proposal_count<T: Config>(institution: InstitutionPalletId) -> u32 {
    pallet::ActiveProposalsByInstitution::<T>::get(institution).len() as u32
}
