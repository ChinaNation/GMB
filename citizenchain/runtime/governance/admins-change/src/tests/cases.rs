#![cfg(test)]

use super::*;

#[test]
fn pending_subject_is_not_exposed_to_active_business_api() {
    new_test_ext().execute_with(|| {
        let institution = pending_subject_id();
        let admin_a = AccountId32::new([211u8; 32]);
        let admin_b = AccountId32::new([212u8; 32]);

        assert_ok!(AdminsChange::do_create_pending_subject(
            institution,
            ORG_REN,
            AdminSubjectKind::PersonalDuoqian,
            vec![admin_a.clone(), admin_b.clone()],
            2,
            admin_a.clone()
        ));

        assert!(!AdminsChange::is_active_subject_admin(
            ORG_REN,
            institution,
            &admin_a
        ));
        assert!(AdminsChange::active_subject_admins(ORG_REN, institution).is_none());
        assert_eq!(
            AdminsChange::pending_subject_admins_for_snapshot(ORG_REN, institution)
                .expect("pending snapshot admins should exist"),
            vec![admin_a.clone(), admin_b.clone()]
        );
        assert_eq!(
            AdminsChange::pending_subject_threshold_for_snapshot(ORG_REN, institution),
            Some(2)
        );

        assert_ok!(AdminsChange::do_activate_subject(institution));
        assert!(AdminsChange::is_active_subject_admin(
            ORG_REN,
            institution,
            &admin_a
        ));
        assert!(
            AdminsChange::pending_subject_admins_for_snapshot(ORG_REN, institution)
                .is_none()
        );
    });
}

#[test]
fn subject_lifecycle_trait_requires_votingengine_scope_for_activation() {
    new_test_ext().execute_with(|| {
        let institution = pending_subject_id();
        let admin_a = AccountId32::new([201u8; 32]);
        let admin_b = AccountId32::new([202u8; 32]);
        let proposal_id = <internal_vote::Pallet<Test> as InternalVoteEngine<
            AccountId32,
        >>::create_pending_subject_internal_proposal_with_snapshot_data(
            admin_a.clone(),
            ORG_REN,
            institution,
            vec![admin_a.clone(), admin_b.clone()],
            2,
            b"org-mgmt",
            b"subject-create".to_vec(),
        )
        .expect("pending subject proposal should be created");

        assert_ok!(AdminsChange::create_pending_subject_for_proposal(
            proposal_id,
            b"org-mgmt",
            institution,
            ORG_REN,
            AdminSubjectKind::PersonalDuoqian,
            vec![admin_a.clone(), admin_b],
            2,
            admin_a.clone()
        ));

        assert_noop!(
            AdminsChange::activate_subject_for_proposal(proposal_id, b"org-mgmt", institution),
            Error::<Test>::InvalidSubjectLifecycleScope
        );

        votingengine::pallet::Proposals::<Test>::mutate(proposal_id, |maybe| {
            let proposal = maybe.as_mut().expect("proposal should exist");
            proposal.status = STATUS_PASSED;
        });
        assert_noop!(
            AdminsChange::activate_subject_for_proposal(proposal_id, b"org-mgmt", institution),
            Error::<Test>::InvalidSubjectLifecycleScope
        );

        votingengine::pallet::CallbackExecutionScopes::<Test>::insert(proposal_id, ());
        assert_ok!(AdminsChange::activate_subject_for_proposal(
            proposal_id,
            b"org-mgmt",
            institution
        ));
        votingengine::pallet::CallbackExecutionScopes::<Test>::remove(proposal_id);
    });
}

#[test]
fn builtin_subjects_cannot_be_closed() {
    new_test_ext().execute_with(|| {
        for (institution, org, admin) in [
            (nrc_pallet_id(), ORG_NRC, nrc_admin(0)),
            (prc_pallet_id(), ORG_PRC, prc_admin(0)),
            (prb_pallet_id(), ORG_PRB, prb_admin(0)),
        ] {
            assert_noop!(
                AdminsChange::do_close_subject(institution),
                Error::<Test>::BuiltinSubjectCannotClose
            );

            let subject = Subjects::<Test>::get(institution)
                .expect("builtin subject should remain stored");
            assert_eq!(subject.kind, AdminSubjectKind::BuiltinInstitution);
            assert_eq!(subject.status, AdminSubjectStatus::Active);
            assert!(AdminsChange::is_active_subject_admin(
                org,
                institution,
                &admin
            ));
        }
    });
}

