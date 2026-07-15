//! 内部投票时长、阈值校验以及阈值生命周期副作用。

use super::*;

impl<T: Config> Pallet<T> {
    pub(crate) fn internal_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    pub(crate) fn ensure_threshold_within_snapshot(
        admins_len: u32,
        threshold: u32,
    ) -> DispatchResult {
        // 普通内部提案仍按账户当前阈值投票，但阈值必须能被本次管理员快照实际达成。
        ensure!(
            threshold > 0 && threshold <= admins_len,
            Error::<T>::InvalidThresholdSnapshot
        );
        Ok(())
    }

    pub(crate) fn ensure_all_admin_threshold(admins_len: u32, threshold: u32) -> DispatchResult {
        // 账户链上注册与注销会改变账户生命周期，必须由该账户快照内全体管理员通过。
        ensure!(
            admins_len > 0 && threshold == admins_len,
            Error::<T>::InvalidThresholdSnapshot
        );
        Ok(())
    }

    pub(crate) fn ensure_dynamic_threshold(admins_len: u32, threshold: u32) -> DispatchResult {
        // 动态阈值只允许严格过半，且不得超过管理员总数；统一用 u64 避免乘法溢出。
        ensure!(admins_len >= 2, Error::<T>::InvalidDynamicThreshold);
        ensure!(
            threshold > 0
                && threshold <= admins_len
                && u64::from(threshold).saturating_mul(2) > u64::from(admins_len),
            Error::<T>::InvalidDynamicThreshold
        );
        Ok(())
    }

    pub(crate) fn bound_and_validate_subject_cids(
        institution_code: InstitutionCode,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
    ) -> Result<ProposalSubjectCidNumbers, DispatchError> {
        let bounded = <votingengine::Pallet<T>>::bound_subject_cid_numbers(subject_cid_numbers)?;
        if is_personal_code(&institution_code) {
            ensure!(
                bounded.is_empty(),
                votingengine::Error::<T>::InvalidInstitution
            );
        } else {
            ensure!(
                !bounded.is_empty(),
                votingengine::Error::<T>::InvalidInstitution
            );
        }
        Ok(bounded)
    }

    pub(crate) fn snapshot_admins_len_or_missing(
        proposal_id: u64,
        institution: T::AccountId,
    ) -> Result<u32, DispatchError> {
        <votingengine::Pallet<T>>::snapshot_admins_len(proposal_id, institution)
            .ok_or(votingengine::Error::<T>::MissingAdminSnapshot.into())
    }
}

impl<T: Config> Pallet<T> {
    pub(crate) fn proposal_code_account(
        proposal_id: u64,
    ) -> Result<(InstitutionCode, T::AccountId), DispatchError> {
        let proposal =
            Proposals::<T>::get(proposal_id).ok_or(votingengine::Error::<T>::ProposalNotFound)?;
        let institution_code = proposal
            .internal_code
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        let account = proposal
            .account_context
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        Ok((institution_code, account))
    }

    pub(crate) fn apply_executed_threshold_side_effect(proposal_id: u64) -> DispatchResult {
        match InternalProposalRoles::<T>::get(proposal_id) {
            Some(InternalProposalRole::LifecycleCreate) => {
                let (institution_code, account) = Self::proposal_code_account(proposal_id)?;
                let threshold = PendingDynamicThresholds::<T>::take(proposal_id)
                    .ok_or(Error::<T>::MissingDynamicThreshold)?;
                ActiveDynamicThresholds::<T>::insert(institution_code, account, threshold);
            }
            Some(InternalProposalRole::LifecycleClose) => {
                let (institution_code, account) = Self::proposal_code_account(proposal_id)?;
                ActiveDynamicThresholds::<T>::remove(institution_code, account);
            }
            Some(InternalProposalRole::AdminChange) => {
                if let Some(pending) = PendingAdminChangeThresholds::<T>::take(proposal_id) {
                    Self::ensure_dynamic_threshold(pending.new_admins_len, pending.new_threshold)?;
                    ActiveDynamicThresholds::<T>::insert(
                        pending.institution_code,
                        pending.account,
                        pending.new_threshold,
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn apply_terminal_threshold_cleanup(proposal_id: u64, status: u8) -> DispatchResult {
        match (InternalProposalRoles::<T>::get(proposal_id), status) {
            (
                Some(InternalProposalRole::LifecycleCreate),
                STATUS_REJECTED | STATUS_EXECUTION_FAILED,
            ) => {
                PendingDynamicThresholds::<T>::remove(proposal_id);
            }
            (
                Some(InternalProposalRole::AdminChange),
                STATUS_REJECTED | STATUS_EXECUTION_FAILED,
            ) => {
                PendingAdminChangeThresholds::<T>::remove(proposal_id);
            }
            (Some(_), STATUS_EXECUTED) | (None, _) => {}
            _ => {}
        }
        Ok(())
    }
}
