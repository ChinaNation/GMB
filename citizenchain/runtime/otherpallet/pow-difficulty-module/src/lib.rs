//! PoW 动态难度调整模块=pow-difficulty-module
//!
//! # 设计原理
//! 参考比特币 Nakamoto 难度调整算法：每隔 DIFFICULTY_ADJUSTMENT_INTERVAL 块，
//! 根据窗口期实际出块总时长与目标时长的比值，按比例调整 PoW 挖矿难度。
//!
//! # 调整公式
//! ```text
//! new_difficulty = old_difficulty × (target_window_ms / actual_window_ms)
//! ```
//! 并限制单次调整幅度在 [old/4, old×4] 范围内，防止难度暴涨或暴跌。
//!
//! # 时序说明
//! - 窗口起始时间戳在调整周期首块的 on_finalize 中记录（此时 pallet_timestamp 已完成时间戳注入）。
//! - 窗口终止时间戳在调整周期末块的 on_finalize 中读取并触发调整。
//! - 节点层通过 PowDifficultyApi Runtime API 读取当前链上难度，替代固定常量。
//! - 当前算法只取窗口首尾两个时间点，不对窗口内每一块做采样；因此制度安全仍依赖时间戳 inherent 的有效性。

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

pub use pallet::*;

// PoW 难度 Runtime API：节点层 SimplePow::difficulty() 通过此接口读取链上实时难度。
sp_api::decl_runtime_apis! {
    pub trait PowDifficultyApi {
        /// 返回当前链上 PoW 挖矿难度值。
        fn current_pow_difficulty() -> u64;
    }
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;
    use primitives::pow_const::{
        DIFFICULTY_ADJUSTMENT_INTERVAL, DIFFICULTY_MAX_ADJUST_FACTOR, DIFFICULTY_MIN_ADJUST_FACTOR,
        POW_INITIAL_DIFFICULTY,
    };
    use sp_runtime::traits::SaturatedConversion;

    use crate::weights::WeightInfo as PowDifficultyWeightInfo;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置：需要 frame_system、pallet_timestamp、genesis_pallet 作为超特征。
    /// pallet_timestamp：读取当前块时间戳。
    /// genesis_pallet：读取链上动态出块目标时间（替代编译期常量）。
    #[pallet::config]
    pub trait Config:
        frame_system::Config<RuntimeEvent: From<Event<Self>>>
        + pallet_timestamp::Config
        + genesis_pallet::Config
    {
        type WeightInfo: crate::weights::WeightInfo;
    }

    // ─── Storage ──────────────────────────────────────────────────────────────

    /// 当前 PoW 挖矿难度值。创世时为 POW_INITIAL_DIFFICULTY，此后由调整算法自动维护。
    /// 正常路径下该值必须始终大于 0；若迁移/脏状态把它写成 0，on_finalize 会兜底修复到至少 1。
    #[pallet::storage]
    #[pallet::getter(fn current_difficulty)]
    pub type CurrentDifficulty<T> = StorageValue<_, u64, ValueQuery, DefaultInitialDifficulty>;

    /// 难度初始默认值（ValueQuery 的 OnEmpty 实现）。
    pub struct DefaultInitialDifficulty;
    impl Get<u64> for DefaultInitialDifficulty {
        fn get() -> u64 {
            POW_INITIAL_DIFFICULTY
        }
    }

    /// 当前调整窗口的起始时间戳（毫秒 Unix 时间）。
    /// None 表示本窗口起始时间尚未记录，将在当前周期首块的 on_finalize 中写入。
    #[pallet::storage]
    pub type WindowStartMs<T> = StorageValue<_, u64, OptionQuery>;

    // ─── Events ───────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 难度调整完成。
        /// [触发区块高度, 旧难度, 新难度, 窗口实际耗时ms, 目标窗口时间ms]
        DifficultyAdjusted {
            block: BlockNumberFor<T>,
            old_difficulty: u64,
            new_difficulty: u64,
            actual_window_ms: u64,
            target_window_ms: u64,
        },
    }

    // ─── Hooks ────────────────────────────────────────────────────────────────

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// try-runtime 状态校验：确保 CurrentDifficulty 始终为正数。
        #[cfg(feature = "try-runtime")]
        fn try_state(_n: BlockNumberFor<T>) -> Result<(), sp_runtime::TryRuntimeError> {
            let diff = CurrentDifficulty::<T>::get();
            frame_support::ensure!(diff > 0, "CurrentDifficulty 不得为 0");
            Ok(())
        }

        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let block_num: u32 = n.saturated_into();
            if block_num == 0 {
                return Weight::zero();
            }

            let interval = DIFFICULTY_ADJUSTMENT_INTERVAL;
            // 中文注释：首个调整块是 interval + 1，因为窗口从 block 1 的时间戳开始计时。
            let is_adjustment_block = block_num > 1 && (block_num - 1) % interval == 0;

            if is_adjustment_block {
                // 中文注释：实际重点在 on_finalize，但 FRAME 只能在 on_initialize 预申报预算。
                <T as Config>::WeightInfo::on_initialize_adjustment()
            } else if WindowStartMs::<T>::get().is_none() {
                <T as Config>::WeightInfo::on_initialize_start_window()
            } else {
                <T as Config>::WeightInfo::on_initialize_idle()
            }
        }

        /// on_finalize 在 pallet_timestamp 的时间戳注入（inherent）之后执行，
        /// 因此可安全调用 pallet_timestamp::Pallet::<T>::now() 获取当前块时间戳。
        fn on_finalize(n: BlockNumberFor<T>) {
            let block_num: u32 = n.saturated_into();
            let now_ms: u64 = pallet_timestamp::Pallet::<T>::now().saturated_into();

            // 跳过创世块（block 0 无时间戳注入）
            if now_ms == 0 {
                return;
            }

            // 中文注释：拒绝空块。每个区块至少包含 1 个固有交易（timestamp::set），
            // 若 extrinsic 总数 ≤ 1 说明没有用户交易，属于空块。
            // 创世块（block 0）无时间戳注入，已在上方 now_ms == 0 处跳过。
            if block_num > 0 {
                let extrinsic_count = frame_system::Pallet::<T>::extrinsic_count();
                assert!(
                    extrinsic_count > 1,
                    "空块不允许上链：区块必须包含至少一笔用户交易"
                );
            }

            let interval = DIFFICULTY_ADJUSTMENT_INTERVAL;

            // 以 block 1 的时间戳作为首窗口起点，则首个有效窗口应在 block (interval + 1)
            // 触发调整，确保窗口跨度恰好覆盖 interval 个区块间隔。
            let is_adjustment_block = block_num > 1 && (block_num - 1) % interval == 0;

            if is_adjustment_block {
                // ── 调整块：计算新难度 ────────────────────────────────────────
                if let Some(start_ms) = WindowStartMs::<T>::get() {
                    let actual_window_ms = now_ms.saturating_sub(start_ms).max(1);
                    // 中文注释：从 genesis-pallet 链上存储读取动态出块目标时间，
                    // 替代编译期常量 DIFFICULTY_TARGET_WINDOW_MS。
                    // 中文注释：.max(1) 防御 genesis-pallet 返回 0 导致 target_window_ms 为 0。
                    let target_block_time =
                        genesis_pallet::Pallet::<T>::target_block_time_ms().max(1);
                    let target_window_ms =
                        DIFFICULTY_ADJUSTMENT_INTERVAL as u64 * target_block_time;
                    let old_difficulty = CurrentDifficulty::<T>::get();
                    // 中文注释：正常情况下 old_difficulty 不会为 0；这里做兜底是为了防止
                    // 迁移错误或脏状态把 clamp 的上下界反转，进而在调整块上触发 panic。
                    let calc_difficulty = old_difficulty.max(1);

                    // 新难度 = 旧难度 × (目标时间 / 实际时间)
                    // 出块过快 → actual < target → 新难度升高（更难挖）
                    // 出块过慢 → actual > target → 新难度降低（更易挖）
                    let new_diff_u128 = (calc_difficulty as u128)
                        .saturating_mul(target_window_ms as u128)
                        / actual_window_ms as u128;

                    // 中文注释：单次调整幅度限制按“参与计算的安全难度”夹紧；
                    // 即便存储里出现 0，也只会被修复为 >= 1，而不会把链直接打崩。
                    let max_diff = calc_difficulty.saturating_mul(DIFFICULTY_MAX_ADJUST_FACTOR);
                    let min_diff = (calc_difficulty / DIFFICULTY_MIN_ADJUST_FACTOR).max(1);
                    let new_diff = new_diff_u128.saturated_into::<u64>();
                    let new_difficulty = new_diff.clamp(min_diff, max_diff);

                    CurrentDifficulty::<T>::put(new_difficulty);

                    Self::deposit_event(Event::DifficultyAdjusted {
                        block: n,
                        old_difficulty,
                        new_difficulty,
                        actual_window_ms,
                        target_window_ms,
                    });
                }
                // 以当前调整块时间戳作为下一窗口起点，避免少算 1 个区块间隔。
                WindowStartMs::<T>::put(now_ms);
            } else {
                // ── 非调整块：若窗口起始未记录，以当前块时间戳为起点 ──────────
                if WindowStartMs::<T>::get().is_none() {
                    WindowStartMs::<T>::put(now_ms);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
    /// 测试用目标窗口时长：与 genesis-pallet 默认的 30_000ms 对齐。
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
        #[runtime::pallet_index(3)]
        pub type GenesisPallet = genesis_pallet;
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

    frame_support::parameter_types! {
        pub const MaxDeclarationLen: u32 = 2048;
    }

    impl genesis_pallet::Config for Test {
        type WeightInfo = ();
        type MaxDeclarationLen = MaxDeclarationLen;
    }

    impl Config for Test {
        type WeightInfo = ();
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| {
            // 中文注释：测试环境下把 genesis-pallet 的出块目标时间
            // 与 pow_const::MILLISECS_PER_BLOCK 对齐，确保难度调整逻辑一致。
            genesis_pallet::TargetBlockTimeMs::<Test>::put(MILLISECS_PER_BLOCK);
        });
        ext
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
}
