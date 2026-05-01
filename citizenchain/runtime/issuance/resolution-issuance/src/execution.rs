//! 决议发行执行、审计记录与暂停维护逻辑。

use crate::pallet::{
    AllocationOf, BalanceOf, Config, Error, Event, EverExecuted, Executed, Pallet, Paused,
    ReasonOf, TotalIssued,
};
use frame_support::{
    dispatch::DispatchResult,
    ensure,
    storage::with_storage_layer,
    traits::{Currency, Get, Imbalance},
};
use sp_runtime::traits::{CheckedAdd, Hash};

impl<T: Config> Pallet<T> {
    pub(crate) fn execute_approved_issuance(
        proposal_id: u64,
        reason: &ReasonOf<T>,
        total_amount: BalanceOf<T>,
        allocations: &AllocationOf<T>,
    ) -> DispatchResult {
        // 中文注释：执行发行必须整笔成功或整笔回滚，不能出现部分账户已到账的半状态。
        with_storage_layer(|| {
            Self::do_execute_inner(proposal_id, reason.as_slice(), total_amount, allocations)
        })
    }

    fn do_execute_inner(
        proposal_id: u64,
        reason: &[u8],
        total_amount: BalanceOf<T>,
        allocations: &AllocationOf<T>,
    ) -> DispatchResult {
        ensure!(!Paused::<T>::get(), Error::<T>::PalletPaused);
        // 中文注释：重放判断只认永久标记 EverExecuted；短期 Executed 可清理但不能释放重放窗口。
        ensure!(
            !EverExecuted::<T>::contains_key(proposal_id),
            Error::<T>::AlreadyExecuted
        );
        ensure!(!reason.is_empty(), Error::<T>::EmptyReason);
        ensure!(
            reason.len() <= T::MaxReasonLen::get() as usize,
            Error::<T>::ReasonTooLong
        );
        Self::validate_execution_allocations(&total_amount, allocations)?;

        let existential_deposit = T::Currency::minimum_balance();
        for item in allocations.iter() {
            // 中文注释：名单唯一、单笔非零和总额匹配已由共享校验负责；执行层只补 ED。
            ensure!(
                item.amount >= existential_deposit,
                Error::<T>::BelowExistentialDeposit
            );
        }
        ensure!(
            total_amount <= T::MaxSingleIssuance::get(),
            Error::<T>::ExceedsSingleIssuanceCap
        );

        let new_total = TotalIssued::<T>::get()
            .checked_add(&total_amount)
            .ok_or(Error::<T>::TotalIssuedOverflow)?;
        ensure!(
            new_total <= T::MaxTotalIssuance::get(),
            Error::<T>::ExceedsTotalIssuanceCap
        );

        let mut total_imbalance =
            <<T as Config>::Currency as Currency<T::AccountId>>::PositiveImbalance::zero();
        for item in allocations.iter() {
            let imbalance = T::Currency::deposit_creating(&item.recipient, item.amount.clone());
            ensure!(imbalance.peek() == item.amount, Error::<T>::DepositFailed);
            total_imbalance.subsume(imbalance);
        }
        // 中文注释：统一 drop 合并后的 imbalance，让 Currency 在这一点完成总发行量记账。
        drop(total_imbalance);

        let current_block = frame_system::Pallet::<T>::block_number();
        EverExecuted::<T>::insert(proposal_id, ());
        Executed::<T>::insert(proposal_id, current_block);
        TotalIssued::<T>::put(new_total);

        let reason_hash = T::Hashing::hash(reason);
        let allocations_hash = T::Hashing::hash_of(&allocations);
        Self::deposit_event(Event::<T>::ResolutionIssuanceExecuted {
            proposal_id,
            total_amount,
            recipient_count: allocations.len() as u32,
            reason_hash,
            allocations_hash,
        });

        Ok(())
    }

    pub(crate) fn clear_executed_marker(proposal_id: u64) -> DispatchResult {
        // 中文注释：这里只清理短期展示/排障记录，不允许触碰永久防重放标记 EverExecuted。
        ensure!(
            Executed::<T>::contains_key(proposal_id),
            Error::<T>::NotExecuted
        );
        Executed::<T>::remove(proposal_id);
        Self::deposit_event(Event::<T>::ExecutedCleared { proposal_id });
        Ok(())
    }

    pub(crate) fn set_pause_state(paused: bool) -> DispatchResult {
        ensure!(Paused::<T>::get() != paused, Error::<T>::AlreadyInState);
        Paused::<T>::put(paused);
        Self::deposit_event(Event::<T>::PausedSet { paused });
        Ok(())
    }
}
