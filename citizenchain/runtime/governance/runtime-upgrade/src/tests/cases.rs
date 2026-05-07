use super::*;

#[test]
fn joint_proposers_can_propose_runtime_upgrade() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            RuntimeUpgrade::propose_runtime_upgrade(
                RuntimeOrigin::signed(outsider()),
                reason_ok(),
                code_ok(),
                10,
                nonce_ok(),
                sig_ok(),
                province_ok(),
                signer_admin_pubkey_ok()
            ),
            sp_runtime::DispatchError::BadOrigin
        );

        assert_ok!(RuntimeUpgrade::propose_runtime_upgrade(
            RuntimeOrigin::signed(nrc_admin()),
            reason_ok(),
            code_ok(),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));

        assert_ok!(RuntimeUpgrade::propose_runtime_upgrade(
            RuntimeOrigin::signed(prc_admin()),
            reason_ok(),
            code_ok(),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));

        assert!(
            votingengine::Pallet::<Test>::get_proposal_data(100).is_some(),
            "NRC proposer should create proposal data"
        );
        assert!(
            votingengine::Pallet::<Test>::get_proposal_data(101).is_some(),
            "PRC proposer should create proposal data"
        );
    });
}

#[test]
fn proposal_data_stored_in_votingengine() {
    new_test_ext().execute_with(|| {
        propose_ok();
        // proposal_id comes from NEXT_JOINT_ID which starts at 100
        assert!(
            votingengine::Pallet::<Test>::get_proposal_data(100).is_some(),
            "proposal data should be stored in voting engine"
        );
        let proposal = decode_proposal(100);
        assert!(matches!(proposal.status, pallet::ProposalStatus::Voting));
        assert!(
            votingengine::Pallet::<Test>::get_proposal_object(100).is_some(),
            "runtime wasm should be stored in proposal object layer"
        );
    });
}

#[test]
fn rejected_joint_vote_marks_proposal_rejected() {
    new_test_ext().execute_with(|| {
        propose_ok();
        // proposal_id == joint_vote_id == 100
        let outcome = call_joint_callback(100, false).expect("callback should succeed");
        assert_eq!(outcome, votingengine::ProposalExecutionOutcome::Executed);
        let p = decode_proposal(100);
        assert!(matches!(p.status, pallet::ProposalStatus::Voting));
    });
}

#[test]
fn approved_joint_vote_executes_runtime_upgrade() {
    new_test_ext().execute_with(|| {
        propose_ok();
        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));

        let p = decode_proposal(100);
        assert!(matches!(p.status, pallet::ProposalStatus::Voting));
        assert!(
            votingengine::Pallet::<Test>::get_proposal_object(100).is_some(),
            "approved proposal should still keep object data for unified cleanup"
        );
        let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
        assert!(code_executed, "runtime code executor should be called");
    });
}

#[test]
fn approved_joint_vote_execution_failure_emits_event() {
    new_test_ext().execute_with(|| {
        propose_ok();
        insert_engine_proposal(100);
        EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);

        assert_ok!(call_joint_callback(100, true));

        let p = decode_proposal(100);
        assert!(matches!(p.status, pallet::ProposalStatus::Voting));
        let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
        assert!(
            !code_executed,
            "runtime code executor should fail in this test"
        );
        // 投票引擎侧应为 STATUS_EXECUTION_FAILED
        let engine_proposal = votingengine::pallet::Proposals::<Test>::get(100).unwrap();
        assert_eq!(
            engine_proposal.status,
            votingengine::STATUS_EXECUTION_FAILED
        );
    });
}

#[test]
fn rejected_joint_vote_retains_object_for_unified_cleanup() {
    new_test_ext().execute_with(|| {
        propose_ok();
        assert_ok!(call_joint_callback(100, false));

        let p = decode_proposal(100);
        assert!(matches!(p.status, pallet::ProposalStatus::Voting));
        assert!(
            votingengine::Pallet::<Test>::get_proposal_object(100).is_some(),
            "rejected proposal object should stay until unified cleanup"
        );
    });
}

#[test]
fn owns_proposal_returns_true_for_own_proposals() {
    new_test_ext().execute_with(|| {
        propose_ok();
        assert!(pallet::Pallet::<Test>::owns_proposal(100));
        assert!(!pallet::Pallet::<Test>::owns_proposal(999));
    });
}

#[test]
fn approved_success_marks_engine_status_executed() {
    new_test_ext().execute_with(|| {
        propose_ok();
        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));

        // 执行成功时在回调作用域内静默写入 EXECUTED，最终事件由投票引擎外层发出。
        let engine_proposal = votingengine::pallet::Proposals::<Test>::get(100).unwrap();
        assert_eq!(
            engine_proposal.status,
            votingengine::STATUS_EXECUTED,
            "success path should mark engine status executed"
        );
    });
}

#[test]
fn joint_vote_callback_requires_voting_status() {
    new_test_ext().execute_with(|| {
        propose_ok();
        insert_engine_proposal(100);
        // First finalize
        assert_ok!(call_joint_callback(100, true));
        // Second finalize should fail - no longer voting
        assert_noop!(
            call_joint_callback(100, true),
            pallet::Error::<Test>::ProposalNotVoting
        );
    });
}

#[test]
fn finalize_nonexistent_proposal_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            call_joint_callback(999, true),
            pallet::Error::<Test>::ProposalNotFound
        );
    });
}

// ─── developer_direct_upgrade 测试 ─────────────────────────────────

#[test]
fn developer_direct_upgrade_allows_joint_proposer_when_enabled() {
    new_test_ext().execute_with(|| {
        assert_ok!(RuntimeUpgrade::developer_direct_upgrade(
            RuntimeOrigin::signed(nrc_admin()),
            code_ok(),
        ));
        let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
        assert!(code_executed, "runtime code executor should be called");

        RUNTIME_CODE_EXECUTED.with(|v| *v.borrow_mut() = false);

        assert_ok!(RuntimeUpgrade::developer_direct_upgrade(
            RuntimeOrigin::signed(prc_admin()),
            code_ok(),
        ));
        let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
        assert!(
            code_executed,
            "PRC proposer should also trigger runtime code executor"
        );
    });
}

#[test]
fn developer_direct_upgrade_fails_when_disabled() {
    new_test_ext().execute_with(|| {
        DEV_UPGRADE_ENABLED.with(|v| *v.borrow_mut() = false);
        assert_noop!(
            RuntimeUpgrade::developer_direct_upgrade(
                RuntimeOrigin::signed(nrc_admin()),
                code_ok(),
            ),
            pallet::Error::<Test>::DeveloperUpgradeDisabled
        );
    });
}

#[test]
fn developer_direct_upgrade_rejects_non_joint_proposer() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            RuntimeUpgrade::developer_direct_upgrade(
                RuntimeOrigin::signed(outsider()),
                code_ok(),
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn developer_direct_upgrade_rejects_empty_code() {
    new_test_ext().execute_with(|| {
        let empty_code: pallet::CodeOf<Test> = vec![].try_into().expect("empty code");
        assert_noop!(
            RuntimeUpgrade::developer_direct_upgrade(
                RuntimeOrigin::signed(nrc_admin()),
                empty_code,
            ),
            pallet::Error::<Test>::EmptyRuntimeCode
        );
    });
}
