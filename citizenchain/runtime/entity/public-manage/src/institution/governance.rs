//! 公权机构治理结果原子应用。
//!
//! 业务模块不能直接改岗位、任职、机构信息或 admins；只能通过 runtime 路由提交
//! 已经完成的 [`InstitutionGovernanceResult`]。本文件只校验实体不变量，不解释任何
//! 提名、选举或其他业务规则。

extern crate alloc;

use admin_primitives::{AdminAccountQuery as _, InstitutionAdminAccountLifecycle as _};
use alloc::{collections::BTreeMap, vec::Vec};
use entity_primitives::{
    InstitutionAdminAssignment, InstitutionAssignmentSource, InstitutionAssignmentStatus,
    InstitutionGovernanceResult, InstitutionRole, InstitutionRoleStatus,
};
use frame_support::{
    dispatch::DispatchResult,
    ensure,
    storage::{with_transaction, TransactionOutcome},
    traits::Get,
};
use sp_std::collections::btree_set::BTreeSet;

use crate::institution::role::{
    AssignmentSourceRefOf, InstitutionRoleOf, RoleAssignmentsOf, RoleCodeOf,
};
use crate::pallet::{
    AccountNameOf, AccountRegisteredCid, CidNumberOf, Config, Error, InstitutionRoleAssignments,
    InstitutionRoles, Institutions, Pallet,
};

