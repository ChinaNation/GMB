#![cfg(test)]

use super::*;

#[test]
fn only_authorized_admin_can_propose() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ResolutionIssuance::propose_issuance(
                RuntimeOrigin::signed(AccountId32::new([2u8; 32])),
                reason_ok(),
                4300,
                allocations_ok(4300)
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn reject_invalid_allocation_count() {
    new_test_ext().execute_with(|| {
        let one = vec![crate::proposal::RecipientAmount {
            recipient: reserve_council_accounts()[0].clone(),
            amount: 1000,
        }];
        let alloc: pallet::AllocationOf<Test> = one.try_into().expect("should fit");
        assert_noop!(
            ResolutionIssuance::propose_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                alloc
            ),
            pallet::Error::<Test>::InvalidAllocationCount
        );
    });
}

#[test]
fn approved_callback_executes_issuance() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300)
        ));

        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));
        assert_eq!(
            votingengine::pallet::Proposals::<Test>::get(100)
                .expect("engine proposal should exist")
                .status,
            votingengine::STATUS_EXECUTED
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 0);
        assert!(pallet::Executed::<Test>::get(100).is_some());
        assert!(pallet::EverExecuted::<Test>::contains_key(100));
        assert_eq!(pallet::TotalIssued::<Test>::get(), 4300);
    });
}

#[test]
fn callback_rejects_non_finalizable_engine_status() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300)
        ));

        insert_engine_proposal_with_status(100, votingengine::STATUS_VOTING);
        assert_noop!(
            call_joint_callback(100, true),
            pallet::Error::<Test>::ProposalNotFinalizable
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 1);
        assert!(!pallet::Executed::<Test>::contains_key(100));
        assert_eq!(pallet::TotalIssued::<Test>::get(), 0);
    });
}

#[test]
fn callback_requires_votingengine_scope() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300)
        ));

        insert_engine_proposal(100);
        assert_noop!(
            ResolutionIssuance::on_joint_vote_finalized(100, true),
            pallet::Error::<Test>::ProposalNotFinalizable
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 1);
        assert!(!pallet::Executed::<Test>::contains_key(100));
    });
}

#[test]
fn second_callback_after_executed_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300)
        ));

        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));
        assert_noop!(
            call_joint_callback(100, true),
            pallet::Error::<Test>::ProposalNotFinalizable
        );
        assert_eq!(
            votingengine::pallet::Proposals::<Test>::get(100)
                .expect("engine proposal should exist")
                .status,
            votingengine::STATUS_EXECUTED
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 0);
        assert_eq!(pallet::TotalIssued::<Test>::get(), 4300);
    });
}

#[test]
fn rejected_callback_does_not_issue() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300)
        ));

        insert_engine_proposal_with_status(100, votingengine::STATUS_REJECTED);
        assert_ok!(call_joint_callback(100, false));
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 0);
        assert!(!pallet::Executed::<Test>::contains_key(100));
        assert_eq!(pallet::TotalIssued::<Test>::get(), 0);
    });
}

#[test]
fn callback_rejects_corrupted_reason_with_reason_too_long() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300)
        ));

        overwrite_proposal_data(
            100,
            crate::proposal::IssuanceProposalData {
                proposer: AccountId32::new([1u8; 32]),
                reason: vec![b'x'; 129],
                total_amount: 4300,
                allocations: allocations_ok(4300).to_vec(),
            },
        );
        insert_engine_proposal(100);
        assert_noop!(
            call_joint_callback(100, true),
            pallet::Error::<Test>::ReasonTooLong
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 1);
        assert!(!pallet::Executed::<Test>::contains_key(100));
        assert_eq!(pallet::TotalIssued::<Test>::get(), 0);
    });
}

#[test]
fn clear_executed_does_not_allow_replay() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300)
        ));
        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));
        assert_ok!(ResolutionIssuance::clear_executed(
            RuntimeOrigin::root(),
            100
        ));
        assert!(!pallet::Executed::<Test>::contains_key(100));
        assert!(pallet::EverExecuted::<Test>::contains_key(100));

        assert_noop!(
            pallet::Pallet::<Test>::execute_approved_issuance(
                100,
                &reason_ok(),
                4300,
                &allocations_ok(4300)
            ),
            pallet::Error::<Test>::AlreadyExecuted
        );
    });
}

#[test]
fn pause_blocks_approved_execution() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300)
        ));
        assert_ok!(ResolutionIssuance::set_paused(RuntimeOrigin::root(), true));
        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));
        assert_eq!(
            votingengine::pallet::Proposals::<Test>::get(100)
                .expect("engine proposal should exist")
                .status,
            votingengine::STATUS_EXECUTION_FAILED
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 0);
        assert!(!pallet::Executed::<Test>::contains_key(100));
        assert_eq!(pallet::TotalIssued::<Test>::get(), 0);
    });
}

#[test]
fn set_allowed_recipients_rejected_when_voting_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300)
        ));
        let recipients: BoundedVec<AccountId32, ConstU32<64>> = reserve_council_accounts()
            .try_into()
            .expect("recipients should fit");
        assert_noop!(
            ResolutionIssuance::set_allowed_recipients(RuntimeOrigin::root(), recipients),
            pallet::Error::<Test>::ActiveVotingProposalsExist
        );
    });
}

#[test]
fn issuance_event_comes_from_unified_pallet() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300)
        ));
        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));

        assert!(frame_system::Pallet::<Test>::events().iter().any(|record| {
            matches!(
                &record.event,
                RuntimeEvent::ResolutionIssuance(
                    pallet::Event::<Test>::ResolutionIssuanceExecuted {
                        proposal_id: 100,
                        ..
                    }
                )
            )
        }));
    });
}

#[test]
fn clear_executed_requires_existing_key() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ResolutionIssuance::clear_executed(RuntimeOrigin::root(), 99),
            pallet::Error::<Test>::NotExecuted
        );
    });
}

#[test]
fn set_paused_same_state_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::set_paused(RuntimeOrigin::root(), true));
        assert_noop!(
            ResolutionIssuance::set_paused(RuntimeOrigin::root(), true),
            pallet::Error::<Test>::AlreadyInState
        );
    });
}
