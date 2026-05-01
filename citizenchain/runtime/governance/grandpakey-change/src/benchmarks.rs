//! GRANDPA 密钥治理模块 Benchmark 定义。
//!
//! Phase 2 整改后投票统一走 `voting-engine::internal_vote`,本模块不再有
//! `vote_replace_grandpa_key` extrinsic。Benchmark 只覆盖"发起提案"、"重试执行"和
//! "清理不可执行提案"三条路径。

#![cfg(feature = "runtime-benchmarks")]

use codec::{Decode, Encode};
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::Pair;
use voting_engine::STATUS_PASSED;

use crate::{
    pallet, reserve_pallet_id_to_bytes, Call, Config, GrandpaKeyReplacementAction,
    InstitutionPalletId, Pallet, CHINA_CB,
};

use crate::Pallet as GrandpaKeyChange;

fn decode_account<T: pallet::Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn prc_institution() -> InstitutionPalletId {
    reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id).expect("PRC institution should be valid")
}

fn prc_admin<T: pallet::Config>(index: usize) -> T::AccountId {
    decode_account::<T>(CHINA_CB[1].duoqian_admins[index])
}

fn seeded_public_key(seed: u8) -> [u8; 32] {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = seed;
    sp_core::ed25519::Pair::from_seed(&seed_bytes).public().0
}

fn propose<T: pallet::Config>(
    institution: InstitutionPalletId,
    proposer: T::AccountId,
    new_key: [u8; 32],
) {
    assert!(GrandpaKeyChange::<T>::propose_replace_grandpa_key(
        RawOrigin::Signed(proposer).into(),
        institution,
        new_key,
    )
    .is_ok());
}

/// 准备统一重试状态(绕开投票路径;benchmark 只测 execute / cancel 本身的开销)。
fn pass_proposal<T: pallet::Config>(proposal_id: u64) {
    voting_engine::Proposals::<T>::mutate(proposal_id, |maybe| {
        if let Some(proposal) = maybe {
            proposal.status = STATUS_PASSED;
        }
    });
    let now = frame_system::Pallet::<T>::block_number();
    voting_engine::ProposalExecutionRetryStates::<T>::insert(
        proposal_id,
        voting_engine::ExecutionRetryState {
            manual_attempts: 0,
            first_auto_failed_at: now,
            retry_deadline: now,
            last_attempt_at: None,
        },
    );
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_replace_grandpa_key() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let new_key = seeded_public_key(11);

        #[extrinsic_call]
        propose_replace_grandpa_key(RawOrigin::Signed(proposer), institution, new_key);

        assert!(voting_engine::Pallet::<T>::get_proposal_data(0).is_some());
    }

    #[benchmark]
    fn execute_replace_grandpa_key() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let caller = prc_admin::<T>(0);
        let new_key = seeded_public_key(13);

        propose::<T>(institution, proposer, new_key);
        pass_proposal::<T>(0);

        #[extrinsic_call]
        execute_replace_grandpa_key(RawOrigin::Signed(caller), 0);
    }

    #[benchmark]
    fn cancel_failed_replace_grandpa_key() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let caller = prc_admin::<T>(1);
        let new_key = seeded_public_key(15);

        propose::<T>(institution, proposer, new_key);
        pass_proposal::<T>(0);

        // 将 old_key 篡改为不存在的 authority,制造"已通过但不可执行"场景。
        let old_raw =
            voting_engine::Pallet::<T>::get_proposal_data(0).expect("proposal data should exist");
        let tag = crate::MODULE_TAG;
        let mut action = GrandpaKeyReplacementAction::decode(&mut &old_raw[tag.len()..])
            .expect("action should decode");
        action.old_key = seeded_public_key(250);
        let mut new_data = sp_runtime::sp_std::vec::Vec::from(tag);
        new_data.extend_from_slice(&action.encode());
        let bounded_data: frame_support::BoundedVec<
            u8,
            <T as voting_engine::Config>::MaxProposalDataLen,
        > = new_data
            .try_into()
            .expect("benchmark proposal data should fit");
        voting_engine::ProposalData::<T>::insert(0, bounded_data);

        #[extrinsic_call]
        cancel_failed_replace_grandpa_key(RawOrigin::Signed(caller), 0);
    }
}