#[test]
fn dynamic_subjects_can_be_closed() {
    new_test_ext().execute_with(|| {
        for (offset, kind) in [
            (0u8, AdminSubjectKind::PersonalDuoqian),
            (1u8, AdminSubjectKind::SfidInstitution),
        ] {
            let mut institution = pending_subject_id();
            institution[0] = institution[0].saturating_add(offset);
            let admin_a = AccountId32::new([221u8.saturating_add(offset); 32]);
            let admin_b = AccountId32::new([231u8.saturating_add(offset); 32]);

            assert_ok!(AdminsChange::do_create_pending_subject(
                institution,
                ORG_REN,
                kind,
                vec![admin_a.clone(), admin_b],
                2,
                admin_a.clone()
            ));
            assert_ok!(AdminsChange::do_activate_subject(institution));
            assert_ok!(AdminsChange::do_close_subject(institution));

            let subject = Subjects::<Test>::get(institution)
                .expect("dynamic subject should remain stored");
            assert_eq!(subject.kind, kind);
            assert_eq!(subject.status, AdminSubjectStatus::Closed);
            assert!(!AdminsChange::is_active_subject_admin(
                ORG_REN,
                institution,
                &admin_a
            ));
            assert!(AdminsChange::active_subject_admins(ORG_REN, institution).is_none());
        }
    });
}

#[test]
fn duoqian_subjects_cannot_use_admin_replacement_entry() {
    new_test_ext().execute_with(|| {
        for (offset, kind) in [
            (0u8, AdminSubjectKind::PersonalDuoqian),
            (1u8, AdminSubjectKind::SfidInstitution),
        ] {
            let mut institution = pending_subject_id();
            institution[0] = institution[0].saturating_add(10u8.saturating_add(offset));
            let admin_a = AccountId32::new([41u8.saturating_add(offset); 32]);
            let admin_b = AccountId32::new([51u8.saturating_add(offset); 32]);
            let new_admin = AccountId32::new([61u8.saturating_add(offset); 32]);

            assert_ok!(AdminsChange::do_create_pending_subject(
                institution,
                ORG_REN,
                kind,
                vec![admin_a.clone(), admin_b.clone()],
                2,
                admin_a.clone()
            ));
            assert_ok!(AdminsChange::do_activate_subject(institution));

            assert_noop!(
                AdminsChange::propose_admin_replacement(
                    RuntimeOrigin::signed(admin_a.clone()),
                    ORG_REN,
                    institution,
                    admin_b,
                    new_admin
                ),
                Error::<Test>::InvalidSubjectKind
            );
        }
    });
}

#[test]
fn nrc_replacement_executes_when_yes_votes_reach_threshold() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let old_admin = nrc_admin(1);
        let new_admin = AccountId32::new([99u8; 32]);

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            old_admin.clone(),
            new_admin.clone()
        ));
        let pid = last_proposal_id();

        for i in 0..13 {
            assert_ok!(cast_vote(nrc_admin(i), pid, true));
        }

        let admins = current_admins(institution);
        assert!(admins.iter().any(|a| a == &new_admin));
        assert!(!admins.iter().any(|a| a == &old_admin));
        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("proposal should exist")
                .status,
            STATUS_EXECUTED
        );
        assert_eq!(finalized_event_count(pid, STATUS_EXECUTED), 1);
    });
}

#[test]
fn non_nrc_admin_cannot_propose_nrc_replacement() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        assert_noop!(
            AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([77u8; 32])
            ),
            Error::<Test>::UnauthorizedAdmin
        );
    });
}

#[test]
fn non_nrc_admin_cannot_vote_nrc_replacement() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            nrc_admin(1),
            AccountId32::new([88u8; 32])
        ));
        let pid = last_proposal_id();

        assert_noop!(
            cast_vote(prc_admin(0), pid, true),
            votingengine::pallet::Error::<Test>::NoPermission
        );
    });
}

