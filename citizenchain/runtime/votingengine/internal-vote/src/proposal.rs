//! 内部投票提案创建、管理员快照和互斥登记。

use super::*;

impl<T: Config> Pallet<T> {
    /// 创建机构内部提案。机构身份只使用 CID，账户只在确有资产执行时写入。
    pub fn do_create_institution_proposal(
        who: T::AccountId,
        institution_code: InstitutionCode,
        actor_cid_number: sp_std::vec::Vec<u8>,
        execution_account: Option<T::AccountId>,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
    ) -> Result<u64, DispatchError> {
        Self::do_create_institution_proposal_with_mutex(
            who,
            institution_code,
            actor_cid_number,
            execution_account,
            subject_cid_numbers,
            InternalProposalMutexKind::Regular,
            InternalProposalRole::General,
        )
    }

    pub(crate) fn do_create_institution_proposal_with_mutex(
        who: T::AccountId,
        institution_code: InstitutionCode,
        actor_cid_number: sp_std::vec::Vec<u8>,
        execution_account: Option<T::AccountId>,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        mutex_kind: InternalProposalMutexKind,
        role: InternalProposalRole,
    ) -> Result<u64, DispatchError> {
        ensure!(
            is_valid_governance_code(&institution_code) && !is_personal_code(&institution_code),
            Error::<T>::InvalidInternalCode
        );
        let actor_cid_number = CidNumber::try_from(actor_cid_number)
            .map_err(|_| votingengine::Error::<T>::InvalidInstitution)?;
        ensure!(
            is_valid_institution_context(institution_code, actor_cid_number.as_slice()),
            votingengine::Error::<T>::InvalidInstitution
        );
        ensure!(
            is_institution_admin::<T>(institution_code, actor_cid_number.as_slice(), &who),
            votingengine::Error::<T>::NoPermission
        );

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());
        let subject_cid_numbers =
            <votingengine::Pallet<T>>::bound_subject_cid_numbers(subject_cid_numbers)?;
        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: votingengine::STATUS_VOTING,
            internal_code: Some(institution_code),
            actor_cid_number: Some(actor_cid_number.clone()),
            execution_account,
            subject_cid_numbers,
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match Self::allocate_and_lock(&proposal, mutex_kind) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            if let Err(err) = <votingengine::Pallet<T>>::snapshot_institution_admins(
                id,
                institution_code,
                actor_cid_number.clone(),
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }
            let subject = ProposalSubject::InstitutionCid(actor_cid_number.clone());
            if !<votingengine::Pallet<T>>::is_admin_in_snapshot(id, subject.clone(), &who) {
                frame_support::defensive!(
                    "do_create_institution_proposal: proposer is missing from CID admin snapshot"
                );
                return TransactionOutcome::Rollback(Err(
                    votingengine::Error::<T>::NoPermission.into()
                ));
            }
            let snapshot_size = match Self::snapshot_admins_len_or_missing(id, subject) {
                Ok(size) => size,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let threshold =
                if let Some(fixed_threshold) = fixed_governance_pass_threshold(&institution_code) {
                    fixed_threshold
                } else if primitives::institution_constraints::is_permanent_singleton_code(
                    &institution_code,
                ) {
                    snapshot_size / 2 + 1
                } else {
                    match active_institution_threshold::<T>(institution_code, &actor_cid_number) {
                        Some(threshold) => threshold,
                        None => {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::MissingDynamicThreshold.into(),
                            ))
                        }
                    }
                };
            let threshold_check =
                if primitives::institution_constraints::is_permanent_singleton_code(
                    &institution_code,
                ) || is_registered_multisig_code(&institution_code)
                {
                    Self::ensure_dynamic_threshold(snapshot_size, threshold)
                } else {
                    Self::ensure_threshold_within_snapshot(snapshot_size, threshold)
                };
            if let Err(err) = threshold_check {
                return TransactionOutcome::Rollback(Err(err));
            }
            Self::finish_proposal_create(id, proposal, end, threshold, role)
        })
    }

    /// 创建个人多签普通内部提案。
    pub fn do_create_personal_proposal(
        who: T::AccountId,
        personal_account: T::AccountId,
    ) -> Result<u64, DispatchError> {
        ensure!(
            is_personal_admin::<T>(personal_account.clone(), &who),
            votingengine::Error::<T>::NoPermission
        );
        Self::do_create_active_personal_proposal(
            who,
            personal_account,
            InternalProposalMutexKind::Regular,
            InternalProposalRole::General,
            false,
        )
    }

    /// 创建个人多签注销提案，要求当前管理员全员通过。
    pub fn do_create_personal_lifecycle_proposal(
        who: T::AccountId,
        personal_account: T::AccountId,
    ) -> Result<u64, DispatchError> {
        ensure!(
            is_personal_admin::<T>(personal_account.clone(), &who),
            votingengine::Error::<T>::NoPermission
        );
        Self::do_create_active_personal_proposal(
            who,
            personal_account,
            InternalProposalMutexKind::Regular,
            InternalProposalRole::PersonalClose,
            true,
        )
    }

    /// 创建个人多签管理员变更提案；新阈值只在提案执行成功后激活。
    pub fn do_create_personal_admin_change_proposal(
        who: T::AccountId,
        personal_account: T::AccountId,
        new_admins_len: u32,
        new_threshold: u32,
    ) -> Result<u64, DispatchError> {
        Self::ensure_dynamic_threshold(new_admins_len, new_threshold)?;
        ensure!(
            is_personal_admin::<T>(personal_account.clone(), &who),
            votingengine::Error::<T>::NoPermission
        );
        let proposal_id = Self::do_create_active_personal_proposal(
            who,
            personal_account.clone(),
            InternalProposalMutexKind::AdminSetMutationExclusive,
            InternalProposalRole::PersonalAdminChange,
            false,
        )?;
        PendingPersonalAdminChangeThresholds::<T>::insert(
            proposal_id,
            PendingPersonalAdminChangeThreshold {
                personal_account,
                new_admins_len,
                new_threshold,
            },
        );
        Ok(proposal_id)
    }

    /// 创建待注册个人多签提案。待注册管理员由调用方在同一事务中提供并锁定。
    pub fn do_create_personal_account_create_proposal(
        who: T::AccountId,
        personal_account: T::AccountId,
        admins: sp_std::vec::Vec<T::AccountId>,
        dynamic_threshold: u32,
    ) -> Result<u64, DispatchError> {
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
        let bounded_admins: BoundedVec<
            T::AccountId,
            <T as votingengine::Config>::MaxAdminsPerInstitution,
        > = admins
            .try_into()
            .map_err(|_| votingengine::Error::<T>::InvalidInstitution)?;

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());
        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: votingengine::STATUS_VOTING,
            internal_code: Some(votingengine::types::PMUL),
            actor_cid_number: None,
            execution_account: Some(personal_account.clone()),
            subject_cid_numbers: ProposalSubjectCidNumbers::default(),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match Self::allocate_and_lock(&proposal, InternalProposalMutexKind::Regular) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            AdminSnapshot::<T>::insert(
                id,
                ProposalSubject::PersonalAccount(personal_account),
                bounded_admins,
            );
            PendingPersonalThresholds::<T>::insert(id, dynamic_threshold);
            Self::finish_proposal_create(
                id,
                proposal,
                end,
                admins_len,
                InternalProposalRole::PersonalCreate,
            )
        })
    }

    fn do_create_active_personal_proposal(
        who: T::AccountId,
        personal_account: T::AccountId,
        mutex_kind: InternalProposalMutexKind,
        role: InternalProposalRole,
        force_all_admin_threshold: bool,
    ) -> Result<u64, DispatchError> {
        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());
        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: votingengine::STATUS_VOTING,
            internal_code: Some(votingengine::types::PMUL),
            actor_cid_number: None,
            execution_account: Some(personal_account.clone()),
            subject_cid_numbers: ProposalSubjectCidNumbers::default(),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match Self::allocate_and_lock(&proposal, mutex_kind) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            if let Err(err) = <votingengine::Pallet<T>>::snapshot_personal_admins(
                id,
                personal_account.clone(),
                false,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }
            let subject = ProposalSubject::PersonalAccount(personal_account.clone());
            if !<votingengine::Pallet<T>>::is_admin_in_snapshot(id, subject.clone(), &who) {
                return TransactionOutcome::Rollback(Err(
                    votingengine::Error::<T>::NoPermission.into()
                ));
            }
            let snapshot_size = match Self::snapshot_admins_len_or_missing(id, subject) {
                Ok(size) => size,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let threshold = if force_all_admin_threshold {
                snapshot_size
            } else {
                match ActivePersonalThresholds::<T>::get(personal_account) {
                    Some(threshold) => threshold,
                    None => {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::MissingDynamicThreshold.into(),
                        ))
                    }
                }
            };
            let threshold_check = if force_all_admin_threshold {
                Self::ensure_all_admin_threshold(snapshot_size, threshold)
            } else {
                Self::ensure_dynamic_threshold(snapshot_size, threshold)
            };
            if let Err(err) = threshold_check {
                return TransactionOutcome::Rollback(Err(err));
            }
            Self::finish_proposal_create(id, proposal, end, threshold, role)
        })
    }

    fn allocate_and_lock(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        mutex_kind: InternalProposalMutexKind,
    ) -> Result<u64, DispatchError> {
        let id = <votingengine::Pallet<T>>::allocate_proposal_id()?;
        votingengine::limit::try_add_active_proposals::<T>(proposal.subject_keys(), id)?;
        for subject in proposal.subject_keys() {
            <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(id, subject, mutex_kind)?;
        }
        Ok(id)
    }

    fn finish_proposal_create(
        id: u64,
        proposal: Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        end: frame_system::pallet_prelude::BlockNumberFor<T>,
        threshold: u32,
        role: InternalProposalRole,
    ) -> TransactionOutcome<Result<u64, DispatchError>> {
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
    }

    pub(crate) fn register_data_and_auto_approve(
        who: T::AccountId,
        proposal_id: u64,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        let now = <frame_system::Pallet<T>>::block_number();
        <votingengine::Pallet<T>>::register_proposal_data(proposal_id, module_tag, data, now)?;
        // 发起人签名发起提案后，投票引擎在同一事务自动记一票赞成。
        Self::do_internal_vote(who, proposal_id, true)?;
        Ok(proposal_id)
    }
}
