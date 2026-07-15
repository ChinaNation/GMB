use super::*;
use core::cell::RefCell;

use frame_support::{
    assert_noop, assert_ok, derive_impl, traits::ConstU32, traits::Hooks, BoundedVec,
};
use frame_system as system;

// 引擎核心 storage / 类型 / trait(住在 votingengine 主 crate)。
// `use super::*` 拉进 internal-vote 自有的 pallet items(Pallet/Event/Error/Config/InternalVotesByAccount/...);
// 这里追加 votingengine 的 storage 与 trait 名,让测试代码用短名引用。
use primitives::cid::code::PRS;
use votingengine::pallet::{
    AdminSnapshot, AutoFinalizeDeadLetters, AutoFinalizeRetryStates, CurrentProposalYear,
    ExecutionRetryDeadlines, NextProposalId, PendingCleanupQueue, PendingCleanupQueueHead,
    PendingCleanupQueueTail, PendingExecutionRetryExpirations, PendingExpiryBucket,
    PendingProposalCleanups, PendingTerminalCleanups, ProposalDisplayId,
    ProposalExecutionRetryStates, Proposals, ProposalsByCid, ProposalsByCode, ProposalsByExpiry,
    ProposalsByOwner, ProposalsByYear, ScheduledCleanupHead, ScheduledCleanupTail,
    ScheduledCleanups, YearProposalCounter,
};
use votingengine::types::{code_bytes, InstitutionCode, NJD, PMUL};
// 测试用机构码:个人多签 / 公权法人 / 私权法人,均属"注册多签动态账户"。
const PERSONAL_CODE: InstitutionCode = PMUL;
const PUBLIC_CODE: InstitutionCode = code_bytes("CGOV");
const PRIVATE_CODE: InstitutionCode = code_bytes("SFLP");
// 六个永久国家单例中的总统府：用于验证“无账户级动态阈值、按提案快照过半”。
const PERMANENT_SINGLETON_CODE: InstitutionCode = PRS;
// joint mode storage 在 joint-vote sub-pallet
use primitives::cid::china::china_cb::CHINA_CB;
use primitives::cid::china::china_ch::CHINA_CH;
use primitives::cid::china::china_sf::{CHINA_SF, NATIONAL_JUDICIAL_YUAN_ADMINS};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage, DispatchError};
use votingengine::traits::{
    InternalAdminProvider, InternalVoteEngine, InternalVoteResultCallback, JointVoteEngine,
    JointVoteResultCallback,
};
use votingengine::{
    PendingCleanupStage, Proposal, ProposalExecutionOutcome, VoteCountU32, VoteCountU64,
    PROPOSAL_KIND_JOINT, STAGE_INTERNAL, STAGE_JOINT, STAGE_REFERENDUM, STATUS_EXECUTED,
    STATUS_EXECUTION_FAILED, STATUS_PASSED, STATUS_REJECTED, STATUS_VOTING,
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
    pub type InternalVote = super;

    #[runtime::pallet_index(3)]
    pub type JointVote = joint_vote;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
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
    type MaxCleanupStepsPerBlock = ConstU32<3>;
    type MaxCleanupActivationsPerBlock = ConstU32<50>;
    type CleanupKeysPerStep = ConstU32<2>;
    type MaxProposalDataLen = ConstU32<4096>;
    type MaxProposalObjectLen = ConstU32<10_240>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type CitizenIdentityReader = TestCitizenIdentityReader;
    type JointVoteResultCallback = TestJointVoteResultCallback;
    type InternalVoteResultCallback = TestInternalVoteResultCallback;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = ();
    type MaxAdminsPerInstitution = ConstU32<32>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
    type TrackHandlers = (InternalVote, (JointVote, ()));
    type LegislationVoteResultCallback = ();
    type ElectionVoteResultCallback = ();
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl joint_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

thread_local! {
    static TEST_POPULATION_COUNT: RefCell<u64> = const { RefCell::new(100) };
}
thread_local! {
    static TEST_NOW_SECS: RefCell<u64> = const { RefCell::new(DEFAULT_TEST_NOW_SECS) };
}
thread_local! {
    static JOINT_CALLBACK_SHOULD_FAIL: RefCell<bool> = const { RefCell::new(false) };
}
thread_local! {
    static JOINT_CALLBACK_OVERRIDE_STATUS: RefCell<Option<u8>> = const { RefCell::new(None) };
}
// 内部投票终态回调测试桩。
// INTERNAL_CALLBACK_SHOULD_FAIL = true → 异步执行回调返回 Err，
//   提案保留 PASSED，等待执行队列按退避策略重试。
// INTERNAL_CALLBACK_LOG 记录每次被调用的 (proposal_id, approved),
//   用于验证回调是否触发 / 触发参数是否正确。
thread_local! {
    static INTERNAL_CALLBACK_SHOULD_FAIL: RefCell<bool> = const { RefCell::new(false) };
}
thread_local! {
    static INTERNAL_CALLBACK_LOG: RefCell<Vec<(u64, bool)>> = const { RefCell::new(Vec::new()) };
}
thread_local! {
    static INTERNAL_CALLBACK_OVERRIDE_STATUS: RefCell<Option<u8>> = const { RefCell::new(None) };
}
thread_local! {
    static INTERNAL_TERMINAL_CLEANUP_LOG: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}
thread_local! {
    static INTERNAL_TERMINAL_CLEANUP_SHOULD_FAIL: RefCell<bool> = const { RefCell::new(false) };
}
thread_local! {
    static REGISTERED_ADMIN_LIST_OVERRIDE: RefCell<Option<Vec<AccountId32>>> = const { RefCell::new(None) };
}

pub struct TestCitizenIdentityReader;
pub struct TestJointVoteResultCallback;
pub struct TestInternalVoteResultCallback;
pub struct TestInternalAdminProvider;

fn pending_account_institution() -> AccountId32 {
    AccountId32::new([77u8; 32])
}

fn pending_account_admin(index: usize) -> AccountId32 {
    match index {
        0 => AccountId32::new([91u8; 32]),
        1 => AccountId32::new([92u8; 32]),
        _ => AccountId32::new([93u8; 32]),
    }
}

fn registered_account_institution() -> AccountId32 {
    AccountId32::new([78u8; 32])
}

fn registered_account_admin(index: usize) -> AccountId32 {
    match index {
        0 => AccountId32::new([81u8; 32]),
        1 => AccountId32::new([82u8; 32]),
        _ => AccountId32::new([83u8; 32]),
    }
}

fn permanent_singleton_institution() -> AccountId32 {
    let singleton = primitives::institution_constraints::singleton_institutions()
        .into_iter()
        .find(|item| item.code == PERMANENT_SINGLETON_CODE)
        .expect("PRS singleton must exist");
    AccountId32::new(singleton.main_account)
}

fn permanent_singleton_admin(index: usize) -> AccountId32 {
    AccountId32::new([101u8.saturating_add(index as u8); 32])
}

fn set_registered_account_threshold(threshold: u32) {
    for institution_code in [PERSONAL_CODE, PUBLIC_CODE, PRIVATE_CODE] {
        ActiveDynamicThresholds::<Test>::insert(
            institution_code,
            registered_account_institution(),
            threshold,
        );
    }
}

fn set_registered_admin_list_override(admins: Vec<AccountId32>) {
    REGISTERED_ADMIN_LIST_OVERRIDE.with(|value| *value.borrow_mut() = Some(admins));
}

impl InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        let who_bytes = who.encode();
        if who_bytes.len() != 32 {
            return false;
        }
        let mut who_arr = [0u8; 32];
        who_arr.copy_from_slice(&who_bytes);

        match institution_code {
            NRC | PRC => CHINA_CB
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false),
            PRB => CHINA_CH
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false),
            NJD => CHINA_SF
                .first()
                .filter(|n| AccountId32::new(n.main_account) == institution)
                .map(|_| {
                    NATIONAL_JUDICIAL_YUAN_ADMINS
                        .iter()
                        .any(|admin| *admin == who_arr)
                })
                .unwrap_or(false),
            PERMANENT_SINGLETON_CODE if institution == permanent_singleton_institution() => {
                (0..3).any(|index| permanent_singleton_admin(index) == *who)
            }
            PERSONAL_CODE | PUBLIC_CODE | PRIVATE_CODE => {
                institution == registered_account_institution()
                    && [
                        registered_account_admin(0),
                        registered_account_admin(1),
                        registered_account_admin(2),
                    ]
                    .iter()
                    .any(|admin| admin == who)
            }
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
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| n.admins.iter().copied().map(AccountId32::new).collect()),
            PRB => CHINA_CH
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| n.admins.iter().copied().map(AccountId32::new).collect()),
            NJD => CHINA_SF
                .first()
                .filter(|n| AccountId32::new(n.main_account) == institution)
                .map(|_| {
                    NATIONAL_JUDICIAL_YUAN_ADMINS
                        .iter()
                        .copied()
                        .map(AccountId32::new)
                        .collect()
                }),
            PERMANENT_SINGLETON_CODE if institution == permanent_singleton_institution() => {
                Some((0..3).map(permanent_singleton_admin).collect())
            }
            PERSONAL_CODE | PUBLIC_CODE | PRIVATE_CODE
                if institution == registered_account_institution() =>
            {
                let override_admins =
                    REGISTERED_ADMIN_LIST_OVERRIDE.with(|value| value.borrow().clone());
                Some(override_admins.unwrap_or_else(|| {
                    sp_std::vec![
                        registered_account_admin(0),
                        registered_account_admin(1),
                        registered_account_admin(2),
                    ]
                }))
            }
            _ => None,
        }
    }

    fn is_pending_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        is_registered_multisig_code(&institution_code)
            && institution == pending_account_institution()
            && [pending_account_admin(0), pending_account_admin(1)]
                .iter()
                .any(|admin| admin == who)
    }

    fn get_pending_admin_list(
        institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<sp_std::vec::Vec<AccountId32>> {
        if !is_registered_multisig_code(&institution_code)
            || institution != pending_account_institution()
        {
            return None;
        }
        Some(sp_std::vec![
            pending_account_admin(0),
            pending_account_admin(1)
        ])
    }
}

