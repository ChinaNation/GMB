#![cfg(test)]

use super::*;
use codec::Encode;
use frame_support::{
    derive_impl,
    traits::{Hooks, Time},
};
use frame_system as system;
use primitives::pow_const::{
    DIFFICULTY_ADJUSTMENT_INTERVAL, DIFFICULTY_MAX_ADJUST_FACTOR, DIFFICULTY_MIN_ADJUST_FACTOR,
    MILLISECS_PER_BLOCK, POW_INITIAL_DIFFICULTY,
};
use sp_runtime::{traits::IdentityLookup, BuildStorage};

type Block = frame_system::mocking::MockBlock<Test>;
/// 测试用目标窗口时长：与 genesis_pallet-pallet 默认的 30_000ms 对齐。
const DIFFICULTY_TARGET_WINDOW_MS: u64 =
    DIFFICULTY_ADJUSTMENT_INTERVAL as u64 * MILLISECS_PER_BLOCK;
const FIRST_ADJUST_BLOCK: u64 = DIFFICULTY_ADJUSTMENT_INTERVAL as u64 + 1;
const SECOND_ADJUST_BLOCK: u64 = FIRST_ADJUST_BLOCK + DIFFICULTY_ADJUSTMENT_INTERVAL as u64;

#[frame_support::runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeTask,
        RuntimeViewFunction
    )]
    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;
    #[runtime::pallet_index(1)]
    pub type Timestamp = pallet_timestamp;
    #[runtime::pallet_index(2)]
    pub type PowDifficulty = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = frame_support::traits::ConstU64<1>;
    type WeightInfo = ();
}

/// 测试用出块目标时间来源:与 genesis 默认 30_000ms(= MILLISECS_PER_BLOCK)对齐,
/// 通过窄 trait 注入,pow-difficulty 单测因此无需注册 genesis pallet、无需 mock 治理栈。
pub struct MockBlockTime;
impl genesis_pallet::TargetBlockTime for MockBlockTime {
    fn target_block_time_ms() -> u64 {
        MILLISECS_PER_BLOCK
    }
}

impl Config for Test {
    type WeightInfo = ();
    type BlockTime = MockBlockTime;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("frame system genesis_pallet storage should build");
    // 出块目标时间由 MockBlockTime 窄 trait 直接提供(= MILLISECS_PER_BLOCK),
    // 不再依赖 genesis pallet 的 storage,故无需在此写入。
    sp_io::TestExternalities::new(storage)
}

fn run_blocks(count: u32, block_time_ms: u64) {
    for _ in 0..count {
        let block = System::block_number() + 1;
        let now_ms = Timestamp::now().saturating_add(block_time_ms);
        System::set_block_number(block);
        Timestamp::set_timestamp(now_ms);
        // 模拟区块含有 2 个 extrinsic（1 inherent + 1 用户交易），绕过空块拒绝检查
        // 模拟 2 个 extrinsic（1 inherent + 1 用户交易）
        sp_io::storage::set(
            frame_support::storage::storage_prefix(b"System", b"ExtrinsicCount").as_ref(),
            &2u32.encode(),
        );
        PowDifficulty::on_finalize(block);
    }
}

fn difficulty_adjusted_events() -> Vec<Event<Test>> {
    System::events()
        .into_iter()
        .filter_map(|r| match r.event {
            RuntimeEvent::PowDifficulty(event) => Some(event),
            _ => None,
        })
        .collect()
}

#[test]
fn first_adjustment_happens_at_interval_plus_one_and_window_is_exact() {
    new_test_ext().execute_with(|| {
        run_blocks(DIFFICULTY_ADJUSTMENT_INTERVAL, MILLISECS_PER_BLOCK);
        assert_eq!(PowDifficulty::current_difficulty(), POW_INITIAL_DIFFICULTY);
        assert!(difficulty_adjusted_events().is_empty());

        run_blocks(1, MILLISECS_PER_BLOCK);
        assert_eq!(PowDifficulty::current_difficulty(), POW_INITIAL_DIFFICULTY);

        System::assert_last_event(RuntimeEvent::PowDifficulty(Event::DifficultyAdjusted {
            block: FIRST_ADJUST_BLOCK,
            old_difficulty: POW_INITIAL_DIFFICULTY,
            new_difficulty: POW_INITIAL_DIFFICULTY,
            actual_window_ms: DIFFICULTY_TARGET_WINDOW_MS,
            target_window_ms: DIFFICULTY_TARGET_WINDOW_MS,
        }));

        assert_eq!(WindowStartMs::<Test>::get(), Some(Timestamp::now()));
    });
}

