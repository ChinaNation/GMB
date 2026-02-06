#![cfg_attr(not(feature = "std"), no_std)]
// ============================================================================
//! 全节点发行：全节点PoW铸块奖励发行制度说明（不可治理）
//! ============================================================================
//! 一、制度定位
//! ---------------------------------------------------------------------------
//! 1. 本模块 `fullnode-pow-reward` 是【系统级、制度性】的货币发行模块；
//! 2. 用于在 Substrate PoW 共识下，对成功铸造新区块的【全节点】发放铸块奖励//!
//! 3. 本模块不属于治理参数范畴，不接受链上治理修改，不依赖任何 Runtime Storage 状态，其发行规则完全由常量与区块高度决定。
//!
//! 二、发行规则（写死于 primitives::pow_const）
//! ---------------------------------------------------------------------------
//! 1. 单块奖励金额：每成功铸造 1 个新区块，系统铸造并发放：9999.00 元数字公民币（即 999,900 分）；
//! 2. 奖励区块高度区间（含首尾）：起始区块高度：1 ，结束区块高度：9,999,999；
//! 3. 发行终止规则（永久）：当区块高度 > 9,999,999 时：系统永久停止全节点 PoW 铸块奖励发行，后续新区块不再产生任何全节点铸块奖励；
//! 4. 发行总量（仅用于审计）：总发行区块数：9,999,999 个，全节点铸块奖励总发行量：999,900 × 9,999,999 = 99,989,990,001.00 元。
//!
//! 三、技术实现原则
//! ---------------------------------------------------------------------------
//! 1. 本模块不参与PoW共识过程，仅消费共识结果，PoW共识由Substrate框架原生实现，通过PreRuntime Digest + FindAuthor获取区块作者；
//! 2. 本模块不使用任何 Storage，不记录已发行数量、不记录已奖励区块、不维护任何可变状态；
//! 3. 奖励发放时机：奖励在区块执行完成后的 on_finalize 阶段发放，属于对“已完成铸块行为”的结算，而非预测性激励；
//! 4. 区块高度作为唯一时间与次数约束，区块高度全网一致、不可篡改的事实状态，不依赖任何人为或治理输入。
//!
//! 四、不可改动声明（对以下内容的任何修改，都会构成【货币发行制度的根本性变更】）
//! ---------------------------------------------------------------------------
//! 1. 单块奖励金额；
//! 2. 奖励起止区块高度；
//! 3. 永久停止发行的规则；
//! 4. 发行触发条件（PoW 铸块）。
//!                                上述内容不得修改//!
//! ============================================================================

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, FindAuthor},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::SaturatedConversion;

    // ------------------------------------------------------------------------
    // 全节点 PoW 发行制度常量（来自 primitives）
    // ------------------------------------------------------------------------
    use primitives::pow_const::{
        FULLNODE_BLOCK_REWARD,
        FULLNODE_REWARD_START_BLOCK,
        FULLNODE_REWARD_END_BLOCK,
    };

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 公民币货币系统（通常对接 pallet-balances 或自定义币模块）
        type Currency: Currency<Self::AccountId>;

        /// PoW 区块作者查找接口（来自 Substrate 共识层）
        type FindAuthor: FindAuthor<Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<
            <T as frame_system::Config>::AccountId,
        >>::Balance;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(n: BlockNumberFor<T>) {
            // 制度前提：本链区块高度使用 u32 表示
            let block_number: u32 = n.saturated_into::<u32>();

            // 是否处于全节点 PoW 奖励区间 [1, 9,999,999]
            if block_number < FULLNODE_REWARD_START_BLOCK
                || block_number > FULLNODE_REWARD_END_BLOCK
            {
                return;
            }

            // 从共识 PreRuntime Digest 中获取 PoW 出块作者
            let digest = <frame_system::Pallet<T>>::digest();
            let pre_runtime_digests =
                digest.logs().iter().filter_map(|d| d.as_pre_runtime());

            let author = match T::FindAuthor::find_author(pre_runtime_digests) {
                Some(a) => a,
                None => return, // 理论上不应发生，发生则不发奖励
            };

            // 发放固定的全节点 PoW 铸块奖励
            let reward: BalanceOf<T> = FULLNODE_BLOCK_REWARD.saturated_into();
            T::Currency::deposit_creating(&author, reward);
        }
    }
}