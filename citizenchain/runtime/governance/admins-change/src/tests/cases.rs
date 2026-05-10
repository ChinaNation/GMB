#![cfg(test)]

use super::*;

// 中文注释：生命周期事件按 subject + org 精确计数，确保索引器可直接按组织分桶。
fn activated_event_count(subject: SubjectId, org: u8) -> usize {
    System::events()
        .iter()
        .filter(|record| {
            matches!(
                &record.event,
                RuntimeEvent::AdminsChange(Event::AdminSubjectActivated {
                    subject: event_subject,
                    org: event_org,
                }) if *event_subject == subject && *event_org == org
            )
        })
        .count()
}

fn pending_removed_event_count(subject: SubjectId, org: u8) -> usize {
    System::events()
        .iter()
        .filter(|record| {
            matches!(
                &record.event,
                RuntimeEvent::AdminsChange(Event::AdminSubjectPendingRemoved {
                    subject: event_subject,
                    org: event_org,
                }) if *event_subject == subject && *event_org == org
            )
        })
        .count()
}

fn closed_event_count(subject: SubjectId, org: u8) -> usize {
    System::events()
        .iter()
        .filter(|record| {
            matches!(
                &record.event,
                RuntimeEvent::AdminsChange(Event::AdminSubjectClosed {
                    subject: event_subject,
                    org: event_org,
                }) if *event_subject == subject && *event_org == org
            )
        })
        .count()
}

#[test]
fn dynamic_threshold_is_derived_from_admin_count() {
    assert_eq!(dynamic_threshold(0), None);
    assert_eq!(dynamic_threshold(1), None);
    assert_eq!(dynamic_threshold(2), Some(2));
    assert_eq!(dynamic_threshold(3), Some(2));
    assert_eq!(dynamic_threshold(4), Some(2));
    assert_eq!(dynamic_threshold(5), Some(3));
}

#[test]
fn institution_account_min_admins_two_works() {
    new_test_ext().execute_with(|| {
        let institution = pending_subject_id();
        let admin_a = AccountId32::new([110u8; 32]);
        let admin_b = AccountId32::new([111u8; 32]);

        assert_ok!(AdminsChange::do_create_pending_subject(
            institution,
            ORG_PUP,
            AdminSubjectKind::InstitutionAccount,
            vec![admin_a.clone(), admin_b],
            admin_a,
        ));
        assert_ok!(AdminsChange::do_activate_subject(institution));
        assert_eq!(
            AdminsChange::active_subject_threshold(ORG_PUP, institution),
            Some(2)
        );
    });
}

#[test]
fn institution_account_threshold_follows_ceil_half() {
    new_test_ext().execute_with(|| {
        for (count, expected_threshold) in [(2u32, 2u32), (3, 2), (4, 2), (5, 3), (6, 3), (7, 4)] {
            let mut institution = pending_subject_id();
            institution[1] = count as u8;
            let admins: Vec<AccountId32> = (0..count)
                .map(|i| AccountId32::new([100u8 + i as u8; 32]))
                .collect();
            let creator = admins[0].clone();

            assert_ok!(AdminsChange::do_create_pending_subject(
                institution,
                ORG_PUP,
                AdminSubjectKind::InstitutionAccount,
                admins,
                creator,
            ));
            assert_ok!(AdminsChange::do_activate_subject(institution));
            assert_eq!(
                AdminsChange::active_subject_threshold(ORG_PUP, institution),
                Some(expected_threshold),
                "admin_count={count} expected ceil(n/2)={expected_threshold}"
            );
        }
    });
}

#[test]
fn institution_account_below_two_admins_rejected() {
    new_test_ext().execute_with(|| {
        let institution = pending_subject_id();
        let admin_a = AccountId32::new([130u8; 32]);

        assert_noop!(
            AdminsChange::do_create_pending_subject(
                institution,
                ORG_PUP,
                AdminSubjectKind::InstitutionAccount,
                vec![admin_a.clone()],
                admin_a,
            ),
            Error::<Test>::InvalidAdminCount
        );
    });
}