const DEFAULT_TEST_NOW_SECS: u64 = 1_782_864_000;

/// 测试用时间提供器：默认返回 2026 年中，可由单测覆盖为指定 UTC 秒。
pub struct TestTimeProvider;
impl frame_support::traits::UnixTime for TestTimeProvider {
    fn now() -> core::time::Duration {
        TEST_NOW_SECS.with(|secs| core::time::Duration::from_secs(*secs.borrow()))
    }
}
impl votingengine::CitizenIdentityReader<AccountId32> for TestCitizenIdentityReader {
    fn can_vote(who: &AccountId32, _scope: &votingengine::PopulationScope) -> bool {
        who == &nrc_admin(0)
    }

    fn can_be_candidate(who: &AccountId32, scope: &votingengine::PopulationScope) -> bool {
        Self::can_vote(who, scope)
    }

    fn population_count(_scope: &votingengine::PopulationScope) -> u64 {
        TEST_POPULATION_COUNT.with(|count| *count.borrow())
    }
}

impl JointVoteResultCallback for TestJointVoteResultCallback {
    fn on_joint_vote_finalized(
        vote_proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        if JOINT_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow()) {
            Err(DispatchError::Other("joint callback failed"))
        } else {
            if let Some(status) = JOINT_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow()) {
                return Ok(match status {
                    STATUS_EXECUTED => ProposalExecutionOutcome::Executed,
                    STATUS_EXECUTION_FAILED => ProposalExecutionOutcome::FatalFailed,
                    STATUS_PASSED => ProposalExecutionOutcome::RetryableFailed,
                    _ => ProposalExecutionOutcome::Ignored,
                });
            }
            let _ = vote_proposal_id;
            Ok(if approved {
                ProposalExecutionOutcome::Executed
            } else {
                ProposalExecutionOutcome::Executed
            })
        }
    }
}

