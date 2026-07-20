#![cfg(test)]

//! election-vote 状态机测试 runtime。
//!
//! 普选使用 citizen-identity 人口数据生成提案快照，互选使用 VotePlan 岗位任职
//! 快照；测试覆盖创建、投票、超时、结果回调和分块清理。

use core::cell::RefCell;

use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU32, ConstU64},
};
use frame_system as system;
use primitives::cid::{
    china::{china_lf::CHINA_LF, china_zf::CHINA_ZF},
    code::{institution_code_from_cid_number, InstitutionCode, FRG, NLG},
};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::{
    CitizenIdentityReader, ElectionProposalFinalizer, ElectionVoteResultCallback,
    InstitutionRoleProvider, InternalAdminProvider, PopulationScope, ProposalExecutionOutcome,
    ProposalTrackHandler, STATUS_PASSED, STATUS_REJECTED,
};

use crate::{
    pallet::{
        ElectionCandidateTallies, ElectionCandidates, ElectionMetaStore, ElectionResults,
        ElectionTallyStore, ElectionVotesByVoter, Error,
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

const ORGANIZER_CODE: InstitutionCode = FRG;
const TARGET_CODE: InstitutionCode = NLG;

fn account(id: u8) -> AccountId32 {
    AccountId32::new([id; 32])
}

fn organizer_admin() -> AccountId32 {
    account(2)
}

fn organizer_cid_number() -> votingengine::types::CidNumber {
    CHINA_ZF
        .iter()
        .find(|entry| institution_code_from_cid_number(entry.cid_number) == Some(ORGANIZER_CODE))
        .expect("federal registry CID should exist")
        .cid_number
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("organizer CID should fit")
}

fn target_cid_number() -> votingengine::types::CidNumber {
    CHINA_LF
        .iter()
        .find(|entry| institution_code_from_cid_number(entry.cid_number) == Some(TARGET_CODE))
        .expect("national legislature CID should exist")
        .cid_number
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("target CID should fit")
}

fn target_admins() -> Vec<AccountId32> {
    vec![account(31), account(32), account(33)]
}

thread_local! {
    static POPULATION_COUNT: RefCell<u64> = const { RefCell::new(3) };
}

pub struct TestCitizenIdentityReader;
pub struct TestInternalAdminProvider;
pub struct TestInstitutionRoleProvider;

const ORGANIZER_ROLE: &[u8] = b"ORGANIZER";
const TARGET_ROLE: &[u8] = b"MEMBER";

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

    fn population_data(scope: &PopulationScope) -> votingengine::PopulationData {
        votingengine::PopulationData {
            scope: scope.clone(),
            eligible_total: POPULATION_COUNT.with(|count| *count.borrow()),
            eligibility_revision: 1,
            eligibility_date: 20_000,
        }
    }

    fn can_vote_at(who: &AccountId32, population_data: &votingengine::PopulationData) -> bool {
        (0..population_data.eligible_total)
            .map(|offset| account(21u8.saturating_add(offset as u8)))
            .any(|voter| &voter == who)
    }
}

impl InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_institution_admin(
        institution_code: InstitutionCode,
        cid_number: &[u8],
        who: &AccountId32,
    ) -> bool {
        institution_code == ORGANIZER_CODE
            && cid_number == organizer_cid_number().as_slice()
            && *who == organizer_admin()
    }

    fn get_institution_admins(
        institution_code: InstitutionCode,
        cid_number: &[u8],
    ) -> Option<Vec<AccountId32>> {
        (institution_code == TARGET_CODE && cid_number == target_cid_number().as_slice())
            .then(target_admins)
    }
}

impl InstitutionRoleProvider<AccountId32> for TestInstitutionRoleProvider {
    fn is_active_assignment(cid_number: &[u8], who: &AccountId32, role_code: &[u8]) -> bool {
        cid_number == organizer_cid_number().as_slice()
            && who == &organizer_admin()
            && role_code == ORGANIZER_ROLE
    }

