use super::*;
use frame_support::traits::GetCallName;
use frame_support::{assert_noop, assert_ok};

use crate::{AccountKind, Error, RESERVED_NAME_FEE, RESERVED_NAME_MAIN};

const ACCOUNT_AMOUNT: Balance = 1_000;
const REGISTRY_FUNDING_BALANCE: Balance = 100_000;
const CUSTOM_ACCOUNT_NAME: &[u8] = "专项账户".as_bytes();

#[test]
fn direct_institution_creation_call_is_permanently_absent() {
    let calls = <pallet::Call<Test> as GetCallName>::get_call_names();
    assert!(!calls.contains(&"propose_create_public_institution"));
}

fn governance_assignment(
    account: AccountId32,
    term_start: u32,
    term_end: u32,
) -> entity_primitives::InstitutionAssignmentTarget<AccountId32> {
    entity_primitives::InstitutionAssignmentTarget {
        admin_account: account,
        term_start,
        term_end,
        assignment_source: entity_primitives::InstitutionAssignmentSource::InstitutionGovernance,
        assignment_source_ref: b"proposal-result".to_vec(),
        assignment_status: entity_primitives::InstitutionAssignmentStatus::Active,
    }
}

fn governance_permission(
    operation: entity_primitives::RolePermissionOperation,
) -> entity_primitives::RolePermissionSpec {
    entity_primitives::RolePermissionSpec {
        business_action_id: entity_primitives::BusinessActionId {
            module_tag: b"pub-mgmt".to_vec(),
            action_code: 3,
        },
        operation,
    }
}

fn fund_registry_account() {
    assert_ok!(Balances::force_set_balance(
        RuntimeOrigin::root(),
        registry_funding_account(),
        REGISTRY_FUNDING_BALANCE,
    ));
}

fn create_cgov(tag: &str) -> pallet::CidNumberOf<Test> {
    let cid = generated_cid(tag, "CGOV");
    let institution_code = code_bytes("CGOV");
    let protocol_accounts =
        crate::institution::accounts::build_required_protocol_accounts::<Test>(&cid)
            .expect("测试协议账户必须可构造");
    let (created_accounts, _, _, _) =
        crate::institution::accounts::validate_initial_accounts::<Test>(&cid, &protocol_accounts)
            .expect("测试协议账户必须合法");
    pallet::Institutions::<Test>::insert(
        &cid,
        crate::InstitutionInfo {
            cid_full_name: cid_full_name("测试公权机构".as_bytes()),
            cid_short_name: cid_short_name("测试机构".as_bytes()),
            town_code: empty_town_code(),
            legal_representative_name: None,
            legal_representative_cid_number: None,
            legal_representative_account: None,
            institution_code,
            created_at: System::block_number(),
        },
    );
    // 固定岗位权限由机构 CID + 岗位码推导，因此先落机构身份，再创建默认 LR 岗位。
    assert_ok!(PublicManage::store_default_legal_representative_role(&cid));
    for account in created_accounts {
        pallet::InstitutionAccounts::<Test>::insert(
            &cid,
            &account.account_name,
            crate::InstitutionAccountInfo {
                address: account.address.clone(),
                initial_balance: account.amount,
                created_at: System::block_number(),
            },
        );
        pallet::AccountRegisteredCid::<Test>::insert(
            &account.address,
            crate::RegisteredInstitution {
                cid_number: cid.clone(),
                account_name: account.account_name,
            },
        );
    }
    let admins = institution_admins(3);
    assert_ok!(PublicManage::set_institution_admins(
        &cid,
        institution_code,
        &admins,
        2,
    ));
    cid
}

