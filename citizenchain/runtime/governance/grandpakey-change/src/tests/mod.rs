#![cfg(test)]

use super::*;
use frame_support::{
    assert_noop, assert_ok, derive_impl, parameter_types,
    traits::{ConstU32, Hooks},
};
use frame_system as system;
use primitives::china::china_cb::CHINA_CB;
use sp_core::{Pair, Void};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::STATUS_EXECUTION_FAILED;

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
    pub type Grandpa = pallet_grandpa;

    #[runtime::pallet_index(2)]
    pub type VotingEngine = votingengine;

    #[runtime::pallet_index(99)]
    pub type InternalVote = internal_vote;

    #[runtime::pallet_index(3)]
    pub type GrandpaKeyChange = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
}

parameter_types! {
    pub const MaxGrandpaAuthorities: u32 = 64;
    pub const MaxGrandpaNominators: u32 = 0;
    pub const MaxSetIdSessionEntries: u64 = 16;
    pub const GrandpaChangeDelay: u64 = 30;
}

impl pallet_grandpa::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = MaxGrandpaAuthorities;
    type MaxNominators = MaxGrandpaNominators;
    type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
    type KeyOwnerProof = Void;
    type EquivocationReportSystem = ();
}

pub struct TestCidEligibility;
pub struct TestPopulationSnapshotVerifier;
pub struct TestInternalAdminProvider;

impl votingengine::CidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
    for TestCidEligibility
{
    fn is_eligible(_binding_id: &<Test as frame_system::Config>::Hash, _who: &AccountId32) -> bool {
        false
    }

    fn verify_and_consume_vote_credential(
        _binding_id: &<Test as frame_system::Config>::Hash,
        _who: &AccountId32,
        _proposal_id: u64,
        _nonce: &[u8],
        _signature: &[u8],
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        false
    }

    fn cleanup_vote_credentials(_proposal_id: u64) {}
}

impl
    votingengine::PopulationSnapshotVerifier<
        AccountId32,
        votingengine::pallet::VoteNonceOf<Test>,
        votingengine::pallet::VoteSignatureOf<Test>,
    > for TestPopulationSnapshotVerifier
{
    fn verify_population_snapshot(
        _who: &AccountId32,
        _eligible_total: u64,
        _nonce: &votingengine::pallet::VoteNonceOf<Test>,
        _signature: &votingengine::pallet::VoteSignatureOf<Test>,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        true
    }
}

impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        let mut who_raw = [0u8; 32];
        who_raw.copy_from_slice(who.as_ref());
        match institution_code {
            NRC | PRC => CHINA_CB
                .iter()
                .find(|node| AccountId32::new(node.main_account) == institution)
                .map(|node| node.admins.iter().any(|admin| *admin == who_raw))
                .unwrap_or(false),
            _ => false,
        }
    }

    fn get_admin_list(
        institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<sp_std::vec::Vec<AccountId32>> {
        match institution_code {
            NRC | PRC => CHINA_CB
                .iter()
                .find(|node| AccountId32::new(node.main_account) == institution)
                .map(|node| {
                    node.admins
                        .iter()
                        .map(|raw| AccountId32::new(*raw))
                        .collect()
                }),
            _ => None,
        }
    }
}

pub struct TestTimeProvider;

impl frame_support::traits::UnixTime for TestTimeProvider {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(1_782_864_000) // 2026-07-01
    }
}

impl votingengine::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type MaxAutoFinalizePerBlock = ConstU32<64>;
    type MaxProposalsPerExpiry = ConstU32<128>;
    type MaxInternalProposalMutexBindings = ConstU32<256>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxCleanupStepsPerBlock = ConstU32<8>;
    type CleanupKeysPerStep = ConstU32<64>;
    type MaxProposalDataLen = ConstU32<256>;
    type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxCleanupQueueBucketLimit = ConstU32<50>;
    type MaxCleanupScheduleOffset = ConstU32<100>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type CidEligibility = TestCidEligibility;
    type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = ();
    type MaxAdminsPerInstitution = ConstU32<32>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
    type InternalFinalizer = InternalVote;
    type InternalCleanup = InternalVote;
    type JointFinalizer = ();
    type JointCleanup = ();
    type LegislationVoteResultCallback = ();
    type LegislationFinalizer = ();
    type LegislationCleanup = ();
}

impl internal_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type GrandpaChangeDelay = GrandpaChangeDelay;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type WeightInfo = ();
}

fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
    vec![
        (
            GrandpaAuthorityId::from(ed25519::Public::from_raw(CHINA_CB[0].grandpa_key)),
            1,
        ),
        (
            GrandpaAuthorityId::from(ed25519::Public::from_raw(CHINA_CB[1].grandpa_key)),
            1,
        ),
        (
            GrandpaAuthorityId::from(ed25519::Public::from_raw(CHINA_CB[2].grandpa_key)),
            1,
        ),
    ]
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    pallet_grandpa::GenesisConfig::<Test> {
        authorities: grandpa_authorities(),
        _config: Default::default(),
    }
    .assimilate_storage(&mut storage)
    .expect("grandpa genesis should assimilate");
    GenesisConfig::<Test>::default()
        .assimilate_storage(&mut storage)
        .expect("grandpakey-change genesis should assimilate");

    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

fn cb_admin(node_index: usize, admin_index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CB[node_index].admins[admin_index])
}

fn cb_pallet_id(node_index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CB[node_index].main_account)
}

fn prc_admin(index: usize) -> AccountId32 {
    cb_admin(1, index)
}

fn prc_pallet_id() -> AccountId32 {
    cb_pallet_id(1)
}

fn valid_public_key(seed: u8) -> [u8; 32] {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = seed;
    ed25519::Pair::from_seed(&seed_bytes).public().0
}

fn identity_public_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    key[0] = 1;
    key
}

fn authority_id_from_key(key: [u8; 32]) -> GrandpaAuthorityId {
    GrandpaAuthorityId::from(ed25519::Public::from_raw(key))
}

fn pass_prc_proposal(node_index: usize, proposal_id: u64) {
    // 中文注释：提案发起人已经由投票引擎自动记一票，这里只补足剩余固定阈值票。
    for admin_index in 1..6 {
        assert_ok!(cast_vote(
            cb_admin(node_index, admin_index),
            proposal_id,
            true
        ));
    }
}

fn finalize_grandpa_at(block: u64) {
    System::set_block_number(block);
    <Grandpa as Hooks<u64>>::on_finalize(block);
}

/// 获取最近一次 create_internal_proposal 分配的 proposal_id。
fn last_proposal_id() -> u64 {
    votingengine::Pallet::<Test>::next_proposal_id().saturating_sub(1)
}

/// 测试辅助:走投票引擎公开 `internal_vote` extrinsic 投票(统一入口)。
fn cast_vote(who: AccountId32, proposal_id: u64, approve: bool) -> DispatchResult {
    frame_support::storage::with_transaction(
        || -> frame_support::storage::TransactionOutcome<DispatchResult> {
            match internal_vote::Pallet::<Test>::do_internal_vote(who, proposal_id, approve) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
            }
        },
    )
}

mod cases;