impl InternalVoteResultCallback for TestInternalVoteResultCallback {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        // 先记日志；thread_local 不参与 storage transaction，仅用于确认回调调用次数。
        INTERNAL_CALLBACK_LOG.with(|log| log.borrow_mut().push((proposal_id, approved)));
        if INTERNAL_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow()) {
            Err(DispatchError::Other("internal callback failed"))
        } else {
            if let Some(status) = INTERNAL_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow()) {
                return Ok(match status {
                    STATUS_EXECUTED => ProposalExecutionOutcome::Executed,
                    STATUS_EXECUTION_FAILED => ProposalExecutionOutcome::FatalFailed,
                    STATUS_PASSED => ProposalExecutionOutcome::RetryableFailed,
                    _ => ProposalExecutionOutcome::Ignored,
                });
            }
            Ok(if approved {
                ProposalExecutionOutcome::RetryableFailed
            } else {
                ProposalExecutionOutcome::Executed
            })
        }
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        if INTERNAL_TERMINAL_CLEANUP_SHOULD_FAIL.with(|flag| *flag.borrow()) {
            return Err(DispatchError::Other("internal terminal cleanup failed"));
        }
        INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow_mut().push(proposal_id));
        Ok(())
    }
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("frame system genesis storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        TEST_POPULATION_COUNT.with(|count| *count.borrow_mut() = 100);
        TEST_NOW_SECS.with(|secs| *secs.borrow_mut() = DEFAULT_TEST_NOW_SECS);
        JOINT_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = false);
        JOINT_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = None);
        INTERNAL_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = false);
        INTERNAL_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = None);
        INTERNAL_CALLBACK_LOG.with(|log| log.borrow_mut().clear());
        INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow_mut().clear());
        INTERNAL_TERMINAL_CLEANUP_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = false);
        REGISTERED_ADMIN_LIST_OVERRIDE.with(|value| *value.borrow_mut() = None);
        set_registered_account_threshold(3);
        System::set_block_number(1);
    });
    ext
}