fn create_cgov_with_custom(tag: &str) -> pallet::CidNumberOf<Test> {
    fund_registry_account();
    let cid = create_cgov(tag);
    grant_close_role(&cid);
    assert_ok!(PublicManage::add_institution_account(
        RuntimeOrigin::signed(creator()),
        cid.clone(),
        account_names_bv(&[CUSTOM_ACCOUNT_NAME]),
        b"REGISTRY-CID".to_vec(),
    ));
    assert_ok!(Balances::force_set_balance(
        RuntimeOrigin::root(),
        account_of(&cid, CUSTOM_ACCOUNT_NAME),
        ACCOUNT_AMOUNT,
    ));
    assert_ok!(Balances::force_set_balance(
        RuntimeOrigin::root(),
        account_of(&cid, RESERVED_NAME_MAIN),
        ACCOUNT_AMOUNT,
    ));
    assert_ok!(Balances::force_set_balance(
        RuntimeOrigin::root(),
        account_of(&cid, RESERVED_NAME_FEE),
        ACCOUNT_AMOUNT,
    ));
    cid
}

#[test]
fn dynamic_role_lifecycle_persists_permissions_and_never_reuses_code() {
    new_test_ext().execute_with(|| {
        use entity_primitives::{
            InstitutionRoleAuthorizationQuery, InstitutionRoleMutation, InstitutionRoleQuery,
            RolePermissionOperation, RoleSubject,
        };

        let cid = create_cgov("dynamic-role-lifecycle");
        let first_code = entity_primitives::generate_dynamic_role_code(cid.as_slice(), 0, 42);
        let action = entity_primitives::BusinessActionId {
            module_tag: b"pub-mgmt".to_vec(),
            action_code: 3,
        };
        assert_ok!(PublicManage::apply_institution_governance_result(
            entity_primitives::InstitutionGovernanceResult {
                institution_code: code_bytes("CGOV"),
                cid_number: cid.to_vec(),
                proposal_id: 42,
                role_mutations: vec![InstitutionRoleMutation::Create {
                    role_name: "财务负责人".as_bytes().to_vec(),
                    term_required: false,
                    permissions: vec![
                        governance_permission(RolePermissionOperation::Propose),
                        governance_permission(RolePermissionOperation::Vote),
                    ],
                    assignments: vec![governance_assignment(admin(0), 0, 0)],
                }],
                assignment_changes: vec![],
                legal_representative_change: None,
                result_source_ref: b"proposal-42".to_vec(),
            }
        ));

        let bounded_code: crate::institution::role::RoleCodeOf =
            first_code.clone().try_into().expect("code fits");
        assert_eq!(pallet::InstitutionRoleNonce::<Test>::get(&cid), 1);
        assert!(pallet::UsedRoleCodes::<Test>::get(&cid, &bounded_code));
        assert!(pallet::InstitutionRoles::<Test>::contains_key(
            &cid,
            &bounded_code
        ));
        assert_eq!(
            pallet::InstitutionRolePermissions::<Test>::get(&cid, &bounded_code).len(),
            2
        );
        let subject = RoleSubject {
            cid_number: cid.to_vec(),
            role_code: first_code.clone(),
        };
        assert!(<PublicManage as InstitutionRoleAuthorizationQuery<
            AccountId32,
        >>::is_authorized(
            &admin(0),
            &subject,
            &action,
            RolePermissionOperation::Propose,
        ));
        assert!(!<PublicManage as InstitutionRoleAuthorizationQuery<
            AccountId32,
        >>::is_authorized(
            &admin(9),
            &subject,
            &action,
            RolePermissionOperation::Propose,
        ));

        assert_ok!(PublicManage::apply_institution_governance_result(
            entity_primitives::InstitutionGovernanceResult {
                institution_code: code_bytes("CGOV"),
                cid_number: cid.to_vec(),
                proposal_id: 43,
                role_mutations: vec![InstitutionRoleMutation::Rename {
                    role_code: first_code.clone(),
                    role_name: "资金负责人".as_bytes().to_vec(),
                }],
                assignment_changes: vec![],
                legal_representative_change: None,
                result_source_ref: b"proposal-43".to_vec(),
            }
        ));
        assert_eq!(
            pallet::InstitutionRoles::<Test>::get(&cid, &bounded_code)
                .expect("role exists")
                .role_name
                .as_slice(),
            "资金负责人".as_bytes()
        );

        assert_ok!(PublicManage::apply_institution_governance_result(
            entity_primitives::InstitutionGovernanceResult {
                institution_code: code_bytes("CGOV"),
                cid_number: cid.to_vec(),
                proposal_id: 44,
                role_mutations: vec![InstitutionRoleMutation::Delete {
                    role_code: first_code.clone(),
                }],
                assignment_changes: vec![],
                legal_representative_change: None,
                result_source_ref: b"proposal-44".to_vec(),
            }
        ));
        assert!(!pallet::InstitutionRoles::<Test>::contains_key(
            &cid,
            &bounded_code
        ));
        assert!(pallet::InstitutionRolePermissions::<Test>::get(&cid, &bounded_code).is_empty());
        assert!(pallet::InstitutionRoleAssignments::<Test>::get(&cid, &bounded_code).is_empty());
        assert!(pallet::UsedRoleCodes::<Test>::get(&cid, &bounded_code));

        assert_ok!(PublicManage::apply_institution_governance_result(
            entity_primitives::InstitutionGovernanceResult {
                institution_code: code_bytes("CGOV"),
                cid_number: cid.to_vec(),
                proposal_id: 42,
                role_mutations: vec![InstitutionRoleMutation::Create {
                    role_name: "新岗位".as_bytes().to_vec(),
                    term_required: false,
                    permissions: vec![governance_permission(RolePermissionOperation::Propose)],
                    assignments: vec![],
                }],
                assignment_changes: vec![],
                legal_representative_change: None,
                result_source_ref: b"proposal-42-second".to_vec(),
            }
        ));
        let second_code = entity_primitives::generate_dynamic_role_code(cid.as_slice(), 1, 42);
        assert_ne!(first_code, second_code);
        assert!(pallet::UsedRoleCodes::<Test>::get(
            &cid,
            crate::institution::role::RoleCodeOf::try_from(second_code).expect("code fits")
        ));
        assert!(
            !<PublicManage as InstitutionRoleQuery<AccountId32>>::is_active_assignment(
                cid.as_slice(),
                &admin(0),
                first_code.as_slice(),
            )
        );
    });
}

