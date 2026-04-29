//! 创世模块=genesis-pallet
//!
//! # 职责
//! 1. 存储链的当前运行阶段（创世期 Genesis / 运行期 Operation）及对应参数：
//!    - 出块目标时间（TargetBlockTimeMs）
//!    - 开发者直升 runtime 开关（DeveloperUpgradeEnabled）
//! 2. 存储创世常量（创世宣言、国名宣言、创世人口），在创世区块中初始化。
//!
//! # 设计原则
//! - 纯存储 + getter + trait，不暴露 extrinsic。
//! - 阶段切换仅通过 runtime 升级迁移（OnRuntimeUpgrade）一次性写入，不设链上调用。
//! - 其他模块（难度调整、矿工门控、runtime-upgrade）各自读本模块的链上值。

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod weights;

pub use pallet::*;

// ─── Runtime API ────────────────────────────────────────────────────────────
// 节点层矿工门控通过此 API 读取链上动态出块时间，替代编译期常量。
sp_api::decl_runtime_apis! {
    pub trait GenesisPalletApi {
        /// 返回当前链上出块目标时间（毫秒）。
        fn target_block_time_ms() -> u64;
    }
}

// ─── DeveloperUpgradeCheck trait ────────────────────────────────────────────
// 供 runtime-upgrade 通过关联类型读取开发者直升开关，不硬耦合。
pub trait DeveloperUpgradeCheck {
    /// 开发者直升是否启用。
    fn is_enabled() -> bool;
}

#[frame_support::pallet]
#[allow(dead_code)] // Events 预留给 on_runtime_upgrade 迁移使用，当前 deposit_event 暂未调用
pub mod pallet {
    use alloc::vec::Vec;
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
        type WeightInfo: crate::weights::WeightInfo;

        /// 创世宣言和国名宣言的最大字节长度。
        #[pallet::constant]
        type MaxDeclarationLen: Get<u32>;
    }

    // ─── 类型 ──────────────────────────────────────────────────────────────

    /// 链运行阶段：Genesis（创世期）或 Operation（运行期）。
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
        /// 创世期：单权威、30 秒出块、开发者可直升 runtime。
        #[default]
        Genesis,
        /// 运行期：44 权威、6 分钟出块、升级必须走联合投票。
        Operation,
    }

    // ─── Storage: 链阶段 ─────────────────────────────────────────────────

    /// 当前链阶段。创世默认 Genesis。
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

    // ─── Storage: 创世常量 ───────────────────────────────────────────────

    /// 创世宣言。
    #[pallet::storage]
    #[pallet::getter(fn citizens_declaration)]
    pub type CitizensDeclaration<T: Config> =
        StorageValue<_, BoundedVec<u8, T::MaxDeclarationLen>, ValueQuery>;

    /// 公民宣言。
    #[pallet::storage]
    #[pallet::getter(fn country_declaration)]
    pub type CountryDeclaration<T: Config> =
        StorageValue<_, BoundedVec<u8, T::MaxDeclarationLen>, ValueQuery>;

    /// 创世人口。
    #[pallet::storage]
    #[pallet::getter(fn citizen_max)]
    pub type CitizenMax<T> = StorageValue<_, u64, ValueQuery>;

    // ─── Genesis Config ─────────────────────────────────────────────────

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// 创世宣言（UTF-8 字节）。
        pub citizens_declaration: Vec<u8>,
        /// 公民宣言（UTF-8 字节）。
        pub country_declaration: Vec<u8>,
        /// 创世人口。
        pub citizen_max: u64,
        #[serde(skip)]
        pub _phantom: core::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let citizens: BoundedVec<u8, T::MaxDeclarationLen> = self
                .citizens_declaration
                .clone()
                .try_into()
                .expect("创世宣言超出 MaxDeclarationLen");
            let country: BoundedVec<u8, T::MaxDeclarationLen> = self
                .country_declaration
                .clone()
                .try_into()
                .expect("公民宣言超出 MaxDeclarationLen");
            CitizensDeclaration::<T>::put(citizens);
            CountryDeclaration::<T>::put(country);
            CitizenMax::<T>::put(self.citizen_max);
        }
    }

    // ─── Events ────────────────────────────────────────────────────────────
    // 中文注释：Events 预留给 on_runtime_upgrade 迁移代码使用（阶段切换时触发）。
    // 当前迁移未实现，deposit_event 暂未被调用。

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
        pub type GenesisPallet = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    frame_support::parameter_types! {
        pub const MaxDeclarationLen: u32 = 2048;
    }

    impl pallet::Config for Test {
        type WeightInfo = ();
        type MaxDeclarationLen = MaxDeclarationLen;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        sp_io::TestExternalities::new(storage)
    }

    #[test]
    fn default_phase_is_genesis() {
        new_test_ext().execute_with(|| {
            assert_eq!(GenesisPallet::phase(), pallet::ChainPhase::Genesis);
        });
    }

    #[test]
    fn default_target_block_time_is_30s() {
        new_test_ext().execute_with(|| {
            assert_eq!(GenesisPallet::target_block_time_ms(), 30_000);
        });
    }

    #[test]
    fn default_developer_upgrade_enabled() {
        new_test_ext().execute_with(|| {
            assert!(GenesisPallet::developer_upgrade_enabled());
        });
    }

    #[test]
    fn developer_upgrade_check_trait_reads_storage() {
        new_test_ext().execute_with(|| {
            assert!(<GenesisPallet as DeveloperUpgradeCheck>::is_enabled());

            pallet::DeveloperUpgradeEnabled::<Test>::put(false);
            assert!(!<GenesisPallet as DeveloperUpgradeCheck>::is_enabled());
        });
    }

    #[test]
    fn storage_can_be_switched_to_operation() {
        new_test_ext().execute_with(|| {
            // 模拟 on_runtime_upgrade 迁移写入
            pallet::Phase::<Test>::put(pallet::ChainPhase::Operation);
            pallet::TargetBlockTimeMs::<Test>::put(360_000u64);
            pallet::DeveloperUpgradeEnabled::<Test>::put(false);

            assert_eq!(GenesisPallet::phase(), pallet::ChainPhase::Operation);
            assert_eq!(GenesisPallet::target_block_time_ms(), 360_000);
            assert!(!GenesisPallet::developer_upgrade_enabled());
            assert!(!<GenesisPallet as DeveloperUpgradeCheck>::is_enabled());
        });
    }

    #[test]
    fn genesis_config_initializes_declarations() {
        let citizens = "创世宣言测试".as_bytes().to_vec();
        let country = "公民宣言测试".as_bytes().to_vec();
        let citizen_max = 1_443_497_378u64;

        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");

        pallet::GenesisConfig::<Test> {
            citizens_declaration: citizens.clone(),
            country_declaration: country.clone(),
            citizen_max,
            _phantom: Default::default(),
        }
        .assimilate_storage(&mut storage)
        .expect("genesis config should assimilate");

        sp_io::TestExternalities::new(storage).execute_with(|| {
            assert_eq!(GenesisPallet::citizens_declaration().to_vec(), citizens);
            assert_eq!(GenesisPallet::country_declaration().to_vec(), country);
            assert_eq!(GenesisPallet::citizen_max(), citizen_max);
        });
    }
}
