use super::*;

#[test]
fn unix_seconds_to_year_uses_utc_gregorian_boundaries() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            VotingEngine::unix_seconds_to_year(1_798_761_600).expect("valid 2027 timestamp"),
            2027
        );
        assert_eq!(
            VotingEngine::unix_seconds_to_year(1_830_297_600).expect("valid 2028 timestamp"),
            2028
        );
        assert_eq!(
            VotingEngine::unix_seconds_to_year(1_861_919_999).expect("valid 2028 timestamp"),
            2028
        );
        assert_eq!(
            VotingEngine::unix_seconds_to_year(1_861_920_000).expect("valid 2029 timestamp"),
            2029
        );
        assert_eq!(
            VotingEngine::unix_seconds_to_year(1_956_528_000).expect("valid 2032 timestamp"),
            2032
        );
    });
}

#[test]
fn leap_year_rules_match_gregorian_calendar() {
    new_test_ext().execute_with(|| {
        assert!(VotingEngine::is_leap_year(2000));
        assert!(!VotingEngine::is_leap_year(2100));
        assert!(VotingEngine::is_leap_year(2400));
        assert_eq!(VotingEngine::days_in_year(2028), 366);
        assert_eq!(VotingEngine::days_in_year(2029), 365);
    });
}

#[test]
fn proposal_id_counter_resets_at_real_utc_year_boundary() {
    // 双层 ID v1:
    // - 主键 NextProposalId 全局单调累加,跨年不重置
    // - 展示号 ProposalDisplayMeta.seq_in_year 跨年自动重置回 0
    // - CurrentProposalYear 跟着系统时间切换
    new_test_ext().execute_with(|| {
        set_test_now_secs(1_830_297_599);
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());
        // 主键纯单调:首个提案 = 0
        assert_eq!(proposal_id, 0);
        // 展示号:2027 年首个,seq_in_year = 0
        let display = votingengine::pallet::ProposalDisplayId::<Test>::get(proposal_id)
            .expect("display id present");
        assert_eq!(display.year, 2027);
        assert_eq!(display.seq_in_year, 0);
        assert_eq!(CurrentProposalYear::<Test>::get(), 2027);
        assert_eq!(YearProposalCounter::<Test>::get(), 1);

        set_test_now_secs(1_830_297_600);
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(1), NRC, nrc_pid());
        // 主键单调累加:第二个提案 = 1(跨年也只 +1)
        assert_eq!(proposal_id, 1);
        // 展示号:2028 年首个,seq_in_year 跨年重置为 0
        let display = votingengine::pallet::ProposalDisplayId::<Test>::get(proposal_id)
            .expect("display id present");
        assert_eq!(display.year, 2028);
        assert_eq!(display.seq_in_year, 0);
        assert_eq!(CurrentProposalYear::<Test>::get(), 2028);
        assert_eq!(YearProposalCounter::<Test>::get(), 1);
        assert_eq!(NextProposalId::<Test>::get(), 2);
    });
}

#[test]
fn internal_proposal_must_be_created_by_same_institution_admin() {
    new_test_ext().execute_with(|| {
        // `create_internal_proposal` 仅由业务模块通过 `InternalVoteEngine` trait
        // 入口调用,这里直接验证 trait 路径的权限校验。
        let outsider = AccountId32::new([7u8; 32]);

        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_general_internal_proposal_with_data(
                outsider,
                NRC,
                nrc_pid(),
                subject_cids_for(NRC, &nrc_pid()),
                b"test",
                b"payload".to_vec(),
            ),
            votingengine::Error::<Test>::NoPermission
        );

        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_general_internal_proposal_with_data(
                prc_admin(0),
                NRC,
                nrc_pid(),
                subject_cids_for(NRC, &nrc_pid()),
                b"test",
                b"payload".to_vec(),
            ),
            votingengine::Error::<Test>::NoPermission
        );

        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());
        // 双层 ID v1:主键纯单调,首个提案 = 0;展示号通过 ProposalDisplayId 反查。
        assert_eq!(proposal_id, 0);
        let display = votingengine::pallet::ProposalDisplayId::<Test>::get(proposal_id)
            .expect("display id present");
        assert_eq!(display.year, 2026);
        assert_eq!(display.seq_in_year, 0);
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .stage,
            STAGE_INTERNAL
        );
    });
}

#[test]
fn active_internal_proposal_rejects_pending_account() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_general_internal_proposal_with_data(
                pending_account_admin(0),
                PERSONAL_CODE,
                pending_account_institution(),
                subject_cids_for(PERSONAL_CODE, &pending_account_institution()),
                b"test",
                b"payload".to_vec(),
            ),
            votingengine::Error::<Test>::InvalidInstitution
        );
    });
}

#[test]
fn governance_internal_proposal_snapshots_fixed_threshold_not_provider() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        // 测试 Provider 对治理机构故意返回 1，这里必须仍写入固定治理阈值。
        assert_eq!(
            InternalThresholdSnapshot::<Test>::get(proposal_id),
            Some(primitives::count_const::NRC_INTERNAL_THRESHOLD)
        );
    });
}

#[test]
fn pending_account_proposal_uses_pending_snapshot_and_threshold() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_pending_account_proposal_via_engine(
            pending_account_admin(0),
            PERSONAL_CODE,
            pending_account_institution(),
        );

        assert_eq!(InternalThresholdSnapshot::<Test>::get(proposal_id), Some(2));
        assert!(VotingEngine::is_admin_in_snapshot(
            proposal_id,
            pending_account_institution(),
            &pending_account_admin(0)
        ));

        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal should exist")
                .status,
            STATUS_VOTING
        );

        assert_ok!(cast_internal_vote_via_extrinsic(
            pending_account_admin(1),
            proposal_id,
            true
        ));
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal should exist")
                .status,
            STATUS_PASSED
        );
    });
}

#[test]
fn institution_account_orgs_use_dynamic_pending_snapshot_and_threshold() {
    new_test_ext().execute_with(|| {
        for institution_code in [PUBLIC_CODE, PRIVATE_CODE] {
            let proposal_id = create_pending_account_proposal_via_engine(
                pending_account_admin(0),
                institution_code,
                pending_account_institution(),
            );

            assert_eq!(InternalThresholdSnapshot::<Test>::get(proposal_id), Some(2));
            assert!(VotingEngine::is_admin_in_snapshot(
                proposal_id,
                pending_account_institution(),
                &pending_account_admin(1)
            ));
        }
    });
}

#[test]
fn pending_account_provider_threshold_requires_all_admins() {
    new_test_ext().execute_with(|| {
        set_pending_account_threshold(1);

        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_registered_account_create_proposal_with_data(
                pending_account_admin(0),
                PERSONAL_CODE,
                pending_account_institution(),
                subject_cids_for(PERSONAL_CODE, &pending_account_institution()),
                sp_std::vec![pending_account_admin(0), pending_account_admin(1)],
                1,
                b"test",
                b"payload".to_vec(),
            ),
            Error::<Test>::InvalidDynamicThreshold
        );
    });
}

#[test]
fn pending_account_snapshot_data_requires_all_admins() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_registered_account_create_proposal_with_data(
                pending_account_admin(0),
                PERSONAL_CODE,
                pending_account_institution(),
                subject_cids_for(PERSONAL_CODE, &pending_account_institution()),
                sp_std::vec![pending_account_admin(0), pending_account_admin(1)],
                1,
                b"test",
                b"payload".to_vec(),
            ),
            Error::<Test>::InvalidDynamicThreshold
        );
    });
}

