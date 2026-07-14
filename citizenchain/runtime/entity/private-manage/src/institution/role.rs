//! 私权机构岗位目录与管理员任职。
//!
//! 私权机构和公权机构复用同一数据结构；差异只在岗位、任职变更的授权来源，
//! 不在 `entity` 中建立任意字符串权限表。

extern crate alloc;

use alloc::vec::Vec;
use entity_primitives::{
    InstitutionAdminAssignment, InstitutionAssignmentSource, InstitutionAssignmentStatus,
    InstitutionRole, InstitutionRoleQuery, InstitutionRoleStatus, ASSIGNMENT_SOURCE_REF_MAX_BYTES,
    INSTITUTION_ROLE_CODE_MAX_BYTES,
};
use frame_support::{ensure, traits::ConstU32, BoundedVec};
use sp_runtime::DispatchError;
use sp_std::collections::btree_set::BTreeSet;

use crate::pallet::{
    AccountNameOf, AdminsOf, CidNumberOf, Config, Error, InstitutionRoleAssignments,
    InstitutionRoles, Pallet,
};

pub type RoleCodeOf = BoundedVec<u8, ConstU32<INSTITUTION_ROLE_CODE_MAX_BYTES>>;
pub type AssignmentSourceRefOf = BoundedVec<u8, ConstU32<ASSIGNMENT_SOURCE_REF_MAX_BYTES>>;
pub type InstitutionRoleOf<T> = InstitutionRole<CidNumberOf<T>, RoleCodeOf, AccountNameOf<T>>;
pub type InstitutionRolesOf<T> = BoundedVec<InstitutionRoleOf<T>, <T as Config>::MaxAdmins>;
pub type InstitutionAdminAssignmentOf<T> = InstitutionAdminAssignment<
    CidNumberOf<T>,
    <T as frame_system::Config>::AccountId,
    RoleCodeOf,
    AssignmentSourceRefOf,
>;
pub type InstitutionAdminAssignmentsOf<T> =
    BoundedVec<InstitutionAdminAssignmentOf<T>, <T as Config>::MaxAdmins>;
pub type RoleAssignmentsOf<T> =
    BoundedVec<InstitutionAdminAssignmentOf<T>, <T as Config>::MaxAdmins>;

impl<T: Config> Pallet<T> {
    /// 注册创建时校验并原子写入岗位、任职，返回去重管理员账户。
    pub fn store_initial_roles_and_assignments(
        cid_number: &CidNumberOf<T>,
        roles: &InstitutionRolesOf<T>,
        assignments: &InstitutionAdminAssignmentsOf<T>,
        expected_source: InstitutionAssignmentSource,
    ) -> Result<AdminsOf<T>, DispatchError> {
        ensure!(!roles.is_empty(), Error::<T>::InstitutionRolesEmpty);
        ensure!(
            !assignments.is_empty(),
            Error::<T>::InstitutionAssignmentsEmpty
        );

        let mut role_codes = BTreeSet::new();
        for role in roles.iter() {
            ensure!(role.cid_number == *cid_number, Error::<T>::RoleCidMismatch);
            ensure!(!role.role_code.is_empty(), Error::<T>::InvalidRoleCode);
            ensure!(!role.role_name.is_empty(), Error::<T>::InvalidRoleName);
            ensure!(
                role.role_status == InstitutionRoleStatus::Active,
                Error::<T>::InitialRoleMustBeActive
            );
            ensure!(
                role_codes.insert(role.role_code.clone()),
                Error::<T>::DuplicateRoleCode
            );
        }

        let mut assignment_keys = BTreeSet::new();
        let mut seen_admin_accounts = BTreeSet::new();
        let mut admin_accounts = Vec::new();
        for assignment in assignments.iter() {
            ensure!(
                assignment.cid_number == *cid_number,
                Error::<T>::AssignmentCidMismatch
            );
            ensure!(
                assignment.assignment_source == expected_source,
                Error::<T>::InvalidAssignmentSource
            );
            ensure!(
                assignment.assignment_status == InstitutionAssignmentStatus::Active,
                Error::<T>::InitialAssignmentMustBeActive
            );
            let role = roles
                .iter()
                .find(|role| role.role_code == assignment.role_code)
                .ok_or(Error::<T>::AssignmentRoleNotFound)?;
            if role.term_required {
                ensure!(
                    assignment.term_start > 0 && assignment.term_end > assignment.term_start,
                    Error::<T>::InvalidAssignmentTerm
                );
            } else {
                ensure!(
                    assignment.term_start == 0 && assignment.term_end == 0,
                    Error::<T>::UnexpectedAssignmentTerm
                );
            }
            ensure!(
                assignment_keys.insert((
                    assignment.admin_account.clone(),
                    assignment.role_code.clone(),
                )),
                Error::<T>::DuplicateAssignment
            );
            if seen_admin_accounts.insert(assignment.admin_account.clone()) {
                // 保持任职载荷顺序，admins 仅做去重派生，不按账户字节重新排序。
                admin_accounts.push(assignment.admin_account.clone());
            }
        }

        for role in roles.iter() {
            ensure!(
                assignments
                    .iter()
                    .any(|assignment| assignment.role_code == role.role_code),
                Error::<T>::RoleHasNoAssignment
            );
        }

        let bounded_admins: AdminsOf<T> = admin_accounts
            .try_into()
            .map_err(|_| Error::<T>::TooManyInstitutionAdmins)?;

        for role in roles.iter() {
            let role_assignments: Vec<InstitutionAdminAssignmentOf<T>> = assignments
                .iter()
                .filter(|assignment| assignment.role_code == role.role_code)
                .cloned()
                .collect();
            let bounded_assignments: RoleAssignmentsOf<T> = role_assignments
                .try_into()
                .map_err(|_| Error::<T>::TooManyInstitutionAdmins)?;
            InstitutionRoles::<T>::insert(cid_number, &role.role_code, role.clone());
            InstitutionRoleAssignments::<T>::insert(
                cid_number,
                &role.role_code,
                bounded_assignments,
            );
        }

        Ok(bounded_admins)
    }
}

