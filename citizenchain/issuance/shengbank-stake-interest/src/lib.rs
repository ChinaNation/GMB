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
        shengbank_nodes_const::SHENG_BANK_NODES, // 固定 43 个省储行多签地址
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

    // ===== 错误 =====
    #[pallet::error]
    pub enum Error<T> {
        /// AccountId 解码失败
        AccountDecodeFailed,
        /// 账户不存在（被 reaped），拒绝发放
        AccountNotExist,
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

            // 只在“年度边界区块”触发，且按年度顺序结算，最多结算到制度上限年限
            if per_year > 0
                && block > 0
                && block % per_year == 0
                && current_year == last_year + 1
                && last_year < SHENGBANK_INTEREST_DURATION_YEARS
            {
                log::info!(
                    target: "runtime::shengbank",
                    "省储行利息年度结算开始 | 年度={} | 区块={:?}",
                    current_year,
                    n
                );

                // 中文注释：执行当年利息发放，并返回读写计数+成功数
                let (reads, writes, success_count) = Self::mint_interest_for_year(current_year);

                // 中文注释：制度要求“43个省储行必须全部成功”，否则本年度不推进结算进度
                let total_count = SHENG_BANK_NODES.len() as u32;
                if success_count == total_count {
                    LastSettledYear::<T>::put(current_year);
                    Self::deposit_event(Event::<T>::ShengBankYearSettled { year: current_year });

                    return T::DbWeight::get().reads_writes(reads + 1, writes + 1);
                }

                // 中文注释：只要有任一地址发放失败，就记录失败事件并保持 LastSettledYear 不变
                Self::deposit_event(Event::<T>::ShengBankYearSettlementFailed {
                    year: current_year,
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
            let reads = 1u64 + SHENG_BANK_NODES.len() as u64;
            let mut writes = 0u64;
            let mut success_count = 0u32;

            let rate_bp = Self::interest_bp_for_year(year);
            if rate_bp == 0 {
                return (reads, writes, success_count);
            }

            for bank in SHENG_BANK_NODES.iter() {
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