#[test]
fn explicit_threshold_proposal_requires_all_snapshot_admins() {
    new_test_ext().execute_with(|| {
        let proposal_id =
            <InternalVote as InternalVoteEngine<AccountId32>>::create_lifecycle_internal_proposal_with_data(
                registered_account_admin(0),
                PERSONAL_CODE,
                registered_account_institution(),
                subject_cids_for(PERSONAL_CODE, &registered_account_institution()),
                b"close",
                b"payload".to_vec(),
            )
            .expect("lifecycle proposal should be created");
        assert_eq!(InternalThresholdSnapshot::<Test>::get(proposal_id), Some(3));
    });
}

#[test]
fn registered_account_threshold_must_not_exceed_snapshot_size() {
    new_test_ext().execute_with(|| {
        set_registered_account_threshold(4);

        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_general_internal_proposal_with_data(
                registered_account_admin(0),
                PERSONAL_CODE,
                registered_account_institution(),
                subject_cids_for(PERSONAL_CODE, &registered_account_institution()),
                b"test",
                b"payload".to_vec(),
            ),
            Error::<Test>::InvalidDynamicThreshold
        );
    });
}

#[test]
fn admin_set_mutation_threshold_must_not_exceed_snapshot_size() {
    new_test_ext().execute_with(|| {
        set_registered_account_threshold(4);

        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_admin_change_internal_proposal_with_data(
                registered_account_admin(0),
                PERSONAL_CODE,
                registered_account_institution(),
                subject_cids_for(PERSONAL_CODE, &registered_account_institution()),
                3,
                2,
                b"test",
                b"payload".to_vec(),
            ),
            Error::<Test>::InvalidDynamicThreshold
        );
    });
}

#[test]
fn snapshot_rejects_empty_admin_list() {
    new_test_ext().execute_with(|| {
        set_registered_admin_list_override(Vec::new());

        assert_noop!(
            VotingEngine::snapshot_institution_admins(
                0,
                PERSONAL_CODE,
                registered_account_institution(),
                false,
            ),
            votingengine::Error::<Test>::MissingAdminSnapshot
        );
    });
}

#[test]
fn snapshot_rejects_duplicate_admin_list() {
    new_test_ext().execute_with(|| {
        set_registered_admin_list_override(sp_std::vec![
            registered_account_admin(0),
            registered_account_admin(0),
            registered_account_admin(1),
        ]);

        assert_noop!(
            VotingEngine::snapshot_institution_admins(
                0,
                PERSONAL_CODE,
                registered_account_institution(),
                false,
            ),
            votingengine::Error::<Test>::InvalidInstitution
        );
    });
}

#[test]
fn registered_account_proposal_snapshots_dynamic_threshold() {
    new_test_ext().execute_with(|| {
        set_registered_account_threshold(3);
        let proposal_id = create_internal_proposal_via_engine(
            registered_account_admin(0),
            PERSONAL_CODE,
            registered_account_institution(),
        );

        assert_eq!(InternalThresholdSnapshot::<Test>::get(proposal_id), Some(3));
        set_registered_account_threshold(2);

        assert_ok!(cast_internal_vote_via_extrinsic(
            registered_account_admin(1),
            proposal_id,
            true
        ));
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_VOTING
        );

        assert_ok!(cast_internal_vote_via_extrinsic(
            registered_account_admin(2),
            proposal_id,
            true
        ));
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_PASSED
        );
    });
}

#[test]
fn institution_account_orgs_snapshot_dynamic_active_threshold() {
    new_test_ext().execute_with(|| {
        for institution_code in [PUBLIC_CODE, PRIVATE_CODE] {
            set_registered_account_threshold(3);
            let proposal_id = create_internal_proposal_via_engine(
                registered_account_admin(0),
                institution_code,
                registered_account_institution(),
            );

            assert_eq!(InternalThresholdSnapshot::<Test>::get(proposal_id), Some(3));
            assert!(VotingEngine::is_admin_in_snapshot(
                proposal_id,
                registered_account_institution(),
                &registered_account_admin(2)
            ));
        }
    });
}

#[test]
fn admin_set_mutation_mutex_blocks_same_subject_regular_proposal() {
    new_test_ext().execute_with(|| {
        let proposal_id =
            create_admin_set_mutation_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());
        let state =
            internal_mutex_for(NRC, nrc_pid()).expect("mutex should exist");
        assert_eq!(state.admin_set_mutation_proposal, Some(proposal_id));

        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_general_internal_proposal_with_data(
                nrc_admin(1),
                NRC,
                nrc_pid(),
                subject_cids_for(NRC, &nrc_pid()),
                b"test",
                b"payload".to_vec(),
            ),
            votingengine::Error::<Test>::AdminSetMutationProposalActive
        );
    });
}

#[test]
fn regular_mutex_blocks_same_subject_admin_set_mutation() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_internal_proposal_via_engine(
            nrc_admin(0),
            NRC,
            nrc_pid(),
        );
        let state = internal_mutex_for(NRC, nrc_pid())
            .expect("mutex should exist");
        assert_eq!(state.regular_active_count, 1);
        assert_eq!(state.admin_set_mutation_proposal, None);

        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_admin_change_internal_proposal_with_data(
                nrc_admin(1),
                NRC,
                nrc_pid(),
                subject_cids_for(NRC, &nrc_pid()),
                primitives::count_const::NRC_ADMIN_COUNT,
                primitives::count_const::NRC_INTERNAL_THRESHOLD,
                b"test",
                b"payload".to_vec(),
            ),
            votingengine::Error::<Test>::RegularInternalProposalActive
        );

        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal should exist")
                .status,
            STATUS_VOTING
        );
    });
}

#[test]
fn regular_internal_proposals_can_coexist_under_same_subject() {
    new_test_ext().execute_with(|| {
        let first = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());
        let second = create_internal_proposal_via_engine(nrc_admin(1), NRC, nrc_pid());

        assert_ne!(first, second);
        let state = internal_mutex_for(NRC, nrc_pid()).expect("mutex should exist");
        assert_eq!(state.regular_active_count, 2);
        assert_eq!(state.admin_set_mutation_proposal, None);
    });
}

#[test]
fn admin_set_mutation_passed_status_keeps_mutex_until_terminal_status() {
    new_test_ext().execute_with(|| {
        let proposal_id =
            create_admin_set_mutation_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal should exist")
                .status,
            STATUS_PASSED
        );
        assert!(internal_mutex_for(NRC, nrc_pid()).is_some());
        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_general_internal_proposal_with_data(
                nrc_admin(1),
                NRC,
                nrc_pid(),
                subject_cids_for(NRC, &nrc_pid()),
                b"test",
                b"payload".to_vec(),
            ),
            votingengine::Error::<Test>::AdminSetMutationProposalActive
        );

        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_EXECUTION_FAILED
        ));
        assert!(internal_mutex_for(NRC, nrc_pid()).is_none());
    });
}