#[test]
fn dynamic_role_name_cannot_duplicate_legal_representative_name() {
    new_test_ext().execute_with(|| {
        use entity_primitives::{InstitutionRoleMutation, RolePermissionOperation};

        let cid = create_cgov("duplicate-lr-name");
        assert_noop!(
            PublicManage::apply_institution_governance_result(
                entity_primitives::InstitutionGovernanceResult {
                    institution_code: code_bytes("CGOV"),
                    cid_number: cid.to_vec(),
                    proposal_id: 78,
                    role_mutations: vec![InstitutionRoleMutation::Create {
                        role_name:
                            primitives::institution_constraints::ROLE_NAME_LEGAL_REPRESENTATIVE
                                .to_vec(),
                        term_required: false,
                        permissions: vec![governance_permission(RolePermissionOperation::Propose,)],
                        assignments: vec![],
                    }],
                    assignment_changes: vec![],
                    legal_representative_change: None,
                    result_source_ref: b"proposal-78".to_vec(),
                }
            ),
            Error::<Test>::DuplicateRoleName
        );
    });
}

#[test]
fn assignment_authorization_respects_inclusive_term_window() {
    new_test_ext().execute_with(|| {
        use entity_primitives::{
            InstitutionRoleMutation, InstitutionRoleQuery, RolePermissionOperation,
        };

        let cid = create_cgov("dynamic-role-term");
        let role_code = entity_primitives::generate_dynamic_role_code(cid.as_slice(), 0, 77);
        assert_ok!(PublicManage::apply_institution_governance_result(
            entity_primitives::InstitutionGovernanceResult {
                institution_code: code_bytes("CGOV"),
                cid_number: cid.to_vec(),
                proposal_id: 77,
                role_mutations: vec![InstitutionRoleMutation::Create {
                    role_name: "任期岗位".as_bytes().to_vec(),
                    term_required: true,
                    permissions: vec![governance_permission(RolePermissionOperation::Vote)],
                    assignments: vec![
                        governance_assignment(admin(0), 20_635, 20_635),
                        governance_assignment(admin(1), 20_600, 20_634),
                    ],
                }],
                assignment_changes: vec![],
                legal_representative_change: None,
                result_source_ref: b"proposal-77".to_vec(),
            }
        ));

        assert!(
            <PublicManage as InstitutionRoleQuery<AccountId32>>::is_active_assignment(
                cid.as_slice(),
                &admin(0),
                role_code.as_slice(),
            )
        );
        assert!(
            !<PublicManage as InstitutionRoleQuery<AccountId32>>::is_active_assignment(
                cid.as_slice(),
                &admin(1),
                role_code.as_slice(),
            )
        );
    });
}

