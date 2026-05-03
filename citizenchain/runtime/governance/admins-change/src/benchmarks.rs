//! 管理员治理模块 Benchmark 定义。
//!
//! Phase 2 整改后投票统一由 `voting-engine::internal_vote` 公开 call 承担,
//! 本模块不再保留独立投票 extrinsic。Benchmark 只覆盖提案创建和
//! 执行重试两条路径。

#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as AdminsChange;
use crate::{
    reserve_pallet_id_to_bytes, BlockNumberFor, Call, Config, InstitutionPalletId, Institutions,
    Pallet, CHINA_CB, ORG_PRC,
};
use codec::Decode;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::{SaturatedConversion, Saturating};

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn prc_institution() -> InstitutionPalletId {
    reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id).expect("PRC institution should be valid")
}

fn prc_admin<T: Config>(index: usize) -> T::AccountId {
    decode_account::<T>(CHINA_CB[1].duoqian_admins[index])
}

fn last_proposal_id<T: Config>() -> u64 {
    voting_engine::Pallet::<T>::next_proposal_id().saturating_sub(1)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_admin_replacement() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let old_admin = prc_admin::<T>(1);
        let new_admin: T::AccountId = frame_benchmarking::account("new_admin", 0, 0);
        let stale_new_admin: T::AccountId = frame_benchmarking::account("stale_new_admin", 0, 0);

        // 先发一个"陈旧"提案,让它自然超时被终结,验证新提案不会冲突。
        assert!(AdminsChange::<T>::propose_admin_replacement(
            RawOrigin::Signed(proposer.clone()).into(),
            ORG_PRC,
            institution,
            old_admin.clone(),
            stale_new_admin,
        )
        .is_ok());

        let stale_proposal_id = last_proposal_id::<T>();
        let end = voting_engine::Pallet::<T>::proposals(stale_proposal_id)
            .expect("stale benchmark proposal should exist")
            .end;
        let one: BlockNumberFor<T> = 1u32.saturated_into();
        frame_system::Pallet::<T>::set_block_number(end.saturating_add(one));
        assert!(voting_engine::Pallet::<T>::finalize_proposal(
            RawOrigin::Signed(proposer.clone()).into(),
            stale_proposal_id,
        )
        .is_ok());

        #[extrinsic_call]
        propose_admin_replacement(
            RawOrigin::Signed(proposer),
            ORG_PRC,
            institution,
            old_admin,
            new_admin,
        );

        let proposal_id = last_proposal_id::<T>();
        assert!(voting_engine::Pallet::<T>::get_proposal_data(proposal_id).is_some());
    }

    /// `execute_admin_replacement` benchmark:
    /// 1. 发起提案 → 自动存入 ProposalData
    /// 2. 手动 mutate Institutions 模拟"管理员列表被污染"的中间态
    /// 3. 手动把提案状态推到 PASSED,触发回调但让自动执行失败
    /// 4. 调 `execute_admin_replacement` 完成补救执行
    #[benchmark]
    fn execute_admin_replacement() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let old_admin = prc_admin::<T>(1);
        let caller = prc_admin::<T>(6);
        let new_admin: T::AccountId = frame_benchmarking::account("new_admin", 2, 0);
        let temp_admin: T::AccountId = frame_benchmarking::account("temp_admin", 0, 0);

        assert!(AdminsChange::<T>::propose_admin_replacement(
            RawOrigin::Signed(proposer).into(),
            ORG_PRC,
            institution,
            old_admin.clone(),
            new_admin,
        )
        .is_ok());
        let proposal_id = last_proposal_id::<T>();

        // 模拟中间态:先把 old_admin 换成 temp_admin,让投票通过回调里的自动执行失败。
        Institutions::<T>::mutate(institution, |maybe_subject| {
            let subject = maybe_subject
                .as_mut()
                .expect("benchmark institution should exist");
            let admins = &mut subject.admins;
            let old_pos = admins
                .iter()
                .position(|admin| admin == &old_admin)
                .expect("benchmark old_admin should exist");
            admins[old_pos] = temp_admin.clone();
        });

        // 中文注释：benchmark 通过 voting-engine 专用 helper 准备统一重试状态，
        // 避免跨 pallet 直接依赖投票引擎内部 storage 结构。
        assert!(
            voting_engine::Pallet::<T>::force_retryable_passed_for_benchmark(proposal_id).is_ok(),
            "benchmark retry state should prepare"
        );

        // 再还原 old_admin(让 execute 逻辑有合法 old_admin 可查)。
        Institutions::<T>::mutate(institution, |maybe_subject| {
            let subject = maybe_subject
                .as_mut()
                .expect("benchmark institution should exist");
            let admins = &mut subject.admins;
            let temp_pos = admins
                .iter()
                .position(|admin| admin == &temp_admin)
                .expect("temporary benchmark admin marker should exist");
            admins[temp_pos] = old_admin.clone();
        });

        assert!(voting_engine::Pallet::<T>::get_proposal_data(proposal_id).is_some());

        #[extrinsic_call]
        execute_admin_replacement(RawOrigin::Signed(caller), proposal_id);
    }
}