#[test]
fn proposal_status_transition_state_machine_is_strict() {
    new_test_ext().execute_with(|| {
        let voting_to_passed = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());
        assert_noop!(
            VotingEngine::set_status_and_emit(voting_to_passed, STATUS_EXECUTED),
            votingengine::Error::<Test>::InvalidProposalStatus
        );
        assert_noop!(
            VotingEngine::set_status_and_emit(voting_to_passed, STATUS_EXECUTION_FAILED),
            votingengine::Error::<Test>::InvalidProposalStatus
        );
        assert_noop!(
            VotingEngine::set_status_and_emit(voting_to_passed, STATUS_VOTING),
            votingengine::Error::<Test>::InvalidProposalStatus
        );
        assert_ok!(VotingEngine::set_status_and_emit(
            voting_to_passed,
            STATUS_PASSED
        ));
        assert_noop!(
            VotingEngine::set_status_and_emit(voting_to_passed, STATUS_REJECTED),
            votingengine::Error::<Test>::InvalidProposalStatus
        );
        assert_noop!(
            VotingEngine::set_status_and_emit(voting_to_passed, STATUS_VOTING),
            votingengine::Error::<Test>::InvalidProposalStatus
        );
        assert_ok!(VotingEngine::set_status_and_emit(
            voting_to_passed,
            STATUS_EXECUTED
        ));
        assert_noop!(
            VotingEngine::set_status_and_emit(voting_to_passed, STATUS_PASSED),
            votingengine::Error::<Test>::InvalidProposalStatus
        );

        let passed_to_failed = create_internal_proposal_via_engine(nrc_admin(1), NRC, nrc_pid());
        assert_ok!(VotingEngine::set_status_and_emit(
            passed_to_failed,
            STATUS_PASSED
        ));
        assert_ok!(VotingEngine::set_status_and_emit(
            passed_to_failed,
            STATUS_EXECUTION_FAILED
        ));
        assert_noop!(
            VotingEngine::set_status_and_emit(passed_to_failed, STATUS_EXECUTED),
            votingengine::Error::<Test>::InvalidProposalStatus
        );

        let rejected = create_internal_proposal_via_engine(nrc_admin(2), NRC, nrc_pid());
        assert_ok!(VotingEngine::set_status_and_emit(rejected, STATUS_REJECTED));
        assert_noop!(
            VotingEngine::set_status_and_emit(rejected, STATUS_PASSED),
            votingengine::Error::<Test>::InvalidProposalStatus
        );
    });
}

#[test]
fn internal_vote_must_be_by_same_institution_admin() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_internal_proposal_via_engine(prb_admin(0), PRB, prb_pid());

        assert_noop!(
            cast_internal_vote_via_extrinsic(nrc_admin(0), proposal_id, true),
            votingengine::Error::<Test>::NoPermission
        );

        assert_ok!(cast_internal_vote_via_extrinsic(
            prb_admin(1),
            proposal_id,
            true
        ));
    });
}

#[test]
fn nrc_internal_vote_passes_at_13_yes_votes() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        for i in 1..12 {
            assert_ok!(cast_internal_vote_via_extrinsic(
                nrc_admin(i),
                proposal_id,
                true
            ));
        }
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_VOTING
        );

        assert_ok!(cast_internal_vote_via_extrinsic(
            nrc_admin(12),
            proposal_id,
            true
        ));
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_PASSED
        );
    });
}

#[test]
fn internal_vote_is_rejected_after_timeout() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_internal_proposal_via_engine(prc_admin(0), PRC, prc_pid());

        let proposal = VotingEngine::proposals(proposal_id).expect("proposal exists");
        System::set_block_number(proposal.end + 1);

        assert_ok!(VotingEngine::finalize_proposal(
            RuntimeOrigin::signed(prc_admin(0)),
            proposal_id,
        ));
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_REJECTED
        );
    });
}

#[test]
fn internal_vote_timeout_is_auto_rejected_on_initialize() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_internal_proposal_via_engine(prc_admin(0), PRC, prc_pid());

        let proposal = VotingEngine::proposals(proposal_id).expect("proposal exists");
        System::set_block_number(proposal.end);
        <VotingEngine as Hooks<u64>>::on_initialize(proposal.end);
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal should exist")
                .status,
            STATUS_VOTING
        );

        let next = proposal.end + 1;
        System::set_block_number(next);
        <VotingEngine as Hooks<u64>>::on_initialize(next);
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal should exist")
                .status,
            STATUS_REJECTED
        );
    });
}

#[test]
fn joint_proposal_must_be_created_by_nrc_or_prc_admin() {
    new_test_ext().execute_with(|| {
        // `create_joint_proposal` 仅由业务模块通过 `JointVoteEngine` trait
        // 入口调用,这里直接验证 trait 路径的权限校验。

        // 外部人员不能创建联合提案
        let outsider = AccountId32::new([9u8; 32]);
        assert_noop!(
            JointVote::prepare_joint_population_snapshot(
                RuntimeOrigin::signed(outsider),
                votingengine::PopulationScope::Country,
            ),
            votingengine::Error::<Test>::NoPermission
        );

        // 省储会管理员可以创建联合提案
        prepare_population_snapshot_for(prc_admin(0), 10);
        assert_ok!(
            <JointVote as JointVoteEngine<AccountId32>>::create_joint_proposal(prc_admin(0))
        );

        // 国储会管理员可以创建联合提案
        prepare_population_snapshot_for(nrc_admin(0), 10);
        assert_ok!(
            <JointVote as JointVoteEngine<AccountId32>>::create_joint_proposal(nrc_admin(0))
        );
    });
}

#[test]
fn joint_proposal_requires_prepared_population_snapshot() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            <JointVote as JointVoteEngine<AccountId32>>::create_joint_proposal(nrc_admin(0)),
            joint_vote::Error::<Test>::PopulationSnapshotNotPrepared
        );
    });
}

#[test]
fn joint_proposal_rejects_stale_population_snapshot() {
    new_test_ext().execute_with(|| {
        prepare_population_snapshot_for(nrc_admin(0), 10);

        // 人口快照只代表准备快照所在区块的公民分母；
        // 隔块创建提案必须拒绝并删除过期缓存。
        System::set_block_number(2);
        assert_eq!(
            <JointVote as JointVoteEngine<AccountId32>>::create_joint_proposal(nrc_admin(0)),
            Err(joint_vote::Error::<Test>::PopulationSnapshotNotCurrent.into())
        );
        assert!(
            !joint_vote::pallet::PendingPopulationSnapshots::<Test>::contains_key(nrc_admin(0))
        );
    });
}

#[test]
fn joint_vote_requires_current_institution_admin() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 10);

        assert_ok!(submit_joint_vote(
            nrc_admin(0),
            proposal_id,
            nrc_pid(),
            true
        ));

        assert_ok!(submit_joint_vote(
            prc_admin(0),
            proposal_id,
            prc_pid(),
            true
        ));

        assert_noop!(
            submit_joint_vote(prc_admin(0), proposal_id, nrc_pid(), true),
            votingengine::Error::<Test>::NoPermission
        );
    });
}

#[test]
fn joint_vote_rejects_duplicate_admin_vote() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 10);

        assert_ok!(submit_joint_vote(
            nrc_admin(0),
            proposal_id,
            nrc_pid(),
            true
        ));

        assert_noop!(
            submit_joint_vote(nrc_admin(0), proposal_id, nrc_pid(), true),
            votingengine::Error::<Test>::AlreadyVoted
        );
    });
}

