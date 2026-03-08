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

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

/// PoW 难度 Runtime API：节点层 SimplePow::difficulty() 通过此接口读取链上实时难度。
sp_api::decl_runtime_apis! {
    pub trait PowDifficultyApi {
        /// 返回当前链上 PoW 挖矿难度值。
        fn current_pow_difficulty() -> u64;
    }
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use primitives::pow_const::{
        DIFFICULTY_ADJUSTMENT_INTERVAL, DIFFICULTY_MAX_ADJUST_FACTOR, DIFFICULTY_MIN_ADJUST_FACTOR,
        DIFFICULTY_TARGET_WINDOW_MS, POW_INITIAL_DIFFICULTY,
    };
    use sp_runtime::traits::SaturatedConversion;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置：需要 frame_system 与 pallet_timestamp 作为超特征，
    /// 以便通过 pallet_timestamp::Pallet::<T>::now() 读取当前块时间戳。
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    // ─── Storage ──────────────────────────────────────────────────────────────

    /// 当前 PoW 挖矿难度值。创世时为 POW_INITIAL_DIFFICULTY，此后由调整算法自动维护。
    #[pallet::storage]
    #[pallet::getter(fn current_difficulty)]
    pub type CurrentDifficulty<T> =
        StorageValue<_, u64, ValueQuery, DefaultInitialDifficulty>;

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
        /// on_finalize 在 pallet_timestamp 的时间戳注入（inherent）之后执行，
        /// 因此可安全调用 pallet_timestamp::Pallet::<T>::now() 获取当前块时间戳。
        fn on_finalize(n: BlockNumberFor<T>) {
            let block_num: u32 = n.saturated_into();
            let now_ms: u64 = pallet_timestamp::Pallet::<T>::now().saturated_into();

            // 跳过创世块（block 0 无时间戳注入）
            if now_ms == 0 {
                return;
            }

            let interval = DIFFICULTY_ADJUSTMENT_INTERVAL;

            if block_num > 0 && block_num % interval == 0 {
                // ── 调整块：计算新难度 ────────────────────────────────────────
                if let Some(start_ms) = WindowStartMs::<T>::get() {
                    let actual_window_ms = now_ms.saturating_sub(start_ms).max(1);
                    let target_window_ms = DIFFICULTY_TARGET_WINDOW_MS;
                    let old_difficulty = CurrentDifficulty::<T>::get();

                    // 新难度 = 旧难度 × (目标时间 / 实际时间)
                    // 出块过快 → actual < target → 新难度升高（更难挖）
                    // 出块过慢 → actual > target → 新难度降低（更易挖）
                    let new_diff_u128 = (old_difficulty as u128)
                        .saturating_mul(target_window_ms as u128)
                        / actual_window_ms as u128;

                    // 单次调整幅度限制：[old/4, old×4]
                    let max_diff = old_difficulty.saturating_mul(DIFFICULTY_MAX_ADJUST_FACTOR);
                    let min_diff = (old_difficulty / DIFFICULTY_MIN_ADJUST_FACTOR).max(1);
                    let new_difficulty = (new_diff_u128 as u64).clamp(min_diff, max_diff);

                    CurrentDifficulty::<T>::put(new_difficulty);

                    Self::deposit_event(Event::DifficultyAdjusted {
                        block: n,
                        old_difficulty,
                        new_difficulty,
                        actual_window_ms,
                        target_window_ms,
                    });
                }
                // 重置窗口：下一块的 on_finalize 会重新记录新窗口起始时间戳
                WindowStartMs::<T>::kill();
            } else {
                // ── 非调整块：若窗口起始未记录，以当前块时间戳为起点 ──────────
                if WindowStartMs::<T>::get().is_none() {
                    WindowStartMs::<T>::put(now_ms);
                }
            }
        }
    }
}
