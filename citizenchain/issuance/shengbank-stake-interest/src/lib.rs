#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::Decode;
    use frame_support::{ensure, pallet_prelude::*, traits::Currency, Blake2_128Concat};
    use frame_system::{ensure_root, pallet_prelude::*};
    use sp_runtime::traits::{SaturatedConversion, Saturating, Zero};
    use sp_std::prelude::*;

    // ===== 引入制度常量 =====
    use primitives::{
        core_const::{
            ENABLE_SHENGBANK_INTEREST_DECAY, SHENGBANK_INITIAL_INTEREST_BP,
            SHENGBANK_INTEREST_DECREASE_BP, SHENGBANK_INTEREST_DURATION_YEARS,
        },
        china::china_ch::{shenfen_id_to_fixed48, CHINA_CH}, // 固定 43 个省储行多签地址
    };

    const AUTO_BACKFILL_MAX_YEARS_PER_BLOCK: u32 = 8;
    const MAX_FORCE_SETTLE_YEARS: u32 = SHENGBANK_INTEREST_DURATION_YEARS;
    const SETTLEMENT_CPU_OP_WEIGHT: u64 = 50_000;

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

    /// 省储行账户覆盖映射：允许治理在不升级 runtime 的前提下更换收款账户。
    #[pallet::storage]
    pub type ShengBankAccountOverrides<T: Config> =
        StorageMap<_, Blake2_128Concat, [u8; 48], T::AccountId, OptionQuery>;

    // ===== Pallet =====
    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // ===== 事件（审计核心）=====
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 单个省储行收到利息
        ShengBankInterestMinted {
            year: u32,
            pallet_id: [u8; 48],
            account: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 省储行账户解码失败或配置无效（链上可审计，不依赖节点日志）。
        ShengBankDecodeFailed {
            year: u32,
            pallet_id: [u8; 48],
        },

        /// shenfen_id 无法编码为固定 48 字节标识（链上配置异常）。
        ShengBankIdEncodeFailed { year: u32, index: u32 },

        /// stake_amount 转换到运行时 Balance 发生饱和截断。
        ShengBankPrincipalOverflow {
            year: u32,
            pallet_id: [u8; 48],
        },

        /// 某一年度结算完成
        ShengBankYearSettled { year: u32 },

        /// 某一年度结算失败（未满足“43个省储行全部成功入账”）
        ShengBankYearSettlementFailed {
            year: u32,
            success_count: u32,
            total_count: u32,
        },

        /// 由 Root 强制推进年度（跳过故障年度）。
        ShengBankYearForceAdvanced { year: u32 },

        /// 省储行收款账户覆盖已更新。
        ShengBankAccountOverrideSet { pallet_id: [u8; 48] },

        /// 省储行收款账户覆盖已清除。
        ShengBankAccountOverrideCleared { pallet_id: [u8; 48] },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidOperationCount,
        InvalidYear,
        UnknownShengBankId,
    }

    // ===== Hooks =====
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            // 按区块高度结算：每满一个年度区块数才触发一次（87_600块/年）
            let block = n.saturated_into::<u64>();
            let per_year = T::BlocksPerYear::get();

            // 快速跳过：非年度边界块无需读取存储。
            if per_year == 0 || block == 0 || block % per_year != 0 {
                return Weight::zero();
            }

            let current_year = Self::current_year(n);
            let last_year = Self::last_settled_year();

            // 只在“年度边界区块”触发，按年度顺序自动补结算，最多结算到制度上限年限。
            if current_year > last_year && last_year < SHENGBANK_INTEREST_DURATION_YEARS {
                let (reads, writes, ops) = Self::settle_next_years(
                    current_year,
                    AUTO_BACKFILL_MAX_YEARS_PER_BLOCK,
                    Some(n),
                );
                return T::DbWeight::get()
                    .reads_writes(reads, writes)
                    .saturating_add(Weight::from_parts(
                        ops.saturating_mul(SETTLEMENT_CPU_OP_WEIGHT),
                        0,
                    ));
            }

            // 默认只读一次 LastSettledYear
            T::DbWeight::get().reads(1)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Root 手动补结算指定年数（用于故障恢复或快速追平）。
        #[pallet::call_index(0)]
        #[pallet::weight(
            T::DbWeight::get()
                .reads_writes(
                    1 + (*max_years as u64) * (1 + CHINA_CH.len() as u64 * 3),
                    1 + (*max_years as u64) * (2 + CHINA_CH.len() as u64 * 3),
                )
                .saturating_add(Weight::from_parts(
                    (*max_years as u64)
                        .saturating_mul(CHINA_CH.len() as u64)
                        .saturating_mul(SETTLEMENT_CPU_OP_WEIGHT),
                    0
                ))
        )]
        pub fn force_settle_years(
            origin: OriginFor<T>,
            max_years: u32,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(
                max_years > 0 && max_years <= MAX_FORCE_SETTLE_YEARS,
                Error::<T>::InvalidOperationCount
            );
            let current_year = Self::current_year(frame_system::Pallet::<T>::block_number());
            let (reads, writes, ops) = Self::settle_next_years(current_year, max_years, None);
            let actual_weight = T::DbWeight::get()
                .reads_writes(reads, writes)
                .saturating_add(Weight::from_parts(
                    ops.saturating_mul(SETTLEMENT_CPU_OP_WEIGHT),
                    0,
                ));
            log::debug!(
                target: "runtime::shengbank",
                "force_settle_years finished | reads={} writes={} ops={}",
                reads,
                writes,
                ops
            );
            Ok(Some(actual_weight).into())
        }

        /// Root 强制推进到指定年度（跳过无法修复的失败年度）。
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn force_advance_year(origin: OriginFor<T>, year: u32) -> DispatchResult {
            ensure_root(origin)?;
            let current = Self::last_settled_year();
            ensure!(
                year > current && year <= SHENGBANK_INTEREST_DURATION_YEARS,
                Error::<T>::InvalidYear
            );
            LastSettledYear::<T>::put(year);
            Self::deposit_event(Event::<T>::ShengBankYearForceAdvanced { year });
            Ok(())
        }

        /// Root 设置省储行收款账户覆盖值。
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_shengbank_account_override(
            origin: OriginFor<T>,
            pallet_id: [u8; 48],
            account: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(Self::is_known_shengbank_id(pallet_id), Error::<T>::UnknownShengBankId);
            ShengBankAccountOverrides::<T>::insert(pallet_id, account);
            Self::deposit_event(Event::<T>::ShengBankAccountOverrideSet { pallet_id });
            Ok(())
        }

        /// Root 清除省储行收款账户覆盖值。
        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn clear_shengbank_account_override(
            origin: OriginFor<T>,
            pallet_id: [u8; 48],
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(Self::is_known_shengbank_id(pallet_id), Error::<T>::UnknownShengBankId);
            ShengBankAccountOverrides::<T>::remove(pallet_id);
            Self::deposit_event(Event::<T>::ShengBankAccountOverrideCleared { pallet_id });
            Ok(())
        }
    }

    // ===== 核心逻辑 =====
    impl<T: Config> Pallet<T> {
        fn is_known_shengbank_id(pallet_id: [u8; 48]) -> bool {
            CHINA_CH.iter().any(|bank| {
                shenfen_id_to_fixed48(bank.shenfen_id)
                    .map(|id| id == pallet_id)
                    .unwrap_or(false)
            })
        }

        fn resolve_bank_account(
            year: u32,
            bank: &primitives::china::china_ch::ChinaCh,
            pallet_id: [u8; 48],
        ) -> Option<T::AccountId> {
            if let Some(override_account) = ShengBankAccountOverrides::<T>::get(pallet_id) {
                return Some(override_account);
            }
            match T::AccountId::decode(&mut &bank.duoqian_address[..]) {
                Ok(a) => Some(a),
                Err(_) => {
                    Self::deposit_event(Event::<T>::ShengBankDecodeFailed { year, pallet_id });
                    log::error!(
                        target: "runtime::shengbank",
                        "省储行账户解码失败: {}",
                        bank.shenfen_id
                    );
                    None
                }
            }
        }

        fn settle_next_years(
            current_year: u32,
            max_years: u32,
            block: Option<BlockNumberFor<T>>,
        ) -> (u64, u64, u64) {
            let mut reads: u64 = 1;
            let mut writes: u64 = 0;
            let mut ops: u64 = 0;
            let mut iterations: u32 = 0;
            let mut last_year = Self::last_settled_year();
            while last_year < current_year
                && last_year < SHENGBANK_INTEREST_DURATION_YEARS
                && iterations < max_years
            {
                let settling_year = last_year + 1;
                if let Some(n) = block {
                    log::info!(
                        target: "runtime::shengbank",
                        "省储行利息年度结算开始 | 结算年度={} | 当前年度={} | 区块={:?}",
                        settling_year,
                        current_year,
                        n
                    );
                }
                let (year_reads, year_writes, success_count, total_count) =
                    Self::mint_interest_for_year(settling_year);
                reads = reads.saturating_add(year_reads);
                writes = writes.saturating_add(year_writes);
                ops = ops.saturating_add(CHINA_CH.len() as u64);
                if success_count == total_count {
                    LastSettledYear::<T>::put(settling_year);
                    writes = writes.saturating_add(1);
                    Self::deposit_event(Event::<T>::ShengBankYearSettled { year: settling_year });
                    writes = writes.saturating_add(1);
                    last_year = settling_year;
                    iterations = iterations.saturating_add(1);
                    continue;
                }
                Self::deposit_event(Event::<T>::ShengBankYearSettlementFailed {
                    year: settling_year,
                    success_count,
                    total_count,
                });
                writes = writes.saturating_add(1);
                break;
            }
            (reads, writes, ops)
        }

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

        /// 核心铸造逻辑（只针对固定省储行地址，支持链上覆盖地址）。
        fn mint_interest_for_year(year: u32) -> (u64, u64, u32, u32) {
            // 中文注释：保守估算每家省储行读：
            // - 覆盖映射读取
            // - 账户余额读取
            // - 总发行量读取
            // - 账户存在性相关读取
            let reads = CHINA_CH.len() as u64 * 4;
            let mut writes = 0u64;
            let mut success_count = 0u32;
            let total_count = CHINA_CH.len() as u32;

            let rate_bp = Self::interest_bp_for_year(year);
            if rate_bp == 0 {
                return (0, writes, total_count, total_count);
            }

            for (idx, bank) in CHINA_CH.iter().enumerate() {
                let Some(pallet_id) = shenfen_id_to_fixed48(bank.shenfen_id) else {
                    Self::deposit_event(Event::<T>::ShengBankIdEncodeFailed {
                        year,
                        index: idx as u32,
                    });
                    log::error!(
                        target: "runtime::shengbank",
                        "shenfen_id 转换失败: {}",
                        bank.shenfen_id
                    );
                    writes = writes.saturating_add(1);
                    continue;
                };
                let Some(account) = Self::resolve_bank_account(year, bank, pallet_id) else {
                    writes = writes.saturating_add(1); // decode-failed event write
                    continue;
                };

                let principal: BalanceOf<T> = bank.stake_amount.saturated_into();
                let principal_back: u128 = principal.saturated_into();
                if principal_back != bank.stake_amount {
                    Self::deposit_event(Event::<T>::ShengBankPrincipalOverflow { year, pallet_id });
                    log::error!(
                        target: "runtime::shengbank",
                        "stake_amount 饱和截断: {}",
                        bank.shenfen_id
                    );
                    writes = writes.saturating_add(1);
                    continue;
                }

                let interest = principal.saturating_mul(rate_bp.into()) / 10_000u32.into();

                if interest.is_zero() {
                    // 利息为0不视为失败，避免把“无应付利息”误判成年度失败。
                    success_count = success_count.saturating_add(1);
                    continue;
                }
                if interest < T::Currency::minimum_balance() {
                    log::warn!(
                        target: "runtime::shengbank",
                        "省储行利息低于 ED，跳过: {}",
                        bank.shenfen_id
                    );
                    success_count = success_count.saturating_add(1);
                    continue;
                }

                // 中文注释：若账户被清理或尚未建户，自动重建对应省储行 pallet_address 后再入账。
                // 中文注释：deposit_creating 返回的 imbalance 在离开作用域时结算，等价于确认增发入账。
                let _imbalance = T::Currency::deposit_creating(&account, interest);
                success_count = success_count.saturating_add(1);
                writes = writes.saturating_add(2); // deposit_creating: account + total_issuance
                writes = writes.saturating_add(1); // minted event

                Self::deposit_event(Event::<T>::ShengBankInterestMinted {
                    year,
                    pallet_id,
                    account,
                    amount: interest,
                });
            }

            (reads, writes, success_count, total_count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pallet::*;
    use codec::Decode;
    use frame_support::{
        assert_noop,
        assert_ok,
        derive_impl,
        traits::{Get, OnFinalize, OnInitialize, VariantCountOf},
    };
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
    use std::{cell::RefCell, thread_local};

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

    thread_local! {
        static BLOCKS_PER_YEAR_FOR_TEST: RefCell<u64> = const { RefCell::new(10) };
    }

    pub struct BlocksPerYearForTest;
    impl Get<u64> for BlocksPerYearForTest {
        fn get() -> u64 {
            BLOCKS_PER_YEAR_FOR_TEST.with(|v| *v.borrow())
        }
    }

    impl Config for Test {
        type Currency = Balances;
        type BlocksPerYear = BlocksPerYearForTest;
    }

    fn set_blocks_per_year(v: u64) {
        BLOCKS_PER_YEAR_FOR_TEST.with(|p| *p.borrow_mut() = v);
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        set_blocks_per_year(10);
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
            &mut &primitives::china::china_ch::CHINA_CH[index].duoqian_address[..],
        )
            .expect("pallet_address must decode")
    }

    #[test]
    fn first_year_should_mint_and_settle() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(LastSettledYear::<Test>::get(), 1);

            let first_bank = &primitives::china::china_ch::CHINA_CH[0];
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
            // 直接跳到第2年边界：应在同一边界块内补结算到第2年。
            System::set_block_number(20);
            ShengBankStakeInterest::on_initialize(20);

            assert_eq!(LastSettledYear::<Test>::get(), 2);

            let first_bank = &primitives::china::china_ch::CHINA_CH[0];
            let account = shengbank_account(0);
            let year1 = first_bank.stake_amount * 100u128 / 10_000u128;
            let year2 = first_bank.stake_amount * 99u128 / 10_000u128;
            assert_eq!(Balances::free_balance(account), year1 + year2);
        });
    }

    #[test]
    fn second_year_should_use_decayed_rate() {
        new_test_ext().execute_with(|| {
            run_to_block(20);
            assert_eq!(LastSettledYear::<Test>::get(), 2);

            let first_bank = &primitives::china::china_ch::CHINA_CH[0];
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

    #[test]
    fn root_can_force_advance_year_for_recovery() {
        new_test_ext().execute_with(|| {
            assert_ok!(ShengBankStakeInterest::force_advance_year(
                RuntimeOrigin::root(),
                5
            ));
            assert_eq!(LastSettledYear::<Test>::get(), 5);
        });
    }

    #[test]
    fn force_advance_year_rejects_noop_and_invalid() {
        new_test_ext().execute_with(|| {
            LastSettledYear::<Test>::put(5);
            assert_noop!(
                ShengBankStakeInterest::force_advance_year(RuntimeOrigin::root(), 5),
                Error::<Test>::InvalidYear
            );
            assert_noop!(
                ShengBankStakeInterest::force_advance_year(RuntimeOrigin::root(), 101),
                Error::<Test>::InvalidYear
            );
        });
    }

    #[test]
    fn root_can_override_shengbank_account() {
        new_test_ext().execute_with(|| {
            let bank = &primitives::china::china_ch::CHINA_CH[0];
            let pallet_id =
                primitives::china::china_ch::shenfen_id_to_fixed48(bank.shenfen_id).expect("id");
            let replacement = AccountId32::new([42u8; 32]);
            assert_ok!(ShengBankStakeInterest::set_shengbank_account_override(
                RuntimeOrigin::root(),
                pallet_id,
                replacement.clone()
            ));

            run_to_block(10);
            let expected = bank.stake_amount * 100u128 / 10_000u128;
            assert_eq!(Balances::free_balance(replacement), expected);
        });
    }

    #[test]
    fn root_can_clear_override_and_restore_default_account() {
        new_test_ext().execute_with(|| {
            let bank = &primitives::china::china_ch::CHINA_CH[0];
            let pallet_id =
                primitives::china::china_ch::shenfen_id_to_fixed48(bank.shenfen_id).expect("id");
            let replacement = AccountId32::new([42u8; 32]);
            let original = shengbank_account(0);

            assert_ok!(ShengBankStakeInterest::set_shengbank_account_override(
                RuntimeOrigin::root(),
                pallet_id,
                replacement.clone()
            ));
            run_to_block(10); // year 1 minted to replacement
            assert_ok!(ShengBankStakeInterest::clear_shengbank_account_override(
                RuntimeOrigin::root(),
                pallet_id
            ));
            run_to_block(20); // year 2 should go to original

            let year1 = bank.stake_amount * 100u128 / 10_000u128;
            let year2 = bank.stake_amount * 99u128 / 10_000u128;
            assert_eq!(Balances::free_balance(replacement), year1);
            assert_eq!(Balances::free_balance(original), year2);
        });
    }

    #[test]
    fn force_settle_years_can_backfill_multiple_years() {
        new_test_ext().execute_with(|| {
            System::set_block_number(50); // current_year = 5
            assert_ok!(ShengBankStakeInterest::force_settle_years(
                RuntimeOrigin::root(),
                3
            ));
            assert_eq!(LastSettledYear::<Test>::get(), 3);
        });
    }

    #[test]
    fn force_settle_years_rejects_zero_count() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                ShengBankStakeInterest::force_settle_years(RuntimeOrigin::root(), 0),
                Error::<Test>::InvalidOperationCount
            );
            assert_noop!(
                ShengBankStakeInterest::force_settle_years(RuntimeOrigin::root(), 101),
                Error::<Test>::InvalidOperationCount
            );
        });
    }

    #[test]
    fn non_root_calls_are_rejected() {
        new_test_ext().execute_with(|| {
            let caller = RuntimeOrigin::signed(AccountId32::new([1u8; 32]));
            let bank = &primitives::china::china_ch::CHINA_CH[0];
            let pallet_id =
                primitives::china::china_ch::shenfen_id_to_fixed48(bank.shenfen_id).expect("id");

            assert_noop!(
                ShengBankStakeInterest::force_settle_years(caller.clone(), 1),
                sp_runtime::DispatchError::BadOrigin
            );
            assert_noop!(
                ShengBankStakeInterest::force_advance_year(caller.clone(), 1),
                sp_runtime::DispatchError::BadOrigin
            );
            assert_noop!(
                ShengBankStakeInterest::set_shengbank_account_override(
                    caller.clone(),
                    pallet_id,
                    AccountId32::new([9u8; 32])
                ),
                sp_runtime::DispatchError::BadOrigin
            );
            assert_noop!(
                ShengBankStakeInterest::clear_shengbank_account_override(caller, pallet_id),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn on_initialize_respects_auto_backfill_cap() {
        new_test_ext().execute_with(|| {
            System::set_block_number(100); // current_year = 10
            ShengBankStakeInterest::on_initialize(100);
            assert_eq!(LastSettledYear::<Test>::get(), 8); // AUTO_BACKFILL_MAX_YEARS_PER_BLOCK
        });
    }

    #[test]
    fn blocks_per_year_zero_disables_settlement() {
        new_test_ext().execute_with(|| {
            set_blocks_per_year(0);
            run_to_block(50);
            assert_eq!(LastSettledYear::<Test>::get(), 0);
            assert_eq!(Balances::free_balance(shengbank_account(0)), 0);
        });
    }
}
