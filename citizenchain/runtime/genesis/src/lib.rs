//! 创世模块=genesis-pallet
//!
//! # 职责
//! 1. 存储链的当前运行阶段（创世期 Genesis / 运行期 Operation）及对应参数：
//!    - 出块目标时间（TargetBlockTimeMs）
//!    - 开发者直升 runtime 开关（DeveloperUpgradeEnabled）
//! 2. 存储创世常量（创世宣言、国名宣言、创世人口），在创世区块中初始化。
//! 3. 创世写入内置公权机构和创世公职人员，只写初始 storage，不承载运行期治理。
//!
//! # 设计原则
//! - 纯存储 + getter + trait，不暴露 extrinsic。
//! - 阶段切换仅通过 runtime 升级迁移（OnRuntimeUpgrade）一次性写入，不设链上调用。
//! - 其他模块（难度调整、矿工门控、runtime-upgrade）各自读本模块的链上值。

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod institution;
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
    pub trait Config:
        frame_system::Config<RuntimeEvent: From<Event<Self>>>
        + public_manage::Config
        + public_admins::Config
    {
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
            crate::institution::build::<T>();
        }
    }

    // ─── Events ────────────────────────────────────────────────────────────
    // Events 预留给 on_runtime_upgrade 迁移代码使用（阶段切换时触发）。
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

    // 本 pallet 不暴露 extrinsic。阶段切换仅通过 OnRuntimeUpgrade 迁移执行。
}

// ─── DeveloperUpgradeCheck 实现 ────────────────────────────────────────────
impl<T: pallet::Config> DeveloperUpgradeCheck for pallet::Pallet<T> {
    fn is_enabled() -> bool {
        pallet::DeveloperUpgradeEnabled::<T>::get()
    }
}

#[cfg(test)]
mod tests;