#[test]
fn joint_vote_uses_fixed_governance_threshold_not_provider() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 10);

        // 测试 Provider 对治理机构故意返回 1；联合投票必须等固定阈值票数才形成机构结果。
        assert_ok!(submit_joint_vote(
            nrc_admin(0),
            proposal_id,
            nrc_pid(),
            true
        ));
        assert_eq!(
            joint_vote::JointVotesByInstitution::<Test>::get(proposal_id, nrc_pid()),
            None
        );

        for i in 1..primitives::count_const::NRC_INTERNAL_THRESHOLD as usize {
            assert_ok!(submit_joint_vote(
                nrc_admin(i),
                proposal_id,
                nrc_pid(),
                true
            ));
        }
        assert_eq!(
            joint_vote::JointVotesByInstitution::<Test>::get(proposal_id, nrc_pid()),
            Some(true)
        );
    });
}

#[test]
fn national_judicial_yuan_uses_fixed_internal_threshold() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_internal_proposal_via_engine(njd_admin(0), NJD, njd_pid());
        assert_eq!(
            InternalThresholdSnapshot::<Test>::get(proposal_id),
            Some(primitives::count_const::NJD_INTERNAL_THRESHOLD)
        );
    });
}

#[test]
fn joint_vote_auto_rejects_institution_when_yes_is_no_longer_reachable() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 10);

        cast_joint_votes_until_finalized(proposal_id, nrc_pid(), false);

        assert_eq!(
            joint_vote::JointVotesByInstitution::<Test>::get(proposal_id, nrc_pid()),
            Some(false)
        );
        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        assert_eq!(proposal.stage, STAGE_REFERENDUM);
        assert_eq!(
            joint_vote::JointTallies::<Test>::get(proposal_id).no,
            primitives::count_const::NRC_JOINT_VOTE_WEIGHT
        );
    });
}

#[test]
fn joint_stage_mutex_blocks_admin_set_mutation_until_citizen_stage() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 10);

        assert!(
            internal_mutex_for(NRC, nrc_pid()).is_some()
        );
        assert!(
            internal_mutex_for(PRC, prc_pid()).is_some()
        );
        assert_noop!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_admin_change_internal_proposal_with_data(
                nrc_admin(1),
                NRC,
                nrc_pid(),
                subject_cids_for(NRC, &nrc_pid()),
                primitives::count_const::NRC_ADMIN_COUNT,
                primitives::count_const::NRC_INTERNAL_THRESHOLD,
                b"test",
                b"payload".to_vec(),
            ),
            votingengine::Error::<Test>::RegularInternalProposalActive
        );

        cast_joint_votes_until_finalized(proposal_id, nrc_pid(), false);
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal should exist")
                .stage,
            STAGE_REFERENDUM
        );
        assert!(
            internal_mutex_for(NRC, nrc_pid()).is_none()
        );
        assert!(
            internal_mutex_for(PRC, prc_pid()).is_none()
        );

        assert_ok!(
            <InternalVote as InternalVoteEngine<AccountId32>>::create_admin_change_internal_proposal_with_data(
                nrc_admin(1),
                NRC,
                nrc_pid(),
                subject_cids_for(NRC, &nrc_pid()),
                primitives::count_const::NRC_ADMIN_COUNT,
                primitives::count_const::NRC_INTERNAL_THRESHOLD,
                b"test",
                b"payload".to_vec(),
            )
        );
    });
}

#[test]
fn population_snapshot_can_be_prepared_for_each_joint_proposal() {
    new_test_ext().execute_with(|| {
        prepare_population_snapshot_for(nrc_admin(0), 10);
        assert_ok!(
            <JointVote as JointVoteEngine<AccountId32>>::create_joint_proposal(nrc_admin(0))
        );

        TEST_POPULATION_COUNT.with(|count| *count.borrow_mut() = 11);
        assert_ok!(JointVote::prepare_joint_population_snapshot(
            RuntimeOrigin::signed(nrc_admin(0)),
            votingengine::PopulationScope::Country,
        ));
    });
}

#[test]
fn citizen_vote_allows_eligible_account() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 100);

        assert_ok!(<joint_vote::Pallet<Test>>::do_jointreferendum_vote(
            nrc_admin(0),
            0,
            true
        ));
        assert_eq!(joint_vote::ReferendumTallies::<Test>::get(0).yes, 1);
        assert!(joint_vote::ReferendumVotesByAccount::<Test>::contains_key(
            0,
            nrc_admin(0)
        ));
    });
}

#[test]
fn citizen_vote_same_account_can_only_vote_once_per_proposal() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 100);

        assert_ok!(<joint_vote::Pallet<Test>>::do_jointreferendum_vote(
            nrc_admin(0),
            0,
            true
        ));

        assert_noop!(
            <joint_vote::Pallet<Test>>::do_jointreferendum_vote(nrc_admin(0), 0, false),
            votingengine::Error::<Test>::AlreadyVoted
        );
    });
}

#[test]
fn citizen_vote_same_account_can_vote_on_different_proposals() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 100);
        insert_citizen_proposal(1, 10, 100);

        assert_ok!(<joint_vote::Pallet<Test>>::do_jointreferendum_vote(
            nrc_admin(0),
            0,
            true
        ));
        assert_ok!(<joint_vote::Pallet<Test>>::do_jointreferendum_vote(
            nrc_admin(0),
            1,
            true
        ));
    });
}

#[test]
fn citizen_vote_rejects_when_eligible_total_not_set_in_proposal() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 0, 100);

        assert_noop!(
            <joint_vote::Pallet<Test>>::do_jointreferendum_vote(nrc_admin(0), 0, true),
            joint_vote::Error::<Test>::CitizenEligibleTotalNotSet
        );
    });
}

#[test]
fn citizen_timeout_with_half_or_less_is_rejected() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 5);
        joint_vote::ReferendumTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });
        System::set_block_number(6);

        assert_ok!(VotingEngine::finalize_proposal(
            RuntimeOrigin::signed(nrc_admin(0)),
            0
        ));
        assert_eq!(
            Proposals::<Test>::get(0)
                .expect("proposal should exist")
                .status,
            STATUS_REJECTED
        );
    });
}

#[test]
fn citizen_timeout_is_auto_rejected_on_initialize() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 5);
        assert_ok!(VotingEngine::schedule_proposal_expiry(0, 5));
        joint_vote::ReferendumTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });

        System::set_block_number(6);
        <VotingEngine as Hooks<u64>>::on_initialize(6);
        assert_eq!(
            Proposals::<Test>::get(0)
                .expect("proposal should exist")
                .status,
            STATUS_REJECTED
        );
    });
}

#[test]
fn citizen_timeout_auto_registers_cleanup_and_clears_referendum_votes() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 5);
        assert_ok!(VotingEngine::schedule_proposal_expiry(0, 5));

        assert_ok!(<joint_vote::Pallet<Test>>::do_jointreferendum_vote(
            nrc_admin(0),
            0,
            true
        ));
        assert!(joint_vote::ReferendumVotesByAccount::<Test>::contains_key(
            0,
            nrc_admin(0)
        ));

        System::set_block_number(6);
        <VotingEngine as Hooks<u64>>::on_initialize(6);

        assert_eq!(
            Proposals::<Test>::get(0)
                .expect("proposal should exist")
                .status,
            STATUS_REJECTED
        );
        assert!(joint_vote::ReferendumVotesByAccount::<Test>::contains_key(
            0,
            nrc_admin(0)
        ));

        let retention = 90u64 * primitives::pow_const::BLOCKS_PER_DAY;
        let cleanup_block = 6 + retention;
        for i in 0..20u64 {
            System::set_block_number(cleanup_block + i);
            <VotingEngine as Hooks<u64>>::on_initialize(cleanup_block + i);
        }
        assert!(!joint_vote::ReferendumVotesByAccount::<Test>::contains_key(
            0,
            nrc_admin(0)
        ));
    });
}

