//! v0 → v1 双层 ID 迁移。
//!
//! 链上现状:
//! - v0 提案 ID 编码 `year × 1_000_000 + counter`,counter ≤ 999_999
//! - 没有 `ProposalDisplayId` 反查表,客户端从主键 `id / 1_000_000` 推年份
//! - 没有反向索引,客户端按 org/institution 查只能扫全表
//!
//! v1 落地后:
//! - 主键 `proposal_id: u64` 纯单调,实质无上限
//! - `ProposalDisplayId[id] = (year, seq_in_year)` 提供展示号反查
//! - 4 张反向索引(org / institution / owner / year)按分类直接迭代
//!
//! 本迁移在 `on_runtime_upgrade` 期间扫所有现存提案:
//! 1. 反推 `(year, seq_in_year)`:旧主键 `id = year * 1_000_000 + seq` 解码
//! 2. 写入 `ProposalDisplayId[id]`
//! 3. 从 `Proposals[id]` 拿 `(internal_org, internal_institution)`
//! 4. 从 `ProposalOwner[id]` 拿 module_tag
//! 5. 写入 4 张反向索引
//!
//! **不动主键** — 现有 v0 ID 保持不变(它们既符合"单调",又能被新格式
//! 解码出展示号),客户端、链下 indexer、历史 event 引用全部继续有效。
//! 之后新创建的提案才用新格式 `NextProposalId` 单调累加。

use core::marker::PhantomData;

use frame_support::pallet_prelude::Weight;
use frame_support::traits::{Get, OnRuntimeUpgrade};

use crate::pallet::{
    Config, ProposalDisplayId, ProposalOwner, Proposals, ProposalsByInstitution, ProposalsByOrg,
    ProposalsByOwner, ProposalsByYear,
};
use crate::ProposalDisplayMeta;

pub struct MigrateToV1<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut weight = Weight::zero();

        // 中文注释:在开发期数据量极小,单块迁移完全够用;真正百万级量级
        // 升级时再分块迁移(走 Substrate 的 LimitedMigration 模式)。
        for (proposal_id, proposal) in Proposals::<T>::iter() {
            // 反推 (year, seq_in_year) — 旧 v0 格式 `id = year * 1_000_000 + seq`
            let year = (proposal_id / 1_000_000) as u16;
            let seq_in_year = (proposal_id % 1_000_000) as u32;

            // 1) 写展示号反查表
            ProposalDisplayId::<T>::insert(
                proposal_id,
                ProposalDisplayMeta { year, seq_in_year },
            );
            weight = weight.saturating_add(T::DbWeight::get().writes(1));

            // 2) 反向索引:按 org / institution / owner / year 各写一条
            if let Some(org) = proposal.internal_org {
                ProposalsByOrg::<T>::insert(org, proposal_id, ());
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }
            if let Some(inst) = proposal.internal_institution {
                ProposalsByInstitution::<T>::insert(inst, proposal_id, ());
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }
            if let Some(owner) = ProposalOwner::<T>::get(proposal_id) {
                ProposalsByOwner::<T>::insert(owner, proposal_id, ());
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            }
            ProposalsByYear::<T>::insert(year, proposal_id, ());
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        }

        weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
        use codec::Encode;
        let count = Proposals::<T>::iter().count() as u64;
        Ok(count.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        use codec::Decode;
        let pre_count = u64::decode(&mut &state[..])
            .map_err(|_| sp_runtime::TryRuntimeError::Other("decode pre count failed"))?;
        let display_count = ProposalDisplayId::<T>::iter().count() as u64;
        if display_count != pre_count {
            return Err(sp_runtime::TryRuntimeError::Other(
                "ProposalDisplayId backfill count mismatch",
            ));
        }
        Ok(())
    }
}
