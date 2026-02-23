#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::Decode;
    use frame_support::{pallet_prelude::*, traits::Currency};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{SaturatedConversion, Saturating, Zero};
    use sp_std::prelude::*;

    // ===== 引入制度常量 =====
    use primitives::{
        core_const::{
            ENABLE_SHENGBANK_INTEREST_DECAY, SHENGBANK_INITIAL_INTEREST_BP,
            SHENGBANK_INTEREST_DECREASE_BP, SHENGBANK_INTEREST_DURATION_YEARS,
        },
        shengbank_nodes_const::CHINACH, // 固定 43 个省储行多签地址
    };

    // ===== 配置 =====
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 原生货币接口
        type Currency: Currency<Self::AccountId>;

        /// 一年对应的区块数（由 runtime 注入）
        #[pallet::constant]
        type BlocksPerYear: Get<u64>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // ===== 存储 =====
    /// 已完成结算的最后年度（0 表示尚未结算任何一年）
    #[pallet::storage]
    #[pallet::getter(fn last_settled_year)]
    pub type LastSettledYear<T> = StorageValue<_, u32, ValueQuery>;

    // ===== Pallet =====
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ===== 事件（审计核心）=====
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 单个省储行收到利息
        ShengBankInterestMinted {
            year: u32,
            pallet_id: Vec<u8>,
            account: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 某一年度结算完成
        ShengBankYearSettled { year: u32 },

        /// 某一年度结算失败（未满足“43个省储行全部成功入账”）
        ShengBankYearSettlementFailed {
            year: u32,
            success_count: u32,
            total_count: u32,
        },
    }

    // ===== Hooks =====
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            // 按区块高度结算：每满一个年度区块数才触发一次（87_600块/年）
            let block = n.saturated_into::<u64>();
            let per_year = T::BlocksPerYear::get();
            let current_year = Self::current_year(n);
            let last_year = Self::last_settled_year();

            // 只在“年度边界区块”触发，且按年度顺序补结算（每次仅推进 1 年），最多结算到制度上限年限
            if per_year > 0
                && block > 0
                && block % per_year == 0
                && current_year > last_year
                && last_year < SHENGBANK_INTEREST_DURATION_YEARS
            {
                let settling_year = last_year + 1;
                log::info!(
                    target: "runtime::shengbank",
                    "省储行利息年度结算开始 | 结算年度={} | 当前年度={} | 区块={:?}",
                    settling_year,
                    current_year,
                    n
                );

                // 中文注释：执行当年利息发放，并返回读写计数+成功数
                let (reads, writes, success_count) = Self::mint_interest_for_year(settling_year);

                // 中文注释：制度要求“43个省储行必须全部成功”，否则本年度不推进结算进度
                let total_count = CHINACH.len() as u32;
                if success_count == total_count {
                    LastSettledYear::<T>::put(settling_year);
                    Self::deposit_event(Event::<T>::ShengBankYearSettled { year: settling_year });

                    return T::DbWeight::get().reads_writes(reads + 1, writes + 1);
                }

                // 中文注释：只要有任一地址发放失败，就记录失败事件并保持 LastSettledYear 不变
                Self::deposit_event(Event::<T>::ShengBankYearSettlementFailed {
                    year: settling_year,
                    success_count,
                    total_count,
                });
                return T::DbWeight::get().reads_writes(reads, writes);
            }

            // 默认只读一次 LastSettledYear
            T::DbWeight::get().reads(1)
        }
    }

    // ===== 核心逻辑 =====
    impl<T: Config> Pallet<T> {
        /// 计算当前区块属于第几年
        fn current_year(block: BlockNumberFor<T>) -> u32 {
            let b = block.saturated_into::<u64>();
            let per_year = T::BlocksPerYear::get();
            if per_year == 0 {
                return 0;
            }
            (b / per_year) as u32
        }

        /// 计算某年的利率（BP，万分比）
        fn interest_bp_for_year(year: u32) -> u32 {
            if !ENABLE_SHENGBANK_INTEREST_DECAY {
                return SHENGBANK_INITIAL_INTEREST_BP;
            }

            if year > SHENGBANK_INTEREST_DURATION_YEARS {
                return 0;
            }

            let decay = year
                .saturating_sub(1)
                .saturating_mul(SHENGBANK_INTEREST_DECREASE_BP);

            SHENGBANK_INITIAL_INTEREST_BP.saturating_sub(decay)
        }

        /// 核心铸造逻辑（只针对 43 个固定省储行多签地址）
        fn mint_interest_for_year(year: u32) -> (u64, u64, u32) {
            // 中文注释：按固定43个省储行估算保守读开销，写开销按成功入账次数累计
            let reads = 1u64 + CHINACH.len() as u64;
            let mut writes = 0u64;
            let mut success_count = 0u32;

            let rate_bp = Self::interest_bp_for_year(year);
            if rate_bp == 0 {
                return (reads, writes, success_count);
            }

            for bank in CHINACH.iter() {
                // 解码省储行交易账户
                let account = match T::AccountId::decode(&mut &bank.pallet_address[..]) {
                    Ok(a) => a,
                    Err(_) => {
                        log::error!(
                            target: "runtime::shengbank",
                            "省储行账户解码失败: {}",
                            bank.pallet_id
                        );
                        continue;
                    }
                };

                let principal: BalanceOf<T> = bank.stake_amount.saturated_into();

                let interest = principal.saturating_mul(rate_bp.into()) / 10_000u32.into();

                if interest.is_zero() {
                    continue;
                }

                // 中文注释：若账户被清理或尚未建户，自动重建对应省储行 pallet_address 后再入账。
                let _imbalance = T::Currency::deposit_creating(&account, interest);
                success_count = success_count.saturating_add(1);
                writes += 1;

                Self::deposit_event(Event::<T>::ShengBankInterestMinted {
                    year,
                    pallet_id: bank.pallet_id.as_bytes().to_vec(),
                    account: account.clone(),
                    amount: interest,
                });
            }

            (reads, writes, success_count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pallet::*;
    use codec::Decode;
    use frame_support::{
        derive_impl,
        traits::{OnFinalize, OnInitialize, VariantCountOf},
    };
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};

    type Block = frame_system::mocking::MockBlock<Test>;
    type Balance = u128;

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
        pub type ShengBankStakeInterest = super;
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

    frame_support::parameter_types! {
        pub const BlocksPerYearForTest: u64 = 10;
    }

    impl Config for Test {
        type Currency = Balances;
        type BlocksPerYear = BlocksPerYearForTest;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    fn run_to_block(n: u64) {
        while System::block_number() < n {
            let b = System::block_number();
            ShengBankStakeInterest::on_finalize(b);
            System::on_finalize(b);
            System::set_block_number(b + 1);
            System::on_initialize(b + 1);
            ShengBankStakeInterest::on_initialize(b + 1);
        }
    }

    fn shengbank_account(index: usize) -> AccountId32 {
        AccountId32::decode(
            &mut &primitives::shengbank_nodes_const::CHINACH[index].pallet_address[..],
        )
            .expect("pallet_address must decode")
    }

    #[test]
    fn first_year_should_mint_and_settle() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(LastSettledYear::<Test>::get(), 1);

            let first_bank = &primitives::shengbank_nodes_const::CHINACH[0];
            let account = shengbank_account(0);
            let expected = first_bank.stake_amount * 100u128 / 10_000u128;
            assert_eq!(Balances::free_balance(account), expected);

            let has_settled_event = System::events().iter().any(|r| {
                matches!(
                    r.event,
                    RuntimeEvent::ShengBankStakeInterest(
                        Event::ShengBankYearSettled { year: 1 }
                    )
                )
            });
            assert!(has_settled_event);
        });
    }

    #[test]
    fn should_backfill_next_unsettled_year_on_later_boundary() {
        new_test_ext().execute_with(|| {
            // 直接跳到第2年边界，验证不会因为 current_year != last+1 而卡死。
            System::set_block_number(20);
            ShengBankStakeInterest::on_initialize(20);

            assert_eq!(LastSettledYear::<Test>::get(), 1);

            let first_bank = &primitives::shengbank_nodes_const::CHINACH[0];
            let account = shengbank_account(0);
            let year1 = first_bank.stake_amount * 100u128 / 10_000u128;
            assert_eq!(Balances::free_balance(account), year1);
        });
    }

    #[test]
    fn second_year_should_use_decayed_rate() {
        new_test_ext().execute_with(|| {
            run_to_block(20);
            assert_eq!(LastSettledYear::<Test>::get(), 2);

            let first_bank = &primitives::shengbank_nodes_const::CHINACH[0];
            let account = shengbank_account(0);
            let year1 = first_bank.stake_amount * 100u128 / 10_000u128;
            let year2 = first_bank.stake_amount * 99u128 / 10_000u128;
            assert_eq!(Balances::free_balance(account), year1 + year2);
        });
    }

    #[test]
    fn should_stop_settling_after_duration_years() {
        new_test_ext().execute_with(|| {
            LastSettledYear::<Test>::put(primitives::core_const::SHENGBANK_INTEREST_DURATION_YEARS);
            let account = shengbank_account(0);
            assert_eq!(Balances::free_balance(account.clone()), 0);

            // current_year = 101（边界块），但因已到年限上限，不应继续发放。
            System::set_block_number(1010);
            ShengBankStakeInterest::on_initialize(1010);

            assert_eq!(
                LastSettledYear::<Test>::get(),
                primitives::core_const::SHENGBANK_INTEREST_DURATION_YEARS
            );
            assert_eq!(Balances::free_balance(account), 0);
        });
    }
}