#[test]
fn citizen_vote_rejects_ineligible_account() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 100);
        let outsider = AccountId32::new([7u8; 32]);

        assert_noop!(
            <joint_vote::Pallet<Test>>::do_jointreferendum_vote(outsider, 0, true),
            joint_vote::Error::<Test>::CitizenNotEligible
        );
    });
}

#[test]
fn citizen_vote_rejects_when_not_in_citizen_stage() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 10);

        assert_noop!(
            <joint_vote::Pallet<Test>>::do_jointreferendum_vote(nrc_admin(0), proposal_id, true),
            votingengine::Error::<Test>::InvalidProposalStage
        );
    });
}

#[test]
fn citizen_vote_passes_immediately_when_yes_exceeds_half() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 100);
        joint_vote::ReferendumTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });

        assert_ok!(<joint_vote::Pallet<Test>>::do_jointreferendum_vote(
            nrc_admin(0),
            0,
            true
        ));

        let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
        assert_eq!(proposal.status, STATUS_EXECUTED);
    });
}

#[test]
fn delayed_cleanup_cleans_referendum_votes_after_retention() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 100);
        joint_vote::ReferendumTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });

        assert_ok!(<joint_vote::Pallet<Test>>::do_jointreferendum_vote(
            nrc_admin(0),
            0,
            true
        ));

        let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
        assert_eq!(proposal.status, STATUS_EXECUTED);
        assert!(joint_vote::ReferendumVotesByAccount::<Test>::contains_key(
            0,
            nrc_admin(0)
        ));

        let retention = 90u64 * primitives::pow_const::BLOCKS_PER_DAY;
        let cleanup_block = retention;
        for i in 0..20u64 {
            System::set_block_number(cleanup_block + i);
            <VotingEngine as Hooks<u64>>::on_initialize(cleanup_block + i);
        }
        assert!(!joint_vote::ReferendumVotesByAccount::<Test>::contains_key(
            0,
            nrc_admin(0)
        ));
    });
}

#[test]
fn citizen_finalize_before_timeout_is_rejected() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 100);
        System::set_block_number(100);

        assert_noop!(
            VotingEngine::finalize_proposal(RuntimeOrigin::signed(nrc_admin(0)), 0),
            votingengine::Error::<Test>::VoteNotExpired
        );
    });
}

#[test]
fn citizen_pass_threshold_function_boundaries_are_correct() {
    assert!(!joint_vote::is_jointreferendum_vote_passed(0, 0));
    assert!(!joint_vote::is_jointreferendum_vote_passed(5, 10));
    assert!(joint_vote::is_jointreferendum_vote_passed(6, 10));

    // 极端大数不能因为 u64 乘法饱和把刚过半误判为未通过。
    let eligible = u64::MAX;
    assert!(!joint_vote::is_jointreferendum_vote_passed(
        eligible / 2,
        eligible
    ));
    assert!(joint_vote::is_jointreferendum_vote_passed(
        eligible / 2 + 1,
        eligible
    ));
}

#[test]
fn citizen_reject_threshold_function_boundaries_are_correct() {
    // eligible_total=0 → 不否决（无意义）
    assert!(!joint_vote::is_jointreferendum_vote_rejected(0, 0));
    // 反对 4/10 = 40% < 50% → 不否决（赞成仍有可能 > 50%）
    assert!(!joint_vote::is_jointreferendum_vote_rejected(4, 10));
    // 反对 5/10 = 50% → 否决（赞成最多 50%，无法严格 > 50%）
    assert!(joint_vote::is_jointreferendum_vote_rejected(5, 10));
    // 反对 6/10 = 60% → 否决
    assert!(joint_vote::is_jointreferendum_vote_rejected(6, 10));

    // 极端大数不能因为 u64 乘法饱和把刚过半误判为未否决。
    let eligible = u64::MAX;
    assert!(!joint_vote::is_jointreferendum_vote_rejected(
        eligible / 2,
        eligible
    ));
    assert!(joint_vote::is_jointreferendum_vote_rejected(
        eligible / 2 + 1,
        eligible
    ));
}

#[test]
fn joint_vote_all_yes_passes_immediately() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 100);

        cast_joint_votes_until_finalized(proposal_id, nrc_pid(), true);

        for (institution, _) in all_prc_institutions() {
            cast_joint_votes_until_finalized(proposal_id, institution, true);
        }
        for (institution, _) in all_prb_institutions() {
            cast_joint_votes_until_finalized(proposal_id, institution, true);
        }

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        assert_eq!(proposal.status, STATUS_EXECUTED);
        assert_eq!(proposal.stage, STAGE_JOINT);
        assert_eq!(
            joint_vote::JointTallies::<Test>::get(proposal_id).yes,
            primitives::count_const::JOINT_VOTE_TOTAL
        );
    });
}

#[test]
fn joint_vote_non_unanimous_moves_to_citizen_immediately_after_one_institution_rejects() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 77);
        cast_joint_votes_until_finalized(proposal_id, nrc_pid(), true);
        let first_prc = all_prc_institutions()
            .first()
            .cloned()
            .expect("there should be at least one prc institution");
        cast_joint_votes_until_finalized(proposal_id, first_prc.0, false);

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        assert_eq!(proposal.stage, STAGE_REFERENDUM);
        assert_eq!(proposal.status, STATUS_VOTING);
        assert_eq!(proposal.start, System::block_number());
        assert_eq!(
            proposal.end,
            proposal.start + primitives::count_const::VOTING_DURATION_BLOCKS as u64
        );
        assert_eq!(proposal.citizen_eligible_total, 77);
        assert_eq!(joint_vote::JointTallies::<Test>::get(proposal_id).no, 1);
    });
}

#[test]
fn joint_vote_timeout_moves_to_citizen_when_not_unanimous() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 88);

        assert_ok!(submit_joint_vote(
            nrc_admin(0),
            proposal_id,
            nrc_pid(),
            true
        ));

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        System::set_block_number(proposal.end + 1);
        assert_ok!(VotingEngine::finalize_proposal(
            RuntimeOrigin::signed(nrc_admin(0)),
            proposal_id
        ));

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        assert_eq!(proposal.stage, STAGE_REFERENDUM);
        assert_eq!(proposal.status, STATUS_VOTING);
        assert_eq!(
            proposal.end,
            (proposal.start + primitives::count_const::VOTING_DURATION_BLOCKS as u64)
        );
    });
}

#[test]
fn joint_vote_timeout_auto_moves_to_citizen_on_initialize() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 88);

        assert_ok!(submit_joint_vote(
            nrc_admin(0),
            proposal_id,
            nrc_pid(),
            true
        ));

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        let expired_at = proposal.end + 1;
        System::set_block_number(expired_at);
        <VotingEngine as Hooks<u64>>::on_initialize(expired_at);

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        assert_eq!(proposal.stage, STAGE_REFERENDUM);
        assert_eq!(proposal.status, STATUS_VOTING);
        assert_eq!(proposal.start, expired_at);
        assert_eq!(
            proposal.end,
            expired_at + primitives::count_const::VOTING_DURATION_BLOCKS as u64
        );
    });
}