#[test]
fn institution_account_requires_org_pup_or_oth() {
    new_test_ext().execute_with(|| {
        let institution = pending_subject_id();
        let admin_a = AccountId32::new([140u8; 32]);
        let admin_b = AccountId32::new([141u8; 32]);

        for org in [ORG_PUP, ORG_OTH] {
            assert_ok!(AdminsChange::do_create_pending_subject(
                {
                    let mut subject = institution;
                    subject[0] = subject[0].saturating_add(org);
                    subject
                },
                org,
                AdminSubjectKind::InstitutionAccount,
                vec![admin_a.clone(), admin_b.clone()],
                admin_a.clone(),
            ));
        }

        for wrong_org in [ORG_NRC, ORG_PRC, ORG_PRB, ORG_REN] {
            assert_noop!(
                AdminsChange::do_create_pending_subject(
                    institution,
                    wrong_org,
                    AdminSubjectKind::InstitutionAccount,
                    vec![admin_a.clone(), admin_b.clone()],
                    admin_a.clone(),
                ),
                Error::<Test>::InvalidSubjectKind
            );
        }
    });
}

#[test]
fn institution_account_at_max_admins_works() {
    new_test_ext().execute_with(|| {
        let institution = pending_subject_id();
        let max =
            <<Test as Config>::MaxAdminsPerInstitution as frame_support::traits::Get<u32>>::get();
        let admins: Vec<AccountId32> = (0..max)
            .map(|i| AccountId32::new([(i & 0xff) as u8; 32]))
            .collect();
        let creator = admins[0].clone();

        assert_ok!(AdminsChange::do_create_pending_subject(
            institution,
            ORG_OTH,
            AdminSubjectKind::InstitutionAccount,
            admins,
            creator,
        ));
        assert_ok!(AdminsChange::do_activate_subject(institution));
        assert_eq!(
            AdminsChange::active_subject_admin_count(ORG_OTH, institution),
            Some(max)
        );
        assert_eq!(
            AdminsChange::active_subject_threshold(ORG_OTH, institution),
            Some(max.saturating_add(1) / 2)
        );
    });
}

#[test]
fn remove_pending_subject_requires_existing_pending_subject() {
    new_test_ext().execute_with(|| {
        let institution = pending_subject_id();
        let admin_a = AccountId32::new([151u8; 32]);
        let admin_b = AccountId32::new([152u8; 32]);

        assert_noop!(
            AdminsChange::do_remove_pending_subject(institution),
            Error::<Test>::InvalidInstitution
        );

        assert_ok!(AdminsChange::do_create_pending_subject(
            institution,
            ORG_REN,
            AdminSubjectKind::PersonalDuoqian,
            vec![admin_a.clone(), admin_b],
            admin_a,
        ));
        assert_ok!(AdminsChange::do_activate_subject(institution));
        assert_noop!(
            AdminsChange::do_remove_pending_subject(institution),
            Error::<Test>::SubjectNotPending
        );
    });
}

#[test]
fn subject_lifecycle_events_include_org() {
    new_test_ext().execute_with(|| {
        let mut institution = pending_subject_id();
        let admin_a = AccountId32::new([161u8; 32]);
        let admin_b = AccountId32::new([162u8; 32]);

        assert_ok!(AdminsChange::do_create_pending_subject(
            institution,
            ORG_REN,
            AdminSubjectKind::PersonalDuoqian,
            vec![admin_a.clone(), admin_b.clone()],
            admin_a.clone(),
        ));
        assert_ok!(AdminsChange::do_activate_subject(institution));
        assert_ok!(AdminsChange::do_close_subject(institution));
        assert_eq!(activated_event_count(institution, ORG_REN), 1);
        assert_eq!(closed_event_count(institution, ORG_REN), 1);

        institution[1] = institution[1].saturating_add(1);
        assert_ok!(AdminsChange::do_create_pending_subject(
            institution,
            ORG_REN,
            AdminSubjectKind::PersonalDuoqian,
            vec![admin_a.clone(), admin_b],
            admin_a,
        ));
        assert_ok!(AdminsChange::do_remove_pending_subject(institution));
        assert_eq!(pending_removed_event_count(institution, ORG_REN), 1);
    });
}

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
        assert!(AdminsChange::pending_subject_admins_for_snapshot(ORG_REN, institution).is_none());
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

            let subject =
                Subjects::<Test>::get(institution).expect("builtin subject should remain stored");
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
        for (offset, org, kind) in [
            (0u8, ORG_REN, AdminSubjectKind::PersonalDuoqian),
            (1u8, ORG_PUP, AdminSubjectKind::InstitutionAccount),
            (2u8, ORG_OTH, AdminSubjectKind::InstitutionAccount),
        ] {
            let mut institution = pending_subject_id();
            institution[0] = institution[0].saturating_add(offset);
            let admin_a = AccountId32::new([221u8.saturating_add(offset); 32]);
            let admin_b = AccountId32::new([231u8.saturating_add(offset); 32]);

            assert_ok!(AdminsChange::do_create_pending_subject(
                institution,
                org,
                kind,
                vec![admin_a.clone(), admin_b],
                admin_a.clone()
            ));
            assert_ok!(AdminsChange::do_activate_subject(institution));
            assert_ok!(AdminsChange::do_close_subject(institution));

            let subject =
                Subjects::<Test>::get(institution).expect("dynamic subject should remain stored");
            assert_eq!(subject.kind, kind);
            assert_eq!(subject.status, AdminSubjectStatus::Closed);
            assert!(!AdminsChange::is_active_subject_admin(
                org,
                institution,
                &admin_a
            ));
            assert!(AdminsChange::active_subject_admins(org, institution).is_none());
        }
    });
}

