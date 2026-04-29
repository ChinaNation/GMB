//! 决议发行共享校验逻辑。

use crate::pallet::{AllocationOf, BalanceOf, Config, Error, Pallet};
use codec::Decode;
use frame_support::{dispatch::DispatchResult, ensure, BoundedVec};
use primitives::china::china_cb::CHINA_CB;
use sp_runtime::traits::{CheckedAdd, Zero};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

impl<T: Config> Pallet<T> {
    pub(crate) fn validate_proposal_allocations(
        total_amount: &BalanceOf<T>,
        allocations: &[crate::proposal::RecipientAmount<T::AccountId, BalanceOf<T>>],
    ) -> DispatchResult {
        ensure!(!allocations.is_empty(), Error::<T>::EmptyAllocations);
        Self::ensure_nonzero_total(total_amount)?;
        let expected = crate::pallet::AllowedRecipients::<T>::get();
        ensure!(!expected.is_empty(), Error::<T>::RecipientsNotConfigured);

        // 中文注释：提案收款人集合必须与链上白名单完全一致，既不能少人，也不能多塞账户。
        let expected_set: BTreeSet<&T::AccountId> = expected.iter().collect();
        ensure!(
            expected_set.len() == expected.len(),
            Error::<T>::DuplicateAllowedRecipient
        );
        ensure!(
            allocations.len() == expected_set.len(),
            Error::<T>::InvalidAllocationCount
        );

        let mut seen: BTreeSet<&T::AccountId> = BTreeSet::new();
        let mut sum = BalanceOf::<T>::zero();
        for item in allocations {
            Self::ensure_nonzero_total(&item.amount)?;
            ensure!(seen.insert(&item.recipient), Error::<T>::DuplicateRecipient);
            ensure!(
                expected_set.contains(&item.recipient),
                Error::<T>::InvalidRecipientSet
            );
            sum = sum
                .checked_add(&item.amount)
                .ok_or(Error::<T>::AllocationOverflow)?;
        }

        ensure!(seen == expected_set, Error::<T>::InvalidRecipientSet);
        ensure!(sum == total_amount.clone(), Error::<T>::TotalMismatch);
        Ok(())
    }

    pub(crate) fn ensure_unique_recipients(recipients: &[T::AccountId]) -> DispatchResult {
        let mut seen: BTreeSet<&T::AccountId> = BTreeSet::new();
        for recipient in recipients {
            ensure!(
                seen.insert(recipient),
                Error::<T>::DuplicateAllowedRecipient
            );
        }
        Ok(())
    }

    /// 中文注释：新名单必须是旧名单的超集（只允许新增，不允许删除）。
    pub(crate) fn ensure_recipients_only_added(
        new_recipients: &BoundedVec<T::AccountId, T::MaxAllocations>,
    ) -> DispatchResult {
        let current = crate::pallet::AllowedRecipients::<T>::get();
        let new_set: BTreeSet<&T::AccountId> = new_recipients.iter().collect();
        for existing in current.iter() {
            ensure!(new_set.contains(existing), Error::<T>::RecipientRemoved);
        }
        Ok(())
    }

    /// 中文注释：所有收款账户必须是 CHINA_CB 省储会地址（跳过索引 0 的 NRC）。
    pub(crate) fn ensure_recipients_in_china_cb(
        recipients: &BoundedVec<T::AccountId, T::MaxAllocations>,
    ) -> DispatchResult {
        let valid_set: BTreeSet<T::AccountId> = CHINA_CB
            .iter()
            .skip(1)
            .filter_map(|node| T::AccountId::decode(&mut &node.main_address[..]).ok())
            .collect();
        for recipient in recipients.iter() {
            ensure!(
                valid_set.contains(recipient),
                Error::<T>::RecipientNotInChinaCb
            );
        }
        Ok(())
    }

    pub(crate) fn decode_default_allowed_recipients(
    ) -> Option<BoundedVec<T::AccountId, T::MaxAllocations>> {
        let recipients: Vec<T::AccountId> = CHINA_CB
            .iter()
            .skip(1)
            .filter_map(|node| T::AccountId::decode(&mut &node.main_address[..]).ok())
            .collect();
        if recipients.is_empty() {
            return None;
        }
        let bounded: BoundedVec<T::AccountId, T::MaxAllocations> = recipients.try_into().ok()?;
        if Self::ensure_unique_recipients(bounded.as_slice()).is_err() {
            return None;
        }
        if Self::ensure_recipients_in_china_cb(&bounded).is_err() {
            return None;
        }
        Some(bounded)
    }

    pub(crate) fn validate_execution_allocations(
        total_amount: &BalanceOf<T>,
        allocations: &AllocationOf<T>,
    ) -> DispatchResult {
        Self::validate_proposal_allocations(total_amount, allocations.as_slice())
    }
}