#[test]
fn joint_vote_timeout_with_unanimous_tally_passes() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 66);
        joint_vote::JointTallies::<Test>::insert(
            proposal_id,
            VoteCountU32 {
                yes: primitives::count_const::JOINT_VOTE_TOTAL,
                no: 0,
            },
        );

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        System::set_block_number(proposal.end + 1);
        assert_ok!(VotingEngine::finalize_proposal(
            RuntimeOrigin::signed(nrc_admin(0)),
            proposal_id
        ));

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        assert_eq!(proposal.status, STATUS_EXECUTED);
        assert_eq!(proposal.stage, STAGE_JOINT);
    });
}

#[test]
fn joint_vote_callback_failure_rolls_back_final_status() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 100);

        set_joint_callback_should_fail(true);
        assert!(VotingEngine::set_status_and_emit(proposal_id, STATUS_PASSED).is_err());

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        assert_eq!(proposal.status, STATUS_VOTING);
        assert_eq!(proposal.stage, STAGE_JOINT);
    });
}

#[test]
fn joint_vote_callback_failure_does_not_cleanup_referendum_votes() {
    new_test_ext().execute_with(|| {
        insert_citizen_proposal(0, 10, 100);
        joint_vote::ReferendumVotesByAccount::<Test>::insert(0, nrc_admin(0), true);
        set_joint_callback_should_fail(true);

        assert!(VotingEngine::set_status_and_emit(0, STATUS_PASSED).is_err());
        assert_eq!(
            Proposals::<Test>::get(0)
                .expect("proposal should exist")
                .status,
            STATUS_VOTING
        );
        assert!(joint_vote::ReferendumVotesByAccount::<Test>::contains_key(
            0,
            nrc_admin(0)
        ));
    });
}

#[test]
fn proposal_finalized_event_uses_status_after_joint_callback_override() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 100);

        set_joint_callback_override_status(Some(STATUS_EXECUTION_FAILED));
        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        assert_eq!(proposal.status, STATUS_EXECUTION_FAILED);

        let finalized = System::events()
            .into_iter()
            .rev()
            .find_map(|record| match record.event {
                RuntimeEvent::VotingEngine(votingengine::Event::ProposalFinalized {
                    proposal_id: event_id,
                    status,
                }) if event_id == proposal_id => Some(status),
                _ => None,
            })
            .expect("proposal finalized event should exist");
        assert_eq!(finalized, STATUS_EXECUTION_FAILED);
        let finalized_count = System::events()
            .into_iter()
            .filter(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::VotingEngine(votingengine::Event::ProposalFinalized {
                        proposal_id: event_id,
                        ..
                    }) if *event_id == proposal_id
                )
            })
            .count();
        assert_eq!(finalized_count, 1);
    });
}

#[test]
fn auto_finalize_requeues_failed_joint_callback() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 66);

        joint_vote::JointTallies::<Test>::insert(
            proposal_id,
            VoteCountU32 {
                yes: primitives::count_const::JOINT_VOTE_TOTAL,
                no: 0,
            },
        );

        let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
        let expired_at = proposal.end + 1;

        set_joint_callback_should_fail(true);
        System::set_block_number(expired_at);
        <VotingEngine as Hooks<u64>>::on_initialize(expired_at);

        assert_eq!(
            Proposals::<Test>::get(proposal_id)
                .expect("proposal should exist")
                .status,
            STATUS_VOTING
        );
        assert_eq!(PendingExpiryBucket::<Test>::get(), Some(expired_at));
        assert_eq!(
            ProposalsByExpiry::<Test>::get(expired_at),
            vec![proposal_id]
        );

        set_joint_callback_should_fail(false);
        let next_block = expired_at + 1;
        System::set_block_number(next_block);
        <VotingEngine as Hooks<u64>>::on_initialize(next_block);

        assert_eq!(
            Proposals::<Test>::get(proposal_id)
                .expect("proposal should exist")
                .status,
            STATUS_EXECUTED
        );
        assert!(PendingExpiryBucket::<Test>::get().is_none());
        assert!(ProposalsByExpiry::<Test>::get(expired_at).is_empty());
    });
}

#[test]
fn auto_finalize_uses_pending_cursor_when_expiry_bucket_exceeds_per_block_limit() {
    new_test_ext().execute_with(|| {
        let end = 5u64;
        let expiry = end + 1;
        let total = 70u64;
        for proposal_id in 0..total {
            insert_citizen_proposal(proposal_id, 10, end);
            assert_ok!(VotingEngine::schedule_proposal_expiry(proposal_id, end));
        }

        System::set_block_number(6);
        <VotingEngine as Hooks<u64>>::on_initialize(6);
        assert_eq!(ProposalsByExpiry::<Test>::get(expiry).len(), 6);
        assert_eq!(PendingExpiryBucket::<Test>::get(), Some(expiry));

        System::set_block_number(7);
        <VotingEngine as Hooks<u64>>::on_initialize(7);
        assert!(ProposalsByExpiry::<Test>::get(expiry).is_empty());
        assert!(PendingExpiryBucket::<Test>::get().is_none());
        for proposal_id in 0..total {
            assert_eq!(
                Proposals::<Test>::get(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
        }
    });
}

#[test]
fn schedule_proposal_expiry_rejects_bucket_overflow() {
    new_test_ext().execute_with(|| {
        let end = 5u64;
        for proposal_id in 0..128u64 {
            assert_ok!(VotingEngine::schedule_proposal_expiry(proposal_id, end));
        }

        assert_noop!(
            VotingEngine::schedule_proposal_expiry(999, end),
            votingengine::Error::<Test>::TooManyProposalsAtExpiry
        );
    });
}

#[test]
fn cleanup_queue_processes_full_due_bucket_without_orphaning_remaining_items() {
    new_test_ext().execute_with(|| {
        let cleanup_block = 77u64;
        let ids: BoundedVec<u64, ConstU32<50>> = (0..50u64)
            .collect::<Vec<_>>()
            .try_into()
            .expect("cleanup queue should fit");
        for proposal_id in ids.iter().copied() {
            insert_citizen_proposal(proposal_id, 10, 100);
        }
        CleanupQueue::<Test>::insert(cleanup_block, ids);

        System::set_block_number(cleanup_block);
        <VotingEngine as Hooks<u64>>::on_initialize(cleanup_block);

        assert!(CleanupQueue::<Test>::get(cleanup_block).is_empty());
        for proposal_id in 0..50u64 {
            assert_eq!(
                PendingProposalCleanups::<Test>::get(proposal_id),
                Some(PendingCleanupStage::AdminSnapshots)
            );
        }
    });
}

#[test]
fn schedule_cleanup_returns_error_when_all_candidate_buckets_are_full() {
    new_test_ext().execute_with(|| {
        let now = System::block_number();
        fill_cleanup_schedule_window(now);

        assert_noop!(
            votingengine::cleanup::schedule_cleanup::<Test>(9_999, now),
            votingengine::Error::<Test>::CleanupQueueFull
        );
    });
}

#[test]
fn terminal_status_rolls_back_when_cleanup_cannot_be_scheduled() {
    new_test_ext().execute_with(|| {
        let proposal_id = 9_999u64;
        let now = System::block_number();
        insert_citizen_proposal(proposal_id, 10, now + 100);
        fill_cleanup_schedule_window(now);

        assert_noop!(
            VotingEngine::set_status_and_emit(proposal_id, STATUS_REJECTED),
            votingengine::Error::<Test>::CleanupQueueFull
        );
        assert_eq!(
            Proposals::<Test>::get(proposal_id)
                .expect("proposal should remain")
                .status,
            STATUS_VOTING
        );
        assert!(PendingProposalCleanups::<Test>::get(proposal_id).is_none());
    });
}

#[test]
fn retry_deadline_keeps_retry_state_when_cleanup_scheduling_fails() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));
        let deadline = ProposalExecutionRetryStates::<Test>::get(proposal_id)
            .expect("retry state should exist")
            .retry_deadline;
        fill_cleanup_schedule_window(deadline);

        System::set_block_number(deadline);
        <VotingEngine as Hooks<u64>>::on_initialize(deadline);

        assert_eq!(
            Proposals::<Test>::get(proposal_id)
                .expect("proposal should remain")
                .status,
            STATUS_PASSED
        );
        assert!(ProposalExecutionRetryStates::<Test>::get(proposal_id).is_some());
        assert!(
            ExecutionRetryDeadlines::<Test>::get(deadline + 1).contains(&proposal_id),
            "cleanup scheduling failure must requeue retry deadline instead of losing it"
        );
    });
}

