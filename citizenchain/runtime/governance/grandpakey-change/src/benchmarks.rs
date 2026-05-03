//! GRANDPA 密钥治理模块 Benchmark 定义。
//!
//! Phase 2 整改后投票统一走 `voting-engine::internal_vote`,本模块不再有
//! `vote_replace_grandpa_key` extrinsic。Benchmark 只覆盖"发起提案"、"重试执行"和
//! "清理不可执行提案"三条路径。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::Pair;

use crate::{pallet, reserve_pallet_id_to_bytes, Call, Config, InstitutionPalletId, Pallet, CHINA_CB};

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

    // execute_replace_grandpa_key / cancel_failed_replace_grandpa_key benchmark
    // 已废弃: 两个 wrapper extrinsic 已统一到 VotingEngine 的 retry/cancel
    // 入口,benchmark 由 voting-engine 自身覆盖。
}