impl<T: Config> InstitutionRoleQuery<T::AccountId> for Pallet<T> {
    fn is_active_assignment(cid_number: &[u8], admin: &T::AccountId, role_code: &[u8]) -> bool {
        let Ok(cid_number) = CidNumberOf::<T>::try_from(cid_number.to_vec()) else {
            return false;
        };
        let Ok(role_code) = RoleCodeOf::try_from(role_code.to_vec()) else {
            return false;
        };
        let Some(role) = InstitutionRoles::<T>::get(&cid_number, &role_code) else {
            return false;
        };
        if role.role_status != InstitutionRoleStatus::Active {
            return false;
        }
        InstitutionRoleAssignments::<T>::get(cid_number, role_code)
            .into_iter()
            .any(|assignment| {
                assignment.assignment_status == InstitutionAssignmentStatus::Active
                    && &assignment.admin_account == admin
            })
    }

    fn active_accounts_for_role(cid_number: &[u8], role_code: &[u8]) -> Vec<T::AccountId> {
        let Ok(cid_number) = CidNumberOf::<T>::try_from(cid_number.to_vec()) else {
            return Vec::new();
        };
        let Ok(role_code) = RoleCodeOf::try_from(role_code.to_vec()) else {
            return Vec::new();
        };
        let Some(role) = InstitutionRoles::<T>::get(&cid_number, &role_code) else {
            return Vec::new();
        };
        if role.role_status != InstitutionRoleStatus::Active {
            return Vec::new();
        }
        InstitutionRoleAssignments::<T>::get(cid_number, role_code)
            .into_iter()
            .filter(|assignment| {
                assignment.assignment_status == InstitutionAssignmentStatus::Active
            })
            .map(|assignment| assignment.admin_account)
            .collect()
    }

    fn active_role_codes(cid_number: &[u8], admin: &T::AccountId) -> Vec<Vec<u8>> {
        let Ok(cid_number) = CidNumberOf::<T>::try_from(cid_number.to_vec()) else {
            return Vec::new();
        };
        InstitutionRoles::<T>::iter_prefix(&cid_number)
            .filter(|(_, role)| role.role_status == InstitutionRoleStatus::Active)
            .filter_map(|(role_code, _)| {
                let active = InstitutionRoleAssignments::<T>::get(&cid_number, &role_code)
                    .into_iter()
                    .any(|assignment| {
                        assignment.assignment_status == InstitutionAssignmentStatus::Active
                            && &assignment.admin_account == admin
                    });
                active.then(|| role_code.into_inner())
            })
            .collect()
    }
}