fn nrc_pid() -> AccountId32 {
    AccountId32::new(CHINA_CB[0].main_account)
}

fn prc_pid() -> AccountId32 {
    AccountId32::new(CHINA_CB[1].main_account)
}

fn prb_pid() -> AccountId32 {
    AccountId32::new(CHINA_CH[0].main_account)
}

fn njd_pid() -> AccountId32 {
    AccountId32::new(CHINA_SF[0].main_account)
}

fn subject_cids_for(
    institution_code: InstitutionCode,
    institution: &AccountId32,
) -> sp_std::vec::Vec<sp_std::vec::Vec<u8>> {
    match institution_code {
        NRC | PRC => CHINA_CB
            .iter()
            .find(|entry| AccountId32::new(entry.main_account) == *institution)
            .map(|entry| sp_std::vec![entry.cid_number.as_bytes().to_vec()])
            .unwrap_or_default(),
        PRB => CHINA_CH
            .iter()
            .find(|entry| AccountId32::new(entry.main_account) == *institution)
            .map(|entry| sp_std::vec![entry.cid_number.as_bytes().to_vec()])
            .unwrap_or_default(),
        NJD => CHINA_SF
            .iter()
            .find(|entry| AccountId32::new(entry.main_account) == *institution)
            .map(|entry| sp_std::vec![entry.cid_number.as_bytes().to_vec()])
            .unwrap_or_default(),
        PERMANENT_SINGLETON_CODE => primitives::institution_constraints::singleton_institutions()
            .into_iter()
            .find(|item| item.code == PERMANENT_SINGLETON_CODE)
            .map(|item| sp_std::vec![item.cid_number.as_bytes().to_vec()])
            .unwrap_or_default(),
        PERSONAL_CODE => sp_std::vec::Vec::new(),
        PUBLIC_CODE => sp_std::vec![b"TEST-PUBLIC-CID".to_vec()],
        PRIVATE_CODE => sp_std::vec![b"TEST-PRIVATE-CID".to_vec()],
        _ => sp_std::vec![b"TEST-OTHER-CID".to_vec()],
    }
}

fn first_subject_cid_for(
    institution_code: InstitutionCode,
    institution: &AccountId32,
) -> votingengine::types::CidNumber {
    let raw = subject_cids_for(institution_code, institution)
        .into_iter()
        .next()
        .expect("institution cid should exist");
    votingengine::Pallet::<Test>::bound_subject_cid_numbers(sp_std::vec![raw])
        .expect("subject cid should be bounded")
        .pop()
        .expect("subject cid should be present")
}

fn internal_mutex_for(
    institution_code: InstitutionCode,
    institution: AccountId32,
) -> Option<votingengine::InternalProposalMutexState> {
    let subject = if institution_code == PERSONAL_CODE {
        votingengine::ProposalSubject::PersonalAccount(institution.clone())
    } else {
        votingengine::ProposalSubject::InstitutionCid(first_subject_cid_for(
            institution_code,
            &institution,
        ))
    };
    VotingEngine::internal_proposal_mutex(subject)
}

fn nrc_admin(index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CB[0].admins[index])
}

