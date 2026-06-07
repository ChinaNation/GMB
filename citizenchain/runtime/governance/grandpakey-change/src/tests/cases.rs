#![cfg(test)]

use super::*;

#[test]
fn weak_small_order_new_key_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                prc_pallet_id(),
                identity_public_key()
            ),
            Error::<Test>::InvalidEd25519Key
        );
    });
}

#[test]
fn passed_proposal_executes_and_cleans_up_state() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let old_key = CurrentGrandpaKeys::<Test>::get(institution.clone())
            .expect("institution should have an initial key");
        let new_key = valid_public_key(31);

        assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
            RuntimeOrigin::signed(prc_admin(0)),
            institution.clone(),
            new_key,
        ));
        let pid = last_proposal_id();

        pass_prc_proposal(1, pid);

        let pending_change = Grandpa::pending_change().expect("change should be scheduled");
        assert_eq!(pending_change.scheduled_at, 1);
        assert_eq!(pending_change.delay, GrandpaChangeDelay::get());
        assert!(pending_change
            .next_authorities
            .iter()
            .any(|(authority, _)| *authority == authority_id_from_key(new_key)));

        assert_eq!(
            CurrentGrandpaKeys::<Test>::get(institution.clone()),
            Some(new_key)
        );
        assert!(GrandpaKeyOwnerByKey::<Test>::get(old_key).is_none());
        assert_eq!(
            GrandpaKeyOwnerByKey::<Test>::get(new_key),
            Some(institution.clone())
        );
        assert!(System::events().iter().any(|record| {
            matches!(
                &record.event,
                RuntimeEvent::GrandpaKeyChange(Event::<Test>::GrandpaKeyReplaced {
                    proposal_id,
                    institution: inst,
                    old_key: replaced_old_key,
                    new_key: replaced_new_key,
                }) if *proposal_id == pid
                    && *inst == institution
                    && *replaced_old_key == old_key
                    && *replaced_new_key == new_key
            )
        }));
    });
}

#[test]
fn passed_proposal_can_be_manually_executed_after_pending_change_clears() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let old_key = CurrentGrandpaKeys::<Test>::get(institution.clone())
            .expect("institution should have an initial key");
        let new_key = valid_public_key(41);

        assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
            RuntimeOrigin::signed(prc_admin(0)),
            institution.clone(),
            new_key,
        ));
        let pid = last_proposal_id();
        assert_ok!(Grandpa::schedule_change(
            grandpa_authorities(),
            GrandpaChangeDelay::get(),
            None,
        ));

        pass_prc_proposal(1, pid);

        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("passed proposal should remain for retries")
                .status,
            STATUS_PASSED
        );
        assert_eq!(
            CurrentGrandpaKeys::<Test>::get(institution.clone()),
            Some(old_key)
        );
        assert!(votingengine::Pallet::<Test>::get_proposal_data(pid).is_some());
        assert!(System::events().iter().any(|record| {
            matches!(
                &record.event,
                RuntimeEvent::GrandpaKeyChange(Event::<Test>::GrandpaKeyExecutionFailed {
                    proposal_id
                }) if *proposal_id == pid
            )
        }));

        finalize_grandpa_at(1 + GrandpaChangeDelay::get());
        assert!(Grandpa::pending_change().is_none());

        assert_ok!(VotingEngine::retry_passed_proposal(
            RuntimeOrigin::signed(prc_admin(0)),
            pid,
        ));

        assert_eq!(
            CurrentGrandpaKeys::<Test>::get(institution.clone()),
            Some(new_key)
        );
        assert!(GrandpaKeyOwnerByKey::<Test>::get(old_key).is_none());
        assert_eq!(
            GrandpaKeyOwnerByKey::<Test>::get(new_key),
            Some(institution.clone())
        );
        assert!(Grandpa::pending_change().is_some());
    });
}

#[test]
fn cancel_failed_replace_grandpa_key_cleans_up_passed_but_invalid_proposal() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let old_key = CurrentGrandpaKeys::<Test>::get(institution.clone())
            .expect("institution should have an initial key");
        let new_key = valid_public_key(51);
        let replacement_authority = valid_public_key(52);

        assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
            RuntimeOrigin::signed(prc_admin(0)),
            institution.clone(),
            new_key,
        ));
        let pid = last_proposal_id();
        assert_ok!(Grandpa::schedule_change(
            vec![
                (authority_id_from_key(CHINA_CB[0].grandpa_key), 1),
                (authority_id_from_key(replacement_authority), 1),
            ],
            GrandpaChangeDelay::get(),
            None,
        ));

        pass_prc_proposal(1, pid);

        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("passed proposal should remain for cleanup")
                .status,
            STATUS_PASSED
        );
        finalize_grandpa_at(1 + GrandpaChangeDelay::get());

        assert_eq!(
            CurrentGrandpaKeys::<Test>::get(institution.clone()),
            Some(old_key)
        );
        assert_eq!(
            Grandpa::grandpa_authorities(),
            vec![
                (authority_id_from_key(CHINA_CB[0].grandpa_key), 1),
                (authority_id_from_key(replacement_authority), 1),
            ]
        );

        assert_ok!(VotingEngine::cancel_passed_proposal(
            RuntimeOrigin::signed(prc_admin(0)),
            pid,
            Default::default(),
        ));
        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("cancelled proposal should remain until cleanup")
                .status,
            STATUS_EXECUTION_FAILED
        );

        assert!(System::events().iter().any(|record| {
            matches!(
                &record.event,
                RuntimeEvent::GrandpaKeyChange(Event::<Test>::FailedProposalCancelled {
                    proposal_id,
                    institution: inst,
                }) if *proposal_id == pid && *inst == institution
            )
        }));
    });
}

