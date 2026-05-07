#![cfg_attr(not(feature = "std"), no_std)]
// ============================================================================
//! 全节点发行：全节点PoW铸块奖励发行制度说明（不可治理）
//! ============================================================================
//! 一、制度定位
//! ---------------------------------------------------------------------------
//! 1. 本模块 `fullnode-issuance` 是【系统级、制度性】的货币发行模块；
//! 2. 用于在 Substrate PoW 共识下，对成功铸造新区块的【全节点】发放铸块奖励//!
//! 3. 本模块不属于治理参数范畴，不接受链上治理修改；
//! 4. 本模块仅依赖最小必要 Runtime Storage（真实出块记录、矿工身份到账户钱包绑定表）用于发奖资格判定，其发行金额与高度规则完全由常量决定。
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
//! 2. 本模块使用最小必要 Storage（真实出块记录、矿工身份到账户钱包的一次性绑定表），不记录已发行数量与已奖励区块；
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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, FindAuthor, Imbalance},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::SaturatedConversion;

    // ------------------------------------------------------------------------
    // 全节点 PoW 发行制度常量（来自 primitives）
    // ------------------------------------------------------------------------
    use primitives::pow_const::{
        FULLNODE_BLOCK_REWARD, FULLNODE_REWARD_END_BLOCK, FULLNODE_REWARD_START_BLOCK,
    };

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 公民币货币系统（通常对接 pallet-balances 或自定义币模块）
        type Currency: Currency<Self::AccountId>;

        /// PoW 区块作者查找接口（来自 Substrate 共识层）
        type FindAuthor: FindAuthor<Self::AccountId>;

        /// 权重信息（由 benchmark 自动生成或手动估算）
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 矿工身份账户（powr）到奖励钱包账户的绑定表。
    #[pallet::storage]
    #[pallet::getter(fn reward_wallet_by_miner)]
    pub type RewardWalletByMiner<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, OptionQuery>;

    /// 矿工身份账户（powr）最近一次真实出块的区块高度。
    #[pallet::storage]
    #[pallet::getter(fn last_authored_block_by_miner)]
    pub type LastAuthoredBlockByMiner<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// powr 矿工身份完成一次性钱包绑定。
        RewardWalletBound {
            miner: T::AccountId,
            wallet: T::AccountId,
        },
        /// 本区块 PoW 奖励已发放到绑定钱包。
        FullnodeIssuanceIssued {
            block: u32,
            miner: T::AccountId,
            wallet: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 本区块奖励跳过：未能从 digest 识别出作者。
        FullnodeIssuanceSkippedNoAuthor { block: u32 },
        /// 矿工身份钱包重新绑定。
        RewardWalletRebound {
            miner: T::AccountId,
            new_wallet: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 同一个矿工身份只允许绑定一次奖励钱包。
        RewardWalletAlreadyBound,
        /// 矿工身份未绑定奖励钱包。
        RewardWalletNotBound,
        /// 奖励钱包不得与矿工身份账户相同。
        RewardWalletCannotBeMiner,
        /// 新奖励钱包必须不同于当前已绑定钱包。
        RewardWalletUnchanged,
        /// 矿工身份尚未在链上产生过真实出块记录。
        MinerNeverAuthoredBlock,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 由矿工身份账户（powr 对应账户）发起一次性绑定。
        ///
        /// 注意：绑定资格来自链上真实出块记录，不读取任何节点本地 keystore。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::bind_reward_wallet())]
        pub fn bind_reward_wallet(origin: OriginFor<T>, wallet: T::AccountId) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            ensure!(
                !RewardWalletByMiner::<T>::contains_key(&miner),
                Error::<T>::RewardWalletAlreadyBound
            );
            ensure!(wallet != miner, Error::<T>::RewardWalletCannotBeMiner);
            ensure!(
                LastAuthoredBlockByMiner::<T>::contains_key(&miner),
                Error::<T>::MinerNeverAuthoredBlock
            );

            // 中文注释：绑定表只决定奖励接收钱包，不改变出块作者身份本身。
            RewardWalletByMiner::<T>::insert(&miner, &wallet);
            Self::deposit_event(Event::<T>::RewardWalletBound { miner, wallet });
            Ok(())
        }

        /// 允许矿工身份账户主动重绑奖励钱包（无需治理权限）。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::rebind_reward_wallet())]
        pub fn rebind_reward_wallet(
            origin: OriginFor<T>,
            new_wallet: T::AccountId,
        ) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            let current_wallet =
                RewardWalletByMiner::<T>::get(&miner).ok_or(Error::<T>::RewardWalletNotBound)?;
            ensure!(new_wallet != miner, Error::<T>::RewardWalletCannotBeMiner);
            ensure!(
                new_wallet != current_wallet,
                Error::<T>::RewardWalletUnchanged
            );
            // 中文注释：重绑后仅影响后续区块奖励，历史已经发放的奖励不会被追溯重定向。
            RewardWalletByMiner::<T>::insert(&miner, &new_wallet);
            Self::deposit_event(Event::<T>::RewardWalletRebound { miner, new_wallet });
            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        #[cfg(feature = "std")]
        fn integrity_test() {
            let reward: BalanceOf<T> = FULLNODE_BLOCK_REWARD.saturated_into();
            let reward_back: u128 = reward.saturated_into();
            assert_eq!(
                reward_back, FULLNODE_BLOCK_REWARD,
                "FULLNODE_BLOCK_REWARD must fit into runtime Balance"
            );
        }

        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let block_number = n.saturated_into::<u64>();
            if block_number >= u64::from(FULLNODE_REWARD_START_BLOCK)
                && block_number <= u64::from(FULLNODE_REWARD_END_BLOCK)
            {
                // 预申报 on_finalize 最坏路径预算：
                // digest + 真实出块记录 + wallet map + balances/issuance + event 相关读写
                T::DbWeight::get().reads_writes(3, 4)
            } else {
                Weight::zero()
            }
        }

        fn on_finalize(n: BlockNumberFor<T>) {
            // 中文注释：区间判断使用 u64，避免把运行时 BlockNumber 强绑定为 u32。
            let block_number_u64 = n.saturated_into::<u64>();

            // 是否处于全节点 PoW 奖励区间 [1, 9,999,999]
            if block_number_u64 < u64::from(FULLNODE_REWARD_START_BLOCK)
                || block_number_u64 > u64::from(FULLNODE_REWARD_END_BLOCK)
            {
                return;
            }
            // 中文注释：固定奖励区间本身写死在 u32 范围内，进入区间后再转为存储和事件字段。
            let block_number: u32 = block_number_u64.saturated_into();

            // 从共识 PreRuntime Digest 中获取 PoW 出块作者
            let digest = <frame_system::Pallet<T>>::digest();
            let pre_runtime_digests = digest.logs().iter().filter_map(|d| d.as_pre_runtime());

            let author = match T::FindAuthor::find_author(pre_runtime_digests) {
                Some(a) => a,
                None => {
                    Self::deposit_event(Event::<T>::FullnodeIssuanceSkippedNoAuthor {
                        block: block_number,
                    });
                    return;
                } // 理论上不应发生，发生则不发奖励
            };

            // 中文注释：只有共识 digest 证明真实出过块的账户，才允许后续绑定奖励钱包。
            LastAuthoredBlockByMiner::<T>::insert(&author, block_number);

            // 已绑定钱包则发到钱包，未绑定则默认发到矿工自身账户。
            let recipient =
                RewardWalletByMiner::<T>::get(&author).unwrap_or_else(|| author.clone());

            // 发放固定的全节点 PoW 铸块奖励
            // 中文注释：奖励金额完全由制度常量决定，绑定表只决定”发给谁”，不影响”发多少”。
            let reward: BalanceOf<T> = FULLNODE_BLOCK_REWARD.saturated_into();
            // 中文注释：deposit_creating 会在钱包尚未建户时自动建户，并同步增加总发行量。
            let imbalance = T::Currency::deposit_creating(&recipient, reward);
            debug_assert_eq!(
                imbalance.peek(),
                reward,
                "deposit_creating must return full reward"
            );
            Self::deposit_event(Event::<T>::FullnodeIssuanceIssued {
                block: block_number,
                miner: author,
                wallet: recipient,
                amount: reward,
            });
        }
    }
}

#[cfg(test)]
mod tests;
