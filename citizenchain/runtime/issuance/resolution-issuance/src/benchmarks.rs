//! 决议发行完整模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::{pallet_prelude::ConstU32, traits::Get, BoundedVec};
use frame_system::RawOrigin;
use primitives::china::china_cb::CHINA_CB;
use sp_runtime::traits::{CheckedAdd, SaturatedConversion, Zero};
use sp_std::{vec, vec::Vec};

use crate::{pallet, AllowedRecipients, Call, Config, Pallet, VotingProposalCount};

fn decode_account<T: pallet::Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn nrc_admin<T: pallet::Config>() -> T::AccountId {
    decode_account::<T>(CHINA_CB[0].duoqian_admins[0])
}

fn prc_recipients<T: pallet::Config>() -> BoundedVec<T::AccountId, T::MaxAllocations> {
    let recipients: Vec<T::AccountId> = CHINA_CB
        .iter()
        .skip(1)
        .map(|node| decode_account::<T>(node.main_address))
        .collect();
    recipients
        .try_into()
        .expect("benchmark recipients should fit MaxAllocations")
}

fn reason_ok<T: pallet::Config>() -> pallet::ReasonOf<T> {
    b"bench-reason".to_vec().try_into().expect("reason fits")
}

fn reason_max<T: pallet::Config>() -> pallet::ReasonOf<T> {
    let len = core::cmp::max(1usize, T::MaxReasonLen::get() as usize);
    vec![b'r'; len].try_into().expect("max reason fits")
}

fn full_allocations<T: pallet::Config>() -> (pallet::AllocationOf<T>, pallet::BalanceOf<T>) {
    let recipients = prc_recipients::<T>();
    let mut allocations: Vec<crate::proposal::RecipientAmount<T::AccountId, pallet::BalanceOf<T>>> =
        Vec::with_capacity(recipients.len());
    let mut total = pallet::BalanceOf::<T>::zero();
    for recipient in recipients {
        let amount: pallet::BalanceOf<T> = 1_000_000u128.saturated_into();
        total = total
            .checked_add(&amount)
            .expect("benchmark total should fit");
        allocations.push(crate::proposal::RecipientAmount { recipient, amount });
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
    vec![b'n'; len].try_into().expect("nonce fits")
}

fn snapshot_sig_ok<T: pallet::Config>() -> pallet::SnapshotSignatureOf<T> {
    let len = core::cmp::max(1usize, T::MaxSnapshotSignatureLength::get() as usize).min(64);
    vec![b's'; len].try_into().expect("signature fits")
}

fn province_ok() -> BoundedVec<u8, ConstU32<64>> {
    b"liaoning"
        .to_vec()
        .try_into()
        .expect("benchmark province should fit")
}

fn signer_admin_pubkey_ok() -> [u8; 32] {
    [7u8; 32]
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
        AllowedRecipients::<T>::put(recipients);
        VotingProposalCount::<T>::put(0u32);

        let reason = reason_max::<T>();
        let (allocations, total_amount) = full_allocations::<T>();
        let nonce = snapshot_nonce_ok::<T>();
        let signature = snapshot_sig_ok::<T>();
        let province = province_ok();
        let signer_admin_pubkey = signer_admin_pubkey_ok();

        #[extrinsic_call]
        propose_resolution_issuance(
            RawOrigin::Signed(proposer),
            reason,
            total_amount,
            allocations,
            10u64,
            nonce,
            signature,
            province,
            signer_admin_pubkey,
        );

        assert_eq!(VotingProposalCount::<T>::get(), 1u32);
    }

    #[benchmark]
    fn clear_executed() {
        let proposal_id = 21u64;
        let reason = reason_ok::<T>();
        let (allocations, total_amount) = full_allocations::<T>();
        Pallet::<T>::execute_approved_issuance(proposal_id, &reason, total_amount, &allocations)
            .expect("benchmark execution should succeed");

        #[extrinsic_call]
        clear_executed(RawOrigin::Root, proposal_id);

        assert!(!crate::Executed::<T>::contains_key(proposal_id));
    }

    #[benchmark]
    fn set_paused() {
        assert!(!crate::Paused::<T>::get());

        #[extrinsic_call]
        set_paused(RawOrigin::Root, true);

        assert!(crate::Paused::<T>::get());
    }
}