#[test]
fn cancel_failed_replace_grandpa_key_rejects_temporarily_blocked_proposal() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let old_key = CurrentGrandpaKeys::<Test>::get(institution.clone())
            .expect("institution should have an initial key");
        let new_key = valid_public_key(71);

        assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
            RuntimeOrigin::signed(prc_admin(0)),
            institution.clone(),
            new_key,
        ));
        let pid = last_proposal_id();
        assert_ok!(Grandpa::schedule_change(
            grandpa_authorities(),
            GrandpaChangeDelay::get(),
            None,
        ));

        pass_prc_proposal(1, pid);

        assert_noop!(
            VotingEngine::cancel_passed_proposal(
                RuntimeOrigin::signed(prc_admin(0)),
                pid,
                Default::default(),
            ),
            Error::<Test>::GrandpaChangePending
        );

        assert_eq!(
            CurrentGrandpaKeys::<Test>::get(institution.clone()),
            Some(old_key)
        );
        assert!(votingengine::Pallet::<Test>::get_proposal_data(pid).is_some());
        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("passed proposal should remain active")
                .status,
            STATUS_PASSED
        );
    });
}

#[test]
fn finalized_vote_fatal_fails_when_old_authority_disappeared() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let old_key = CurrentGrandpaKeys::<Test>::get(institution.clone())
            .expect("institution should have an initial key");
        let new_key = valid_public_key(72);
        let replacement_authority = valid_public_key(73);

        assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
            RuntimeOrigin::signed(prc_admin(0)),
            institution.clone(),
            new_key,
        ));
        let pid = last_proposal_id();

        // 中文注释：模拟其他治理动作已经把提案绑定的旧 authority 替换掉。
        assert_ok!(Grandpa::schedule_change(
            vec![
                (authority_id_from_key(CHINA_CB[0].grandpa_key), 1),
                (authority_id_from_key(replacement_authority), 1),
                (authority_id_from_key(CHINA_CB[2].grandpa_key), 1),
            ],
            GrandpaChangeDelay::get(),
            None,
        ));
        finalize_grandpa_at(1 + GrandpaChangeDelay::get());
        assert!(Grandpa::pending_change().is_none());

        pass_prc_proposal(1, pid);

        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("fatal failed proposal should remain until cleanup")
                .status,
            STATUS_EXECUTION_FAILED
        );
        assert_eq!(
            CurrentGrandpaKeys::<Test>::get(institution.clone()),
            Some(old_key)
        );
        assert!(GrandpaKeyOwnerByKey::<Test>::get(new_key).is_none());
        assert!(System::events().iter().any(|record| {
            matches!(
                &record.event,
                RuntimeEvent::GrandpaKeyChange(Event::<Test>::GrandpaKeyExecutionFailed {
                    proposal_id
                }) if *proposal_id == pid
            )
        }));
    });
}

#[test]
fn finalized_vote_fatal_fails_when_new_key_collides_after_first_execution() {
    new_test_ext().execute_with(|| {
        let first_institution = cb_pallet_id(1);
        let second_institution = cb_pallet_id(2);
        let first_old_key = CurrentGrandpaKeys::<Test>::get(first_institution.clone())
            .expect("first institution should have an initial key");
        let second_old_key = CurrentGrandpaKeys::<Test>::get(second_institution.clone())
            .expect("second institution should have an initial key");
        let shared_new_key = valid_public_key(74);

        assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
            RuntimeOrigin::signed(cb_admin(1, 0)),
            first_institution.clone(),
            shared_new_key,
        ));
        let first_pid = last_proposal_id();
        assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
            RuntimeOrigin::signed(cb_admin(2, 0)),
            second_institution.clone(),
            shared_new_key,
        ));
        let second_pid = last_proposal_id();

        pass_prc_proposal(1, first_pid);
        assert_eq!(
            CurrentGrandpaKeys::<Test>::get(first_institution.clone()),
            Some(shared_new_key)
        );
        assert_eq!(
            GrandpaKeyOwnerByKey::<Test>::get(shared_new_key),
            Some(first_institution.clone())
        );
        finalize_grandpa_at(1 + GrandpaChangeDelay::get());
        assert!(Grandpa::pending_change().is_none());

        pass_prc_proposal(2, second_pid);

        assert_eq!(
            votingengine::Pallet::<Test>::proposals(second_pid)
                .expect("colliding proposal should remain until cleanup")
                .status,
            STATUS_EXECUTION_FAILED
        );
        assert_eq!(
            CurrentGrandpaKeys::<Test>::get(second_institution.clone()),
            Some(second_old_key)
        );
        assert_eq!(
            GrandpaKeyOwnerByKey::<Test>::get(shared_new_key),
            Some(first_institution.clone())
        );
        assert!(GrandpaKeyOwnerByKey::<Test>::get(first_old_key).is_none());
        assert!(System::events().iter().any(|record| {
            matches!(
                &record.event,
                RuntimeEvent::GrandpaKeyChange(Event::<Test>::GrandpaKeyExecutionFailed {
                    proposal_id
                }) if *proposal_id == second_pid
            )
        }));
    });
}

