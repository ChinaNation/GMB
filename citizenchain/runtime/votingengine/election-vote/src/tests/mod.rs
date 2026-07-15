#![cfg(test)]

//! election-vote 状态机测试 runtime。
//!
//! 普选使用 citizen-identity 抽象提供人口与资格快照，互选使用 admins provider
//! 提供机构完整管理员快照；测试覆盖创建、投票、超时、结果回调和分块清理。

use core::cell::RefCell;

use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU32, ConstU64},
};
use frame_system as system;
use primitives::cid::code::InstitutionCode;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::{
    CitizenIdentityReader, ElectionProposalFinalizer, ElectionVoteResultCallback,
    InternalAdminProvider, PopulationScope, ProposalExecutionOutcome, ProposalTrackHandler,
    STATUS_PASSED, STATUS_REJECTED,
};

use crate::{
    pallet::{
        ElectionCandidateTallies, ElectionCandidates, ElectionMetaStore, ElectionResults,
        ElectionTallyStore, ElectionVoters, ElectionVotesByVoter, Error,
    },
    types::ElectionMode,
};

type Block = frame_system::mocking::MockBlock<Test>;

#[frame_support::runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask,
        RuntimeViewFunction
    )]
    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;

    #[runtime::pallet_index(1)]
    pub type VotingEngine = votingengine;

    #[runtime::pallet_index(2)]
    pub type ElectionVote = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
}

const ORGANIZER_CODE: InstitutionCode = *b"CGOV";
const TARGET_CODE: InstitutionCode = *b"NLG\0";

fn account(id: u8) -> AccountId32 {
    AccountId32::new([id; 32])
}

fn organizer() -> AccountId32 {
    account(1)
}

fn organizer_admin() -> AccountId32 {
    account(2)
}

fn target() -> AccountId32 {
    account(3)
}

fn target_admins() -> Vec<AccountId32> {
    vec![account(31), account(32), account(33)]
}

thread_local! {
    static POPULATION_COUNT: RefCell<u64> = const { RefCell::new(3) };
}

pub struct TestCitizenIdentityReader;
pub struct TestInternalAdminProvider;
pub struct TestInstitutionQuery;

impl CitizenIdentityReader<AccountId32> for TestCitizenIdentityReader {
    fn can_vote(who: &AccountId32, _scope: &PopulationScope) -> bool {
        *who != account(250)
    }

    fn can_be_candidate(who: &AccountId32, _scope: &PopulationScope) -> bool {
        *who != account(251)
    }

    fn population_count(_scope: &PopulationScope) -> u64 {
        POPULATION_COUNT.with(|count| *count.borrow())
    }
}

impl InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        institution_code == ORGANIZER_CODE
            && institution == organizer()
            && *who == organizer_admin()
    }

    fn get_admin_list(
        institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<Vec<AccountId32>> {
        (institution_code == TARGET_CODE && institution == target()).then(target_admins)
    }
}

impl entity_primitives::InstitutionMultisigQuery<AccountId32> for TestInstitutionQuery {
    fn lookup_cid(addr: &AccountId32) -> Option<Vec<u8>> {
        let bytes: &[u8] = addr.as_ref();
        Some([b"TEST-ELECTION-".as_slice(), &bytes[..1]].concat())
    }

    fn lookup_org(_addr: &AccountId32) -> Option<InstitutionCode> {
        None
    }

    fn lookup_admin_config(
        _addr: &AccountId32,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<AccountId32>> {
        None
    }

    fn is_active(_addr: &AccountId32) -> bool {
        true
    }
}

pub struct TestTimeProvider;

impl frame_support::traits::UnixTime for TestTimeProvider {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(1_782_864_000)
    }
}