#[test]
fn replaced_new_admin_can_propose_next_replacement() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let old_admin = nrc_admin(1);
        let new_admin = AccountId32::new([66u8; 32]);

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            old_admin,
            new_admin.clone()
        ));
        let pid = last_proposal_id();
        for i in 0..13 {
            assert_ok!(cast_vote(nrc_admin(i), pid, true));
        }

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(new_admin),
            ORG_NRC,
            institution,
            nrc_admin(2),
            AccountId32::new([67u8; 32])
        ));
    });
}

#[test]
fn prc_replacement_executes_when_yes_votes_reach_threshold() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let old_admin = prc_admin(1);
        let new_admin = AccountId32::new([55u8; 32]);

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(prc_admin(0)),
            ORG_PRC,
            institution,
            old_admin.clone(),
            new_admin.clone()
        ));
        let pid = last_proposal_id();

        // 省储会内部投票阈值：>=6
        for i in 0..6 {
            assert_ok!(cast_vote(prc_admin(i), pid, true));
        }

        let admins = current_admins(institution);
        assert!(admins.iter().any(|a| a == &new_admin));
        assert!(!admins.iter().any(|a| a == &old_admin));
    });
}

#[test]
fn prb_replacement_executes_when_yes_votes_reach_threshold() {
    new_test_ext().execute_with(|| {
        let institution = prb_pallet_id();
        let old_admin = prb_admin(1);
        let new_admin = AccountId32::new([56u8; 32]);

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(prb_admin(0)),
            ORG_PRB,
            institution,
            old_admin.clone(),
            new_admin.clone()
        ));
        let pid = last_proposal_id();

        // 省储行内部投票阈值：>=6
        for i in 0..6 {
            assert_ok!(cast_vote(prb_admin(i), pid, true));
        }

        let admins = current_admins(institution);
        assert!(admins.iter().any(|a| a == &new_admin));
        assert!(!admins.iter().any(|a| a == &old_admin));
    });
}

#[test]
fn non_prc_admin_cannot_propose_or_vote_prc_replacement() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();

        assert_noop!(
            AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(prb_admin(0)),
                ORG_PRC,
                institution,
                prc_admin(1),
                AccountId32::new([57u8; 32])
            ),
            Error::<Test>::UnauthorizedAdmin
        );

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(prc_admin(0)),
            ORG_PRC,
            institution,
            prc_admin(1),
            AccountId32::new([58u8; 32])
        ));
        let pid = last_proposal_id();

        assert_noop!(
            cast_vote(prb_admin(0), pid, true),
            votingengine::pallet::Error::<Test>::NoPermission
        );
    });
}

#[test]
fn non_prb_admin_cannot_propose_or_vote_prb_replacement() {
    new_test_ext().execute_with(|| {
        let institution = prb_pallet_id();

        assert_noop!(
            AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_PRB,
                institution,
                prb_admin(1),
                AccountId32::new([59u8; 32])
            ),
            Error::<Test>::UnauthorizedAdmin
        );

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(prb_admin(0)),
            ORG_PRB,
            institution,
            prb_admin(1),
            AccountId32::new([60u8; 32])
        ));
        let pid = last_proposal_id();

        assert_noop!(
            cast_vote(prc_admin(0), pid, true),
            votingengine::pallet::Error::<Test>::NoPermission
        );
    });
}

#[test]
fn regular_internal_proposal_blocks_admin_replacement() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        assert_ok!(<internal_vote::Pallet<Test> as InternalVoteEngine<
            AccountId32,
        >>::create_internal_proposal(
            nrc_admin(0), ORG_NRC, institution,
        ));

        assert_noop!(
            AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(1)),
                ORG_NRC,
                institution,
                nrc_admin(2),
                AccountId32::new([89u8; 32])
            ),
            votingengine::pallet::Error::<Test>::RegularInternalProposalActive
        );
    });
}

