//! 反向索引(spec_version v1)。
//!
//! 链上 4 张反向索引让客户端按"分类"O(分类内规模)迭代提案,不用扫全表:
//! - **`ProposalsByOrg[org][id]`** — 按 ORG_NRC / ORG_PRC / ORG_PRB / ORG_REN 反查
//! - **`ProposalsByInstitution[institution_pallet_id][id]`** — 按机构主体反查
//!   (如某省储行所有提案、某个多签账户所有提案)
//! - **`ProposalsByOwner[module_tag][id]`** — 按业务模块 MODULE_TAG 反查
//!   (如"只看 runtime 升级提案"、"只看决议销毁提案")
//! - **`ProposalsByYear[year][id]`** — 按创建年份反查(历史归档视图)
//!
//! **写入时机**:`register_proposal_data` 末尾(提案对外业务数据落地后立刻写)。
//! **删除时机**:`cleanup_proposal_indexes`(终态/90 天保留期清理路径)。

use frame_support::pallet_prelude::BoundedVec;

use crate::pallet::{
    self, ProposalDisplayId, ProposalOwner, Proposals, ProposalsByInstitution, ProposalsByOrg,
    ProposalsByOwner, ProposalsByYear,
};
use crate::SubjectId;

impl<T: pallet::Config> pallet::Pallet<T> {
    /// 写入四张反向索引。
    ///
    /// 由 `register_proposal_data` 在创建阶段同事务调用,保证:
    /// - 任一提案写入 `ProposalData` 后,4 张反向索引立即可查
    /// - 不写 `ProposalData` 的占位提案(仅创建阶段失败回滚的)不会污染索引
    pub fn register_proposal_indexes(
        proposal_id: u64,
        org: Option<u8>,
        institution: Option<SubjectId>,
        module_tag: BoundedVec<u8, T::MaxModuleTagLen>,
        year: u16,
    ) {
        if let Some(org) = org {
            ProposalsByOrg::<T>::insert(org, proposal_id, ());
        }
        if let Some(inst) = institution {
            ProposalsByInstitution::<T>::insert(inst, proposal_id, ());
        }
        ProposalsByOwner::<T>::insert(module_tag, proposal_id, ());
        ProposalsByYear::<T>::insert(year, proposal_id, ());
    }

    /// 释放该提案在 4 张反向索引中的所有条目 + ProposalDisplayId。
    ///
    /// 由清理路径(`cleanup` / 90 天保留期)调用。需要从 `Proposals` /
    /// `ProposalOwner` / `ProposalDisplayId` 反查 (org, institution, owner, year),
    /// 顺序无关紧要,所有 4 张索引 + 展示号表一次清干净。
    ///
    /// **必须在 `Proposals[id]` / `ProposalOwner[id]` / `ProposalDisplayId[id]`
    /// 自身被删除之前调用**(否则反查不到分类键无法清索引)。
    pub fn cleanup_proposal_indexes(proposal_id: u64) {
        if let Some(proposal) = Proposals::<T>::get(proposal_id) {
            if let Some(org) = proposal.internal_org {
                ProposalsByOrg::<T>::remove(org, proposal_id);
            }
            if let Some(inst) = proposal.internal_institution {
                ProposalsByInstitution::<T>::remove(inst, proposal_id);
            }
        }
        if let Some(owner) = ProposalOwner::<T>::get(proposal_id) {
            ProposalsByOwner::<T>::remove(owner, proposal_id);
        }
        if let Some(meta) = ProposalDisplayId::<T>::get(proposal_id) {
            ProposalsByYear::<T>::remove(meta.year, proposal_id);
        }
        ProposalDisplayId::<T>::remove(proposal_id);
    }
}