impl votingengine::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type MaxAutoFinalizePerBlock = ConstU32<64>;
    type MaxAutoFinalizeWeightPerBlock = votingengine::weights::BlockWeightFraction<Test, 4>;
    type MaxExecutionWeightPerBlock = votingengine::weights::BlockWeightFraction<Test, 4>;
    type MaxCleanupWeightPerBlock = votingengine::weights::BlockWeightFraction<Test, 8>;
    type MaxProposalsPerExpiry = ConstU32<128>;
    type MaxInternalProposalMutexBindings = ConstU32<256>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxCleanupStepsPerBlock = ConstU32<8>;
    type CleanupKeysPerStep = ConstU32<2>;
    type MaxProposalDataLen = ConstU32<1024>;
    type MaxProposalObjectLen = ConstU32<{ 64 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxCleanupActivationsPerBlock = ConstU32<50>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type CitizenIdentityReader = TestCitizenIdentityReader;
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = ();
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = ();
    type MaxAdminsPerInstitution = ConstU32<16>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
    type TrackHandlers = (ElectionVote, ());
    type LegislationVoteResultCallback = ();
    type ElectionVoteResultCallback = ElectionVote;
}

impl crate::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxElectionOfficeCodeLen = ConstU32<32>;
    type MaxElectionCandidates = ConstU32<8>;
    type MaxElectionVoters = ConstU32<8>;
    type InstitutionQuery = TestInstitutionQuery;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    POPULATION_COUNT.with(|count| *count.borrow_mut() = 3);
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn office_code() -> crate::pallet::ElectionOfficeCodeOf<Test> {
    b"speaker".to_vec().try_into().expect("bounded office code")
}

fn create_popular(candidates: Vec<AccountId32>, voters: Vec<AccountId32>) -> u64 {
    ElectionVote::do_create_popular_election(
        organizer_admin(),
        ORGANIZER_CODE,
        organizer(),
        TARGET_CODE,
        target(),
        office_code(),
        7,
        1,
        10,
        20,
        PopulationScope::Country,
        candidates,
        voters,
    )
    .expect("popular election should be created")
}

fn create_mutual() -> u64 {
    let admins = target_admins();
    ElectionVote::do_create_mutual_election(
        organizer_admin(),
        ORGANIZER_CODE,
        organizer(),
        TARGET_CODE,
        target(),
        office_code(),
        8,
        1,
        10,
        20,
        vec![admins[0].clone(), admins[1].clone()],
        admins,
    )
    .expect("mutual election should be created")
}

#[test]
fn popular_election_uses_population_snapshot_and_generates_result() {
    new_test_ext().execute_with(|| {
        let candidates = vec![account(11), account(12)];
        let voters = vec![account(21), account(22), account(23)];
        let proposal_id = create_popular(candidates.clone(), voters.clone());

        assert_ok!(ElectionVote::cast_popular_vote(
            RuntimeOrigin::signed(voters[0].clone()),
            proposal_id,
            candidates[0].clone()
        ));
        assert_ok!(ElectionVote::cast_popular_vote(
            RuntimeOrigin::signed(voters[1].clone()),
            proposal_id,
            candidates[0].clone()
        ));
        assert_ok!(ElectionVote::cast_popular_vote(
            RuntimeOrigin::signed(voters[2].clone()),
            proposal_id,
            candidates[1].clone()
        ));

        let proposal = votingengine::pallet::Proposals::<Test>::get(proposal_id).unwrap();
        assert_eq!(proposal.status, STATUS_PASSED);
        assert_eq!(proposal.citizen_eligible_total, 3);
        let winners = ElectionResults::<Test>::get(proposal_id).unwrap();
        assert_eq!(winners[0].account, candidates[0]);
        assert_eq!(winners[0].votes, 2);
        assert_eq!(
            ElectionVote::on_election_vote_finalized(proposal_id, true),
            Ok(ProposalExecutionOutcome::Executed)
        );
    });
}