    fn active_accounts_for_role(cid_number: &[u8], role_code: &[u8]) -> Vec<AccountId32> {
        if cid_number == target_cid_number().as_slice() && role_code == TARGET_ROLE {
            target_admins()
        } else if cid_number == organizer_cid_number().as_slice() && role_code == ORGANIZER_ROLE {
            vec![organizer_admin()]
        } else {
            Vec::new()
        }
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
    type MaxAutoFinalizeWeightPerBlock = votingengine::BlockWeightFraction<Test, 4>;
    type MaxExecutionWeightPerBlock = votingengine::BlockWeightFraction<Test, 4>;
    type MaxCleanupWeightPerBlock = votingengine::BlockWeightFraction<Test, 8>;
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
    type InstitutionRoleProvider = TestInstitutionRoleProvider;
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

fn vote_plan(mutual: bool) -> votingengine::types::VotePlanOf<AccountId32> {
    let owner: frame_support::BoundedVec<
        u8,
        ConstU32<{ entity_primitives::BUSINESS_MODULE_TAG_MAX_BYTES }>,
    > = b"election-campaign"
        .to_vec()
        .try_into()
        .expect("owner fits");
    let proposer =
        votingengine::AuthorizationSubject::Institution(entity_primitives::RoleSubject {
            cid_number: organizer_cid_number(),
            role_code: ORGANIZER_ROLE.to_vec().try_into().expect("role fits"),
        });
    let voters = mutual
        .then(|| {
            votingengine::AuthorizationSubject::Institution(entity_primitives::RoleSubject {
                cid_number: target_cid_number(),
                role_code: TARGET_ROLE.to_vec().try_into().expect("role fits"),
            })
        })
        .into_iter()
        .collect();
    votingengine::types::VotePlanOf::try_new(
        entity_primitives::BusinessActionId {
            module_tag: owner.clone(),
            action_code: 0,
        },
        owner,
        proposer,
        voters,
        votingengine::VotingEngineKind::Election,
        [7u8; 32],
    )
    .expect("election plan valid")
}

fn create_popular(candidates: Vec<AccountId32>) -> u64 {
    ElectionVote::do_create_popular_election(
        organizer_admin(),
        vote_plan(false),
        organizer_cid_number(),
        target_cid_number(),
        office_code(),
        7,
        1,
        10,
        20,
        PopulationScope::Country,
        candidates,
    )
    .expect("popular election should be created")
}

fn create_mutual() -> u64 {
    let admins = target_admins();
    ElectionVote::do_create_mutual_election(
        organizer_admin(),
        vote_plan(true),
        organizer_cid_number(),
        target_cid_number(),
        office_code(),
        8,
        1,
        10,
        20,
        vec![admins[0].clone(), admins[1].clone()],
    )
    .expect("mutual election should be created")
}

#[test]
fn popular_election_uses_population_snapshot_and_generates_result() {
    new_test_ext().execute_with(|| {
        let candidates = vec![account(11), account(12)];
        let voters = vec![account(21), account(22), account(23)];
        let proposal_id = create_popular(candidates.clone());

        // 创建后人口增长不能把新账户塞进既有普选；Popular 不保存全量选民表。
        POPULATION_COUNT.with(|count| *count.borrow_mut() = 4);
        assert_noop!(
            ElectionVote::cast_popular_vote(
                RuntimeOrigin::signed(account(24)),
                proposal_id,
                candidates[0].clone()
            ),
            Error::<Test>::VoterNotEligible
        );

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
        assert_eq!(
            VotingEngine::population_eligible_total_of(proposal_id),
            Some(3)
        );
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
fn mutual_election_uses_role_assignment_snapshot() {
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
fn creation_rejects_vote_plan_for_wrong_target_role() {
    new_test_ext().execute_with(|| {
        let admins = target_admins();
        assert_noop!(
            ElectionVote::do_create_mutual_election(
                organizer_admin(),
                vote_plan(false),
                organizer_cid_number(),
                target_cid_number(),
                office_code(),
                8,
                1,
                10,
                20,
                vec![admins[0].clone()],
            ),
            Error::<Test>::InvalidVotePlan
        );
    });
}

#[test]
fn popular_creation_rejects_ineligible_accounts_and_bad_shape() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ElectionVote::do_create_popular_election(
                organizer_admin(),
                vote_plan(false),
                organizer_cid_number(),
                target_cid_number(),
                office_code(),
                7,
                1,
                10,
                20,
                PopulationScope::Country,
                vec![account(11), account(251)]
            ),
            Error::<Test>::CandidateNotEligible
        );
        assert_noop!(
            ElectionVote::do_create_popular_election(
                account(9),
                vote_plan(false),
                organizer_cid_number(),
                target_cid_number(),
                office_code(),
                7,
                1,
                10,
                20,
                PopulationScope::Country,
                vec![account(11), account(12)]
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
        let proposal_id = create_popular(candidates.clone());
        assert_noop!(
            ElectionVote::cast_popular_vote(
                RuntimeOrigin::signed(account(99)),
                proposal_id,
                candidates[0].clone()
            ),
            Error::<Test>::VoterNotEligible
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
        let proposal_id = create_popular(vec![account(11), account(12)]);
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
        let proposal_id = create_popular(candidates.clone());
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
        let proposal_id = create_popular(vec![account(11), account(12)]);
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
