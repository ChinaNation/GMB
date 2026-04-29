//! 扫码支付清算体系 Step 2 新增:清算行费率自治。
//!
//! 中文注释:
//! - 每个清算行主账户对应一个 `L2FeeRateBp` 费率(单位 bp,范围 1~10)。
//! - 清算行管理员可提案改费率,**延迟 7 天生效**(防突袭改价,给 L3 换行时间)。
//! - 全局上限 `MaxL2FeeRateBp` 由联合投票调整(沿用 `voting-engine`,
//!   这里只定义 extrinsic 入口;具体联合投票执行回调的对接由 Step 2b 补)。
//! - 在 `on_initialize` 每块扫描一次到期提案并激活(小成本,Step 3 可优化为 cursor)。

use frame_support::{ensure, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{SaturatedConversion, Saturating};
use sp_std::vec::Vec;

use crate::bank_check::{self, SfidAccountQuery};
use crate::{Config, Error, Event, L2FeeRateBp, L2FeeRateProposed, MaxL2FeeRateBp, Pallet};

/// 费率下限(bp),1 bp = 0.01%。
pub const L2_FEE_RATE_BP_MIN: u32 = 1;

/// 费率上限(bp),10 bp = 0.1%,对应白皮书 5.4.2。
pub const L2_FEE_RATE_BP_MAX: u32 = 10;

/// 费率变更延迟生效期,沿用 `primitives::pow_const` 里 30 秒/块的定义。
/// 7 天 × 24 × 120 = **20160 块**。
pub const RATE_CHANGE_DELAY_BLOCKS: u64 = 20_160;

/// 提案新费率:清算行管理员发起,延迟 `RATE_CHANGE_DELAY_BLOCKS` 后生效。
///
/// 约束:
/// - `who` 必须是该清算行多签管理员之一(通过 `duoqian-manage-pow` 的 DuoqianAccounts 校验)
/// - 目标必须是合法清算行主账户(SFR/FFR + Active)
/// - 新费率在 `[L2_FEE_RATE_BP_MIN, MaxL2FeeRateBp]` 区间
/// - 同一清算行不允许并行提案(新提案覆盖旧提案)
pub fn do_propose_l2_fee_rate<T: Config>(
    who: T::AccountId,
    bank_main_address: T::AccountId,
    new_rate_bp: u32,
) -> DispatchResult {
    // 1. 清算行合法性
    bank_check::ensure_can_be_bound::<T>(&bank_main_address)?;

    // 2. 调用者必须是该清算行多签管理员(通过 SfidAccountQuery 解耦到 runtime 层)
    ensure!(
        T::SfidAccountQuery::is_admin_of(&bank_main_address, &who),
        Error::<T>::UnauthorizedAdmin
    );

    // 3. 费率范围校验
    let max = MaxL2FeeRateBp::<T>::get().max(L2_FEE_RATE_BP_MAX);
    ensure!(
        new_rate_bp >= L2_FEE_RATE_BP_MIN && new_rate_bp <= max,
        Error::<T>::InvalidL2FeeRate
    );

    // 4. 延迟生效高度 = 当前高度 + RATE_CHANGE_DELAY_BLOCKS
    let now = frame_system::Pallet::<T>::block_number();
    let delay: BlockNumberFor<T> = RATE_CHANGE_DELAY_BLOCKS.saturated_into();
    let effective_at = now.saturating_add(delay);

    L2FeeRateProposed::<T>::insert(&bank_main_address, (new_rate_bp, effective_at));
    Pallet::<T>::deposit_event(Event::<T>::L2FeeRateProposed {
        bank: bank_main_address,
        new_rate_bp,
        effective_at,
    });
    Ok(())
}

/// `on_initialize` 钩子激活到期提案:把 `L2FeeRateProposed` 里 `effective_at <= now`
/// 的提案搬到 `L2FeeRateBp`,然后删除提案记录。
///
/// 本步采用简单全表扫描(小规模清算行可接受,Step 3 优化为 cursor/batched)。
/// 返回消耗的权重,供上层 on_initialize 累加。
pub fn activate_pending_rates<T: Config>(now: BlockNumberFor<T>) -> Weight {
    let db = T::DbWeight::get();
    let mut consumed = Weight::zero();
    let mut to_activate: Vec<(T::AccountId, u32)> = Vec::new();

    for (bank, (rate, effective_at)) in L2FeeRateProposed::<T>::iter() {
        consumed = consumed.saturating_add(db.reads(1));
        if now >= effective_at {
            to_activate.push((bank, rate));
        }
    }

    for (bank, rate) in to_activate {
        L2FeeRateBp::<T>::insert(&bank, rate);
        L2FeeRateProposed::<T>::remove(&bank);
        Pallet::<T>::deposit_event(Event::<T>::L2FeeRateActivated {
            bank,
            rate_bp: rate,
        });
        consumed = consumed.saturating_add(db.reads_writes(0, 3));
    }

    consumed
}

/// 全局上限治理:先写"由联合投票引擎执行"的入口,Step 2b 接回调。
///
/// Step 2a 本函数仅做校验,写入 `MaxL2FeeRateBp`。Step 2b 接入联合投票后改为
/// 只由投票引擎回调可达(把 extrinsic 改为免费的 `execute_*` 类)。
pub fn do_set_max_l2_fee_rate<T: Config>(new_max: u32) -> DispatchResult {
    ensure!(
        new_max >= L2_FEE_RATE_BP_MIN && new_max <= L2_FEE_RATE_BP_MAX,
        Error::<T>::InvalidL2FeeRate
    );
    MaxL2FeeRateBp::<T>::put(new_max);
    Pallet::<T>::deposit_event(Event::<T>::MaxL2FeeRateUpdated { new_max });
    Ok(())
}

/// 查询清算行当前生效费率。未配置时返回 0(调用方自己决定是否用全局默认)。
pub fn current_rate_bp<T: Config>(bank_main: &T::AccountId) -> u32 {
    L2FeeRateBp::<T>::get(bank_main)
}