fn njd_admin(index: usize) -> AccountId32 {
    AccountId32::new(NATIONAL_JUDICIAL_YUAN_ADMINS[index])
}

fn all_prc_institutions() -> Vec<(AccountId32, AccountId32)> {
    CHINA_CB
        .iter()
        .skip(1)
        .map(|n| {
            (
                AccountId32::new(n.main_account),
                AccountId32::new(n.admins[0]),
            )
        })
        .collect()
}

fn all_prb_institutions() -> Vec<(AccountId32, AccountId32)> {
    CHINA_CH
        .iter()
        .map(|n| {
            (
                AccountId32::new(n.main_account),
                AccountId32::new(n.admins[0]),
            )
        })
        .collect()
}

fn prc_admin(index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CB[1].admins[index])
}

fn prb_admin(index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CH[0].admins[index])
}

fn institution_admins(institution: AccountId32) -> Vec<AccountId32> {
    CHINA_CB
        .iter()
        .find(|n| AccountId32::new(n.main_account) == institution)
        .map(|n| n.admins.iter().copied().map(AccountId32::new).collect())
        .or_else(|| {
            CHINA_CH
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| n.admins.iter().copied().map(AccountId32::new).collect())
        })
        .expect("institution should have admins")
}

fn institution_threshold(institution: AccountId32) -> usize {
    if institution == nrc_pid() {
        return primitives::count_const::NRC_INTERNAL_THRESHOLD as usize;
    }
    if CHINA_CB
        .iter()
        .skip(1)
        .any(|n| AccountId32::new(n.main_account) == institution)
    {
        return primitives::count_const::PRC_INTERNAL_THRESHOLD as usize;
    }
    if CHINA_CH
        .iter()
        .any(|n| AccountId32::new(n.main_account) == institution)
    {
        return primitives::count_const::PRB_INTERNAL_THRESHOLD as usize;
    }
    panic!("unknown institution");
}

fn cast_joint_votes_until_finalized(proposal_id: u64, institution: AccountId32, approve: bool) {
    let admins = institution_admins(institution.clone());
    let threshold = institution_threshold(institution.clone());
    let required_votes = if approve {
        threshold
    } else {
        admins.len().saturating_sub(threshold).saturating_add(1)
    };
    for admin in admins.into_iter().take(required_votes) {
        assert_ok!(submit_joint_vote(
            admin,
            proposal_id,
            institution.clone(),
            approve
        ));
    }
    let now = System::block_number();
    <VotingEngine as Hooks<u64>>::on_initialize(now);
}

fn submit_joint_vote(
    who: AccountId32,
    proposal_id: u64,
    institution: AccountId32,
    approve: bool,
) -> DispatchResult {
    // 测试 helper 直调底层 do_joint_vote,绕过 extrinsic 签名包装。
    <joint_vote::Pallet<Test>>::do_joint_vote(who, proposal_id, institution, approve)
}

fn prepare_population_snapshot_for(who: AccountId32, eligible_total: u64) {
    TEST_POPULATION_COUNT.with(|count| *count.borrow_mut() = eligible_total);
    assert_ok!(JointVote::prepare_joint_population_snapshot(
        RuntimeOrigin::signed(who),
        votingengine::PopulationScope::Country,
    ));
}

fn create_joint_proposal_for(who: AccountId32, eligible_total: u64) -> u64 {
    prepare_population_snapshot_for(who.clone(), eligible_total);
    <JointVote as JointVoteEngine<AccountId32>>::create_joint_proposal(who)
        .expect("joint proposal should be created")
}

fn set_joint_callback_should_fail(should_fail: bool) {
    JOINT_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = should_fail);
}

fn set_joint_callback_override_status(status: Option<u8>) {
    JOINT_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = status);
}

fn set_internal_callback_override_status(status: Option<u8>) {
    INTERNAL_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = status);
}

fn set_internal_terminal_cleanup_should_fail(should_fail: bool) {
    INTERNAL_TERMINAL_CLEANUP_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = should_fail);
}

fn set_test_now_secs(secs: u64) {
    TEST_NOW_SECS.with(|value| *value.borrow_mut() = secs);
}

fn create_internal_proposal_via_engine(
    who: AccountId32,
    institution_code: InstitutionCode,
    institution: AccountId32,
) -> u64 {
    <InternalVote as InternalVoteEngine<AccountId32>>::create_general_internal_proposal_with_data(
        who,
        institution_code,
        institution.clone(),
        subject_cids_for(institution_code, &institution),
        b"test",
        b"payload".to_vec(),
    )
    .expect("internal proposal should be created")
}