#[test]
fn dynamic_subjects_can_use_admin_set_change_entry() {
    new_test_ext().execute_with(|| {
        for (offset, org, kind) in [
            (0u8, ORG_REN, AdminSubjectKind::PersonalDuoqian),
            (1u8, ORG_PUP, AdminSubjectKind::InstitutionAccount),
            (2u8, ORG_OTH, AdminSubjectKind::InstitutionAccount),
        ] {
            let mut institution = pending_subject_id();
            institution[0] = institution[0].saturating_add(10u8.saturating_add(offset));
            let admin_a = AccountId32::new([41u8.saturating_add(offset); 32]);
            let admin_b = AccountId32::new([51u8.saturating_add(offset); 32]);
            let new_admin = AccountId32::new([61u8.saturating_add(offset); 32]);

            assert_ok!(AdminsChange::do_create_pending_subject(
                institution,
                org,
                kind,
                vec![admin_a.clone(), admin_b.clone()],
                admin_a.clone()
            ));
            assert_ok!(AdminsChange::do_activate_subject(institution));

            assert_ok!(propose_admin_set_replacement(
                RuntimeOrigin::signed(admin_a.clone()),
                org,
                institution,
                admin_b,
                new_admin
            ));
        }
    });
}

#[test]
fn dynamic_subject_set_change_can_add_delete_and_recalculate_threshold() {
    new_test_ext().execute_with(|| {
        let institution = pending_subject_id();
        let admin_a = AccountId32::new([71u8; 32]);
        let admin_b = AccountId32::new([72u8; 32]);
        let admin_c = AccountId32::new([73u8; 32]);

        assert_ok!(AdminsChange::do_create_pending_subject(
            institution,
            ORG_PUP,
            AdminSubjectKind::InstitutionAccount,
            vec![admin_a.clone(), admin_b.clone()],
            admin_a.clone()
        ));
        assert_ok!(AdminsChange::do_activate_subject(institution));
        assert_eq!(
            AdminsChange::active_subject_threshold(ORG_PUP, institution),
            Some(2)
        );

        assert_ok!(AdminsChange::propose_admin_set_change(
            RuntimeOrigin::signed(admin_a.clone()),
            ORG_PUP,
            institution,
            bounded_admins(vec![admin_a.clone(), admin_b.clone(), admin_c.clone()])
        ));
        let add_pid = last_proposal_id();
        assert_ok!(cast_vote(admin_a.clone(), add_pid, true));
        assert_ok!(cast_vote(admin_b.clone(), add_pid, true));
        assert_eq!(
            AdminsChange::active_subject_admin_count(ORG_PUP, institution),
            Some(3)
        );
        assert_eq!(
            AdminsChange::active_subject_threshold(ORG_PUP, institution),
            Some(2)
        );

        assert_ok!(AdminsChange::propose_admin_set_change(
            RuntimeOrigin::signed(admin_c.clone()),
            ORG_PUP,
            institution,
            bounded_admins(vec![admin_a.clone(), admin_c.clone()])
        ));
        let delete_pid = last_proposal_id();
        assert_ok!(cast_vote(admin_a.clone(), delete_pid, true));
        assert_ok!(cast_vote(admin_c.clone(), delete_pid, true));
        assert_eq!(
            AdminsChange::active_subject_admins(ORG_PUP, institution).unwrap(),
            vec![admin_a, admin_c]
        );
        assert_eq!(
            AdminsChange::active_subject_threshold(ORG_PUP, institution),
            Some(2)
        );
    });
}

#[test]
fn sfid_institution_is_not_valid_admin_subject_kind() {
    new_test_ext().execute_with(|| {
        let institution = pending_subject_id();
        let admin_a = AccountId32::new([241u8; 32]);
        let admin_b = AccountId32::new([242u8; 32]);

        for org in [ORG_REN, ORG_PUP, ORG_OTH] {
            assert_noop!(
                AdminsChange::do_create_pending_subject(
                    institution,
                    org,
                    AdminSubjectKind::SfidInstitution,
                    vec![admin_a.clone(), admin_b.clone()],
                    admin_a.clone()
                ),
                Error::<Test>::InvalidSubjectKind
            );
        }
        assert_eq!(
            derived_threshold(AdminSubjectKind::SfidInstitution, ORG_PUP, 2),
            None
        );
    });
}

