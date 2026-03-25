//! 链阶段控制模块=chain-phase-control
//!
//! # 职责
//! 存储链的当前运行阶段（Development / Production）及对应参数：
//! - 出块目标时间（TargetBlockTimeMs）
//! - 开发者直升 runtime 开关（DeveloperUpgradeEnabled）
//!
//! # 设计原则
//! - 纯存储 + getter + trait，不暴露 extrinsic。
//! - 阶段切换仅通过 runtime 升级迁移（OnRuntimeUpgrade）一次性写入，不设链上调用。
//! - 其他模块（难度调整、矿工门控、runtime-root-upgrade）各自读本模块的链上值。

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

pub use pallet::*;

// ─── Runtime API ────────────────────────────────────────────────────────────
// 节点层矿工门控通过此 API 读取链上动态出块时间，替代编译期常量。
sp_api::decl_runtime_apis! {
    pub trait ChainPhaseApi {
        /// 返回当前链上出块目标时间（毫秒）。
        fn target_block_time_ms() -> u64;
    }
}

// ─── DeveloperUpgradeCheck trait ────────────────────────────────────────────
// 供 runtime-root-upgrade 通过关联类型读取开发者直升开关，不硬耦合。
pub trait DeveloperUpgradeCheck {
    /// 开发者直升是否启用。
    fn is_enabled() -> bool;
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
        type WeightInfo: crate::weights::WeightInfo;
    }

    // ─── 类型 ──────────────────────────────────────────────────────────────

    /// 链运行阶段：Development（开发期）或 Production（运行期）。
    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        Default,
    )]
    pub enum ChainPhase {
        /// 开发期：单权威、30 秒出块、开发者可直升 runtime。
        #[default]
        Development,
        /// 运行期：44 权威、6 分钟出块、升级必须走联合投票。
        Production,
    }

    // ─── Storage ───────────────────────────────────────────────────────────

    /// 当前链阶段。创世默认 Development。
    #[pallet::storage]
    #[pallet::getter(fn phase)]
    pub type Phase<T> = StorageValue<_, ChainPhase, ValueQuery>;

    /// 出块目标时间（毫秒）。创世默认 30,000（30 秒）。
    #[pallet::storage]
    #[pallet::getter(fn target_block_time_ms)]
    pub type TargetBlockTimeMs<T> = StorageValue<_, u64, ValueQuery, DefaultTargetBlockTime>;

    /// 出块目标时间默认值（ValueQuery 的 OnEmpty 实现）。
    pub struct DefaultTargetBlockTime;
    impl Get<u64> for DefaultTargetBlockTime {
        fn get() -> u64 {
            30_000
        }
    }

    /// 开发者直升 runtime 开关。创世默认 true（开启）。
    #[pallet::storage]
    #[pallet::getter(fn developer_upgrade_enabled)]
    pub type DeveloperUpgradeEnabled<T> = StorageValue<_, bool, ValueQuery, DefaultDevUpgrade>;

    /// 开发者直升默认值。
    pub struct DefaultDevUpgrade;
    impl Get<bool> for DefaultDevUpgrade {
        fn get() -> bool {
            true
        }
    }

    // ─── Events ────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 链阶段已切换。
        PhaseChanged { from: ChainPhase, to: ChainPhase },
        /// 出块目标时间已变更。
        TargetBlockTimeChanged { old_ms: u64, new_ms: u64 },
        /// 开发者直升开关已变更。
        DeveloperUpgradeToggled { enabled: bool },
    }

    // 中文注释：本 pallet 不暴露 extrinsic。阶段切换仅通过 OnRuntimeUpgrade 迁移执行。
}

// ─── DeveloperUpgradeCheck 实现 ────────────────────────────────────────────
impl<T: pallet::Config> DeveloperUpgradeCheck for pallet::Pallet<T> {
    fn is_enabled() -> bool {
        pallet::DeveloperUpgradeEnabled::<T>::get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::derive_impl;
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, BuildStorage};

    type Block = frame_system::mocking::MockBlock<Test>;

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
        pub type ChainPhaseControl = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    impl pallet::Config for Test {
        type WeightInfo = ();
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        sp_io::TestExternalities::new(storage)
    }

    #[test]
    fn default_phase_is_development() {
        new_test_ext().execute_with(|| {
            assert_eq!(ChainPhaseControl::phase(), pallet::ChainPhase::Development);
        });
    }

    #[test]
    fn default_target_block_time_is_30s() {
        new_test_ext().execute_with(|| {
            assert_eq!(ChainPhaseControl::target_block_time_ms(), 30_000);
        });
    }

    #[test]
    fn default_developer_upgrade_enabled() {
        new_test_ext().execute_with(|| {
            assert!(ChainPhaseControl::developer_upgrade_enabled());
        });
    }

    #[test]
    fn developer_upgrade_check_trait_reads_storage() {
        new_test_ext().execute_with(|| {
            assert!(<ChainPhaseControl as DeveloperUpgradeCheck>::is_enabled());

            pallet::DeveloperUpgradeEnabled::<Test>::put(false);
            assert!(!<ChainPhaseControl as DeveloperUpgradeCheck>::is_enabled());
        });
    }

    #[test]
    fn storage_can_be_switched_to_production() {
        new_test_ext().execute_with(|| {
            // 模拟 on_runtime_upgrade 迁移写入
            pallet::Phase::<Test>::put(pallet::ChainPhase::Production);
            pallet::TargetBlockTimeMs::<Test>::put(360_000u64);
            pallet::DeveloperUpgradeEnabled::<Test>::put(false);

            assert_eq!(ChainPhaseControl::phase(), pallet::ChainPhase::Production);
            assert_eq!(ChainPhaseControl::target_block_time_ms(), 360_000);
            assert!(!ChainPhaseControl::developer_upgrade_enabled());
            assert!(!<ChainPhaseControl as DeveloperUpgradeCheck>::is_enabled());
        });
    }
}
