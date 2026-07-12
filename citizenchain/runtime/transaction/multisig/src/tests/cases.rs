#![cfg(test)]

use super::*;

#[test]
fn nrc_transfer_executes_when_internal_vote_reaches_threshold() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let inst_account = institution_account(&institution);
        let dest = beneficiary();

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest.clone(),
            1_000,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        let vote_pairs = nrc_pass_pairs();
        assert_ok!(cast_transfer_votes_n(
            &vote_pairs[1..],
            nrc_pass_count().saturating_sub(1),
            pid,
        ));

        // 转账已执行（含手续费 10）
        assert_eq!(Balances::free_balance(&inst_account), 8_990);
        assert_eq!(Balances::free_balance(&dest), 1_000);
        // 提案数据仍保留（由 votingengine 延迟清理）
        assert!(votingengine::Pallet::<Test>::get_proposal_data(pid).is_some());
    });
}

#[test]
fn prc_transfer_executes_when_internal_vote_reaches_threshold() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let inst_account = institution_account(&institution);
        let dest = beneficiary();

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(prc_admin(0)),
            PRC,
            institution,
            dest.clone(),
            2_000,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        let vote_pairs = prc_pass_pairs();
        assert_ok!(cast_transfer_votes_n(
            &vote_pairs[1..],
            prc_pass_count().saturating_sub(1),
            pid,
        ));

        assert_eq!(Balances::free_balance(&inst_account), 7_990);
        assert_eq!(Balances::free_balance(&dest), 2_000);
        assert!(votingengine::Pallet::<Test>::get_proposal_data(pid).is_some());
    });
}

#[test]
fn prb_transfer_executes_when_internal_vote_reaches_threshold() {
    new_test_ext().execute_with(|| {
        let institution = prb_pallet_id();
        let inst_account = institution_account(&institution);
        let dest = beneficiary();

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(prb_admin(0)),
            PRB,
            institution,
            dest.clone(),
            3_000,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        let vote_pairs = prb_pass_pairs();
        assert_ok!(cast_transfer_votes_n(
            &vote_pairs[1..],
            prb_pass_count().saturating_sub(1),
            pid,
        ));

        assert_eq!(Balances::free_balance(&inst_account), 6_990);
        assert_eq!(Balances::free_balance(&dest), 3_000);
        assert!(votingengine::Pallet::<Test>::get_proposal_data(pid).is_some());
    });
}

#[test]
fn registered_account_transfer_executes_when_internal_vote_reaches_threshold() {
    new_test_ext().execute_with(|| {
        let personal_account = registered_account();
        let dest = beneficiary();
        let admins = BoundedVec::try_from(vec![
            registered_account_admin(0),
            registered_account_admin(1),
            registered_account_admin(2),
        ])
        .expect("admins should fit");

        personal_manage::PersonalAccounts::<Test>::insert(
            &personal_account,
            personal_manage::PersonalAccount {
                creator: registered_account_admin(0),
                account_name: b"personal"
                    .to_vec()
                    .try_into()
                    .expect("account name should fit"),
                created_at: 1,
                status: personal_manage::PersonalStatus::Active,
            },
        );
        personal_admins::AdminAccounts::<Test>::insert(
            personal_account.clone(),
            admin_primitives::AdminAccount {
                cid_number: Default::default(),
                institution_code: PERSONAL_CODE,
                kind: admin_primitives::AdminAccountKind::PersonalMultisig,
                admins,
                creator: registered_account_admin(0),
                created_at: 1,
                updated_at: 1,
                status: admin_primitives::AdminAccountStatus::Active,
            },
        );
        internal_vote::ActiveDynamicThresholds::<Test>::insert(
            PERSONAL_CODE,
            personal_account.clone(),
            2,
        );
        let _ = Balances::deposit_creating(&personal_account, 10_000);

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(registered_account_admin(0)),
            PERSONAL_CODE,
            personal_account.clone(),
            dest.clone(),
            1_500,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        let vote_pairs = registered_account_pairs(2);
        assert_ok!(cast_transfer_votes_n(&vote_pairs[1..], 1, pid,));

        assert_eq!(Balances::free_balance(&personal_account), 8_490);
        assert_eq!(Balances::free_balance(&dest), 1_500);
        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("proposal should exist")
                .status,
            STATUS_EXECUTED
        );
    });
}