impl<T: Config> Pallet<T> {
    /// 原子应用公权机构岗位、任职和法定代表人最终状态，并从有效任职派生 admins。
    pub fn apply_institution_governance_result(
        result: InstitutionGovernanceResult<T::AccountId>,
    ) -> DispatchResult {
        ensure!(
            admin_primitives::is_public_admin_code(&result.institution_code),
            Error::<T>::InvalidInstitutionCode
        );
        ensure!(
            !result.role_changes.is_empty()
                || !result.assignment_changes.is_empty()
                || result.legal_representative_change.is_some(),
            Error::<T>::GovernanceResultEmpty
        );
        ensure!(
            !result.result_source_ref.is_empty(),
            Error::<T>::AssignmentSourceRefEmpty
        );
        ensure!(
            result.role_changes.len() as u32 <= T::MaxAdmins::get()
                && result.assignment_changes.len() as u32 <= T::MaxAdmins::get(),
            Error::<T>::TooManyGovernanceChanges
        );
        let result_source_ref: AssignmentSourceRefOf = result
            .result_source_ref
            .try_into()
            .map_err(|_| Error::<T>::AssignmentSourceRefEmpty)?;

        let registered = AccountRegisteredCid::<T>::get(&result.institution_account)
            .ok_or(Error::<T>::InvalidAssignmentResultInstitution)?;
        let cid_number = registered.cid_number;
        let main_account = Self::resolve_admin_account_for_account(&result.institution_account)
            .ok_or(Error::<T>::InvalidAssignmentResultInstitution)?;
        ensure!(
            main_account == result.institution_account,
            Error::<T>::InvalidAssignmentResultInstitution
        );
        let institution = Institutions::<T>::get(&cid_number)
            .ok_or(Error::<T>::InvalidAssignmentResultInstitution)?;
        ensure!(
            institution.institution_code == result.institution_code
                && institution.status == entity_primitives::InstitutionLifecycleStatus::Active,
            Error::<T>::InvalidAssignmentResultInstitution
        );

        let mut final_roles = InstitutionRoles::<T>::iter_prefix(&cid_number)
            .collect::<BTreeMap<RoleCodeOf, InstitutionRoleOf<T>>>();
        let mut role_changes = BTreeMap::<RoleCodeOf, InstitutionRoleOf<T>>::new();
        for change in result.role_changes {
            // 五类创世机构岗位定义由治理骨架固定；依法轮换只改变任职账户。
            ensure!(
                !primitives::cid::code::is_fixed_governance_code(&result.institution_code),
                Error::<T>::FixedRoleDefinitionImmutable
            );
            ensure!(!change.role_code.is_empty(), Error::<T>::InvalidRoleCode);
            ensure!(!change.role_name.is_empty(), Error::<T>::InvalidRoleName);
            let role_code: RoleCodeOf = change
                .role_code
                .try_into()
                .map_err(|_| Error::<T>::InvalidRoleCode)?;
            let role_name: AccountNameOf<T> = change
                .role_name
                .try_into()
                .map_err(|_| Error::<T>::InvalidRoleName)?;
            ensure!(
                !role_changes.contains_key(&role_code),
                Error::<T>::DuplicateGovernanceRoleChange
            );
            let role = InstitutionRole {
                cid_number: cid_number.clone(),
                role_code: role_code.clone(),
                role_name,
                term_required: change.term_required,
                role_status: change.role_status,
            };
            role_changes.insert(role_code.clone(), role.clone());
            final_roles.insert(role_code, role);
        }
        ensure!(
            final_roles.len() as u32 <= T::MaxAdmins::get(),
            Error::<T>::TooManyGovernanceChanges
        );

        let mut assignment_changes = BTreeMap::<RoleCodeOf, RoleAssignmentsOf<T>>::new();
        for change in result.assignment_changes {
            ensure!(!change.role_code.is_empty(), Error::<T>::InvalidRoleCode);
            let role_code: RoleCodeOf = change
                .role_code
                .try_into()
                .map_err(|_| Error::<T>::InvalidRoleCode)?;
            let role = final_roles
                .get(&role_code)
                .ok_or(Error::<T>::AssignmentRoleNotFound)?;
            ensure!(
                !assignment_changes.contains_key(&role_code),
                Error::<T>::DuplicateGovernanceAssignmentChange
            );
            ensure!(
                change.assignments.len() as u32 <= T::MaxAdmins::get(),
                Error::<T>::TooManyInstitutionAdmins
            );
            let mut seen_accounts = BTreeSet::new();
            let mut stored_assignments = Vec::with_capacity(change.assignments.len());
            for target in change.assignments {
                ensure!(
                    target.assignment_status == InstitutionAssignmentStatus::Active,
                    Error::<T>::InitialAssignmentMustBeActive
                );
                ensure!(
                    matches!(
                        target.assignment_source,
                        InstitutionAssignmentSource::PopularElection
                            | InstitutionAssignmentSource::MutualElection
                            | InstitutionAssignmentSource::NominationAppointment
                    ),
                    Error::<T>::InvalidAssignmentSource
                );
                ensure!(
                    !target.assignment_source_ref.is_empty(),
                    Error::<T>::AssignmentSourceRefEmpty
                );
                ensure!(
                    seen_accounts.insert(target.admin_account.clone()),
                    Error::<T>::DuplicateAssignment
                );
                Self::ensure_governance_assignment_term(role, target.term_start, target.term_end)?;
                let assignment_source_ref: AssignmentSourceRefOf = target
                    .assignment_source_ref
                    .try_into()
                    .map_err(|_| Error::<T>::AssignmentSourceRefEmpty)?;
                stored_assignments.push(InstitutionAdminAssignment {
                    cid_number: cid_number.clone(),
                    admin_account: target.admin_account,
                    role_code: role_code.clone(),
                    term_start: target.term_start,
                    term_end: target.term_end,
                    assignment_source: target.assignment_source,
                    assignment_source_ref,
                    assignment_status: target.assignment_status,
                });
            }
            let bounded: RoleAssignmentsOf<T> = stored_assignments
                .try_into()
                .map_err(|_| Error::<T>::TooManyInstitutionAdmins)?;
            assignment_changes.insert(role_code, bounded);
        }

        let mut assignment_order = Vec::new();
        let mut desired_admins = BTreeSet::new();
        for (role_code, role) in &final_roles {
            let assignments = assignment_changes
                .get(role_code)
                .cloned()
                .unwrap_or_else(|| InstitutionRoleAssignments::<T>::get(&cid_number, role_code));
            if role.role_status == InstitutionRoleStatus::Inactive {
                ensure!(
                    assignments.is_empty(),
                    Error::<T>::InactiveRoleHasAssignments
                );
                continue;
            }
            for assignment in &assignments {
                ensure!(
                    assignment.assignment_status == InstitutionAssignmentStatus::Active,
                    Error::<T>::InitialAssignmentMustBeActive
                );
                Self::ensure_governance_assignment_term(
                    role,
                    assignment.term_start,
                    assignment.term_end,
                )?;
                if desired_admins.insert(assignment.admin_account.clone()) {
                    assignment_order.push(assignment.admin_account.clone());
                }
            }
            if primitives::cid::code::is_fixed_governance_code(&result.institution_code) {
                let seats = primitives::governance_skeleton::fixed_role_seats(
                    result.institution_code,
                    role_code.as_slice(),
                )
                .ok_or(Error::<T>::InvalidRoleCode)?;
                ensure!(
                    assignments.len() == seats as usize,
                    Error::<T>::FixedRoleSeatsMismatch
                );
            }
        }
        ensure!(
            !desired_admins.is_empty(),
            Error::<T>::InvalidAssignmentResultAdmins
        );
        ensure!(
            desired_admins.len() as u32 <= T::MaxAdmins::get(),
            Error::<T>::TooManyInstitutionAdmins
        );

        // 尽量保留既有 admins 顺序，只在尾部追加新账户；管理员真源仍是最终任职集合。
        let current_admins = T::AdminAccountQuery::active_account_admins(
            result.institution_code,
            main_account.clone(),
        )
        .ok_or(Error::<T>::InvalidAssignmentResultInstitution)?;
        let mut remaining = desired_admins;
        let mut derived_admins = Vec::new();
        for admin in current_admins.into_iter().chain(assignment_order) {
            if remaining.remove(&admin) {
                derived_admins.push(admin);
            }
        }
        ensure!(
            remaining.is_empty(),
            Error::<T>::InvalidAssignmentResultAdmins
        );

        let legal_representative_change = result
            .legal_representative_change
            .map(|change| {
                ensure!(
                    !change.legal_representative_name.is_empty(),
                    Error::<T>::EmptyLegalRepresentativeName
                );
                ensure!(
                    !change.legal_representative_cid_number.is_empty(),
                    Error::<T>::EmptyLegalRepresentativeCidNumber
                );
                let name: AccountNameOf<T> = change
                    .legal_representative_name
                    .try_into()
                    .map_err(|_| Error::<T>::EmptyLegalRepresentativeName)?;
                let citizen_cid: CidNumberOf<T> = change
                    .legal_representative_cid_number
                    .try_into()
                    .map_err(|_| Error::<T>::EmptyLegalRepresentativeCidNumber)?;
                Ok::<_, sp_runtime::DispatchError>((
                    name,
                    citizen_cid,
                    change.legal_representative_account,
                ))
            })
            .transpose()?;
        let role_changes_len = role_changes.len() as u32;
        let assignment_changes_len = assignment_changes.len() as u32;
        let admins_len = derived_admins.len() as u32;
        let legal_representative_updated = legal_representative_change.is_some();

        with_transaction(|| {
            for (role_code, role) in &role_changes {
                InstitutionRoles::<T>::insert(&cid_number, role_code, role.clone());
            }
            for (role_code, assignments) in &assignment_changes {
                InstitutionRoleAssignments::<T>::insert(
                    &cid_number,
                    role_code,
                    assignments.clone(),
                );
            }
            if let Some((name, citizen_cid, account)) = &legal_representative_change {
                Institutions::<T>::mutate(&cid_number, |maybe| {
                    if let Some(info) = maybe {
                        info.legal_representative_name = Some(name.clone());
                        info.legal_representative_cid_number = Some(citizen_cid.clone());
                        info.legal_representative_account = Some(account.clone());
                    }
                });
            }
            if let Err(err) = T::AdminLifecycle::sync_active_institution_admins_from_assignments(
                crate::MODULE_TAG,
                main_account.clone(),
                cid_number.to_vec(),
                result.institution_code,
                derived_admins.clone(),
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }
            Self::deposit_event(crate::pallet::Event::<T>::InstitutionGovernanceApplied {
                cid_number,
                institution_account: main_account,
                role_changes: role_changes_len,
                assignment_changes: assignment_changes_len,
                admins_len,
                legal_representative_updated,
                result_source_ref,
            });
            TransactionOutcome::Commit(Ok(()))
        })
    }

    fn ensure_governance_assignment_term(
        role: &InstitutionRoleOf<T>,
        term_start: u32,
        term_end: u32,
    ) -> DispatchResult {
        if role.term_required {
            ensure!(
                term_start > 0 && term_end > term_start,
                Error::<T>::InvalidAssignmentTerm
            );
        } else {
            ensure!(
                term_start == 0 && term_end == 0,
                Error::<T>::UnexpectedAssignmentTerm
            );
        }
        Ok(())
    }
}
