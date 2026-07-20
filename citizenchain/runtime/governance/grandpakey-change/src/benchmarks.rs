//! GRANDPA 密钥治理模块 Benchmark 定义。
//!
//! 投票统一走 `votingengine::internal_vote`,本模块只覆盖"发起提案"、
//! "重试执行"和"清理不可执行提案"三条路径。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::Pair;

use crate::{pallet, Call, Config, Pallet, CHINA_CB};

fn decode_account<T: pallet::Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn prc_cid() -> votingengine::types::CidNumber {
    CHINA_CB[1]
        .cid_number
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("PRC CID fits")
}

fn prc_admin<T: pallet::Config>(index: usize) -> T::AccountId {
    decode_account::<T>(CHINA_CB[1].admins[index])
}

fn seeded_public_key(seed: u8) -> [u8; 32] {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = seed;
    sp_core::ed25519::Pair::from_seed(&seed_bytes).public().0
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_replace_grandpa_key() {
        let actor_cid_number = prc_cid();
        let proposer = prc_admin::<T>(0);
        let new_key = seeded_public_key(11);

        #[extrinsic_call]
        propose_replace_grandpa_key(
            RawOrigin::Signed(proposer),
            actor_cid_number,
            primitives::governance_skeleton::ROLE_CODE_COMMITTEE_MEMBER
                .to_vec()
                .try_into()
                .expect("benchmark role fits"),
            new_key,
        );

        assert!(votingengine::Pallet::<T>::get_proposal_data(0).is_some());
    }

    // 重试/取消已通过提案的 wrapper extrinsic 已废弃,统一到 VotingEngine
    // 的 retry/cancel 入口,benchmark 由 votingengine 自身覆盖。
}