#[test]
fn vote_does_not_rollback_when_auto_execute_fails() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let old_admin = nrc_admin(1);
        let new_admin = AccountId32::new([61u8; 32]);

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            old_admin.clone(),
            new_admin
        ));
        let pid = last_proposal_id();

        Subjects::<Test>::mutate(institution, |maybe| {
            let subject = maybe.as_mut().expect("institution should exist");
            let admins = &mut subject.admins;
            let pos = admins
                .iter()
                .position(|a| a == &old_admin)
                .expect("old_admin should be in admins");
            admins[pos] = nrc_admin(18);
        });

        for i in [0usize, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] {
            assert_ok!(cast_vote(nrc_admin(i), pid, true));
        }

        let proposal =
            votingengine::Pallet::<Test>::proposals(pid).expect("proposal should exist");
        assert_eq!(proposal.status, STATUS_EXECUTION_FAILED);
        assert_eq!(finalized_event_count(pid, STATUS_EXECUTION_FAILED), 1);
        assert!(
            votingengine::Pallet::<Test>::internal_proposal_mutex(ORG_NRC, institution)
                .is_none()
        );
        let data = votingengine::Pallet::<Test>::get_proposal_data(pid)
            .expect("proposal data should exist");
        assert!(votingengine::Pallet::<Test>::is_proposal_owner(
            pid, MODULE_TAG
        ));
        let _action = AdminReplacementAction::<AccountId32>::decode(&mut &data[..])
            .expect("should decode");
        assert_noop!(
            VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(nrc_admin(0)), pid),
            votingengine::pallet::Error::<Test>::ProposalNotRetryable
        );
    });
}

#[test]
fn org_mismatch_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_PRC,
                nrc_pallet_id(),
                nrc_admin(1),
                AccountId32::new([74u8; 32])
            ),
            Error::<Test>::InstitutionOrgMismatch
        );
    });
}

#[test]
fn reject_vote_does_not_trigger_execution() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let old_admin = nrc_admin(1);
        let new_admin = AccountId32::new([75u8; 32]);

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            old_admin.clone(),
            new_admin.clone()
        ));
        let pid = last_proposal_id();

        assert_ok!(cast_vote(nrc_admin(2), pid, false));

        let admins = current_admins(institution);
        assert!(admins.iter().any(|a| a == &old_admin));
        assert!(!admins.iter().any(|a| a == &new_admin));
        assert!(
            votingengine::Pallet::<Test>::get_proposal_data(pid).is_some(),
            "proposal data should exist"
        );
    });
}

#[test]
fn propose_fails_when_old_admin_missing() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                nrc_pallet_id(),
                AccountId32::new([201u8; 32]),
                AccountId32::new([202u8; 32])
            ),
            Error::<Test>::OldAdminNotFound
        );
    });
}

#[test]
fn propose_fails_when_new_admin_already_exists() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                nrc_pallet_id(),
                nrc_admin(1),
                nrc_admin(2)
            ),
            Error::<Test>::NewAdminAlreadyExists
        );
    });
}

#[test]
fn executed_proposal_cannot_be_executed_again() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            nrc_admin(1),
            AccountId32::new([203u8; 32])
        ));
        let pid = last_proposal_id();

        for i in 0..13 {
            assert_ok!(cast_vote(nrc_admin(i), pid, true));
        }

        assert_noop!(
            VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(nrc_admin(0)), pid),
            votingengine::pallet::Error::<Test>::ProposalNotRetryable
        );
    });
}

#[test]
fn rejected_proposal_does_not_block_new_proposal() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            nrc_admin(1),
            AccountId32::new([206u8; 32])
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

        // 中文注释：投票引擎全局限额管控后，被拒绝的提案不再阻塞同机构新提案。
        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            nrc_admin(2),
            AccountId32::new([207u8; 32])
        ));
    });
}

