#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use codec::Decode;
    use frame_support::{ensure, pallet_prelude::*, traits::Currency};
    use frame_system::{ensure_root, pallet_prelude::*};
    use sp_runtime::traits::{CheckedMul, SaturatedConversion, Zero};
    use sp_std::prelude::*;

    // ===== 引入制度常量 =====
    use primitives::{
        china::china_ch::CHINA_CH, // 固定 43 个省储行多签地址
        core_const::{
            ENABLE_SHENGBANK_INTEREST_DECAY, SHENGBANK_INITIAL_INTEREST_BP,
            SHENGBANK_INTEREST_DECREASE_BP, SHENGBANK_INTEREST_DURATION_YEARS,
        },
    };

    // 中文注释：自动路径只允许每个年度边界块结算 1 年，避免历史欠账集中压进单块。
    const AUTO_BACKFILL_MAX_YEARS_PER_BLOCK: u32 = 1;
    // 中文注释：Root 补结算保留批处理能力，但必须分批执行，避免单笔交易结算 100 年。
    const MAX_FORCE_SETTLE_YEARS: u32 = 8;
    // 中文注释：省储行利息制度当前固定启用逐年递减，禁止保留关闭递减的死分支。
    const _: () = assert!(
        ENABLE_SHENGBANK_INTEREST_DECAY,
        "ENABLE_SHENGBANK_INTEREST_DECAY must stay true"
    );

    // ===== 配置 =====
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 原生货币接口
        type Currency: Currency<Self::AccountId>;

        /// 一年对应的区块数（由 runtime 注入）
        #[pallet::constant]
        type BlocksPerYear: Get<u64>;

        /// 权重信息（通常由 benchmark 生成）
        type WeightInfo: crate::weights::WeightInfo;
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
            account_bytes: [u8; 32],
            account: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 省储行账户解码失败或配置无效（链上可审计，不依赖节点日志）。
        ShengBankDecodeFailed { year: u32, account_bytes: [u8; 32] },

        /// stake_amount 转换到运行时 Balance 发生饱和截断。
        ShengBankPrincipalOverflow { year: u32, account_bytes: [u8; 32] },

        /// 利率乘法发生溢出，跳过该省储行本年度铸币并让年度结算失败。
        ShengBankInterestOverflow { year: u32, account_bytes: [u8; 32] },

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

        /// 省储行利息低于 Existential Deposit，跳过发币以防 dust 账户（链上可审计）。
        ShengBankInterestBelowED {
            year: u32,
            account_bytes: [u8; 32],
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidOperationCount,
        InvalidYear,
    }

    // ===== Hooks =====
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            // 按区块高度结算：每满 BLOCKS_PER_YEAR（白皮书定义 87,600）个区块触发一次年度结算
            let block = n.saturated_into::<u64>();
            let per_year = T::BlocksPerYear::get();

            // 快速跳过：非年度边界块无需读取存储。
            if per_year == 0 || block == 0 || block % per_year != 0 {
                return Weight::zero();
            }

            let current_year = Self::current_year(n);
            let last_year = Self::last_settled_year();

            // 只在“年度边界区块”触发，按年度顺序自动补结算。
            if current_year > last_year && last_year < SHENGBANK_INTEREST_DURATION_YEARS {
                // 中文注释：自动结算的最坏路径由 benchmark 权重覆盖，不再使用手写读写估算。
                let _ = Self::settle_next_years(
                    current_year,
                    AUTO_BACKFILL_MAX_YEARS_PER_BLOCK,
                    Some(n),
                );
                return T::WeightInfo::on_initialize_settlement();
            }

            T::WeightInfo::on_initialize_boundary_noop()
        }

        /// try-runtime 状态校验：确保 LastSettledYear 不超过制度年限上限。
        #[cfg(feature = "try-runtime")]
        fn try_state(_n: BlockNumberFor<T>) -> Result<(), sp_runtime::TryRuntimeError> {
            let last = LastSettledYear::<T>::get();
            frame_support::ensure!(
                last <= SHENGBANK_INTEREST_DURATION_YEARS,
                "LastSettledYear 超过制度年限上限"
            );
            Ok(())
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Root 手动补结算指定年数（用于故障恢复或快速追平）。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::force_settle_years(*max_years))]
        pub fn force_settle_years(
            origin: OriginFor<T>,
            max_years: u32,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            // 中文注释：手动补结算只允许推进有限年数，避免一次交易做过多年度工作。
            ensure!(
                max_years > 0 && max_years <= MAX_FORCE_SETTLE_YEARS,
                Error::<T>::InvalidOperationCount
            );
            let current_year = Self::current_year(frame_system::Pallet::<T>::block_number());
            let (reads, writes, ops) = Self::settle_next_years(current_year, max_years, None);
            log::debug!(
                target: "runtime::shengbank",
                "force_settle_years finished | reads={} writes={} ops={}",
                reads,
                writes,
                ops
            );
            // 中文注释：实际扣重保持使用声明的 benchmark 权重，避免用运行时手写估算低报。
            Ok(().into())
        }

        /// Root 强制推进到指定年度（跳过无法修复的失败年度）。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::force_advance_year())]
        pub fn force_advance_year(origin: OriginFor<T>, year: u32) -> DispatchResult {
            ensure_root(origin)?;
            let current = Self::last_settled_year();
            let current_year = Self::current_year(frame_system::Pallet::<T>::block_number());
            // 中文注释：force advance 只用于跳过“已经到期但无法修复”的故障年度，
            // 不能越过当前链上时间提前跳过未来尚未到期的年度。
            ensure!(
                year > current && year <= current_year && year <= SHENGBANK_INTEREST_DURATION_YEARS,
                Error::<T>::InvalidYear
            );
            LastSettledYear::<T>::put(year);
            Self::deposit_event(Event::<T>::ShengBankYearForceAdvanced { year });
            Ok(())
        }
    }

    // ===== 核心逻辑 =====
    impl<T: Config> Pallet<T> {
        /// 解析省储行收款地址：只能是 CHINA_CH 中硬编码的该省多签地址，不可覆盖。
        fn resolve_bank_account(
            year: u32,
            bank: &primitives::china::china_ch::ChinaCh,
            account_bytes: [u8; 32],
        ) -> Option<T::AccountId> {
            match T::AccountId::decode(&mut &bank.main_address[..]) {
                Ok(a) => Some(a),
                Err(_) => {
                    Self::deposit_event(Event::<T>::ShengBankDecodeFailed {
                        year,
                        account_bytes,
                    });
                    log::error!(
                        target: "runtime::shengbank",
                        "省储行账户解码失败: {}",
                        bank.sfid_number
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
            // 中文注释：必须按年度顺序逐年推进；只要中间某一年失败，就停止后续年度，
            // 避免出现“后一年已发、前一年未发”的跨年错位。
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
                    Self::deposit_event(Event::<T>::ShengBankYearSettled {
                        year: settling_year,
                    });
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
            // 中文注释：第 1 个年度边界块对应 year=1；例如 block=per_year 时开始结算第 1 年。
            (b / per_year) as u32
        }

        /// 计算某年的利率（BP，万分比）
        fn interest_bp_for_year(year: u32) -> u32 {
            debug_assert!(
                year >= 1 && year <= SHENGBANK_INTEREST_DURATION_YEARS,
                "settlement year must stay inside shengbank interest duration"
            );

            // 中文注释：第 1 年使用初始利率，从第 2 年开始按固定 BP 递减，最低不会小于 0。
            let decay = year
                .saturating_sub(1)
                .saturating_mul(SHENGBANK_INTEREST_DECREASE_BP);

            SHENGBANK_INITIAL_INTEREST_BP.saturating_sub(decay)
        }

        /// 核心铸造逻辑（只针对固定省储行多签地址，不可覆盖）。
        fn mint_interest_for_year(year: u32) -> (u64, u64, u32, u32) {
            // 中文注释：这里的读写计数只保留给调试日志；真实区块权重以 benchmark 产物为准。
            // 保守估算每家省储行读：
            // - 账户余额读取
            // - 总发行量读取
            // - 账户存在性相关读取
            let reads = CHINA_CH.len() as u64 * 3;
            let mut writes = 0u64;
            let mut success_count = 0u32;
            let total_count = CHINA_CH.len() as u32;

            let rate_bp = Self::interest_bp_for_year(year);
            if rate_bp == 0 {
                return (0, writes, total_count, total_count);
            }

            for bank in CHINA_CH.iter() {
                let Some(account) = Self::resolve_bank_account(year, bank, bank.main_address)
                else {
                    writes = writes.saturating_add(1); // decode-failed event write
                    continue;
                };

                let principal: BalanceOf<T> = bank.stake_amount.saturated_into();
                let principal_back: u128 = principal.saturated_into();
                if principal_back != bank.stake_amount {
                    Self::deposit_event(Event::<T>::ShengBankPrincipalOverflow {
                        year,
                        account_bytes: bank.main_address,
                    });
                    log::error!(
                        target: "runtime::shengbank",
                        "stake_amount 饱和截断: {}",
                        bank.sfid_number
                    );
                    writes = writes.saturating_add(1);
                    continue;
                }

                // 中文注释：利率乘法必须显式检查溢出，避免 saturating_mul 静默铸出异常大额。
                let rate: BalanceOf<T> = rate_bp.into();
                let Some(gross_interest) = principal.checked_mul(&rate) else {
                    Self::deposit_event(Event::<T>::ShengBankInterestOverflow {
                        year,
                        account_bytes: bank.main_address,
                    });
                    log::error!(
                        target: "runtime::shengbank",
                        "省储行利息乘法溢出: {}",
                        bank.sfid_number
                    );
                    writes = writes.saturating_add(1);
                    continue;
                };
                let interest = gross_interest / 10_000u32.into();

                if interest.is_zero() {
                    // 利息为0不视为失败，避免把“无应付利息”误判成年度失败。
                    success_count = success_count.saturating_add(1);
                    continue;
                }
                if interest < T::Currency::minimum_balance() {
                    // 中文注释：当前省储行固定 stake_amount 下不会命中这个分支；
                    // 这里保留为防御性兜底，避免未来参数变化时创建 dust 账户。
                    Self::deposit_event(Event::<T>::ShengBankInterestBelowED {
                        year,
                        account_bytes: bank.main_address,
                        amount: interest,
                    });
                    log::warn!(
                        target: "runtime::shengbank",
                        "省储行利息低于 ED，跳过: {}",
                        bank.sfid_number
                    );
                    writes = writes.saturating_add(1); // event write
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
                    account_bytes: bank.main_address,
                    account,
                    amount: interest,
                });
            }

            (reads, writes, success_count, total_count)
        }
    }
}

#[cfg(test)]
mod tests;