// ========================================================================
// 补充的错误路径和边界测试
// ========================================================================

#[test]
fn propose_rejects_zero_key() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                prc_pallet_id(),
                [0u8; 32],
            ),
            Error::<Test>::NewKeyIsZero
        );
    });
}

#[test]
fn propose_rejects_unchanged_key() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let current_key = CurrentGrandpaKeys::<Test>::get(institution.clone())
            .expect("institution should have key");
        assert_noop!(
            GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution.clone(),
                current_key,
            ),
            Error::<Test>::NewKeyUnchanged
        );
    });
}

#[test]
fn propose_rejects_key_owned_by_other_institution() {
    new_test_ext().execute_with(|| {
        // CHINA_CB[0] 是国储会的 key，用它作为省储会的 new_key 应失败
        let nrc_key = CHINA_CB[0].grandpa_key;
        assert_noop!(
            GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                prc_pallet_id(),
                nrc_key,
            ),
            Error::<Test>::NewKeyAlreadyUsed
        );
    });
}

#[test]
fn propose_rejects_unauthorized_admin() {
    new_test_ext().execute_with(|| {
        // 使用一个不在 duoqian_admins 中的随机账户
        let outsider = AccountId32::new([99u8; 32]);
        assert_noop!(
            GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(outsider),
                prc_pallet_id(),
                valid_public_key(80),
            ),
            Error::<Test>::UnauthorizedAdmin
        );
    });
}

#[test]
fn propose_rejects_invalid_institution() {
    new_test_ext().execute_with(|| {
        let fake_institution = AccountId32::new([99u8; 32]);
        assert_noop!(
            GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                fake_institution,
                valid_public_key(81),
            ),
            Error::<Test>::InvalidInstitution
        );
    });
}

#[test]
fn execute_rejects_non_passed_proposal() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let new_key = valid_public_key(82);
        assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
            RuntimeOrigin::signed(prc_admin(0)),
            institution.clone(),
            new_key,
        ));
        let pid = last_proposal_id();
        // 不投票，直接尝试执行
        assert_noop!(
            VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(prc_admin(0)), pid,),
            votingengine::pallet::Error::<Test>::ProposalNotRetryable
        );
    });
}

#[test]
fn cancel_rejects_still_executable_proposal() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let new_key = valid_public_key(83);

        // 先制造 pending change 阻塞
        assert_ok!(Grandpa::schedule_change(
            grandpa_authorities(),
            GrandpaChangeDelay::get(),
            None,
        ));

        assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
            RuntimeOrigin::signed(prc_admin(0)),
            institution.clone(),
            new_key,
        ));
        let pid = last_proposal_id();

        // 投票通过，自动执行因 pending change 失败
        pass_prc_proposal(1, pid);
        assert!(System::events().iter().any(|r| matches!(
            &r.event,
            RuntimeEvent::GrandpaKeyChange(Event::<Test>::GrandpaKeyExecutionFailed { .. })
        )));

        // 清除 pending change
        finalize_grandpa_at(1 + GrandpaChangeDelay::get());
        assert!(Grandpa::pending_change().is_none());

        // 提案仍可执行，不允许取消
        assert_noop!(
            VotingEngine::cancel_passed_proposal(
                RuntimeOrigin::signed(prc_admin(0)),
                pid,
                Default::default(),
            ),
            Error::<Test>::ProposalStillExecutable
        );
    });
}

#[test]
fn vote_rejects_unauthorized_admin() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let new_key = valid_public_key(85);
        assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
            RuntimeOrigin::signed(prc_admin(0)),
            institution.clone(),
            new_key,
        ));
        let pid = last_proposal_id();
        let outsider = AccountId32::new([98u8; 32]);
        assert_noop!(
            cast_vote(outsider, pid, true),
            votingengine::pallet::Error::<Test>::NoPermission
        );
    });
}
