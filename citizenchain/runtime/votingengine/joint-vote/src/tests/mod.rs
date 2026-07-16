use super::*;

use frame_support::{assert_ok, derive_impl, traits::ConstU32, traits::Hooks};
use frame_system as system;
use primitives::cid::china::{china_cb::CHINA_CB, china_ch::CHINA_CH};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage, DispatchError};
use votingengine::{
    traits::{
        CitizenIdentityReader, InternalAdminProvider, JointVoteEngine, JointVoteResultCallback,
    },
    ProposalExecutionOutcome, STAGE_REFERENDUM, STATUS_EXECUTED, STATUS_PASSED, STATUS_VOTING,
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
    pub type JointVote = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
}

pub struct TestTimeProvider;
impl frame_support::traits::UnixTime for TestTimeProvider {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(1_783_987_200)
    }
}

pub struct TestCitizenIdentityReader;
impl CitizenIdentityReader<AccountId32> for TestCitizenIdentityReader {
    fn can_vote(who: &AccountId32, _scope: &PopulationScope) -> bool {
        referendum_voters().contains(who)
    }

    fn can_be_candidate(who: &AccountId32, scope: &PopulationScope) -> bool {
        Self::can_vote(who, scope)
    }

    fn population_count(_scope: &PopulationScope) -> u64 {
        referendum_voters().len() as u64
    }

    fn create_population_snapshot(_scope: &PopulationScope) -> Result<(u64, u64), DispatchError> {
        Ok((7, referendum_voters().len() as u64))
    }

    fn can_vote_at(who: &AccountId32, snapshot_id: u64) -> bool {
        snapshot_id == 7 && referendum_voters().contains(who)
    }
}

pub struct TestAdminProvider;
impl TestAdminProvider {
    fn institution_admins(
        institution_code: votingengine::InstitutionCode,
        cid_number: &[u8],
    ) -> Option<Vec<AccountId32>> {
        match institution_code {
            votingengine::NRC | votingengine::PRC => CHINA_CB
                .iter()
                .find(|entry| entry.cid_number.as_bytes() == cid_number)
                .map(|entry| entry.admins.iter().copied().map(AccountId32::new).collect()),
            votingengine::PRB => CHINA_CH
                .iter()
                .find(|entry| entry.cid_number.as_bytes() == cid_number)
                .map(|entry| entry.admins.iter().copied().map(AccountId32::new).collect()),
            _ => None,
        }
    }
}

impl InternalAdminProvider<AccountId32> for TestAdminProvider {
    fn is_institution_admin(
        institution_code: votingengine::InstitutionCode,
        cid_number: &[u8],
        who: &AccountId32,
    ) -> bool {
        Self::institution_admins(institution_code, cid_number)
            .map(|admins| admins.contains(who))
            .unwrap_or(false)
    }

    fn get_institution_admins(
        institution_code: votingengine::InstitutionCode,
        cid_number: &[u8],
    ) -> Option<Vec<AccountId32>> {
        Self::institution_admins(institution_code, cid_number)
    }
}

pub struct TestJointCallback;
impl JointVoteResultCallback for TestJointCallback {
    fn on_joint_vote_finalized(
        _vote_proposal_id: u64,
        _approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        Ok(ProposalExecutionOutcome::Executed)
    }
}

impl votingengine::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type MaxAutoFinalizePerBlock = ConstU32<128>;
    type MaxAutoFinalizeWeightPerBlock = votingengine::BlockWeightFraction<Test, 2>;
    type MaxExecutionWeightPerBlock = votingengine::BlockWeightFraction<Test, 2>;
    type MaxCleanupWeightPerBlock = votingengine::BlockWeightFraction<Test, 4>;
    type MaxProposalsPerExpiry = ConstU32<128>;
    type MaxInternalProposalMutexBindings = ConstU32<256>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxCleanupStepsPerBlock = ConstU32<4>;
    type MaxCleanupActivationsPerBlock = ConstU32<64>;
    type CleanupKeysPerStep = ConstU32<8>;
    type MaxProposalDataLen = ConstU32<4096>;
    type MaxProposalObjectLen = ConstU32<10_240>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type CitizenIdentityReader = TestCitizenIdentityReader;
    type JointVoteResultCallback = TestJointCallback;
    type InternalVoteResultCallback = ();
    type InternalAdminProvider = TestAdminProvider;
    type InternalAdminsLenProvider = ();
    type MaxAdminsPerInstitution = ConstU32<32>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
    type TrackHandlers = (JointVote, ());
    type LegislationVoteResultCallback = ();
    type ElectionVoteResultCallback = ();
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("frame system genesis storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn referendum_voters() -> [AccountId32; 3] {
    [
        AccountId32::new([201; 32]),
        AccountId32::new([202; 32]),
        AccountId32::new([203; 32]),
    ]
}

fn nrc_admin() -> AccountId32 {
    AccountId32::new(CHINA_CB[0].admins[0])
}

fn nrc_cid_number() -> Vec<u8> {
    CHINA_CB[0].cid_number.as_bytes().to_vec()
}

fn all_joint_institutions() -> Vec<(votingengine::InstitutionCode, Vec<u8>)> {
    CHINA_CB
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let code = if index == 0 {
                votingengine::NRC
            } else {
                votingengine::PRC
            };
            (code, entry.cid_number.as_bytes().to_vec())
        })
        .chain(
            CHINA_CH
                .iter()
                .map(|entry| (votingengine::PRB, entry.cid_number.as_bytes().to_vec())),
        )
        .collect()
}

