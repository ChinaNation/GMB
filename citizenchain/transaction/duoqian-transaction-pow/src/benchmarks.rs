//! 多签交易模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_support::{
    traits::{Get, UnfilteredDispatchable},
    BoundedVec,
};
use frame_system::RawOrigin;
use sp_runtime::traits::{Hash, SaturatedConversion, Zero};
use sp_std::vec;

use crate::{
    AdminApprovalsOf, BalanceOf, BlockNumberFor, Call, ChainDomainHash, Config, DuoqianAdminsOf,
    Pallet, SfidIdOf,
};

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_sfid_institution() {
        let operator: T::AccountId = frame_benchmarking::account("operator", 0, 0);
        let sfid_id: SfidIdOf<T> = BoundedVec::default();
        ChainDomainHash::<T>::put(T::Hashing::hash(b"duoqian-benchmark-domain"));

        #[block]
        {
            let call = Call::<T>::register_sfid_institution {
                sfid_id: sfid_id.clone(),
            };
            assert!(call
                .dispatch_bypass_filter(RawOrigin::Signed(operator.clone()).into())
                .is_err());
        }
    }

    #[benchmark]
    fn create_duoqian() {
        let caller: T::AccountId = frame_benchmarking::account("caller", 0, 0);
        let sfid_id: SfidIdOf<T> =
            BoundedVec::try_from(vec![b'b']).expect("bounded sfid id should fit");
        let duoqian_admins: DuoqianAdminsOf<T> = BoundedVec::default();
        let approvals: AdminApprovalsOf<T> = BoundedVec::default();
        let amount = T::MinCreateAmount::get();
        let expires_at: BlockNumberFor<T> = 1u32.saturated_into();

        #[block]
        {
            let call = Call::<T>::create_duoqian {
                sfid_id: sfid_id.clone(),
                admin_count: 1,
                duoqian_admins: duoqian_admins.clone(),
                threshold: 1,
                amount,
                expires_at,
                approvals: approvals.clone(),
            };
            assert!(call
                .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())
                .is_err());
        }
    }

    #[benchmark]
    fn close_duoqian() {
        let caller: T::AccountId = frame_benchmarking::account("caller", 1, 0);
        let duoqian_address: T::AccountId = frame_benchmarking::account("duoqian", 0, 0);
        let approvals: AdminApprovalsOf<T> = BoundedVec::default();
        let min_balance: BalanceOf<T> = Zero::zero();
        let expires_at: BlockNumberFor<T> = 1u32.saturated_into();

        #[block]
        {
            let call = Call::<T>::close_duoqian {
                duoqian_address: duoqian_address.clone(),
                beneficiary: duoqian_address.clone(),
                min_balance,
                expires_at,
                approvals: approvals.clone(),
            };
            assert!(call
                .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())
                .is_err());
        }
    }
}