fn create_pending_account_proposal_via_engine(
    who: AccountId32,
    institution_code: InstitutionCode,
    institution: AccountId32,
) -> u64 {
    <InternalVote as InternalVoteEngine<AccountId32>>::create_registered_account_create_proposal_with_data(
        who,
        institution_code,
        institution.clone(),
        subject_cids_for(institution_code, &institution),
        sp_std::vec![pending_account_admin(0), pending_account_admin(1)],
        2,
        b"test",
        b"payload".to_vec(),
    )
    .expect("pending account proposal should be created")
}

fn create_admin_set_mutation_proposal_via_engine(
    who: AccountId32,
    institution_code: InstitutionCode,
    institution: AccountId32,
) -> u64 {
    let new_threshold = if is_registered_multisig_code(&institution_code) {
        2
    } else {
        votingengine::types::fixed_governance_pass_threshold(&institution_code)
            .expect("fixed threshold")
    };
    <InternalVote as InternalVoteEngine<AccountId32>>::create_admin_change_internal_proposal_with_data(
        who,
        institution_code,
        institution.clone(),
        subject_cids_for(institution_code, &institution),
        <TestInternalAdminProvider as InternalAdminProvider<AccountId32>>::get_admin_list(
            institution_code,
            institution,
        )
        .map(|admins| admins.len() as u32)
        .unwrap_or(3),
        new_threshold,
        b"test",
        b"payload".to_vec(),
    )
    .expect("admin-set mutation proposal should be created")
}

/// 测试辅助:走公开 `internal_vote` extrinsic 投票。
///
/// 管理员投票只能通过公开 call,
/// 此函数包裹 `RuntimeOrigin::signed(who)` 让测试代码保持简洁。
fn cast_internal_vote_via_extrinsic(
    who: AccountId32,
    proposal_id: u64,
    approve: bool,
) -> DispatchResult {
    // 测试 helper 直调底层 do_internal_vote；保留 extrinsic dispatch 的事务语义。
    let result = frame_support::storage::with_transaction(
        || -> frame_support::storage::TransactionOutcome<DispatchResult> {
            match Pallet::<Test>::do_internal_vote(who, proposal_id, approve) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
            }
        },
    );
    if result.is_ok() {
        let now = System::block_number();
        <VotingEngine as Hooks<u64>>::on_initialize(now);
    }
    result
}

/// 执行当前区块的维护钩子，让 PASSED 提案进入一次异步业务执行尝试。
fn process_current_block() {
    let now = System::block_number();
    <VotingEngine as Hooks<u64>>::on_initialize(now);
}

fn insert_joint_referendum_proposal(proposal_id: u64, eligible_total: u64, end: u64) {
    Proposals::<Test>::insert(
        proposal_id,
        Proposal {
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_REFERENDUM,
            status: STATUS_VOTING,
            internal_code: None,
            account_context: None,
            subject_cid_numbers: Default::default(),
            start: System::block_number(),
            end,
            citizen_eligible_total: eligible_total,
        },
    );
    joint_vote::ReferendumScopes::<Test>::insert(
        proposal_id,
        votingengine::PopulationScope::Country,
    );
}

fn full_retry_deadline_bucket(seed: u64) -> BoundedVec<u64, ConstU32<128>> {
    (0..128u64)
        .map(|index| 2_000_000 + seed.saturating_mul(128) + index)
        .collect::<Vec<_>>()
        .try_into()
        .expect("test retry deadline bucket should fit capacity")
}

fn exhaust_cleanup_sequence(_current_block: u64) {
    ScheduledCleanupTail::<Test>::put(u64::MAX);
}

fn reset_cleanup_sequence(_current_block: u64) {
    ScheduledCleanupTail::<Test>::put(0);
}

fn fill_retry_deadline_window(from: u64) {
    for offset in 0..100u64 {
        ExecutionRetryDeadlines::<Test>::insert(from + offset, full_retry_deadline_bucket(offset));
    }
}

fn clear_retry_deadline_window(from: u64) {
    for offset in 0..100u64 {
        ExecutionRetryDeadlines::<Test>::remove(from + offset);
    }
}

mod cases;
mod dual_id;