#[test]
fn retry_deadline_enters_pending_queue_when_reschedule_window_is_full() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));
        let deadline = ProposalExecutionRetryStates::<Test>::get(proposal_id)
            .expect("retry state should exist")
            .retry_deadline;
        fill_cleanup_schedule_window(deadline);
        fill_retry_deadline_window(deadline + 1);

        System::set_block_number(deadline);
        <VotingEngine as Hooks<u64>>::on_initialize(deadline);

        assert!(ProposalExecutionRetryStates::<Test>::get(proposal_id).is_some());
        assert_eq!(
            PendingExecutionRetryExpirations::<Test>::get(proposal_id),
            Some(deadline)
        );
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal should remain")
                .status,
            STATUS_PASSED
        );

        clear_cleanup_schedule_window(deadline + 1);
        clear_retry_deadline_window(deadline + 1);
        System::set_block_number(deadline + 1);
        <VotingEngine as Hooks<u64>>::on_initialize(deadline + 1);

        assert!(PendingExecutionRetryExpirations::<Test>::get(proposal_id).is_none());
        assert!(ProposalExecutionRetryStates::<Test>::get(proposal_id).is_none());
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal should exist")
                .status,
            STATUS_EXECUTION_FAILED
        );
    });
}

#[test]
fn delayed_cleanup_chunks_cleanup_across_blocks() {
    new_test_ext().execute_with(|| {
        let proposal_id = 42u64;
        let citizen_accounts = [
            AccountId32::new([201u8; 32]),
            AccountId32::new([202u8; 32]),
            AccountId32::new([203u8; 32]),
        ];

        insert_citizen_proposal(proposal_id, 10, 100);
        joint_vote::JointVotesByInstitution::<Test>::insert(proposal_id, nrc_pid(), true);
        joint_vote::JointVotesByInstitution::<Test>::insert(proposal_id, prc_pid(), true);
        joint_vote::JointVotesByInstitution::<Test>::insert(proposal_id, prb_pid(), true);
        for account in citizen_accounts.iter() {
            joint_vote::ReferendumVotesByAccount::<Test>::insert(proposal_id, account, true);
        }

        // 投票通过后由 callback 返回 Executed，终态会注册 90 天后清理。
        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));
        // 此时 PendingProposalCleanups 尚未设置（要等 90 天后 process_cleanup_queue 触发）
        assert!(PendingProposalCleanups::<Test>::get(proposal_id).is_none());

        // set_status_and_emit 在 block 0 调用，cleanup_at = 0 + retention
        let retention = 90u64 * primitives::pow_const::BLOCKS_PER_DAY;
        let cleanup_block = retention;
        // 运行多轮 on_initialize 直到清理完成
        for i in 0..20u64 {
            System::set_block_number(cleanup_block + i);
            <VotingEngine as Hooks<u64>>::on_initialize(cleanup_block + i);
            if PendingProposalCleanups::<Test>::get(proposal_id).is_none()
                && Proposals::<Test>::get(proposal_id).is_none()
            {
                break;
            }
        }

        assert!(PendingProposalCleanups::<Test>::get(proposal_id).is_none());
        for account in citizen_accounts.iter() {
            assert!(!joint_vote::ReferendumVotesByAccount::<Test>::contains_key(
                proposal_id,
                account
            ));
        }
    });
}

// ──── 公开 internal_vote extrinsic + InternalVoteResultCallback ────

/// 重置内部投票回调测试桩状态,避免用例间污染。
fn reset_internal_callback_state() {
    INTERNAL_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = false);
    INTERNAL_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = None);
    INTERNAL_CALLBACK_LOG.with(|log| log.borrow_mut().clear());
    INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow_mut().clear());
    set_internal_terminal_cleanup_should_fail(false);
}

#[test]
fn internal_vote_public_call_casts_vote() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        assert_ok!(cast_internal_vote_via_extrinsic(
            nrc_admin(1),
            proposal_id,
            true
        ));

        assert!(InternalVotesByAccount::<Test>::contains_key(
            proposal_id,
            &nrc_admin(0)
        ));
        assert_eq!(InternalTallies::<Test>::get(proposal_id).yes, 2);
        assert_eq!(InternalTallies::<Test>::get(proposal_id).no, 0);
    });
}

#[test]
fn internal_vote_rejects_non_admin() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        // 非 NRC 管理员(比如 PRB 的管理员)不能投 NRC 的内部提案。
        assert_noop!(
            cast_internal_vote_via_extrinsic(prb_admin(0), proposal_id, true),
            votingengine::Error::<Test>::NoPermission
        );
        assert_eq!(InternalTallies::<Test>::get(proposal_id).yes, 1);
    });
}

#[test]
fn internal_vote_rejects_double_vote() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());
        assert_noop!(
            cast_internal_vote_via_extrinsic(nrc_admin(0), proposal_id, false),
            votingengine::Error::<Test>::AlreadyVoted
        );
    });
}

#[test]
fn internal_vote_passes_triggers_callback_approved_true() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        // NRC 阈值 13 票;投 13 票赞成使提案进入 STATUS_PASSED。
        for i in 1..13 {
            assert_ok!(cast_internal_vote_via_extrinsic(
                nrc_admin(i),
                proposal_id,
                true
            ));
        }

        // 回调被触发且 approved = true。
        let log = INTERNAL_CALLBACK_LOG.with(|log| log.borrow().clone());
        assert_eq!(log, vec![(proposal_id, true)]);
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_PASSED
        );
    });
}

#[test]
fn internal_vote_early_rejection_triggers_callback_approved_false() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        // NRC 总管理员 19 人,阈值 13 票。7 票反对 → 剩余 12 人全同意也到不了 13,
        // 触发提前否决。
        for i in 1..8 {
            assert_ok!(cast_internal_vote_via_extrinsic(
                nrc_admin(i),
                proposal_id,
                false
            ));
        }

        // 回调被触发且 approved = false。
        let log = INTERNAL_CALLBACK_LOG.with(|log| log.borrow().clone());
        assert_eq!(log, vec![(proposal_id, false)]);
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_REJECTED
        );
    });
}

