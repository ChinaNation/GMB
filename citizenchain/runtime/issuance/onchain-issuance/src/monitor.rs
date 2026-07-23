//! NRC 监管动作(freeze / unfreeze / confiscate / forceTransfer / forceClose)— 框架占位。
//!
//! 与 ADR-011 v2 第 5.1 / 5.6 节对齐:
//! - 监管动作走 **JointVote**(NRC admin 多签 + 全民兜底)
//! - propose origin 校验:`actor_cid_number == NRC` 且
//!   `ensure!(proposer_account_id ∈ AdminAccounts[actor_cid_number].admins)`
//! - 强制销毁倒计时 30 天:写入 `ForceCloseSchedule[expire_block].push(asset_id)`,
//!   `on_finalize(n)` 通过 `take(n)` O(1) 处理,不全表扫描 Assets
//!
//! 当前框架阶段只搭函数签名 + doc 占位,实装在后续任务卡 B 完成。

use crate::pallet::{BalanceOf, Config};
use crate::proposal::{
    MonitorConfiscateProposal, MonitorForceCloseProposal, MonitorForceTransferProposal,
    MonitorFreezeProposal,
};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::BlockNumberFor;

/// NRC 监管:冻结特定持仓(调 pallet_assets::freeze + emit MonitorFrozen)。
pub fn execute_monitor_freeze<T: Config>(
    _proposal: MonitorFreezeProposal<T::AccountId>,
) -> DispatchResult {
    // TODO: implement business logic
    Ok(())
}

/// NRC 监管:解冻持仓(调 pallet_assets::thaw + emit MonitorUnfrozen)。
pub fn execute_monitor_unfreeze<T: Config>(
    _proposal: MonitorFreezeProposal<T::AccountId>,
) -> DispatchResult {
    // TODO: implement business logic
    Ok(())
}

/// NRC 监管:强制 burn(扣押,调 pallet_assets::burn_from + emit MonitorConfiscated)。
pub fn execute_monitor_confiscate<T: Config>(
    _proposal: MonitorConfiscateProposal<T::AccountId, BalanceOf<T>>,
) -> DispatchResult {
    // TODO: implement business logic
    Ok(())
}

/// NRC 监管:强制划转(追赃,调 pallet_assets::transfer 跳过 from_account_id 同意)。
pub fn execute_monitor_force_transfer<T: Config>(
    _proposal: MonitorForceTransferProposal<T::AccountId, BalanceOf<T>>,
) -> DispatchResult {
    // TODO: implement business logic
    Ok(())
}

/// NRC 监管:整币封禁入调度队列(30 天后由 on_finalize 销毁余额)。
///
/// 实装时 `expire_block = current_block + 30 * DAYS`,
/// `ForceCloseSchedule::mutate(expire_block, |list| list.try_push(asset_id))`,
/// `Assets[asset_id].state = ForceClosed { close_block: expire_block }`(同事务)。
/// 30 天后 `on_finalize(expire_block)` 取出 list 逐一执行 `pallet_assets::start_destroy`。
pub fn execute_monitor_force_close<T: Config>(
    _proposal: MonitorForceCloseProposal,
) -> DispatchResult {
    // TODO: implement business logic
    Ok(())
}

/// `on_finalize(n)` 处理到期 ForceClose 队列。
///
/// O(1) `take(n)` 取出当前块到期的 asset_id 列表 → 逐一 destroy。
/// 不扫主 Assets 表。
pub fn process_force_close_schedule_on_finalize<T: Config>(_block: BlockNumberFor<T>) {
    // TODO: implement business logic
    // let scheduled = ForceCloseSchedule::<T>::take(_block);
    // for asset_id in scheduled.iter() { pallet_assets::start_destroy(asset_id); ... }
}

/// 监管 callback 入口:VotingEngine JointVote 通过后路由到对应 execute_monitor_*。
///
/// propose origin 校验(proposer_account_id ∈ NRC admins)已在 propose 阶段完成,callback 不再校验。
pub fn dispatch_joint_callback<T: Config>(
    _action: [u8; 4],
    _proposal_data: &[u8],
) -> DispatchResult {
    // TODO: route by ACTION constant to_account_id execute_monitor_freeze / unfreeze / ...
    Ok(())
}