#[test]
fn mutual_election_uses_complete_admin_snapshot() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_mutual();
        let admins = target_admins();
        for voter in admins.iter() {
            assert_ok!(ElectionVote::cast_mutual_vote(
                RuntimeOrigin::signed(voter.clone()),
                proposal_id,
                admins[0].clone()
            ));
        }

        let meta = ElectionMetaStore::<Test>::get(proposal_id).unwrap();
        assert_eq!(meta.mode, ElectionMode::Mutual);
        assert_eq!(
            votingengine::pallet::Proposals::<Test>::get(proposal_id)
                .unwrap()
                .status,
            STATUS_PASSED
        );
    });
}

#[test]
fn creation_rejects_untrusted_or_incomplete_snapshots() {
    new_test_ext().execute_with(|| {
        let candidates = vec![account(11), account(12)];
        let voters = vec![account(21), account(22), account(23)];
        POPULATION_COUNT.with(|count| *count.borrow_mut() = 4);
        assert_noop!(
            ElectionVote::do_create_popular_election(
                organizer_admin(),
                ORGANIZER_CODE,
                organizer(),
                TARGET_CODE,
                target(),
                office_code(),
                7,
                1,
                10,
                20,
                PopulationScope::Country,
                candidates.clone(),
                voters,
            ),
            Error::<Test>::ElectionSnapshotMismatch
        );

        let admins = target_admins();
        assert_noop!(
            ElectionVote::do_create_mutual_election(
                organizer_admin(),
                ORGANIZER_CODE,
                organizer(),
                TARGET_CODE,
                target(),
                office_code(),
                8,
                1,
                10,
                20,
                vec![admins[0].clone()],
                vec![admins[0].clone(), admins[1].clone()],
            ),
            Error::<Test>::ElectionSnapshotMismatch
        );
    });
}

#[test]
fn popular_creation_rejects_ineligible_accounts_and_bad_shape() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ElectionVote::do_create_popular_election(
                organizer_admin(),
                ORGANIZER_CODE,
                organizer(),
                TARGET_CODE,
                target(),
                office_code(),
                7,
                1,
                10,
                20,
                PopulationScope::Country,
                vec![account(11), account(12)],
                vec![account(21), account(22), account(250)]
            ),
            Error::<Test>::VoterNotEligible
        );
        assert_noop!(
            ElectionVote::do_create_popular_election(
                organizer_admin(),
                ORGANIZER_CODE,
                organizer(),
                TARGET_CODE,
                target(),
                office_code(),
                7,
                1,
                10,
                20,
                PopulationScope::Country,
                vec![account(11), account(251)],
                vec![account(21), account(22), account(23)]
            ),
            Error::<Test>::CandidateNotEligible
        );
        assert_noop!(
            ElectionVote::do_create_popular_election(
                account(9),
                ORGANIZER_CODE,
                organizer(),
                TARGET_CODE,
                target(),
                office_code(),
                7,
                1,
                10,
                20,
                PopulationScope::Country,
                vec![account(11), account(12)],
                vec![account(21), account(22), account(23)]
            ),
            Error::<Test>::NotOrganizerAdmin
        );
    });
}

#[test]
fn cast_rejects_wrong_voter_candidate_stage_and_duplicate_vote() {
    new_test_ext().execute_with(|| {
        let candidates = vec![account(11), account(12)];
        let voters = vec![account(21), account(22), account(23)];
        let proposal_id = create_popular(candidates.clone(), voters.clone());
        assert_noop!(
            ElectionVote::cast_popular_vote(
                RuntimeOrigin::signed(account(99)),
                proposal_id,
                candidates[0].clone()
            ),
            Error::<Test>::VoterNotInSnapshot
        );
        assert_noop!(
            ElectionVote::cast_popular_vote(
                RuntimeOrigin::signed(voters[0].clone()),
                proposal_id,
                account(99)
            ),
            Error::<Test>::CandidateNotInSnapshot
        );
        assert_noop!(
            ElectionVote::cast_mutual_vote(
                RuntimeOrigin::signed(voters[0].clone()),
                proposal_id,
                candidates[0].clone()
            ),
            votingengine::Error::<Test>::InvalidProposalStage
        );
        assert_ok!(ElectionVote::cast_popular_vote(
            RuntimeOrigin::signed(voters[0].clone()),
            proposal_id,
            candidates[0].clone()
        ));
        assert_noop!(
            ElectionVote::cast_popular_vote(
                RuntimeOrigin::signed(voters[0].clone()),
                proposal_id,
                candidates[0].clone()
            ),
            votingengine::Error::<Test>::AlreadyVoted
        );
    });
}