#[test]
fn institution_account_transfer_executes_when_internal_vote_reaches_threshold() {
    new_test_ext().execute_with(|| {
        let institution = registered_institution_account();
        let inst_account = registered_institution_account();
        let dest = beneficiary();
        let admins = BoundedVec::try_from(vec![
            registered_institution_admin(0),
            registered_institution_admin(1),
            registered_institution_admin(2),
        ])
        .expect("admins should fit");

        insert_active_registered_institution_account(&inst_account, admins);
        let _ = Balances::deposit_creating(&inst_account, 10_000);

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(registered_institution_admin(0)),
            PRIVATE_CODE,
            institution,
            dest.clone(),
            2_000,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        let vote_pairs = registered_institution_pairs(2);
        assert_ok!(cast_transfer_votes_n(&vote_pairs[1..], 1, pid,));

        assert_eq!(Balances::free_balance(&inst_account), 7_990);
        assert_eq!(Balances::free_balance(&dest), 2_000);
        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("proposal should exist")
                .status,
            STATUS_EXECUTED
        );
    });
}

#[test]
fn institution_account_rejects_personal_org() {
    new_test_ext().execute_with(|| {
        let institution = registered_institution_account();
        let inst_account = registered_institution_account();
        let admins = BoundedVec::try_from(vec![
            registered_institution_admin(0),
            registered_institution_admin(1),
            registered_institution_admin(2),
        ])
        .expect("admins should fit");

        insert_active_registered_institution_account(&inst_account, admins);
        let _ = Balances::deposit_creating(&inst_account, 10_000);

        assert_noop!(
            MultisigTransfer::propose_transfer(
                RuntimeOrigin::signed(registered_institution_admin(0)),
                PERSONAL_CODE,
                institution.clone(),
                beneficiary(),
                1_000,
                BoundedVec::default(),
            ),
            Error::<Test>::InstitutionCodeMismatch
        );
    });
}

#[test]
fn unknown_account_cannot_be_used_as_transfer_source() {
    new_test_ext().execute_with(|| {
        let institution = AccountId32::new([0x77; 32]);

        assert_noop!(
            MultisigTransfer::propose_transfer(
                RuntimeOrigin::signed(registered_institution_admin(0)),
                PERSONAL_CODE,
                institution.clone(),
                beneficiary(),
                1_000,
                BoundedVec::default(),
            ),
            Error::<Test>::InvalidInstitution
        );
    });
}

#[test]
fn zero_amount_is_rejected() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let dest = beneficiary();

        assert_noop!(
            MultisigTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                NRC,
                institution.clone(),
                dest,
                0,
                BoundedVec::default(),
            ),
            Error::<Test>::ZeroAmount
        );
    });
}

#[test]
fn self_transfer_is_rejected() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let inst_account = institution_account(&institution);

        assert_noop!(
            MultisigTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                NRC,
                institution,
                inst_account,
                100,
                BoundedVec::default(),
            ),
            Error::<Test>::SelfTransferNotAllowed
        );
    });
}

#[test]
fn insufficient_balance_is_rejected_on_propose() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let dest = beneficiary();

        // 余额 10_000，fee=10，ED=1：最多 amount=9_989（9_989+10+1=10_000）
        // amount=9_990 时 required=9_990+10+1=10_001 > 10_000 → 拒绝
        assert_noop!(
            MultisigTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                NRC,
                institution.clone(),
                dest.clone(),
                9_990,
                BoundedVec::default(),
            ),
            Error::<Test>::InsufficientBalance
        );

        // amount=9_989 时 required=9_989+10+1=10_000 → 刚好通过
        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest,
            9_989,
            BoundedVec::default(),
        ));
    });
}

#[test]
fn multiple_proposals_allowed_within_limit() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let dest = beneficiary();

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution.clone(),
            dest.clone(),
            100,
            BoundedVec::default(),
        ));

        // 活跃提案数限制由 votingengine 全局管控（上限 10），第二个提案可以成功
        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest,
            200,
            BoundedVec::default(),
        ));
    });
}