fn account_of(cid: &pallet::CidNumberOf<Test>, name: &[u8]) -> AccountId32 {
    pallet::InstitutionAccounts::<Test>::get(cid, account_name(name))
        .expect("institution account must exist")
        .address
}

#[test]
fn update_info_and_add_account_keep_cid_as_only_entity_key() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("maintain-cid");
        assert_ok!(PublicManage::update_institution_info(
            RuntimeOrigin::signed(creator()),
            cid.clone(),
            cid_full_name("更新后的机构全称".as_bytes()),
            cid_short_name("更新简称".as_bytes()),
            b"REGISTRY-CID".to_vec(),
        ));
        let updated = pallet::Institutions::<Test>::get(&cid).expect("institution remains");
        assert_eq!(
            updated.cid_full_name.as_slice(),
            "更新后的机构全称".as_bytes()
        );

        let added_name = "新增账户".as_bytes();
        assert_ok!(PublicManage::add_institution_account(
            RuntimeOrigin::signed(creator()),
            cid.clone(),
            account_names_bv(&[added_name]),
            b"REGISTRY-CID".to_vec(),
        ));
        let added = account_of(&cid, added_name);
        assert_eq!(
            pallet::AccountRegisteredCid::<Test>::get(&added)
                .expect("new reverse index")
                .cid_number,
            cid
        );
        assert_eq!(Balances::free_balance(added), 0);
    });
}

#[test]
fn add_account_rejects_protocol_names_and_duplicate_custom_names() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("add-invalid");
        assert_noop!(
            PublicManage::add_institution_account(
                RuntimeOrigin::signed(creator()),
                cid.clone(),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                b"REGISTRY-CID".to_vec(),
            ),
            Error::<Test>::ReservedAccountName
        );
        assert_noop!(
            PublicManage::add_institution_account(
                RuntimeOrigin::signed(creator()),
                cid,
                account_names_bv(&["重复账户".as_bytes(), "重复账户".as_bytes()]),
                b"REGISTRY-CID".to_vec(),
            ),
            Error::<Test>::DuplicateAccountName
        );
    });
}

#[test]
fn derive_account_distinguishes_protocol_and_custom_account_kinds() {
    new_test_ext().execute_with(|| {
        let cid = generated_cid("derive-kinds", "CGOV");
        let (_, main_kind) =
            PublicManage::derive_institution_account(cid.as_slice(), RESERVED_NAME_MAIN).unwrap();
        let (_, fee_kind) =
            PublicManage::derive_institution_account(cid.as_slice(), RESERVED_NAME_FEE).unwrap();
        let (_, custom_kind) =
            PublicManage::derive_institution_account(cid.as_slice(), CUSTOM_ACCOUNT_NAME).unwrap();
        assert!(matches!(main_kind, AccountKind::InstitutionMain { .. }));
        assert!(matches!(fee_kind, AccountKind::InstitutionFee { .. }));
        assert!(matches!(custom_kind, AccountKind::InstitutionNamed { .. }));
    });
}

#[test]
fn close_requires_matching_actor_cid_and_an_institution_admin() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("close-auth");
        let custom = account_of(&cid, CUSTOM_ACCOUNT_NAME);
        let other_cid = generated_cid("close-auth-other", "CGOV");
        assert_noop!(
            propose_named_account_close(
                RuntimeOrigin::signed(admin(0)),
                other_cid,
                custom.clone(),
                beneficiary(),
            ),
            Error::<Test>::NotInstitutionAccount
        );
        assert_noop!(
            propose_named_account_close(
                RuntimeOrigin::signed(admin(9)),
                cid,
                custom,
                beneficiary(),
            ),
            Error::<Test>::PermissionDenied
        );
    });
}