#[test]
fn raises_difficulty_when_blocks_are_too_fast() {
    new_test_ext().execute_with(|| {
        run_blocks(DIFFICULTY_ADJUSTMENT_INTERVAL + 1, MILLISECS_PER_BLOCK / 2);
        assert_eq!(
            PowDifficulty::current_difficulty(),
            POW_INITIAL_DIFFICULTY * 2
        );
    });
}

#[test]
fn lowers_difficulty_when_blocks_are_too_slow() {
    new_test_ext().execute_with(|| {
        run_blocks(DIFFICULTY_ADJUSTMENT_INTERVAL + 1, MILLISECS_PER_BLOCK * 2);
        assert_eq!(
            PowDifficulty::current_difficulty(),
            POW_INITIAL_DIFFICULTY / 2
        );
    });
}

#[test]
fn clamps_to_adjustment_bounds() {
    new_test_ext().execute_with(|| {
        let old = 100u64;
        CurrentDifficulty::<Test>::put(old);
        WindowStartMs::<Test>::put(999);
        System::set_block_number(FIRST_ADJUST_BLOCK);
        Timestamp::set_timestamp(1_000);
        // 模拟 2 个 extrinsic（1 inherent + 1 用户交易）
        sp_io::storage::set(
            frame_support::storage::storage_prefix(b"System", b"ExtrinsicCount").as_ref(),
            &2u32.encode(),
        );
        PowDifficulty::on_finalize(FIRST_ADJUST_BLOCK);
        assert_eq!(
            PowDifficulty::current_difficulty(),
            old * DIFFICULTY_MAX_ADJUST_FACTOR
        );

        CurrentDifficulty::<Test>::put(old);
        WindowStartMs::<Test>::put(0);
        System::set_block_number(SECOND_ADJUST_BLOCK);
        Timestamp::set_timestamp(1_000_000_000);
        // 模拟 2 个 extrinsic（1 inherent + 1 用户交易）
        sp_io::storage::set(
            frame_support::storage::storage_prefix(b"System", b"ExtrinsicCount").as_ref(),
            &2u32.encode(),
        );
        PowDifficulty::on_finalize(SECOND_ADJUST_BLOCK);
        assert_eq!(
            PowDifficulty::current_difficulty(),
            old / DIFFICULTY_MIN_ADJUST_FACTOR
        );
    });
}

#[test]
fn saturating_cast_prevents_u128_to_u64_wraparound() {
    new_test_ext().execute_with(|| {
        CurrentDifficulty::<Test>::put(u64::MAX - 1);
        WindowStartMs::<Test>::put(999);
        System::set_block_number(FIRST_ADJUST_BLOCK);
        Timestamp::set_timestamp(1_000);
        // 模拟 2 个 extrinsic（1 inherent + 1 用户交易）
        sp_io::storage::set(
            frame_support::storage::storage_prefix(b"System", b"ExtrinsicCount").as_ref(),
            &2u32.encode(),
        );

        PowDifficulty::on_finalize(FIRST_ADJUST_BLOCK);

        assert_eq!(PowDifficulty::current_difficulty(), u64::MAX);
    });
}

#[test]
fn zero_difficulty_storage_is_repaired_without_panic() {
    new_test_ext().execute_with(|| {
        CurrentDifficulty::<Test>::put(0);
        WindowStartMs::<Test>::put(0);
        System::set_block_number(FIRST_ADJUST_BLOCK);
        Timestamp::set_timestamp(DIFFICULTY_TARGET_WINDOW_MS);
        // 模拟 2 个 extrinsic（1 inherent + 1 用户交易）
        sp_io::storage::set(
            frame_support::storage::storage_prefix(b"System", b"ExtrinsicCount").as_ref(),
            &2u32.encode(),
        );

        PowDifficulty::on_finalize(FIRST_ADJUST_BLOCK);

        assert_eq!(PowDifficulty::current_difficulty(), 1);
        System::assert_last_event(RuntimeEvent::PowDifficulty(Event::DifficultyAdjusted {
            block: FIRST_ADJUST_BLOCK,
            old_difficulty: 0,
            new_difficulty: 1,
            actual_window_ms: DIFFICULTY_TARGET_WINDOW_MS,
            target_window_ms: DIFFICULTY_TARGET_WINDOW_MS,
        }));
    });
}

#[test]
#[should_panic(expected = "空块不允许上链")]
fn rejects_empty_block() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(30_000);
        // 测试环境 extrinsic_count 为 0，触发空块拒绝
        PowDifficulty::on_finalize(1);
    });
}
