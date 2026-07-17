use super::*;
use frame_support::{assert_noop, assert_ok, BoundedVec};
use votingengine::{types::code_bytes, InternalVoteEngine as _};

#[test]
fn create_uses_cid_as_the_only_institution_identity() {
    new_test_ext().execute_with(|| {
        let cid_number = generated_cid("private-create", "SFLP");
        let code = code_bytes("SFLP");
        assert_ok!(create_institution(
            cid_number.clone(),
            code,
            initial_accounts(&[
                (crate::RESERVED_NAME_MAIN, 0),
                (crate::RESERVED_NAME_FEE, 0)
            ]),
        ));

        let main_account = account_of(&cid_number, crate::RESERVED_NAME_MAIN);
        let fee_account = account_of(&cid_number, crate::RESERVED_NAME_FEE);
        assert_ne!(main_account, fee_account);
        assert!(pallet::Institutions::<Test>::contains_key(&cid_number));
        assert_eq!(
            pallet::InstitutionAccounts::<Test>::get(
                &cid_number,
                account_name(crate::RESERVED_NAME_MAIN)
            )
            .map(|item| item.address),
            Some(main_account.clone())
        );
        assert_eq!(
            pallet::AccountRegisteredCid::<Test>::get(&fee_account).map(|item| item.cid_number),
            Some(cid_number.clone())
        );

        // 管理员和投票阈值均按 CID 寻址，机构账户不再充当管理员根或阈值 key。
        assert_eq!(
            PrivateAdmins::institution_admins(code, cid_number.as_slice()),
            Some(alloc::vec![admin(1), admin(2)])
        );
        assert_eq!(
            InternalVote::active_institution_threshold(code, cid_number.as_slice()),
            Some(2)
        );
        assert!(pallet::InstitutionRoles::<Test>::contains_key(
            &cid_number,
            crate::RoleCodeOf::try_from(
                primitives::institution_constraints::ROLE_CODE_LEGAL_REPRESENTATIVE.to_vec()
            )
            .expect("LR 岗位码必须受界")
        ));
        assert!(pallet::InstitutionRoleAssignments::<Test>::get(
            &cid_number,
            crate::RoleCodeOf::try_from(
                primitives::institution_constraints::ROLE_CODE_LEGAL_REPRESENTATIVE.to_vec()
            )
            .expect("LR 岗位码必须受界")
        )
        .is_empty());
    });
}

#[test]
fn protocol_accounts_are_automatically_created_with_zero_balance() {
    new_test_ext().execute_with(|| {
        let zero_cid = generated_cid("private-zero", "SFLP");
        assert_ok!(create_institution(
            zero_cid.clone(),
            code_bytes("SFLP"),
            initial_accounts(&[
                (crate::RESERVED_NAME_MAIN, 0),
                (crate::RESERVED_NAME_FEE, 0)
            ]),
        ));
        assert_eq!(
            pallet::InstitutionAccounts::<Test>::get(
                &zero_cid,
                account_name(crate::RESERVED_NAME_MAIN)
            )
            .expect("主账户必须存在")
            .initial_balance,
            0
        );

        assert_eq!(
            pallet::InstitutionAccounts::<Test>::get(
                &zero_cid,
                account_name(crate::RESERVED_NAME_FEE)
            )
            .expect("费用账户必须存在")
            .initial_balance,
            0
        );
    });
}

#[test]
fn creation_derives_protocol_accounts_without_client_account_input() {
    new_test_ext().execute_with(|| {
        let cid_number = generated_cid("private-missing-fee", "SFLP");
        assert_ok!(create_institution(
            cid_number.clone(),
            code_bytes("SFLP"),
            initial_accounts(&[]),
        ));
        assert!(pallet::InstitutionAccounts::<Test>::contains_key(
            &cid_number,
            account_name(crate::RESERVED_NAME_MAIN)
        ));
        assert!(pallet::InstitutionAccounts::<Test>::contains_key(
            &cid_number,
            account_name(crate::RESERVED_NAME_FEE)
        ));
    });
}