#[test]
fn internal_vote_callback_not_called_before_threshold() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        // 投 12 票赞成(阈值 13),未达阈值不应触发回调。
        for i in 1..12 {
            assert_ok!(cast_internal_vote_via_extrinsic(
                nrc_admin(i),
                proposal_id,
                true
            ));
        }

        let log = INTERNAL_CALLBACK_LOG.with(|log| log.borrow().clone());
        assert!(log.is_empty(), "未达阈值回调不应被调用: {:?}", log);
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_VOTING
        );
    });
}

#[test]
fn internal_vote_callback_err_rolls_back_status() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        // 前 12 票赞成(未达阈值,不触发回调,不受 SHOULD_FAIL 影响)。
        for i in 1..12 {
            assert_ok!(cast_internal_vote_via_extrinsic(
                nrc_admin(i),
                proposal_id,
                true
            ));
        }

        // 第 13 票达阈值,回调会被触发;置 SHOULD_FAIL 让回调返回 Err。
        INTERNAL_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = true);
        assert!(cast_internal_vote_via_extrinsic(nrc_admin(12), proposal_id, true).is_err());

        // 提案状态、票数必须整体回滚到投票中 + 12 票。
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_VOTING
        );
        assert_eq!(InternalTallies::<Test>::get(proposal_id).yes, 12);
        assert!(!InternalVotesByAccount::<Test>::contains_key(
            proposal_id,
            &nrc_admin(12)
        ));
    });
}

#[test]
fn manual_retry_third_failure_marks_execution_failed() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));
        assert_eq!(
            ProposalExecutionRetryStates::<Test>::get(proposal_id)
                .expect("retry state should exist")
                .manual_attempts,
            0
        );

        assert_ok!(VotingEngine::retry_passed_proposal(
            RuntimeOrigin::signed(nrc_admin(0)),
            proposal_id
        ));
        assert_eq!(
            ProposalExecutionRetryStates::<Test>::get(proposal_id)
                .expect("retry state should remain")
                .manual_attempts,
            1
        );
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_PASSED
        );

        assert_ok!(VotingEngine::retry_passed_proposal(
            RuntimeOrigin::signed(nrc_admin(1)),
            proposal_id
        ));
        assert_eq!(
            ProposalExecutionRetryStates::<Test>::get(proposal_id)
                .expect("retry state should remain")
                .manual_attempts,
            2
        );

        assert_ok!(VotingEngine::retry_passed_proposal(
            RuntimeOrigin::signed(nrc_admin(2)),
            proposal_id
        ));
        assert!(ProposalExecutionRetryStates::<Test>::get(proposal_id).is_none());
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_EXECUTION_FAILED
        );

        let third_retry_outcome = System::events()
            .into_iter()
            .rev()
            .find_map(|record| match record.event {
                RuntimeEvent::VotingEngine(votingengine::Event::ProposalExecutionRetried {
                    proposal_id: event_id,
                    manual_attempts: 3,
                    outcome,
                }) if event_id == proposal_id => Some(outcome),
                _ => None,
            })
            .expect("third retry event should exist");
        assert_eq!(third_retry_outcome, STATUS_EXECUTION_FAILED);
    });
}

#[test]
fn default_cancel_callback_rejects_passed_retry_proposal() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));
        assert_noop!(
            VotingEngine::cancel_passed_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                proposal_id,
                b"not allowed"
                    .to_vec()
                    .try_into()
                    .expect("reason should fit")
            ),
            votingengine::Error::<Test>::ProposalCancellationNotAllowed
        );
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_PASSED
        );
    });
}

#[test]
fn automatic_fatal_failed_runs_execution_failed_terminal_hook() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        set_internal_callback_override_status(Some(STATUS_EXECUTION_FAILED));
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_EXECUTION_FAILED
        );
        let cleanup_log = INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow().clone());
        assert_eq!(cleanup_log, vec![proposal_id]);
    });
}

#[test]
fn execution_failed_terminal_cleanup_error_is_queued_and_retried() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        set_internal_callback_override_status(Some(STATUS_EXECUTION_FAILED));
        set_internal_terminal_cleanup_should_fail(true);
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        // 终态可以成立，但业务侧执行失败清理通知失败时必须留下重试入口。
        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_EXECUTION_FAILED
        );
        assert!(PendingTerminalCleanups::<Test>::contains_key(proposal_id));
        assert!(INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow().is_empty()));

        set_internal_terminal_cleanup_should_fail(false);
        System::set_block_number(2);
        <VotingEngine as Hooks<u64>>::on_initialize(2);

        assert!(!PendingTerminalCleanups::<Test>::contains_key(proposal_id));
        let cleanup_log = INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow().clone());
        assert_eq!(cleanup_log, vec![proposal_id]);
        assert!(System::events().into_iter().any(|record| matches!(
            record.event,
            RuntimeEvent::VotingEngine(votingengine::Event::ProposalTerminalCleanupCompleted {
                proposal_id: event_id
            }) if event_id == proposal_id
        )));
    });
}

#[test]
fn joint_retryable_outcome_is_forced_to_execution_failed() {
    new_test_ext().execute_with(|| {
        set_joint_callback_override_status(Some(STATUS_PASSED));
        let proposal_id = create_joint_proposal_for(nrc_admin(0), 10);

        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_EXECUTION_FAILED
        );
        assert!(
            ProposalExecutionRetryStates::<Test>::get(proposal_id).is_none(),
            "joint proposal must not enter internal retry state"
        );
    });
}

#[test]
fn execution_retry_deadline_expires_to_execution_failed() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        let proposal_id = create_internal_proposal_via_engine(nrc_admin(0), NRC, nrc_pid());

        assert_ok!(VotingEngine::set_status_and_emit(
            proposal_id,
            STATUS_PASSED
        ));
        let deadline = ProposalExecutionRetryStates::<Test>::get(proposal_id)
            .expect("retry state should exist")
            .retry_deadline;

        System::set_block_number(deadline);
        <VotingEngine as Hooks<u64>>::on_initialize(deadline);

        assert!(ProposalExecutionRetryStates::<Test>::get(proposal_id).is_none());
        assert_eq!(
            VotingEngine::proposals(proposal_id)
                .expect("proposal exists")
                .status,
            STATUS_EXECUTION_FAILED
        );
        assert!(System::events().into_iter().any(|record| matches!(
            record.event,
            RuntimeEvent::VotingEngine(votingengine::Event::ProposalExecutionRetryExpired {
                proposal_id: event_id
            }) if event_id == proposal_id
        )));
    });
}

#[test]
fn internal_vote_rejects_wrong_stage_joint_proposal() {
    new_test_ext().execute_with(|| {
        reset_internal_callback_state();
        // 手工写一个 kind=JOINT 的提案,用 internal_vote 去投 → 应拒绝。
        let proposal_id = 999u64;
        let now = <frame_system::Pallet<Test>>::block_number();
        Proposals::<Test>::insert(
            proposal_id,
            Proposal {
                kind: PROPOSAL_KIND_JOINT,
                stage: STAGE_JOINT,
                status: STATUS_VOTING,
                internal_code: None,
                account_context: None,
                subject_cid_numbers: Default::default(),
                start: now,
                end: now + 100,
                citizen_eligible_total: 0,
            },
        );

        assert_noop!(
            cast_internal_vote_via_extrinsic(nrc_admin(0), proposal_id, true),
            votingengine::Error::<Test>::InvalidProposalKind
        );
    });
}