#[test]
fn executed_transfer_does_not_block_new_proposal() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let dest = beneficiary();

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution.clone(),
            dest.clone(),
            100,
            BoundedVec::default(),
        ));
        let pid1 = last_proposal_id();

        let vote_pairs = nrc_pass_pairs();
        assert_ok!(cast_transfer_votes_n(
            &vote_pairs[1..],
            nrc_pass_count().saturating_sub(1),
            pid1,
        ));

        // 转账已执行，可以创建新提案
        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest,
            200,
            BoundedVec::default(),
        ));
    });
}

#[test]
fn rejected_proposal_does_not_block_new_proposal() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let dest = beneficiary();

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution.clone(),
            dest.clone(),
            100,
            BoundedVec::default(),
        ));
        let pid1 = last_proposal_id();

        let end = votingengine::Pallet::<Test>::proposals(pid1)
            .expect("proposal should exist")
            .end;
        System::set_block_number(end + 1);
        assert_ok!(votingengine::Pallet::<Test>::finalize_proposal(
            RuntimeOrigin::signed(nrc_admin(0)),
            pid1
        ));
        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid1)
                .expect("proposal should exist")
                .status,
            STATUS_REJECTED
        );

        // 被拒绝后可以创建新提案
        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest,
            50,
            BoundedVec::default(),
        ));
    });
}

#[test]
fn existential_deposit_is_preserved() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let inst_account = institution_account(&institution);
        let dest = beneficiary();

        // 余额 10_000，ED=1，手续费=10，提案 9_989 刚好使剩余 = ED
        // required = 9_989 + 10(fee) + 1(ED) = 10_000
        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest.clone(),
            9_989,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        let vote_pairs = nrc_pass_pairs();
        assert_ok!(cast_transfer_votes_n(
            &vote_pairs[1..],
            nrc_pass_count().saturating_sub(1),
            pid,
        ));

        assert_eq!(Balances::free_balance(&inst_account), 1);
        assert_eq!(Balances::free_balance(&dest), 9_989);
    });
}

#[test]
fn retry_passed_transfer_succeeds_after_failed_auto_execution() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let inst_account = institution_account(&institution);
        let dest = beneficiary();

        // 余额 10_000,提案 9_000(预检通过),然后在投票通过前转走 9_000。
        // 使余额仅 1_000,自动执行因余额不足失败,但提案保留,可统一手动重试。
        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest.clone(),
            9_000,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        // 投票通过前转走余额,使自动执行失败。
        let drain_dest = AccountId32::new([88u8; 32]);
        let _ = Balances::deposit_creating(&drain_dest, 1);
        assert_ok!(<Balances as frame_support::traits::Currency<_>>::transfer(
            &inst_account,
            &drain_dest,
            9_000,
            frame_support::traits::ExistenceRequirement::KeepAlive,
        ));
        assert_eq!(Balances::free_balance(&inst_account), 1_000);

        // 投票达阈值后自动执行,但 try_execute_transfer 因余额不足失败。
        // 提案仍为 PASSED,转账未执行。
        let vote_pairs = nrc_pass_pairs();
        assert_ok!(cast_transfer_votes_n(
            &vote_pairs[1..],
            nrc_pass_count().saturating_sub(1),
            pid,
        ));
        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("proposal should exist")
                .status,
            STATUS_PASSED
        );
        assert_eq!(Balances::free_balance(&dest), 0);
        assert!(votingengine::Pallet::<Test>::get_proposal_data(pid).is_some());

        // 补充余额后通过投票引擎统一入口手动重试。
        let _ = Balances::deposit_creating(&inst_account, 9_000);
        assert_eq!(Balances::free_balance(&inst_account), 10_000);
        assert_ok!(VotingEngine::retry_passed_proposal(
            RuntimeOrigin::signed(nrc_admin(0)),
            pid
        ));
        // 转账成功：9_000 转出 + 10 手续费
        assert_eq!(Balances::free_balance(&inst_account), 990);
        assert_eq!(Balances::free_balance(&dest), 9_000);
    });
}

