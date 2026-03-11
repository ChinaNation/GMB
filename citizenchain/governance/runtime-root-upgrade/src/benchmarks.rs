//! 运行时升级模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use sp_runtime::traits::Hash;

use crate::pallet::{
    Config, GovToJointVote, JointVoteToGov, NextProposalId, Pallet, Proposal, ProposalStatus,
    Proposals, RetryCount,
};

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_runtime_upgrade() {
        let proposer = decode_account::<T>(primitives::china::china_cb::CHINA_CB[0].admins[0]);
        let reason: crate::pallet::ReasonOf<T> = b"benchmark reason"
            .to_vec()
            .try_into()
            .expect("reason should fit");
        let code: crate::pallet::CodeOf<T> = sp_runtime::sp_std::vec![1, 2, 3]
            .try_into()
            .expect("code should fit");
        let code_hash = T::Hashing::hash(code.as_slice());

        #[block]
        {
            let proposal_id = NextProposalId::<T>::get();
            NextProposalId::<T>::put(proposal_id.checked_add(1).expect("no overflow"));
            let proposal = Proposal::<T> {
                proposer: proposer.clone(),
                reason,
                code_hash,
                code,
                status: ProposalStatus::Voting,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            GovToJointVote::<T>::insert(proposal_id, 100u64);
            JointVoteToGov::<T>::insert(100u64, proposal_id);
        }

        assert!(Proposals::<T>::contains_key(0));
    }

    #[benchmark]
    fn finalize_joint_vote() {
        let proposer = decode_account::<T>(primitives::china::china_cb::CHINA_CB[0].admins[0]);
        let reason: crate::pallet::ReasonOf<T> = b"benchmark reason"
            .to_vec()
            .try_into()
            .expect("reason should fit");
        let code: crate::pallet::CodeOf<T> = sp_runtime::sp_std::vec![1, 2, 3]
            .try_into()
            .expect("code should fit");
        let code_hash = T::Hashing::hash(code.as_slice());

        let proposal = Proposal::<T> {
            proposer,
            reason,
            code_hash,
            code,
            status: ProposalStatus::Voting,
        };
        Proposals::<T>::insert(0u64, proposal);
        GovToJointVote::<T>::insert(0u64, 100u64);
        JointVoteToGov::<T>::insert(100u64, 0u64);

        #[block]
        {
            Pallet::<T>::apply_joint_vote_result(0, false).expect("finalize should succeed");
        }

        let p = Proposals::<T>::get(0).expect("proposal should exist");
        assert!(matches!(p.status, ProposalStatus::Rejected));
    }

    #[benchmark]
    fn retry_failed_execution() {
        let proposer = decode_account::<T>(primitives::china::china_cb::CHINA_CB[0].admins[0]);
        let reason: crate::pallet::ReasonOf<T> = b"benchmark reason"
            .to_vec()
            .try_into()
            .expect("reason should fit");
        let code: crate::pallet::CodeOf<T> = sp_runtime::sp_std::vec![1, 2, 3]
            .try_into()
            .expect("code should fit");
        let code_hash = T::Hashing::hash(code.as_slice());

        let proposal = Proposal::<T> {
            proposer,
            reason,
            code_hash,
            code,
            status: ProposalStatus::ExecutionFailed,
        };
        Proposals::<T>::insert(0u64, proposal);
        RetryCount::<T>::insert(0u64, 0u32);

        #[block]
        {
            // 基准测试仅衡量存储操作成本，跳过 RuntimeCodeExecutor 跨模块调用。
            let retries = RetryCount::<T>::get(0);
            RetryCount::<T>::insert(0u64, retries.saturating_add(1));
        }
    }
}
