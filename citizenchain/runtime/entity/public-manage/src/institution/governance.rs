//! 公权机构治理结果原子应用。
//!
//! 业务模块不能直接改岗位、任职、机构信息或 admins；只能通过 runtime 路由提交
//! 已经完成的 [`InstitutionGovernanceResult`]。本文件只校验实体不变量，不解释任何
//! 提名、选举或其他业务规则。

extern crate alloc;

use admin_primitives::InstitutionAdminQuery as _;
use alloc::{collections::BTreeMap, vec::Vec};
use entity_primitives::{
    InstitutionAdminAssignment, InstitutionAssignmentSource, InstitutionAssignmentStatus,
    InstitutionGovernanceResult, InstitutionLegalRepresentativeChange, InstitutionRole,
    InstitutionRoleStatus,
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
    AccountNameOf, CidNumberOf, Config, Error, InstitutionRoleAssignments, InstitutionRoles,
    Institutions, Pallet,
};

enum LegalRepresentativeTarget<T: Config> {
    Set(AccountNameOf<T>, CidNumberOf<T>, T::AccountId),
    Clear,
}

impl<T: Config> Pallet<T> {
    /// 原子应用公权机构岗位、任职和法定代表人最终状态；管理员集合保持独立。
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

        let cid_number: CidNumberOf<T> = result
            .cid_number
            .clone()
            .try_into()
            .map_err(|_| Error::<T>::InvalidAssignmentResultInstitution)?;
        let institution = Institutions::<T>::get(&cid_number)
            .ok_or(Error::<T>::InvalidAssignmentResultInstitution)?;
        ensure!(
            institution.institution_code == result.institution_code,
            Error::<T>::InvalidAssignmentResultInstitution
        );
        let protected_institution = primitives::governance_skeleton::fixed_institution_by_identity(
            result.institution_code,
            cid_number.as_slice(),
        )
        .is_some();
        let member_composition =
            primitives::institution_constraints::member_composition_by_identity(
                result.institution_code,
                cid_number.as_slice(),
            );
        let current_admins = T::InstitutionAdminQuery::institution_admins(
            result.institution_code,
            cid_number.as_slice(),
        )
        .ok_or(Error::<T>::InvalidAssignmentResultInstitution)?;
        let current_admin_set = current_admins.iter().cloned().collect::<BTreeSet<_>>();

