#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32};
use frame_system as system;
use primitives::china::china_cb::CHINA_CB;
use primitives::china::china_ch::CHINA_CH;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::{
    types::{ORG_NRC, ORG_OTH, ORG_PRB, ORG_PRC, ORG_PUP, ORG_REN},
    InternalVoteEngine, STATUS_EXECUTED, STATUS_EXECUTION_FAILED, STATUS_PASSED, STATUS_REJECTED,
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
    pub type AdminsChange = super;

    #[runtime::pallet_index(3)]
    pub type InternalVote = internal_vote;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
}

pub struct TestSfidEligibility;
impl votingengine::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
    for TestSfidEligibility
{
    fn is_eligible(_binding_id: &<Test as frame_system::Config>::Hash, _who: &AccountId32) -> bool {
        true
    }

    fn verify_and_consume_vote_credential(
        _binding_id: &<Test as frame_system::Config>::Hash,
        _who: &AccountId32,
        _proposal_id: u64,
        _nonce: &[u8],
        _signature: &[u8],
        _issuer_sfid_number: &[u8],
        _issuer_main_account: &AccountId32,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        true
    }
}

pub struct TestPopulationSnapshotVerifier;
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
        _issuer_sfid_number: &[u8],
        _issuer_main_account: &AccountId32,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        true
    }
}

pub struct TestInternalAdminProvider;
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(org: u8, institution: AccountId32, who: &AccountId32) -> bool {
        pallet::Pallet::<Test>::is_active_account_admin(org, institution, who)
    }

    fn get_admin_list(org: u8, institution: AccountId32) -> Option<Vec<AccountId32>> {
        pallet::Pallet::<Test>::active_account_admins(org, institution)
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
    type MaxCleanupQueueBucketLimit = ConstU32<50>;
    type MaxCleanupScheduleOffset = ConstU32<100>;
    type CleanupKeysPerStep = ConstU32<64>;
    type MaxProposalDataLen = ConstU32<{ 8 * 1024 }>;
    type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type SfidEligibility = TestSfidEligibility;
    type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
    type JointVoteResultCallback = ();
    // mock runtime 必须把本模块的 Executor 挂上,
    // 否则内部提案通过后业务执行回调不会触发,端到端测试失败。
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
}

impl internal_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<32>;
    type MaxPersonalAccountAdmins = ConstU32<16>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    GenesisConfig::<Test>::default()
        .assimilate_storage(&mut storage)
        .expect("admins-change genesis should assimilate");
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn nrc_admin(index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CB[0].admins[index])
}

fn prc_admin(index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CB[1].admins[index])
}

fn nrc_pallet_id() -> AccountId32 {
    AccountId32::new(CHINA_CB[0].main_account)
}

fn prc_pallet_id() -> AccountId32 {
    AccountId32::new(CHINA_CB[1].main_account)
}

fn prb_pallet_id() -> AccountId32 {
    AccountId32::new(CHINA_CH[0].main_account)
}

fn prb_admin(index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CH[0].admins[index])
}

fn pending_account_id() -> AccountId32 {
    AccountId32::new([42u8; 32])
}

fn pending_account_with_offset(offset: u8) -> AccountId32 {
    let mut raw = [42u8; 32];
    raw[0] = raw[0].saturating_add(offset);
    AccountId32::new(raw)
}

fn pending_account_with_second_byte(value: u8) -> AccountId32 {
    let mut raw = [42u8; 32];
    raw[1] = value;
    AccountId32::new(raw)
}

/// 获取最近一次 create_internal_proposal 分配的 proposal_id。
fn last_proposal_id() -> u64 {
    votingengine::Pallet::<Test>::next_proposal_id().saturating_sub(1)
}

fn current_admins(institution: AccountId32) -> Vec<AccountId32> {
    AdminAccounts::<Test>::get(institution)
        .expect("admin account should be stored")
        .admins
        .into_inner()
}

fn bounded_admins(admins: Vec<AccountId32>) -> AdminsOf<Test> {
    admins
        .try_into()
        .expect("test admin list should fit MaxAdminsPerInstitution")
}

fn current_vote_threshold(org: u8, account: AccountId32) -> u32 {
    votingengine::types::fixed_governance_pass_threshold(org)
        .or_else(|| internal_vote::ActiveDynamicThresholds::<Test>::get(org, account))
        .unwrap_or(2)
}

fn propose_admin_set_replacement(
    origin: RuntimeOrigin,
    org: u8,
    account: AccountId32,
    old_admin: AccountId32,
    new_admin: AccountId32,
) -> DispatchResult {
    let mut admins = current_admins(account.clone());
    let old_pos = admins
        .iter()
        .position(|admin| admin == &old_admin)
        .expect("old admin must exist in test account");
    admins[old_pos] = new_admin;
    let threshold = current_vote_threshold(org, account.clone());
    AdminsChange::propose_admin_set_change(origin, org, account, bounded_admins(admins), threshold)
}

fn mark_proposal_passed_without_callback(proposal_id: u64) {
    votingengine::pallet::Proposals::<Test>::mutate(proposal_id, |maybe| {
        let proposal = maybe.as_mut().expect("proposal should exist");
        proposal.status = STATUS_PASSED;
    });
    let now = System::block_number();
    votingengine::ProposalExecutionRetryStates::<Test>::insert(
        proposal_id,
        votingengine::ExecutionRetryState {
            manual_attempts: 0,
            first_auto_failed_at: now,
            retry_deadline: now,
            last_attempt_at: None,
        },
    );
}

/// 测试辅助:走投票引擎公开 `internal_vote` extrinsic 投票(Phase 2 后的统一入口)。
///
/// 替代旧的业务模块专属投票入口——业务模块不再持有投票 call,
/// 所有管理员通过投票引擎的公开 call 直接投票,通过后由 `InternalVoteExecutor` 回调
/// 执行业务。
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

fn finalized_event_count(proposal_id: u64, expected_status: u8) -> usize {
    System::events()
        .into_iter()
        .filter(|record| {
            matches!(
                &record.event,
                RuntimeEvent::VotingEngine(votingengine::Event::ProposalFinalized {
                    proposal_id: event_id,
                    status,
                }) if *event_id == proposal_id && *status == expected_status
            )
        })
        .count()
}

mod cases;