#[test]
fn timeout_is_rejected_before_expiry_then_finalizes_no_vote_election() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_popular(
            vec![account(11), account(12)],
            vec![account(21), account(22), account(23)],
        );
        let proposal = votingengine::pallet::Proposals::<Test>::get(proposal_id).unwrap();
        assert_noop!(
            ElectionVote::finalize_election_popular_timeout(&proposal, proposal_id),
            votingengine::Error::<Test>::VoteNotExpired
        );
        System::set_block_number(proposal.end + 1);
        assert_ok!(VotingEngine::finalize_proposal(
            RuntimeOrigin::signed(account(99)),
            proposal_id
        ));
        assert_eq!(
            votingengine::pallet::Proposals::<Test>::get(proposal_id)
                .unwrap()
                .status,
            STATUS_REJECTED
        );
    });
}

#[test]
fn election_cleanup_removes_all_track_storage() {
    new_test_ext().execute_with(|| {
        let candidates = vec![account(11), account(12)];
        let voters = vec![account(21), account(22), account(23)];
        let proposal_id = create_popular(candidates.clone(), voters.clone());
        assert_ok!(ElectionVote::cast_popular_vote(
            RuntimeOrigin::signed(voters[0].clone()),
            proposal_id,
            candidates[0].clone()
        ));

        assert!(
            <ElectionVote as ProposalTrackHandler<u64, AccountId32>>::cleanup_chunk(
                votingengine::PROPOSAL_KIND_ELECTION,
                proposal_id,
                1,
            )
            .is_some()
        );
        let _ = <ElectionVote as ProposalTrackHandler<u64, AccountId32>>::cleanup_chunk(
            votingengine::PROPOSAL_KIND_ELECTION,
            proposal_id,
            8,
        );
        assert_eq!(
            <ElectionVote as ProposalTrackHandler<u64, AccountId32>>::cleanup_terminal(
                votingengine::PROPOSAL_KIND_ELECTION,
                proposal_id,
            ),
            Some(())
        );

        assert!(!ElectionMetaStore::<Test>::contains_key(proposal_id));
        assert!(!ElectionCandidates::<Test>::contains_key(proposal_id));
        assert!(!ElectionResults::<Test>::contains_key(proposal_id));
        assert!(ElectionVotesByVoter::<Test>::iter_prefix(proposal_id)
            .next()
            .is_none());
        assert!(ElectionVoters::<Test>::iter_prefix(proposal_id)
            .next()
            .is_none());
        assert!(ElectionCandidateTallies::<Test>::iter_prefix(proposal_id)
            .next()
            .is_none());
        assert!(!ElectionTallyStore::<Test>::contains_key(proposal_id));
    });
}

#[test]
fn result_callback_fails_closed_for_incomplete_or_foreign_result() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            ElectionVote::on_election_vote_finalized(999, true),
            Ok(ProposalExecutionOutcome::Ignored)
        );
        let proposal_id = create_popular(
            vec![account(11), account(12)],
            vec![account(21), account(22), account(23)],
        );
        assert_eq!(
            ElectionVote::on_election_vote_finalized(proposal_id, true),
            Ok(ProposalExecutionOutcome::FatalFailed)
        );
        assert_eq!(
            ElectionVote::on_election_vote_finalized(proposal_id, false),
            Ok(ProposalExecutionOutcome::Executed)
        );
    });
}