#[test]
fn nrc_set_change_executes_when_yes_votes_reach_threshold() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let old_admin = nrc_admin(1);
        let new_admin = AccountId32::new([99u8; 32]);

        assert_ok!(propose_admin_set_replacement(
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
fn non_nrc_admin_cannot_propose_nrc_set_change() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        assert_noop!(
            propose_admin_set_replacement(
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
fn non_nrc_admin_cannot_vote_nrc_set_change() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        assert_ok!(propose_admin_set_replacement(
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
fn replaced_new_admin_can_propose_next_set_change() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let old_admin = nrc_admin(1);
        let new_admin = AccountId32::new([66u8; 32]);

        assert_ok!(propose_admin_set_replacement(
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

        assert_ok!(propose_admin_set_replacement(
            RuntimeOrigin::signed(new_admin),
            ORG_NRC,
            institution,
            nrc_admin(2),
            AccountId32::new([67u8; 32])
        ));
    });
}

#[test]
fn prc_set_change_executes_when_yes_votes_reach_threshold() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();
        let old_admin = prc_admin(1);
        let new_admin = AccountId32::new([55u8; 32]);

        assert_ok!(propose_admin_set_replacement(
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
fn prb_set_change_executes_when_yes_votes_reach_threshold() {
    new_test_ext().execute_with(|| {
        let institution = prb_pallet_id();
        let old_admin = prb_admin(1);
        let new_admin = AccountId32::new([56u8; 32]);

        assert_ok!(propose_admin_set_replacement(
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
fn non_prc_admin_cannot_propose_or_vote_prc_set_change() {
    new_test_ext().execute_with(|| {
        let institution = prc_pallet_id();

        assert_noop!(
            propose_admin_set_replacement(
                RuntimeOrigin::signed(prb_admin(0)),
                ORG_PRC,
                institution,
                prc_admin(1),
                AccountId32::new([57u8; 32])
            ),
            Error::<Test>::UnauthorizedAdmin
        );

        assert_ok!(propose_admin_set_replacement(
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
fn non_prb_admin_cannot_propose_or_vote_prb_set_change() {
    new_test_ext().execute_with(|| {
        let institution = prb_pallet_id();

        assert_noop!(
            propose_admin_set_replacement(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_PRB,
                institution,
                prb_admin(1),
                AccountId32::new([59u8; 32])
            ),
            Error::<Test>::UnauthorizedAdmin
        );

        assert_ok!(propose_admin_set_replacement(
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
fn regular_internal_proposal_blocks_admin_set_change() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        assert_ok!(<internal_vote::Pallet<Test> as InternalVoteEngine<
            AccountId32,
        >>::create_internal_proposal(
            nrc_admin(0), ORG_NRC, institution,
        ));

        assert_noop!(
            propose_admin_set_replacement(
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

        assert_ok!(propose_admin_set_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            old_admin.clone(),
            new_admin
        ));
        let pid = last_proposal_id();

        Subjects::<Test>::mutate(institution, |maybe| {
            let subject = maybe.as_mut().expect("institution should exist");
            subject.status = AdminSubjectStatus::Closed;
        });

        for i in [0usize, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] {
            assert_ok!(cast_vote(nrc_admin(i), pid, true));
        }

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal should exist");
        assert_eq!(proposal.status, STATUS_EXECUTION_FAILED);
        assert_eq!(finalized_event_count(pid, STATUS_EXECUTION_FAILED), 1);
        assert!(
            votingengine::Pallet::<Test>::internal_proposal_mutex(ORG_NRC, institution).is_none()
        );
        let data = votingengine::Pallet::<Test>::get_proposal_data(pid)
            .expect("proposal data should exist");
        assert!(votingengine::Pallet::<Test>::is_proposal_owner(
            pid, MODULE_TAG
        ));
        let _action =
            AdminSetChangeAction::<AdminsOf<Test>>::decode(&mut &data[..]).expect("should decode");
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
            propose_admin_set_replacement(
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

        assert_ok!(propose_admin_set_replacement(
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
fn propose_fails_when_admin_set_unchanged() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        assert_noop!(
            AdminsChange::propose_admin_set_change(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                bounded_admins(current_admins(institution))
            ),
            Error::<Test>::AdminSetUnchanged
        );
    });
}

#[test]
fn propose_fails_when_admin_set_only_reordered() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        let mut admins = current_admins(institution);
        admins.swap(0, 1);

        assert_noop!(
            AdminsChange::propose_admin_set_change(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                bounded_admins(admins)
            ),
            Error::<Test>::AdminSetUnchanged
        );
    });
}

#[test]
fn propose_fails_when_admin_set_has_duplicate_admin() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            propose_admin_set_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                nrc_pallet_id(),
                nrc_admin(1),
                nrc_admin(2)
            ),
            Error::<Test>::DuplicateAdmin
        );
    });
}

#[test]
fn executed_proposal_cannot_be_executed_again() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();

        assert_ok!(propose_admin_set_replacement(
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
        assert_ok!(propose_admin_set_replacement(
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
        assert_ok!(propose_admin_set_replacement(
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

        assert_ok!(propose_admin_set_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            old_admin.clone(),
            new_admin.clone()
        ));
        let pid = last_proposal_id();

        Subjects::<Test>::mutate(institution, |maybe| {
            let subject = maybe.as_mut().expect("institution should exist");
            subject.status = AdminSubjectStatus::Closed;
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
            votingengine::Pallet::<Test>::internal_proposal_mutex(ORG_NRC, institution).is_none()
        );

        Subjects::<Test>::mutate(institution, |maybe| {
            let subject = maybe.as_mut().expect("institution should exist");
            subject.status = AdminSubjectStatus::Active;
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
fn execute_admin_set_change_rejects_wrong_proposal_kind_or_stage() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();

        assert_ok!(propose_admin_set_replacement(
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
fn execute_admin_set_change_rejects_proposal_metadata_mismatch() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();

        assert_ok!(propose_admin_set_replacement(
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

        assert_ok!(propose_admin_set_replacement(
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
            AdminsChange::propose_admin_set_change(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                [0u8; 48],
                bounded_admins(vec![nrc_admin(0), AccountId32::new([205u8; 32])])
            ),
            Error::<Test>::InvalidInstitution
        );
    });
}

/// 全员替换循环：连续替换 NRC 后六位管理员（idx 13..19），保持
/// 前 13 位作为投票者。验证 admin 数量恒为 NRC_ADMIN_COUNT、
/// 新人入名单、旧人出名单、互斥锁每轮正确释放。
#[test]
fn nrc_full_cycle_set_change_keeps_admin_count_stable() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();
        assert_eq!(current_admins(institution).len() as u32, NRC_ADMIN_COUNT);

        for i in 13..NRC_ADMIN_COUNT as usize {
            let old_admin = nrc_admin(i);
            let new_admin = AccountId32::new([180u8 + i as u8; 32]);

            assert_ok!(propose_admin_set_replacement(
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
            assert!(
                admins.contains(&new_admin),
                "round {i}: new admin must be in list"
            );
            assert!(
                !admins.contains(&old_admin),
                "round {i}: old admin must be out"
            );
            assert!(
                votingengine::Pallet::<Test>::internal_proposal_mutex(ORG_NRC, institution)
                    .is_none(),
                "round {i}: mutex must be released after finalize"
            );
        }
    });
}

/// 互斥锁回归：同机构、同 org 在第一个 admin-set_change 提案
/// 进行中时,第二个 propose 必须被 AdminSetMutationProposalActive 拦下。
#[test]
fn concurrent_nrc_admin_set_changes_blocked_by_mutex() {
    new_test_ext().execute_with(|| {
        let institution = nrc_pallet_id();

        assert_ok!(propose_admin_set_replacement(
            RuntimeOrigin::signed(nrc_admin(0)),
            ORG_NRC,
            institution,
            nrc_admin(13),
            AccountId32::new([220u8; 32])
        ));

        assert_noop!(
            propose_admin_set_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(14),
                AccountId32::new([221u8; 32])
            ),
            votingengine::pallet::Error::<Test>::AdminSetMutationProposalActive
        );

        assert_noop!(
            propose_admin_set_replacement(
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
fn prc_set_change_isolates_provinces() {
    new_test_ext().execute_with(|| {
        let prc_a = prc_pallet_id();
        // CHINA_CB[0]=NRC, [1]=辽宁(prc_pallet_id), 取另一省作为对照。
        let prc_b = subject_id_from_sfid_number(CHINA_CB[2].sfid_number)
            .expect("second prc institution should map");
        let prc_b_initial = current_admins(prc_b);

        let old_admin = prc_admin(1);
        let new_admin = AccountId32::new([240u8; 32]);
        assert_ok!(propose_admin_set_replacement(
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
