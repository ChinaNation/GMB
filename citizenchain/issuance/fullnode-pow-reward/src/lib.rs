#![cfg_attr(not(feature = "std"), no_std)]
// ============================================================================
//! 全节点发行：全节点PoW铸块奖励发行制度说明（不可治理）
//! ============================================================================
//! 一、制度定位
//! ---------------------------------------------------------------------------
//! 1. 本模块 `fullnode-pow-reward` 是【系统级、制度性】的货币发行模块；
//! 2. 用于在 Substrate PoW 共识下，对成功铸造新区块的【全节点】发放铸块奖励//!
//! 3. 本模块不属于治理参数范畴，不接受链上治理修改；
//! 4. 本模块仅依赖最小必要 Runtime Storage（矿工身份到账户钱包绑定表）用于发奖资格判定，其发行金额与高度规则完全由常量决定。
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
//! 2. 本模块使用最小必要 Storage（矿工身份到账户钱包的一次性绑定表），不记录已发行数量与已奖励区块；
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
        FULLNODE_BLOCK_REWARD, FULLNODE_REWARD_END_BLOCK, FULLNODE_REWARD_START_BLOCK,
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
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 矿工身份账户（powr）到奖励钱包账户的绑定表。
    #[pallet::storage]
    #[pallet::getter(fn reward_wallet_by_miner)]
    pub type RewardWalletByMiner<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// powr 矿工身份完成一次性钱包绑定。
        RewardWalletBound {
            miner: T::AccountId,
            wallet: T::AccountId,
        },
        /// 本区块 PoW 奖励已发放到绑定钱包。
        PowRewardIssued {
            block: u32,
            miner: T::AccountId,
            wallet: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 同一个矿工身份只允许绑定一次奖励钱包。
        RewardWalletAlreadyBound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 由矿工身份账户（powr 对应账户）发起一次性绑定。
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn bind_reward_wallet(origin: OriginFor<T>, wallet: T::AccountId) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            ensure!(
                !RewardWalletByMiner::<T>::contains_key(&miner),
                Error::<T>::RewardWalletAlreadyBound
            );

            RewardWalletByMiner::<T>::insert(&miner, &wallet);
            Self::deposit_event(Event::<T>::RewardWalletBound { miner, wallet });
            Ok(())
        }
    }

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
            let pre_runtime_digests = digest.logs().iter().filter_map(|d| d.as_pre_runtime());

            let author = match T::FindAuthor::find_author(pre_runtime_digests) {
                Some(a) => a,
                None => return, // 理论上不应发生，发生则不发奖励
            };

            // 仅向已绑定钱包的矿工发放奖励；未绑定则不发放。
            let wallet = match RewardWalletByMiner::<T>::get(&author) {
                Some(w) => w,
                None => return,
            };

            // 发放固定的全节点 PoW 铸块奖励
            let reward: BalanceOf<T> = FULLNODE_BLOCK_REWARD.saturated_into();
            let _imbalance = T::Currency::deposit_creating(&wallet, reward);
            Self::deposit_event(Event::<T>::PowRewardIssued {
                block: block_number,
                miner: author,
                wallet,
                amount: reward,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pallet::*;
    use frame_support::{
        assert_noop, assert_ok, derive_impl,
        traits::{Hooks, VariantCountOf},
    };
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
    use std::{cell::RefCell, thread_local};

    type Block = frame_system::mocking::MockBlock<Test>;
    type Balance = u128;

    thread_local! {
        static MOCK_AUTHOR: RefCell<Option<AccountId32>> = const { RefCell::new(None) };
    }

    pub struct MockFindAuthor;

    impl frame_support::traits::FindAuthor<AccountId32> for MockFindAuthor {
        fn find_author<'a, I>(_digests: I) -> Option<AccountId32>
        where
            I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
        {
            MOCK_AUTHOR.with(|v| v.borrow().clone())
        }
    }

    #[frame_support::runtime]
    mod runtime {
        #[runtime::runtime]
        #[runtime::derive(
            RuntimeCall,
            RuntimeEvent,
            RuntimeError,
            RuntimeOrigin,
            RuntimeFreezeReason,
            RuntimeHoldReason,
            RuntimeSlashReason,
            RuntimeLockId,
            RuntimeTask,
            RuntimeViewFunction
        )]
        pub struct Test;

        #[runtime::pallet_index(0)]
        pub type System = frame_system;
        #[runtime::pallet_index(1)]
        pub type Balances = pallet_balances;
        #[runtime::pallet_index(2)]
        pub type FullnodePowReward = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type AccountData = pallet_balances::AccountData<Balance>;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Nonce = u64;
    }

    impl pallet_balances::Config for Test {
        type MaxLocks = frame_support::traits::ConstU32<0>;
        type MaxReserves = frame_support::traits::ConstU32<0>;
        type ReserveIdentifier = [u8; 8];
        type Balance = Balance;
        type RuntimeEvent = RuntimeEvent;
        type DustRemoval = ();
        type ExistentialDeposit = frame_support::traits::ConstU128<1>;
        type AccountStore = System;
        type WeightInfo = ();
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
    }

    impl Config for Test {
        type Currency = Balances;
        type FindAuthor = MockFindAuthor;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    fn account(n: u8) -> AccountId32 {
        AccountId32::new([n; 32])
    }

    #[test]
    fn bind_reward_wallet_only_once() {
        new_test_ext().execute_with(|| {
            let miner = account(1);
            let wallet = account(2);
            let wallet2 = account(3);

            assert_ok!(FullnodePowReward::bind_reward_wallet(
                RuntimeOrigin::signed(miner.clone()),
                wallet.clone()
            ));
            assert_eq!(RewardWalletByMiner::<Test>::get(&miner), Some(wallet));

            assert_noop!(
                FullnodePowReward::bind_reward_wallet(RuntimeOrigin::signed(miner), wallet2),
                Error::<Test>::RewardWalletAlreadyBound
            );
        });
    }

    #[test]
    fn reward_issued_within_range_when_bound() {
        new_test_ext().execute_with(|| {
            let miner = account(11);
            let wallet = account(22);
            assert_ok!(FullnodePowReward::bind_reward_wallet(
                RuntimeOrigin::signed(miner.clone()),
                wallet.clone()
            ));
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner.clone()));

            // 起始边界块 1 应发放奖励
            <FullnodePowReward as Hooks<u64>>::on_finalize(1);
            assert_eq!(Balances::free_balance(wallet.clone()), primitives::pow_const::FULLNODE_BLOCK_REWARD);

            let has_event = System::events().iter().any(|r| {
                matches!(
                    r.event,
                    RuntimeEvent::FullnodePowReward(Event::PowRewardIssued { block: 1, .. })
                )
            });
            assert!(has_event);
        });
    }

    #[test]
    fn no_reward_when_not_bound() {
        new_test_ext().execute_with(|| {
            let miner = account(33);
            let wallet = account(44);
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner));

            <FullnodePowReward as Hooks<u64>>::on_finalize(1);
            assert_eq!(Balances::free_balance(wallet), 0);
        });
    }

    #[test]
    fn no_reward_outside_reward_range() {
        new_test_ext().execute_with(|| {
            let miner = account(55);
            let wallet = account(66);
            assert_ok!(FullnodePowReward::bind_reward_wallet(
                RuntimeOrigin::signed(miner.clone()),
                wallet.clone()
            ));
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner));

            // 区块 0 不发放
            <FullnodePowReward as Hooks<u64>>::on_finalize(0);
            assert_eq!(Balances::free_balance(wallet.clone()), 0);

            // 超出结束高度不发放
            <FullnodePowReward as Hooks<u64>>::on_finalize(
                (primitives::pow_const::FULLNODE_REWARD_END_BLOCK + 1).into(),
            );
            assert_eq!(Balances::free_balance(wallet), 0);
        });
    }
}
