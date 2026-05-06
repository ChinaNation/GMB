//! Benchmarks for `joint-vote` pallet。
//!
//! 包含 cast_admin / cast_referendum / finalize_joint_timeout /
//! finalize_jointreferendum_timeout 的权重 benchmark 占位 + migration v1
//! 的负载 benchmark。weight 实测靠 substrate-benchmark-cli 跑出再覆盖
//! `weights.rs` 的手工保守上界。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_support::traits::OnRuntimeUpgrade;
use sp_io::hashing::twox_128;
use sp_std::vec::Vec;

use crate::migrations::v1::MigrateV0ToV1;
use crate::{Config, Pallet};

const OLD_PALLET: &[u8] = b"VotingEngine";
const NEW_PALLET: &[u8] = b"JointVote";
/// (旧名 in VotingEngine, 新名 in JointVote)。Citizen* 一对在迁移过程中改名为 Referendum*。
const STORAGES: &[(&[u8], &[u8])] = &[
    (b"JointVotesByAdmin", b"JointVotesByAdmin"),
    (b"JointInstitutionTallies", b"JointInstitutionTallies"),
    (b"JointVotesByInstitution", b"JointVotesByInstitution"),
    (b"JointTallies", b"JointTallies"),
    (b"CitizenVotesByBindingId", b"ReferendumVotesByBindingId"),
    (b"CitizenTallies", b"ReferendumTallies"),
    (b"UsedPopulationSnapshotNonce", b"UsedPopulationSnapshotNonce"),
];

fn build_prefix(pallet: &[u8], storage: &[u8]) -> Vec<u8> {
    let mut p = Vec::with_capacity(32);
    p.extend_from_slice(&twox_128(pallet));
    p.extend_from_slice(&twox_128(storage));
    p
}

#[benchmarks]
mod benchmarks {
    use super::*;

    /// migration v0 → v1 负载 benchmark。`n` 是单 storage 的 entry 数;
    /// 7 个 storage 总计 7·n 条 key。CLI `--steps 50 --repeat 20` 可拟合
    /// weight = a + b·n 线性关系。
    #[benchmark]
    fn migration_v0_to_v1(n: Linear<0, 10_000>) {
        for (old_name, _) in STORAGES {
            let prefix = build_prefix(OLD_PALLET, old_name);
            for i in 0..n {
                let mut key = prefix.clone();
                key.extend_from_slice(&i.to_le_bytes());
                sp_io::storage::set(&key, &i.to_le_bytes());
            }
        }

        #[block]
        {
            <MigrateV0ToV1<T> as OnRuntimeUpgrade>::on_runtime_upgrade();
        }

        for (_, new_name) in STORAGES {
            let new_prefix = build_prefix(NEW_PALLET, new_name);
            if n > 0 {
                let mut probe = new_prefix.clone();
                probe.extend_from_slice(&0u32.to_le_bytes());
                assert!(sp_io::storage::get(&probe).is_some());
            }
        }
    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}