fn admins_for(
    institution_code: votingengine::InstitutionCode,
    cid_number: &[u8],
) -> Vec<AccountId32> {
    TestAdminProvider::institution_admins(institution_code, cid_number)
        .expect("joint institution should have admins")
}

fn create_joint_proposal() -> u64 {
    assert_ok!(JointVote::prepare_joint_population_snapshot(
        RuntimeOrigin::signed(nrc_admin()),
        nrc_cid_number().try_into().expect("NRC CID should fit"),
        PopulationScope::Country,
    ));
    <JointVote as JointVoteEngine<AccountId32>>::create_joint_proposal(
        nrc_admin(),
        nrc_cid_number(),
    )
    .expect("joint proposal should be created")
}

fn finalize_institution(
    proposal_id: u64,
    institution_code: votingengine::InstitutionCode,
    cid_number: Vec<u8>,
    approve: bool,
) {
    let admins = admins_for(institution_code, &cid_number);
    let threshold = votingengine::types::fixed_governance_pass_threshold(&institution_code)
        .expect("fixed institution threshold") as usize;
    let required = if approve {
        threshold
    } else {
        admins.len().saturating_sub(threshold).saturating_add(1)
    };
    for admin in admins.into_iter().take(required) {
        assert_ok!(JointVote::cast_admin(
            RuntimeOrigin::signed(admin),
            proposal_id,
            cid_number
                .clone()
                .try_into()
                .expect("institution CID should fit"),
            approve,
        ));
    }
}

#[test]
fn joint_internal_requires_all_105_weight() {
    assert!(!is_joint_unanimous(JOINT_VOTE_PASS_THRESHOLD - 1));
    assert!(is_joint_unanimous(JOINT_VOTE_PASS_THRESHOLD));
}

#[test]
fn joint_referendum_uses_strict_majority() {
    assert!(!is_jointreferendum_vote_passed(50, 100));
    assert!(is_jointreferendum_vote_passed(51, 100));
    assert!(is_jointreferendum_vote_rejected(50, 100));
    assert!(!is_jointreferendum_vote_rejected(49, 100));
}

#[test]
fn joint_referendum_fails_closed_without_population() {
    assert!(!is_jointreferendum_vote_passed(1, 0));
    assert!(!is_jointreferendum_vote_rejected(1, 0));
}

#[test]
fn all_105_weight_via_cast_admin_passes_and_executes() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal();
        for (code, institution) in all_joint_institutions() {
            finalize_institution(proposal_id, code, institution, true);
        }

        assert_eq!(
            JointTallies::<Test>::get(proposal_id).yes,
            JOINT_VOTE_PASS_THRESHOLD
        );
        assert_eq!(
            VotingEngine::proposals(proposal_id).unwrap().status,
            STATUS_PASSED
        );

        <VotingEngine as Hooks<u64>>::on_initialize(System::block_number());
        assert_eq!(
            VotingEngine::proposals(proposal_id).unwrap().status,
            STATUS_EXECUTED
        );
    });
}

#[test]
fn one_institution_rejection_via_cast_admin_enters_referendum() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal();
        let cid_number = CHINA_CB[1].cid_number.as_bytes().to_vec();
        finalize_institution(proposal_id, votingengine::PRC, cid_number, false);

        let proposal = VotingEngine::proposals(proposal_id).expect("proposal should exist");
        assert_eq!(proposal.stage, STAGE_REFERENDUM);
        assert_eq!(proposal.status, STATUS_VOTING);
    });
}

#[test]
fn joint_internal_timeout_via_public_finalizer_enters_referendum() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal();
        let end = VotingEngine::proposals(proposal_id).unwrap().end;
        System::set_block_number(end + 1);

        assert_ok!(VotingEngine::finalize_proposal(
            RuntimeOrigin::signed(AccountId32::new([250; 32])),
            proposal_id,
        ));

        let proposal = VotingEngine::proposals(proposal_id).expect("proposal should exist");
        assert_eq!(proposal.stage, STAGE_REFERENDUM);
        assert_eq!(proposal.status, STATUS_VOTING);
    });
}

#[test]
fn cast_referendum_extrinsic_uses_frozen_snapshot_and_strict_majority() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal();
        let cid_number = CHINA_CB[1].cid_number.as_bytes().to_vec();
        finalize_institution(proposal_id, votingengine::PRC, cid_number, false);

        let voters = referendum_voters();
        assert_ok!(JointVote::cast_referendum(
            RuntimeOrigin::signed(voters[0].clone()),
            proposal_id,
            true,
        ));
        assert_eq!(
            VotingEngine::proposals(proposal_id).unwrap().status,
            STATUS_VOTING
        );
        assert_ok!(JointVote::cast_referendum(
            RuntimeOrigin::signed(voters[1].clone()),
            proposal_id,
            true,
        ));

        assert_eq!(ReferendumTallies::<Test>::get(proposal_id).yes, 2);
        assert_eq!(
            VotingEngine::proposals(proposal_id).unwrap().status,
            STATUS_PASSED
        );
    });
}

#[test]
fn newly_added_voter_cannot_enter_existing_snapshot() {
    new_test_ext().execute_with(|| {
        let proposal_id = create_joint_proposal();
        let cid_number = CHINA_CB[1].cid_number.as_bytes().to_vec();
        finalize_institution(proposal_id, votingengine::PRC, cid_number, false);

        let post_snapshot_account = AccountId32::new([204; 32]);
        assert!(JointVote::cast_referendum(
            RuntimeOrigin::signed(post_snapshot_account),
            proposal_id,
            true,
        )
        .is_err());
        assert!(!ReferendumVotesByAccount::<Test>::contains_key(
            proposal_id,
            AccountId32::new([204; 32]),
        ));
    });
}