#[test]
fn update_and_add_account_keep_cid_as_the_target_key() {
    new_test_ext().execute_with(|| {
        let cid_number = generated_cid("private-maintain", "SFLP");
        assert_ok!(create_institution(
            cid_number.clone(),
            code_bytes("SFLP"),
            initial_accounts(&[
                (crate::RESERVED_NAME_MAIN, 0),
                (crate::RESERVED_NAME_FEE, 0)
            ]),
        ));

        assert_ok!(PrivateManage::update_institution_info(
            RuntimeOrigin::signed(registrar()),
            cid_number.clone(),
            account_name("更新后的机构全称".as_bytes()),
            account_name("更新简称".as_bytes()),
            register_nonce(b"update-nonce"),
            valid_signature(),
            b"GD001-FRG00-000000001-2026".to_vec(),
            [7u8; 32],
            "广东省".as_bytes().to_vec(),
            "荔湾市".as_bytes().to_vec(),
        ));
        assert_eq!(
            pallet::Institutions::<Test>::get(&cid_number)
                .expect("机构必须存在")
                .cid_short_name,
            account_name("更新简称".as_bytes())
        );

        let names = BoundedVec::try_from(alloc::vec![account_name("专项账户".as_bytes())])
            .expect("账户名列表必须受界");
        assert_ok!(PrivateManage::add_institution_account(
            RuntimeOrigin::signed(registrar()),
            cid_number.clone(),
            names,
            register_nonce(b"add-account-nonce"),
            valid_signature(),
            b"GD001-FRG00-000000001-2026".to_vec(),
            [7u8; 32],
            "广东省".as_bytes().to_vec(),
            "荔湾市".as_bytes().to_vec(),
        ));
        let named_account = account_of(&cid_number, "专项账户".as_bytes());
        assert_eq!(
            pallet::AccountRegisteredCid::<Test>::get(named_account)
                .map(|item| (item.cid_number, item.account_name)),
            Some((cid_number, account_name("专项账户".as_bytes())))
        );
    });
}

#[test]
fn only_named_account_can_be_closed_and_institution_stays_alive() {
    new_test_ext().execute_with(|| {
        let cid_number = generated_cid("private-close-named", "SFLP");
        assert_ok!(create_institution(
            cid_number.clone(),
            code_bytes("SFLP"),
            initial_accounts(&[
                (crate::RESERVED_NAME_MAIN, 0),
                // 关闭操作的链上费必须由本机构费用账户支付；余额不足时不得回落管理员。
                (crate::RESERVED_NAME_FEE, 1_000),
                ("项目账户".as_bytes(), 1_000),
            ]),
        ));
        let named_account = account_of(&cid_number, "项目账户".as_bytes());
        let fee_account = account_of(&cid_number, crate::RESERVED_NAME_FEE);
        let admin_balance_before = Balances::free_balance(admin(1));

        assert_ok!(PrivateManage::propose_close_private_institution(
            RuntimeOrigin::signed(admin(1)),
            cid_number.clone(),
            named_account.clone(),
            beneficiary(),
            register_nonce(b"close-named-nonce"),
            close_signature(),
            b"GD001-FRG00-000000001-2026".to_vec(),
            [7u8; 32],
        ));
        let proposal_id = VotingEngine::next_proposal_id().saturating_sub(1);
        assert_ok!(cast_yes_votes(proposal_id));

        assert_eq!(Balances::free_balance(&fee_account), 990);
        assert_eq!(Balances::free_balance(beneficiary()), 1_100);
        assert_eq!(Balances::free_balance(admin(1)), admin_balance_before);
        assert!(!pallet::AccountRegisteredCid::<Test>::contains_key(
            &named_account
        ));
        assert!(!pallet::InstitutionAccounts::<Test>::contains_key(
            &cid_number,
            account_name("项目账户".as_bytes())
        ));
        assert!(pallet::Institutions::<Test>::contains_key(&cid_number));
        assert!(pallet::InstitutionAccounts::<Test>::contains_key(
            &cid_number,
            account_name(crate::RESERVED_NAME_MAIN)
        ));
        assert_eq!(
            PrivateAdmins::institution_admins(code_bytes("SFLP"), cid_number.as_slice()),
            Some(alloc::vec![admin(1), admin(2)])
        );
    });
}

