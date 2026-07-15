//! 内部投票提案创建、管理员快照和互斥登记。

use super::*;

impl<T: Config> Pallet<T> {
    pub fn do_create_registered_account_create_proposal(
        who: T::AccountId,
        institution_code: InstitutionCode,
        institution: T::AccountId,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        admins: sp_std::vec::Vec<T::AccountId>,
        dynamic_threshold: u32,
    ) -> Result<u64, DispatchError> {
        ensure!(
            is_registered_multisig_code(&institution_code)
                && !primitives::institution_constraints::is_permanent_singleton_code(
                    &institution_code
                ),
            Error::<T>::InvalidInternalCode
        );
        ensure!(
            !admins.is_empty(),
            votingengine::Error::<T>::MissingAdminSnapshot
        );
        ensure!(
            admins.iter().any(|admin| admin == &who),
            votingengine::Error::<T>::NoPermission
        );
        for i in 0..admins.len() {
            for j in i.saturating_add(1)..admins.len() {
                ensure!(
                    admins[i] != admins[j],
                    votingengine::Error::<T>::InvalidInstitution
                );
            }
        }
        let admins_len = admins.len() as u32;
        Self::ensure_dynamic_threshold(admins_len, dynamic_threshold)?;
        let lifecycle_threshold = admins_len;
        let bounded_admins: BoundedVec<
            T::AccountId,
            <T as votingengine::Config>::MaxAdminsPerInstitution,
        > = admins
            .try_into()
            .map_err(|_| votingengine::Error::<T>::InvalidInstitution)?;

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());
        let subject_cid_numbers =
            Self::bound_and_validate_subject_cids(institution_code, subject_cid_numbers)?;
        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: votingengine::STATUS_VOTING,
            internal_code: Some(institution_code),
            account_context: Some(institution.clone()),
            subject_cid_numbers,
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match <votingengine::Pallet<T>>::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let subjects = proposal.subject_keys();
            if let Err(err) = votingengine::limit::try_add_active_proposals::<T>(subjects, id) {
                return TransactionOutcome::Rollback(Err(err));
            }
            for subject in proposal.subject_keys() {
                if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                    id,
                    subject,
                    InternalProposalMutexKind::Regular,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }
            }

            AdminSnapshot::<T>::insert(id, institution.clone(), bounded_admins);
            InternalThresholdSnapshot::<T>::insert(id, lifecycle_threshold);
            // 待激活阈值必须绑定具体提案，避免同一机构并发注册时互相覆盖。
            PendingDynamicThresholds::<T>::insert(id, dynamic_threshold);
            InternalProposalRoles::<T>::insert(id, InternalProposalRole::LifecycleCreate);
            Proposals::<T>::insert(id, proposal);
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_INTERNAL,
                STAGE_INTERNAL,
                end,
            );
            TransactionOutcome::Commit(Ok(id))
        })
    }

    pub fn do_create_general_internal_proposal(
        who: T::AccountId,
        institution_code: InstitutionCode,
        institution: T::AccountId,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
    ) -> Result<u64, DispatchError> {
        Self::do_create_active_account_internal_proposal(
            who,
            institution_code,
            institution.clone(),
            subject_cid_numbers,
            InternalProposalMutexKind::Regular,
            InternalProposalRole::General,
            None,
        )
    }

    pub fn do_create_lifecycle_internal_proposal(
        who: T::AccountId,
        institution_code: InstitutionCode,
        institution: T::AccountId,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
    ) -> Result<u64, DispatchError> {
        ensure!(
            is_registered_multisig_code(&institution_code)
                && !primitives::institution_constraints::is_permanent_singleton_code(
                    &institution_code
                ),
            Error::<T>::InvalidInternalCode
        );
        Self::do_create_active_account_internal_proposal(
            who,
            institution_code,
            institution,
            subject_cid_numbers,
            InternalProposalMutexKind::Regular,
            InternalProposalRole::LifecycleClose,
            Some(true),
        )
    }

    pub fn do_create_admin_change_internal_proposal(
        who: T::AccountId,
        institution_code: InstitutionCode,
        institution: T::AccountId,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        new_admins_len: u32,
        new_threshold: u32,
    ) -> Result<u64, DispatchError> {
        ensure!(
            !primitives::institution_constraints::is_permanent_singleton_code(&institution_code),
            Error::<T>::InvalidInternalCode
        );
        if is_registered_multisig_code(&institution_code) {
            Self::ensure_dynamic_threshold(new_admins_len, new_threshold)?;
        } else {
            ensure!(
                fixed_governance_pass_threshold(&institution_code) == Some(new_threshold),
                Error::<T>::InvalidDynamicThreshold
            );
        }
        let proposal_id = Self::do_create_active_account_internal_proposal(
            who,
            institution_code,
            institution.clone(),
            subject_cid_numbers,
            InternalProposalMutexKind::AdminSetMutationExclusive,
            InternalProposalRole::AdminChange,
            Some(false),
        )?;
        if is_registered_multisig_code(&institution_code) {
            PendingAdminChangeThresholds::<T>::insert(
                proposal_id,
                PendingAdminChangeThreshold {
                    institution_code,
                    account: institution,
                    new_admins_len,
                    new_threshold,
                },
            );
        }
        Ok(proposal_id)
    }

    pub(crate) fn do_create_active_account_internal_proposal(
        who: T::AccountId,
        institution_code: InstitutionCode,
        institution: T::AccountId,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        mutex_kind: InternalProposalMutexKind,
        role: InternalProposalRole,
        force_all_admin_threshold: Option<bool>,
    ) -> Result<u64, DispatchError> {
        ensure!(
            is_valid_governance_code(&institution_code),
            Error::<T>::InvalidInternalCode
        );
        ensure!(
            is_valid_account_context::<T>(institution_code, institution.clone()),
            votingengine::Error::<T>::InvalidInstitution
        );
        ensure!(
            is_internal_admin::<T>(institution_code, institution.clone(), &who),
            votingengine::Error::<T>::NoPermission
        );
        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());
        let subject_cid_numbers =
            Self::bound_and_validate_subject_cids(institution_code, subject_cid_numbers)?;

        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: votingengine::STATUS_VOTING,
            internal_code: Some(institution_code),
            account_context: Some(institution.clone()),
            subject_cid_numbers,
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match <votingengine::Pallet<T>>::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            let subjects = proposal.subject_keys();
            if let Err(err) = votingengine::limit::try_add_active_proposals::<T>(subjects, id) {
                return TransactionOutcome::Rollback(Err(err));
            }
            for subject in proposal.subject_keys() {
                if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                    id, subject, mutex_kind,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }
            }

            if let Err(err) = <votingengine::Pallet<T>>::snapshot_institution_admins(
                id,
                institution_code,
                institution.clone(),
                false,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }
            if !<votingengine::Pallet<T>>::is_admin_in_snapshot(id, institution.clone(), &who) {
                frame_support::defensive!(
                    "do_create_internal_proposal: proposer is missing from admin snapshot"
                );
                return TransactionOutcome::Rollback(Err(
                    votingengine::Error::<T>::NoPermission.into()
                ));
            }
            let snapshot_size = match Self::snapshot_admins_len_or_missing(id, institution.clone())
            {
                Ok(size) => size,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let threshold = if force_all_admin_threshold.unwrap_or(false) {
                snapshot_size
            } else if let Some(fixed_threshold) = fixed_governance_pass_threshold(&institution_code)
            {
                fixed_threshold
            } else if primitives::institution_constraints::is_permanent_singleton_code(
                &institution_code,
            ) {
                // 六个国家级永久单例没有账户级动态阈值；普通内部事项在提案创建时
                // 直接对当前 admins 快照计算最小严格过半，并只写提案阈值快照。
                snapshot_size / 2 + 1
            } else {
                match active_internal_threshold::<T>(institution_code, institution.clone()) {
                    Some(threshold) => threshold,
                    None => {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::InvalidInternalCode.into()
                        ))
                    }
                }
            };
            let threshold_check = if force_all_admin_threshold.unwrap_or(false) {
                Self::ensure_all_admin_threshold(snapshot_size, threshold)
            } else if primitives::institution_constraints::is_permanent_singleton_code(
                &institution_code,
            ) {
                Self::ensure_dynamic_threshold(snapshot_size, threshold)
            } else if is_registered_multisig_code(&institution_code) {
                Self::ensure_dynamic_threshold(snapshot_size, threshold)
            } else {
                Self::ensure_threshold_within_snapshot(snapshot_size, threshold)
            };
            if let Err(err) = threshold_check {
                return TransactionOutcome::Rollback(Err(err));
            }
            InternalThresholdSnapshot::<T>::insert(id, threshold);
            InternalProposalRoles::<T>::insert(id, role);

            Proposals::<T>::insert(id, proposal);
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_INTERNAL,
                STAGE_INTERNAL,
                end,
            );
            TransactionOutcome::Commit(Ok(id))
        })
    }

    pub(crate) fn register_data_and_auto_approve(
        who: T::AccountId,
        proposal_id: u64,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        let now = <frame_system::Pallet<T>>::block_number();
        <votingengine::Pallet<T>>::register_proposal_data(proposal_id, module_tag, data, now)?;
        // 发起人签名发起提案后，投票引擎在同一事务自动记一票赞成，
        // 用户不需要再发第二笔“同意”交易。
        Self::do_internal_vote(who, proposal_id, true)?;
        Ok(proposal_id)
    }
}
