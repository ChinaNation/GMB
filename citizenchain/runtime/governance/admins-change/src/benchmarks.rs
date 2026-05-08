//! 管理员治理模块 Benchmark 定义。
//!
//! 投票统一走 `votingengine::internal_vote`,本模块只覆盖提案创建和
//! 执行重试两条路径。

#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as AdminsChange;
use crate::{
    subject_id_from_sfid_number, BlockNumberFor, Call, Config, Pallet, SubjectId, CHINA_CB, ORG_PRC,
};
use codec::Decode;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::{SaturatedConversion, Saturating};

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn prc_institution() -> SubjectId {
    subject_id_from_sfid_number(CHINA_CB[1].sfid_number).expect("PRC institution should be valid")
}

fn prc_admin<T: Config>(index: usize) -> T::AccountId {
    decode_account::<T>(CHINA_CB[1].duoqian_admins[index])
}

fn last_proposal_id<T: Config>() -> u64 {
    votingengine::Pallet::<T>::next_proposal_id().saturating_sub(1)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_admin_set_change() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let new_admin: T::AccountId = frame_benchmarking::account("new_admin", 0, 0);
        let stale_new_admin: T::AccountId = frame_benchmarking::account("stale_new_admin", 0, 0);
        let subject =
            crate::Subjects::<T>::get(institution).expect("benchmark genesis subject should exist");
        let mut stale_admins = subject.admins.clone();
        stale_admins[1] = stale_new_admin;
        let mut new_admins = subject.admins;
        new_admins[1] = new_admin;

        // 先发一个"陈旧"提案,让它自然超时被终结,验证新提案不会冲突。
        assert!(AdminsChange::<T>::propose_admin_set_change(
            RawOrigin::Signed(proposer.clone()).into(),
            ORG_PRC,
            institution,
            stale_admins,
        )
        .is_ok());

        let stale_proposal_id = last_proposal_id::<T>();
        let end = votingengine::Pallet::<T>::proposals(stale_proposal_id)
            .expect("stale benchmark proposal should exist")
            .end;
        let one: BlockNumberFor<T> = 1u32.saturated_into();
        frame_system::Pallet::<T>::set_block_number(end.saturating_add(one));
        assert!(votingengine::Pallet::<T>::finalize_proposal(
            RawOrigin::Signed(proposer.clone()).into(),
            stale_proposal_id,
        )
        .is_ok());

        #[extrinsic_call]
        propose_admin_set_change(
            RawOrigin::Signed(proposer),
            ORG_PRC,
            institution,
            new_admins,
        );

        let proposal_id = last_proposal_id::<T>();
        assert!(votingengine::Pallet::<T>::get_proposal_data(proposal_id).is_some());
    }
}