#[test]
fn rejected_close_is_cleaned_only_by_votingengine_callback() {
    new_test_ext().execute_with(|| {
        let cid_number = generated_cid("private-close-rejected", "SFLP");
        assert_ok!(create_institution(
            cid_number.clone(),
            code_bytes("SFLP"),
            initial_accounts(&[
                (crate::RESERVED_NAME_MAIN, 0),
                (crate::RESERVED_NAME_FEE, 1_000),
                ("项目账户".as_bytes(), 1_000),
            ]),
        ));
        let named_account = account_of(&cid_number, "项目账户".as_bytes());

        assert_ok!(PrivateManage::propose_close_private_institution(
            RuntimeOrigin::signed(admin(1)),
            cid_number.clone(),
            named_account.clone(),
            beneficiary(),
            register_nonce(b"close-rejected-nonce"),
            close_signature(),
            b"GD001-FRG00-000000001-2026".to_vec(),
            [7u8; 32],
        ));
        let proposal_id = VotingEngine::next_proposal_id().saturating_sub(1);
        assert_eq!(
            pallet::InstitutionPendingClose::<Test>::get(&named_account),
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
            &named_account
        ));
        assert!(pallet::InstitutionAccounts::<Test>::contains_key(
            &cid_number,
            account_name("项目账户".as_bytes()),
        ));
    });
}

#[test]
fn protocol_account_close_is_rejected() {
    new_test_ext().execute_with(|| {
        let cid_number = generated_cid("private-close-main", "SFLP");
        assert_ok!(create_institution(
            cid_number.clone(),
            code_bytes("SFLP"),
            initial_accounts(&[
                (crate::RESERVED_NAME_MAIN, 0),
                (crate::RESERVED_NAME_FEE, 0)
            ]),
        ));
        let main_account = account_of(&cid_number, crate::RESERVED_NAME_MAIN);

        assert_noop!(
            PrivateManage::propose_close_private_institution(
                RuntimeOrigin::signed(admin(1)),
                cid_number,
                main_account,
                beneficiary(),
                register_nonce(b"close-main-nonce"),
                close_signature(),
                b"GD001-FRG00-000000001-2026".to_vec(),
                [7u8; 32],
            ),
            pallet::Error::<Test>::CannotCloseProtectedInstitution
        );
    });
}

#[test]
fn account_operation_rejects_actor_cid_mismatch() {
    new_test_ext().execute_with(|| {
        let cid_number = generated_cid("private-actor", "SFLP");
        assert_ok!(create_institution(
            cid_number.clone(),
            code_bytes("SFLP"),
            initial_accounts(&[
                (crate::RESERVED_NAME_MAIN, 0),
                (crate::RESERVED_NAME_FEE, 0),
                ("项目账户".as_bytes(), 0),
            ]),
        ));
        let named_account = account_of(&cid_number, "项目账户".as_bytes());
        let other_cid = generated_cid("private-other-actor", "SFLP");

        assert_noop!(
            PrivateManage::propose_close_private_institution(
                RuntimeOrigin::signed(admin(1)),
                other_cid,
                named_account,
                beneficiary(),
                register_nonce(b"wrong-actor-nonce"),
                close_signature(),
                b"GD001-FRG00-000000001-2026".to_vec(),
                [7u8; 32],
            ),
            pallet::Error::<Test>::NotInstitutionAccount
        );
    });
}

#[test]
fn non_admin_cannot_start_institution_account_close() {
    new_test_ext().execute_with(|| {
        let cid_number = generated_cid("private-close-auth", "SFLP");
        assert_ok!(create_institution(
            cid_number.clone(),
            code_bytes("SFLP"),
            initial_accounts(&[
                (crate::RESERVED_NAME_MAIN, 0),
                (crate::RESERVED_NAME_FEE, 0),
                ("项目账户".as_bytes(), 0),
            ]),
        ));
        let named_account = account_of(&cid_number, "项目账户".as_bytes());

        assert_noop!(
            PrivateManage::propose_close_private_institution(
                RuntimeOrigin::signed(admin(8)),
                cid_number,
                named_account,
                beneficiary(),
                register_nonce(b"non-admin-close"),
                close_signature(),
                b"GD001-FRG00-000000001-2026".to_vec(),
                [7u8; 32],
            ),
            pallet::Error::<Test>::PermissionDenied
        );
    });
}