#[test]
fn failed_auto_execute_enters_terminal_status_and_cannot_retry() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let old_admin = nrc_admin(1);
        let new_admin = AccountId32::new([208u8; 32]);

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            old_admin.clone(),
            new_admin.clone()
        ));
        let pid = last_proposal_id();

        Subjects::<Test>::mutate(institution, |maybe| {
            let subject = maybe.as_mut().expect("institution should exist");
            let admins = &mut subject.admins;
            let old_pos = admins
                .iter()
                .position(|a| a == &old_admin)
                .expect("old_admin should be in admins");
            admins[old_pos] = nrc_admin(18);
        });

        for i in [0usize, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] {
            assert_ok!(cast_vote(nrc_admin(i), pid, true));
        }

        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("proposal should exist")
                .status,
            STATUS_EXECUTION_FAILED
        );
        assert!(votingengine::Pallet::<Test>::get_proposal_data(pid).is_some());
        assert!(
            votingengine::Pallet::<Test>::internal_proposal_mutex(ORG_NRC, institution)
                .is_none()
        );

        Subjects::<Test>::mutate(institution, |maybe| {
            let subject = maybe.as_mut().expect("institution should exist");
            let admins = &mut subject.admins;
            let restore_pos = admins
                .iter()
                .position(|a| a == &nrc_admin(18))
                .expect("temporary admin marker should exist");
            admins[restore_pos] = old_admin.clone();
        });

        assert_noop!(
            VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(nrc_admin(0)), pid),
            votingengine::pallet::Error::<Test>::ProposalNotRetryable
        );
        let admins = current_admins(institution);
        assert!(!admins.iter().any(|a| a == &new_admin));
        assert!(admins.iter().any(|a| a == &old_admin));
    });
}

#[test]
fn execute_admin_replacement_rejects_wrong_proposal_kind_or_stage() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            nrc_admin(1),
            AccountId32::new([209u8; 32])
        ));
        let pid = last_proposal_id();
        mark_proposal_passed_without_callback(pid);

        votingengine::pallet::Proposals::<Test>::mutate(pid, |maybe| {
            let proposal = maybe.as_mut().expect("proposal should exist");
            proposal.kind = votingengine::PROPOSAL_KIND_JOINT;
        });
        assert_noop!(
            VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(nrc_admin(0)), pid),
            votingengine::pallet::Error::<Test>::ProposalOwnerMissing
        );

        votingengine::pallet::Proposals::<Test>::mutate(pid, |maybe| {
            let proposal = maybe.as_mut().expect("proposal should exist");
            proposal.kind = votingengine::PROPOSAL_KIND_INTERNAL;
            proposal.stage = votingengine::STAGE_JOINT;
        });
        assert_ok!(VotingEngine::retry_passed_proposal(
            RuntimeOrigin::signed(nrc_admin(0)),
            pid
        ));
        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("proposal should exist")
                .status,
            STATUS_EXECUTION_FAILED
        );
    });
}

#[test]
fn execute_admin_replacement_rejects_proposal_metadata_mismatch() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            nrc_admin(1),
            AccountId32::new([210u8; 32])
        ));
        let pid = last_proposal_id();
        mark_proposal_passed_without_callback(pid);

        votingengine::pallet::Proposals::<Test>::mutate(pid, |maybe| {
            let proposal = maybe.as_mut().expect("proposal should exist");
            proposal.internal_institution = Some(prc_pallet_id());
        });
        assert_noop!(
            VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(nrc_admin(0)), pid),
            votingengine::pallet::Error::<Test>::NoPermission
        );

        votingengine::pallet::Proposals::<Test>::mutate(pid, |maybe| {
            let proposal = maybe.as_mut().expect("proposal should exist");
            proposal.internal_institution = Some(institution);
            proposal.internal_org = Some(ORG_PRC);
        });
        assert_ok!(VotingEngine::retry_passed_proposal(
            RuntimeOrigin::signed(nrc_admin(0)),
            pid
        ));
        assert_eq!(
            votingengine::Pallet::<Test>::proposals(pid)
                .expect("proposal should exist")
                .status,
            STATUS_EXECUTION_FAILED
        );
    });
}

#[test]
fn vote_below_threshold_does_not_trigger_execution() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let old_admin = nrc_admin(1);
        let new_admin = AccountId32::new([204u8; 32]);

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            old_admin.clone(),
            new_admin.clone()
        ));
        let pid = last_proposal_id();

        assert_ok!(cast_vote(nrc_admin(2), pid, true));

        let admins = current_admins(institution);
        assert!(admins.iter().any(|a| a == &old_admin));
        assert!(!admins.iter().any(|a| a == &new_admin));
        assert!(
            votingengine::Pallet::<Test>::get_proposal_data(pid).is_some(),
            "proposal data should exist"
        );
    });
}

