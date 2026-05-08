//! 创建费押金机制(ADR-011 v2 第六节)。
//!
//! - 费率单一权威源:`primitives::fee_policy::ONCHAIN_ASSET_CREATE_FEE`(100_000 FEN = 1000 元)
//! - 仅 `propose_issue` 走本路径;mint/burn/transfer/close 走 VotingEngine InternalVote 自身的
//!   `VOTE_FLAT_FEE = 1 元/次`(由 OnchainTxAmountExtractor 处理),与本文件无关
//! - NRC 监管 5 动作 propose Free
//!
//! ## 三态押金机制
//!
//! 1. `reserve_creation_deposit(proposer, proposal_id)`:propose_issue 阶段,
//!    从 proposer 账户 `Currency::reserve` 锁定 1000 GMB,写入 `IssueDeposit` storage
//! 2. `release_creation_deposit_to_nrc(proposal_id)`:callback 通过阶段,
//!    `unreserve` 后 transfer 到 `NrcFeeAccountProvider::nrc_fee_account()`
//! 3. `refund_creation_deposit(proposal_id)`:callback 否决/过期阶段,
//!    `unreserve` 退还原 proposer
//!
//! 三态走完后 `IssueDeposit::remove(proposal_id)` 清理 storage。

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, ReservableCurrency},
};

use crate::pallet::{BalanceOf, Config, Error, Event, IssueDeposit, NrcFeeAccountProvider, Pallet};

/// 链上发行代币创建费(单一权威源转译)。
pub fn issue_creation_fee<T: Config>() -> BalanceOf<T>
where
    BalanceOf<T>: From<u128>,
{
    BalanceOf::<T>::from(primitives::fee_policy::ONCHAIN_ASSET_CREATE_FEE)
}

/// 阶段 1:propose_issue 时 reserve 1000 GMB,写入 IssueDeposit storage。
///
/// 中文注释:`Currency::reserve` 失败原因仅可能是 issuer 余额不足,链端在 ensure 链上拦截。
/// reserve 后 GMB 仍属 proposer,只是被锁定,直到 callback 决定 release/refund。
pub fn reserve_creation_deposit<T: Config>(
    proposer: &T::AccountId,
    proposal_id: u64,
) -> DispatchResult
where
    BalanceOf<T>: From<u128>,
{
    let amount: BalanceOf<T> = issue_creation_fee::<T>();
    T::Currency::reserve(proposer, amount)
        .map_err(|_| Error::<T>::InsufficientBalanceForDeposit)?;
    IssueDeposit::<T>::insert(proposal_id, (proposer.clone(), amount));
    Pallet::<T>::deposit_event(Event::IssueDepositReserved {
        proposal_id,
        who: proposer.clone(),
        amount,
    });
    Ok(())
}

/// 阶段 2:callback 通过时,unreserve 后 transfer 给 NRC fee_address。
///
/// 中文注释:实际净效果 = 押金从 proposer 永久转入 NRC。
/// 步骤拆为 unreserve + transfer 而非直接 `repatriate_reserved`,
/// 保证 transfer 失败时余额仍可见(便于审计)。
pub fn release_creation_deposit_to_nrc<T: Config>(proposal_id: u64) -> DispatchResult
where
    BalanceOf<T>: From<u128>,
{
    let (proposer, amount) =
        IssueDeposit::<T>::take(proposal_id).ok_or(Error::<T>::AssetNotFound)?;
    let nrc_fee =
        T::NrcFeeAccountProvider::nrc_fee_account().ok_or(Error::<T>::NrcFeeAccountMissing)?;
    let _ = T::Currency::unreserve(&proposer, amount);
    T::Currency::transfer(
        &proposer,
        &nrc_fee,
        amount,
        ExistenceRequirement::AllowDeath,
    )
    .map_err(|_| Error::<T>::AssetsInternal)?;
    Pallet::<T>::deposit_event(Event::IssueDepositCharged {
        proposal_id,
        who: proposer,
        amount,
    });
    Ok(())
}

/// 阶段 3:callback 否决/过期时,unreserve 退还原 proposer。
pub fn refund_creation_deposit<T: Config>(proposal_id: u64) -> DispatchResult {
    let (proposer, amount) =
        IssueDeposit::<T>::take(proposal_id).ok_or(Error::<T>::AssetNotFound)?;
    let _ = T::Currency::unreserve(&proposer, amount);
    Pallet::<T>::deposit_event(Event::IssueDepositRefunded {
        proposal_id,
        who: proposer,
        amount,
    });
    Ok(())
}