#[test]
fn retry_passed_transfer_rejects_non_passed_proposal() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let dest = beneficiary();

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest,
            100,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        // 提案仍在投票中，不能手动重试。
        assert_noop!(
            VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(nrc_admin(0)), pid),
            votingengine::Error::<Test>::ProposalNotRetryable
        );
    });
}

#[test]
fn retry_passed_transfer_rejects_non_admin() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let inst_account = institution_account(&institution);
        let dest = beneficiary();
        let outsider = AccountId32::new([88u8; 32]);
        let _ = Balances::deposit_creating(&outsider, 1);

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest.clone(),
            100,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        // 减余额使自动执行失败
        let drain_dest = AccountId32::new([77u8; 32]);
        let _ = Balances::deposit_creating(&drain_dest, 1);
        assert_ok!(<Balances as frame_support::traits::Currency<_>>::transfer(
            &inst_account,
            &drain_dest,
            9_900,
            frame_support::traits::ExistenceRequirement::KeepAlive,
        ));

        let vote_pairs = nrc_pass_pairs();
        assert_ok!(cast_transfer_votes_n(
            &vote_pairs[1..],
            nrc_pass_count().saturating_sub(1),
            pid,
        ));

        // 自动执行失败，补充余额
        assert_eq!(Balances::free_balance(&dest), 0);
        let _ = Balances::deposit_creating(&inst_account, 10_000);

        // 统一重试入口只允许快照管理员手动重试。
        assert_noop!(
            VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(outsider), pid),
            votingengine::Error::<Test>::NoPermission
        );
        assert_eq!(Balances::free_balance(&dest), 0);
    });
}

#[test]
fn executed_transfer_cannot_be_executed_again() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let dest = beneficiary();

        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest.clone(),
            1_000,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        let vote_pairs = nrc_pass_pairs();
        assert_ok!(cast_transfer_votes_n(
            &vote_pairs[1..],
            nrc_pass_count().saturating_sub(1),
            pid,
        ));

        // 自动执行成功，状态变为 EXECUTED
        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("proposal should exist")
                .status,
            STATUS_EXECUTED
        );

        // 已执行提案再次走统一重试入口应被拒绝。
        assert_noop!(
            VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(nrc_admin(0)), pid),
            votingengine::Error::<Test>::ProposalNotRetryable
        );
    });
}

#[test]
fn protected_account_is_rejected() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let protected = AccountId32::new([77u8; 32]);

        // 标记为受保护地址
        PROTECTED_ACCOUNT.with(|pa| *pa.borrow_mut() = Some(protected.clone()));

        assert_noop!(
            MultisigTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                NRC,
                institution,
                protected,
                100,
                BoundedVec::default(),
            ),
            Error::<Test>::BeneficiaryIsProtectedAddress
        );
    });
}

#[test]
fn institution_spend_guard_blocks_transfer_proposal() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let source = institution_account(&institution);
        let dest = beneficiary();
        DENIED_SPEND_SOURCE.with(|blocked| *blocked.borrow_mut() = Some(source.clone()));

        assert_noop!(
            MultisigTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                NRC,
                institution,
                dest,
                100,
                BoundedVec::default(),
            ),
            Error::<Test>::InstitutionSpendNotAllowed
        );

        DENIED_SPEND_SOURCE.with(|blocked| *blocked.borrow_mut() = None);
    });
}

#[test]
fn fee_respects_minimum_on_small_amount() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let inst_account = institution_account(&institution);
        let dest = beneficiary();

        // amount=1, 费率计算 1×0.1%=0.001 < 最低 10 分，手续费应为 10
        // required = 1 + 10 + 1(ED) = 12
        assert_ok!(MultisigTransfer::propose_transfer(
            RuntimeOrigin::signed(nrc_admin(0)),
            NRC,
            institution,
            dest.clone(),
            1,
            BoundedVec::default(),
        ));
        let pid = last_proposal_id();

        let vote_pairs = nrc_pass_pairs();
        assert_ok!(cast_transfer_votes_n(
            &vote_pairs[1..],
            nrc_pass_count().saturating_sub(1),
            pid,
        ));

        // 余额 10_000 - 1(转账) - 10(最低手续费) = 9_989
        assert_eq!(Balances::free_balance(&inst_account), 9_989);
        assert_eq!(Balances::free_balance(&dest), 1);
    });
}
