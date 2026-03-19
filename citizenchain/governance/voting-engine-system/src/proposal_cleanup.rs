//! 提案定时清理调度模块。
//!
//! 投票引擎统一负责清理所有提案相关数据（投票数据 + 业务数据），
//! 业务模块不需要实现任何清理逻辑。
//!
//! ## 清理策略
//!
//! - 提案完成（通过/拒绝/过期）时，调用 `schedule_cleanup` 注册延迟清理
//! - 清理时间 = 完成时区块 + 90 天区块数
//! - 每个区块 `on_initialize` 检查 `CleanupQueue[当前区块]`，到期后触发清理
//! - 每区块最多触发 `MAX_TRIGGERS_PER_BLOCK` 个提案进入清理流程
//!
//! ## 清理执行
//!
//! 本模块只负责**调度**（何时触发），实际数据删除全部委托给
//! `PendingProposalCleanups` 分块状态机（`process_pending_cleanup_steps`），
//! 保证大量投票记录（如公民投票上万条）能分多个区块完成清理。
//!
//! 清理流程：
//! 1. `schedule_cleanup` → 写入 `CleanupQueue[cleanup_at]`
//! 2. `on_initialize` → `process_cleanup_queue` → 从队列取出 proposal_id
//! 3. 释放活跃提案名额 + 删除 core/业务数据 + 注册 `PendingProposalCleanups`
//! 4. `process_pending_cleanup_steps` 分块清理投票记录（InternalVotes → JointVotes → CitizenVotes → VoteCredentials → FinalCleanup）

use crate::pallet::{self, Config};
use crate::PendingCleanupStage;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::{One, Saturating};

/// 提案完成后保留天数。
const RETENTION_DAYS: u32 = 90;

/// 每区块最多从 CleanupQueue 触发清理的提案数量。
const MAX_TRIGGERS_PER_BLOCK: u32 = 5;

/// CleanupQueue 单个区块的 BoundedVec 上限。
const QUEUE_CAPACITY: u32 = 50;

/// 计算保留期限对应的区块数。
fn retention_blocks<T: Config>() -> BlockNumberFor<T> {
    let blocks_per_day: BlockNumberFor<T> =
        (primitives::pow_const::BLOCKS_PER_DAY as u32).into();
    let days: BlockNumberFor<T> = RETENTION_DAYS.into();
    blocks_per_day.saturating_mul(days)
}

/// 注册延迟清理：提案完成时调用，90 天后自动清理。
/// 如果目标区块的队列已满（50 个），自动顺延到下一个区块，保证不丢失。
/// 连续 100 个区块都满时返回错误（实际几乎不可能发生）。
pub fn schedule_cleanup<T: Config>(
    proposal_id: u64,
    current_block: BlockNumberFor<T>,
) -> frame_support::pallet_prelude::DispatchResult {
    let base = current_block.saturating_add(retention_blocks::<T>());
    let mut target = base;

    // 尝试最多 100 个区块的偏移，找到有空位的队列
    for _ in 0..100u32 {
        let success = pallet::CleanupQueue::<T>::mutate(target, |ids| {
            ids.try_push(proposal_id).is_ok()
        });
        if success {
            return Ok(());
        }
        target = target.saturating_add(BlockNumberFor::<T>::one());
    }

    // 极端情况：连续 100 个区块都满了（几乎不可能）。
    // 使用 defensive! 在 debug/test 模式下 panic 提示开发者，生产模式仅记录日志。
    // 数据不会被清理但不影响链运行。
    frame_support::defensive!("schedule_cleanup: all queues full, proposal may not be cleaned up");
    Ok(())
}

/// 在 `on_initialize` 中调用。
/// 检查当前区块是否有到期清理任务，有则触发（注册到 PendingProposalCleanups）。
/// 未处理完的保留在队列中，下个区块继续。
pub fn process_cleanup_queue<T: Config>(now: BlockNumberFor<T>) -> Weight {
    let db_weight = T::DbWeight::get();
    let mut weight = db_weight.reads(1); // 读取 CleanupQueue[now]

    let queue = pallet::CleanupQueue::<T>::get(now);
    if queue.is_empty() {
        return weight;
    }

    let total = queue.len();
    let to_process = core::cmp::min(total, MAX_TRIGGERS_PER_BLOCK as usize);

    for i in 0..to_process {
        let proposal_id = queue[i];
        weight = weight.saturating_add(trigger_cleanup::<T>(proposal_id));
    }

    if to_process >= total {
        // 全部处理完，删除队列条目
        pallet::CleanupQueue::<T>::remove(now);
        weight = weight.saturating_add(db_weight.writes(1));
    } else {
        // 未处理完，保留剩余部分
        let remaining: BoundedVec<u64, ConstU32<QUEUE_CAPACITY>> = queue
            .into_inner()
            .into_iter()
            .skip(to_process)
            .collect::<sp_std::vec::Vec<_>>()
            .try_into()
            .unwrap_or_default();
        pallet::CleanupQueue::<T>::insert(now, remaining);
        weight = weight.saturating_add(db_weight.writes(1));
    }

    weight
}

/// 触发单个提案的清理：释放活跃名额 + 删除立即可删的数据 + 注册分块清理。
fn trigger_cleanup<T: Config>(proposal_id: u64) -> Weight {
    let db_weight = T::DbWeight::get();
    let mut weight = db_weight.reads(1); // 读取 Proposal

    // 1. 释放活跃提案名额（兜底）
    if let Some(proposal) = pallet::Proposals::<T>::get(proposal_id) {
        if let Some(institution) = proposal.internal_institution {
            crate::active_proposal_limit::remove_active_proposal::<T>(institution, proposal_id);
            weight = weight.saturating_add(db_weight.reads_writes(1, 1));
        }
    }

    // 2. 注册到分块清理状态机。
    //    所有提案（无论内部/联合）统一从 InternalVotes 阶段开始：
    //    InternalVotes → JointVotes → CitizenVotes → VoteCredentials → FinalCleanup
    //    如果某阶段没有数据（比如内部提案没有 JointVotes），clear_prefix 返回空结果，
    //    自动跳到下一阶段，不会卡住。最后 FinalCleanup 删除核心数据和业务数据。
    if !pallet::PendingProposalCleanups::<T>::contains_key(proposal_id) {
        pallet::PendingProposalCleanups::<T>::insert(
            proposal_id,
            PendingCleanupStage::InternalVotes,
        );
        weight = weight.saturating_add(db_weight.reads_writes(1, 1));
    }

    weight
}
