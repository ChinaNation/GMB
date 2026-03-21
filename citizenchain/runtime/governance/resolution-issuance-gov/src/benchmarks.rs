//! 决议发行治理模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::{Decode, Encode};
use frame_benchmarking::v2::*;
use frame_support::{traits::Get, BoundedVec};
use frame_system::RawOrigin;
use primitives::china::china_cb::CHINA_CB;
use sp_std::{vec, vec::Vec};

use crate::{pallet, AllowedRecipients, Call, Config, Pallet, VotingProposalCount};

fn decode_account<T: pallet::Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn nrc_admin<T: pallet::Config>() -> T::AccountId {
    decode_account::<T>(CHINA_CB[0].admins[0])
}

fn prc_recipients<T: pallet::Config>() -> BoundedVec<T::AccountId, T::MaxAllocations> {
    let recipients: Vec<T::AccountId> = CHINA_CB
        .iter()
        .skip(1)
        .map(|node| decode_account::<T>(node.duoqian_address))
        .collect();
    recipients
        .try_into()
        .expect("benchmark recipients should fit MaxAllocations")
}

fn reason_ok<T: pallet::Config>() -> pallet::ReasonOf<T> {
    b"bench-reason"
        .to_vec()
        .try_into()
        .expect("benchmark reason should fit")
}

fn reason_max<T: pallet::Config>() -> pallet::ReasonOf<T> {
    let len = core::cmp::max(1usize, T::MaxReasonLen::get() as usize);
    vec![b'r'; len]
        .try_into()
        .expect("max benchmark reason should fit")
}

fn one_allocation<T: pallet::Config>() -> pallet::AllocationOf<T> {
    let recipient = decode_account::<T>(CHINA_CB[1].duoqian_address);
    let alloc = vec![pallet::RecipientAmount {
        recipient,
        amount: 1_000_000u128,
    }];
    alloc
        .try_into()
        .expect("benchmark allocations should fit MaxAllocations")
}

fn full_allocations<T: pallet::Config>() -> (pallet::AllocationOf<T>, u128) {
    let recipients = prc_recipients::<T>();
    let mut allocations: Vec<pallet::RecipientAmount<T::AccountId>> =
        Vec::with_capacity(recipients.len());
    let mut total = 0u128;
    for recipient in recipients {
        let amount = 1_000_000u128;
        total = total.saturating_add(amount);
        allocations.push(pallet::RecipientAmount { recipient, amount });
    }
    (
        allocations
            .try_into()
            .expect("benchmark allocations should fit MaxAllocations"),
        total,
    )
}

fn snapshot_nonce_ok<T: pallet::Config>() -> pallet::SnapshotNonceOf<T> {
    let len = core::cmp::max(1usize, T::MaxSnapshotNonceLength::get() as usize).min(16);
    vec![b'n'; len]
        .try_into()
        .expect("benchmark nonce should fit")
}

fn snapshot_sig_ok<T: pallet::Config>() -> pallet::SnapshotSignatureOf<T> {
    let len = core::cmp::max(1usize, T::MaxSnapshotSignatureLength::get() as usize).min(64);
    vec![b's'; len]
        .try_into()
        .expect("benchmark signature should fit")
}

/// 在投票引擎 ProposalData 中直接写入带 MODULE_TAG 前缀的业务数据，用于 benchmark setup。
fn insert_proposal_data_for_benchmark<T: pallet::Config>(
    proposal_id: u64,
    proposer: T::AccountId,
    reason: &pallet::ReasonOf<T>,
    total_amount: u128,
    allocations: &pallet::AllocationOf<T>,
) {
    let data = pallet::IssuanceProposalData {
        proposer,
        reason: reason.to_vec(),
        total_amount,
        allocations: allocations.to_vec(),
    };
    let mut encoded = Vec::from(crate::MODULE_TAG);
    encoded.extend_from_slice(&data.encode());
    voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, encoded)
        .expect("benchmark store_proposal_data should succeed");
    voting_engine_system::Pallet::<T>::store_proposal_meta(
        proposal_id,
        frame_system::Pallet::<T>::block_number(),
    );
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_allowed_recipients() {
        VotingProposalCount::<T>::put(0u32);
        let recipients = prc_recipients::<T>();

        #[extrinsic_call]
        set_allowed_recipients(RawOrigin::Root, recipients.clone());

        assert_eq!(AllowedRecipients::<T>::get(), recipients);
    }

    #[benchmark]
    fn propose_resolution_issuance() {
        let proposer = nrc_admin::<T>();
        let recipients = prc_recipients::<T>();
        AllowedRecipients::<T>::put(recipients.clone());
        VotingProposalCount::<T>::put(0u32);

        let reason = reason_max::<T>();
        let (allocations, total_amount) = full_allocations::<T>();
        let nonce = snapshot_nonce_ok::<T>();
        let signature = snapshot_sig_ok::<T>();

        #[extrinsic_call]
        propose_resolution_issuance(
            RawOrigin::Signed(proposer),
            reason,
            total_amount,
            allocations,
            10u64,
            nonce,
            signature,
        );

        assert_eq!(VotingProposalCount::<T>::get(), 1u32);
    }

    #[benchmark]
    fn finalize_joint_vote_approved() {
        let proposal_id = 11u64;
        let proposer = nrc_admin::<T>();
        let reason = reason_max::<T>();
        let (allocations, total_amount) = full_allocations::<T>();
        insert_proposal_data_for_benchmark::<T>(
            proposal_id,
            proposer,
            &reason,
            total_amount,
            &allocations,
        );
        VotingProposalCount::<T>::put(1u32);

        #[extrinsic_call]
        finalize_joint_vote(RawOrigin::Root, proposal_id, true);

        assert_eq!(VotingProposalCount::<T>::get(), 0u32);
    }

    #[benchmark]
    fn finalize_joint_vote_rejected() {
        let proposal_id = 12u64;
        let proposer = nrc_admin::<T>();
        let reason = reason_ok::<T>();
        let allocations = one_allocation::<T>();
        insert_proposal_data_for_benchmark::<T>(
            proposal_id,
            proposer,
            &reason,
            1_000_000u128,
            &allocations,
        );
        VotingProposalCount::<T>::put(1u32);

        #[extrinsic_call]
        finalize_joint_vote(RawOrigin::Root, proposal_id, false);

        assert_eq!(VotingProposalCount::<T>::get(), 0u32);
    }
}