        let mut final_roles = InstitutionRoles::<T>::iter_prefix(&cid_number)
            .collect::<BTreeMap<RoleCodeOf, InstitutionRoleOf<T>>>();
        let mut role_changes = BTreeMap::<RoleCodeOf, InstitutionRoleOf<T>>::new();
        for change in result.role_changes {
            // 89 个受保护创世机构的岗位定义由治理骨架固定；依法轮换只改变任职账户。
            // 其他机构即使机构类型相同，也由 runtime 治理结果动态调整岗位结构。
            ensure!(
                !protected_institution,
                Error::<T>::FixedRoleDefinitionImmutable
            );
            ensure!(!change.role_code.is_empty(), Error::<T>::InvalidRoleCode);
            ensure!(!change.role_name.is_empty(), Error::<T>::InvalidRoleName);
            let role_code: RoleCodeOf = change
                .role_code
                .try_into()
                .map_err(|_| Error::<T>::InvalidRoleCode)?;
            ensure!(
                !primitives::institution_constraints::is_legal_representative_role(
                    role_code.as_slice()
                ),
                Error::<T>::FixedRoleDefinitionImmutable
            );
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
                    current_admin_set.contains(&target.admin_account),
                    Error::<T>::InvalidAssignmentResultAdmins
                );
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
                            | InstitutionAssignmentSource::InstitutionGovernance
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
                    current_admin_set.contains(&assignment.admin_account),
                    Error::<T>::InvalidAssignmentResultAdmins
                );
                ensure!(
                    assignment.assignment_status == InstitutionAssignmentStatus::Active,
                    Error::<T>::InitialAssignmentMustBeActive
                );
                Self::ensure_governance_assignment_term(
                    role,
                    assignment.term_start,
                    assignment.term_end,
                )?;
            }
            if protected_institution {
                let seats = primitives::governance_skeleton::fixed_role_seats_by_identity(
                    result.institution_code,
                    cid_number.as_slice(),
                    role_code.as_slice(),
                )
                .ok_or(Error::<T>::InvalidRoleCode)?;
                ensure!(
                    assignments.len() == seats as usize,
                    Error::<T>::FixedRoleSeatsMismatch
                );
            }
        }
        if let Some(spec) = member_composition {
            let required_role_code: RoleCodeOf = spec
                .role_code
                .to_vec()
                .try_into()
                .map_err(|_| Error::<T>::InvalidRoleCode)?;
            let required_role = final_roles
                .get(&required_role_code)
                .ok_or(Error::<T>::RequiredMemberRoleMissing)?;
            ensure!(
                required_role.role_status == InstitutionRoleStatus::Active,
                Error::<T>::RequiredMemberRoleMissing
            );
            ensure!(
                required_role.role_name.as_slice() == spec.role_name,
                Error::<T>::RequiredMemberRoleNameMismatch
            );
            let required_assignments = assignment_changes
                .get(&required_role_code)
                .cloned()
                .unwrap_or_else(|| {
                    InstitutionRoleAssignments::<T>::get(&cid_number, &required_role_code)
                });
            ensure!(
                required_assignments.len() >= spec.min_members as usize
                    && required_assignments.len() <= spec.max_members as usize,
                Error::<T>::RequiredMemberCountOutOfRange
            );
            let required_admins = required_assignments
                .iter()
                .map(|assignment| assignment.admin_account.clone())
                .collect::<BTreeSet<_>>();
            ensure!(
                required_admins == current_admin_set,
                Error::<T>::NonMemberAdminForbidden
            );
        }

        let legal_representative_change = result
            .legal_representative_change
            .map(|change| {
                match change {
                    InstitutionLegalRepresentativeChange::Set {
                        legal_representative_name,
                        legal_representative_cid_number,
                        legal_representative_account,
                    } => {
                        ensure!(
                            !legal_representative_name.is_empty(),
                            Error::<T>::EmptyLegalRepresentativeName
                        );
                        ensure!(
                            !legal_representative_cid_number.is_empty(),
                            Error::<T>::EmptyLegalRepresentativeCidNumber
                        );
                        let name: AccountNameOf<T> = legal_representative_name
                            .try_into()
                            .map_err(|_| Error::<T>::EmptyLegalRepresentativeName)?;
                        let citizen_cid: CidNumberOf<T> = legal_representative_cid_number
                            .try_into()
                            .map_err(|_| Error::<T>::EmptyLegalRepresentativeCidNumber)?;
                        Ok::<_, sp_runtime::DispatchError>(LegalRepresentativeTarget::<T>::Set(
                            name,
                            citizen_cid,
                            legal_representative_account,
                        ))
                    }
                    // 解除法定代表人只清空 InstitutionInfo 三字段，不影响 LR 岗位本身。
                    InstitutionLegalRepresentativeChange::Clear => {
                        Ok(LegalRepresentativeTarget::<T>::Clear)
                    }
                }
            })
            .transpose()?;
        let role_changes_len = role_changes.len() as u32;
        let assignment_changes_len = assignment_changes.len() as u32;
        let admins_len = current_admins.len() as u32;
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
            if let Some(change) = legal_representative_change {
                Institutions::<T>::mutate(&cid_number, |maybe| {
                    if let Some(info) = maybe {
                        match change {
                            LegalRepresentativeTarget::Set(name, citizen_cid, account) => {
                                info.legal_representative_name = Some(name);
                                info.legal_representative_cid_number = Some(citizen_cid);
                                info.legal_representative_account = Some(account);
                            }
                            LegalRepresentativeTarget::Clear => {
                                info.legal_representative_name = None;
                                info.legal_representative_cid_number = None;
                                info.legal_representative_account = None;
                            }
                        }
                    }
                });
            }
            Self::deposit_event(crate::pallet::Event::<T>::InstitutionGovernanceApplied {
                cid_number,
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