#[test]
fn protocol_accounts_cannot_be_closed() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("close-protocol");
        for account in [
            account_of(&cid, RESERVED_NAME_MAIN),
            account_of(&cid, RESERVED_NAME_FEE),
        ] {
            assert_noop!(
                propose_named_account_close(
                    RuntimeOrigin::signed(admin(0)),
                    cid.clone(),
                    account,
                    beneficiary(),
                ),
                Error::<Test>::CannotCloseProtectedInstitution
            );
        }
    });
}

#[test]
fn approved_close_removes_only_custom_account() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("close-custom");
        let main = account_of(&cid, RESERVED_NAME_MAIN);
        let fee = account_of(&cid, RESERVED_NAME_FEE);
        let custom = account_of(&cid, CUSTOM_ACCOUNT_NAME);
        let beneficiary_account = beneficiary();
        let admin_balance_before = Balances::free_balance(admin(0));

        assert_ok!(propose_named_account_close(
            RuntimeOrigin::signed(admin(0)),
            cid.clone(),
            custom.clone(),
            beneficiary_account.clone(),
        ));
        let proposal_id = last_proposal_id();
        assert_ok!(cast_yes_votes(&[admin(1), admin(2)], 2, proposal_id));

        assert!(!pallet::AccountRegisteredCid::<Test>::contains_key(&custom));
        assert!(!pallet::InstitutionAccounts::<Test>::contains_key(
            &cid,
            account_name(CUSTOM_ACCOUNT_NAME),
        ));
        assert!(pallet::AccountRegisteredCid::<Test>::contains_key(&main));
        assert!(pallet::AccountRegisteredCid::<Test>::contains_key(&fee));
        assert!(pallet::Institutions::<Test>::contains_key(&cid));
        assert!(public_admins::AdminAccounts::<Test>::contains_key(&cid));
        assert_eq!(Balances::free_balance(&fee), 990);
        assert_eq!(Balances::free_balance(beneficiary_account), ACCOUNT_AMOUNT);
        assert_eq!(Balances::free_balance(admin(0)), admin_balance_before);
    });
}

#[test]
fn rejected_close_is_cleaned_only_by_votingengine_callback() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("close-rejected");
        let custom = account_of(&cid, CUSTOM_ACCOUNT_NAME);

        assert_ok!(propose_named_account_close(
            RuntimeOrigin::signed(admin(0)),
            cid.clone(),
            custom.clone(),
            beneficiary(),
        ));
        let proposal_id = last_proposal_id();
        assert_eq!(
            pallet::InstitutionPendingClose::<Test>::get(&custom),
            Some(proposal_id)
        );

        assert_eq!(
            <crate::InternalVoteExecutor<Test> as votingengine::InternalVoteResultCallback>::on_internal_vote_finalized(
                proposal_id,
                false,
            ),
            Ok(votingengine::ProposalExecutionOutcome::Executed)
        );
        assert!(!pallet::InstitutionPendingClose::<Test>::contains_key(
            &custom
        ));
        assert!(pallet::InstitutionAccounts::<Test>::contains_key(
            &cid,
            account_name(CUSTOM_ACCOUNT_NAME),
        ));
    });
}

#[test]
fn duplicate_close_proposal_is_rejected_while_pending() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("close-pending");
        let custom = account_of(&cid, CUSTOM_ACCOUNT_NAME);
        // 首次发起成功后 InstitutionPendingClose 命中,重复发起同账户关闭必须被拒。
        assert_ok!(propose_named_account_close(
            RuntimeOrigin::signed(admin(0)),
            cid.clone(),
            custom.clone(),
            beneficiary(),
        ));
        assert_noop!(
            propose_named_account_close(
                RuntimeOrigin::signed(admin(0)),
                cid,
                custom,
                beneficiary(),
            ),
            Error::<Test>::CloseAlreadyPending
        );
    });
}