#[test]
fn invalid_institution_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                [0u8; 48],
                nrc_admin(1),
                AccountId32::new([205u8; 32])
            ),
            Error::<Test>::InvalidInstitution
        );
    });
}

/// 全员替换循环：连续替换 NRC 后六位管理员（idx 13..19），保持
/// 前 13 位作为投票者。验证 admin 数量恒为 NRC_ADMIN_COUNT、
/// 新人入名单、旧人出名单、互斥锁每轮正确释放。
#[test]
fn nrc_full_cycle_replacement_keeps_admin_count_stable() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        assert_eq!(
            current_admins(institution).len() as u32,
            NRC_ADMIN_COUNT
        );

        for i in 13..NRC_ADMIN_COUNT as usize {
            let old_admin = nrc_admin(i);
            let new_admin = AccountId32::new([180u8 + i as u8; 32]);

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));
            let pid = last_proposal_id();
            for v in 0..NRC_INTERNAL_THRESHOLD as usize {
                assert_ok!(cast_vote(nrc_admin(v), pid, true));
            }

            let admins = current_admins(institution);
            assert_eq!(
                admins.len() as u32,
                NRC_ADMIN_COUNT,
                "round {i}: admin count must stay at NRC_ADMIN_COUNT"
            );
            assert!(admins.contains(&new_admin), "round {i}: new admin must be in list");
            assert!(!admins.contains(&old_admin), "round {i}: old admin must be out");
            assert!(
                votingengine::Pallet::<Test>::internal_proposal_mutex(ORG_NRC, institution)
                    .is_none(),
                "round {i}: mutex must be released after finalize"
            );
        }
    });
}

/// 互斥锁回归：同机构、同 org 在第一个 admin-replacement 提案
/// 进行中时,第二个 propose 必须被 AdminSetMutationProposalActive 拦下。
#[test]
fn concurrent_nrc_admin_replacements_blocked_by_mutex() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();

        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            nrc_admin(13),
            AccountId32::new([220u8; 32])
        ));

        assert_noop!(
            AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(14),
                AccountId32::new([221u8; 32])
            ),
            votingengine::pallet::Error::<Test>::AdminSetMutationProposalActive
        );

        assert_noop!(
            AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(1)),
                ORG_NRC,
                institution,
                nrc_admin(13),
                AccountId32::new([222u8; 32])
            ),
            votingengine::pallet::Error::<Test>::AdminSetMutationProposalActive
        );
    });
}

/// 跨省隔离：PRC 一个省的管理员替换不得影响另一个省的管理员名单。
#[test]
fn prc_replacement_isolates_provinces() {
    new_test_ext().execute_with(|| {
        let prc_a = prc_pallet_id();
        // CHINA_CB[0]=NRC, [1]=辽宁(prc_pallet_id), 取另一省作为对照。
        let prc_b = subject_id_from_sfid_number(CHINA_CB[2].sfid_number)
            .expect("second prc institution should map");
        let prc_b_initial = current_admins(prc_b);

        let old_admin = prc_admin(1);
        let new_admin = AccountId32::new([240u8; 32]);
        assert_ok!(AdminsChange::propose_admin_replacement(
            RuntimeOrigin::signed(prc_admin(0)),
            ORG_PRC,
            prc_a,
            old_admin.clone(),
            new_admin.clone()
        ));
        let pid = last_proposal_id();
        for i in 0..PRC_INTERNAL_THRESHOLD as usize {
            assert_ok!(cast_vote(prc_admin(i), pid, true));
        }

        let prc_a_after = current_admins(prc_a);
        assert!(prc_a_after.contains(&new_admin));
        assert!(!prc_a_after.contains(&old_admin));

        let prc_b_after = current_admins(prc_b);
        assert_eq!(
            prc_b_initial, prc_b_after,
            "B 省管理员名单不得被 A 省替换影响"
        );
    });
}
