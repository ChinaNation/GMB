use super::*;
use codec::Encode;
use core::cell::RefCell;
use primitives::derive::subject_id_from_sfid_number;
use std::collections::BTreeSet;

use frame_support::{
    assert_noop, assert_ok, derive_impl, traits::ConstU32, traits::Hooks, BoundedVec,
};
use frame_system as system;

// 引擎核心 storage / 类型 / trait(住在 votingengine 主 crate)。
// `use super::*` 拉进 internal-vote 自有的 pallet items(Pallet/Event/Error/Config/InternalVotesByAccount/...);
// 这里追加 votingengine 的 storage 与 trait 名,让测试代码用短名引用。
use votingengine::pallet::{
    CleanupQueue, CurrentProposalYear, ExecutionRetryDeadlines, NextProposalId,
    PendingExecutionRetryExpirations, PendingExpiryBucket, PendingProposalCleanups,
    ProposalDisplayId, ProposalExecutionRetryStates, Proposals, ProposalsByExpiry,
    ProposalsByInstitution, ProposalsByOrg, ProposalsByOwner, ProposalsByYear, YearProposalCounter,
};
// joint mode storage 在 joint-vote sub-pallet
use primitives::china::china_cb::CHINA_CB;
use primitives::china::china_ch::CHINA_CH;
use sp_runtime::{traits::Hash, traits::IdentityLookup, AccountId32, BuildStorage, DispatchError};
use votingengine::traits::{
    InternalAdminProvider, InternalThresholdProvider, InternalVoteEngine,
    InternalVoteResultCallback, JointVoteEngine, JointVoteResultCallback,
    PopulationSnapshotVerifier,
};
use votingengine::SfidEligibility;
use votingengine::{
    PendingCleanupStage, Proposal, ProposalExecutionOutcome, SubjectId, VoteCountU32, VoteCountU64,
    VoteCredentialCleanup, PROPOSAL_KIND_INTERNAL, PROPOSAL_KIND_JOINT, STAGE_INTERNAL,
    STAGE_JOINT, STAGE_REFERENDUM, STATUS_EXECUTED, STATUS_EXECUTION_FAILED, STATUS_PASSED,
    STATUS_REJECTED, STATUS_VOTING,
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
    type MaxProposalsPerExpiry = ConstU32<128>;
    type MaxInternalProposalMutexBindings = ConstU32<256>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxCleanupStepsPerBlock = ConstU32<3>;
    type MaxCleanupQueueBucketLimit = ConstU32<50>;
    type MaxCleanupScheduleOffset = ConstU32<100>;
    type CleanupKeysPerStep = ConstU32<2>;
    type MaxProposalDataLen = ConstU32<4096>;
    type MaxProposalObjectLen = ConstU32<10_240>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type SfidEligibility = TestSfidEligibility;
    type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
    type JointVoteResultCallback = TestJointVoteResultCallback;
    type InternalVoteResultCallback = TestInternalVoteResultCallback;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminCountProvider = ();
    type InternalThresholdProvider = TestInternalThresholdProvider;
    type MaxAdminsPerInstitution = ConstU32<32>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
    type InternalFinalizer = InternalVote;
    type InternalCleanup = InternalVote;
    type JointFinalizer = JointVote;
    type JointCleanup = JointVote;
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
    static USED_VOTE_NONCES: RefCell<BTreeSet<(u64, Vec<u8>, Vec<u8>)>> = RefCell::new(BTreeSet::new());
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
// Phase 1 新增:内部投票终态回调测试桩。
// INTERNAL_CALLBACK_SHOULD_FAIL = true → on_internal_vote_finalized 返回 Err,
//   触发 set_status_and_emit 回滚;用于验证事务原子性。
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
    static REGISTERED_DUOQIAN_THRESHOLD: RefCell<u32> = const { RefCell::new(3) };
}
thread_local! {
    static PENDING_DUOQIAN_THRESHOLD: RefCell<u32> = const { RefCell::new(2) };
}
thread_local! {
    static REGISTERED_ADMIN_LIST_OVERRIDE: RefCell<Option<Vec<AccountId32>>> = const { RefCell::new(None) };
}

pub struct TestSfidEligibility;
pub struct TestPopulationSnapshotVerifier;
pub struct TestJointVoteResultCallback;
pub struct TestInternalVoteResultCallback;
pub struct TestInternalAdminProvider;
pub struct TestInternalThresholdProvider;

fn pending_subject_institution() -> SubjectId {
    [77u8; 48]
}

fn pending_subject_admin(index: usize) -> AccountId32 {
    match index {
        0 => AccountId32::new([91u8; 32]),
        1 => AccountId32::new([92u8; 32]),
        _ => AccountId32::new([93u8; 32]),
    }
}

fn registered_subject_institution() -> SubjectId {
    [78u8; 48]
}

fn registered_subject_admin(index: usize) -> AccountId32 {
    match index {
        0 => AccountId32::new([81u8; 32]),
        1 => AccountId32::new([82u8; 32]),
        _ => AccountId32::new([83u8; 32]),
    }
}

fn set_registered_duoqian_threshold(threshold: u32) {
    REGISTERED_DUOQIAN_THRESHOLD.with(|value| *value.borrow_mut() = threshold);
}

fn set_pending_duoqian_threshold(threshold: u32) {
    PENDING_DUOQIAN_THRESHOLD.with(|value| *value.borrow_mut() = threshold);
}

fn set_registered_admin_list_override(admins: Vec<AccountId32>) {
    REGISTERED_ADMIN_LIST_OVERRIDE.with(|value| *value.borrow_mut() = Some(admins));
}

impl InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(org: u8, institution: SubjectId, who: &AccountId32) -> bool {
        let who_bytes = who.encode();
        if who_bytes.len() != 32 {
            return false;
        }
        let mut who_arr = [0u8; 32];
        who_arr.copy_from_slice(&who_bytes);

        match org {
            ORG_NRC | ORG_PRC => CHINA_CB
                .iter()
                .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
                .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false),
            ORG_PRB => CHINA_CH
                .iter()
                .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
                .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false),
            ORG_REN => {
                institution == registered_subject_institution()
                    && [
                        registered_subject_admin(0),
                        registered_subject_admin(1),
                        registered_subject_admin(2),
                    ]
                    .iter()
                    .any(|admin| admin == who)
            }
            _ => false,
        }
    }

    fn get_admin_list(org: u8, institution: SubjectId) -> Option<sp_std::vec::Vec<AccountId32>> {
        match org {
            ORG_NRC | ORG_PRC => CHINA_CB
                .iter()
                .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
                .map(|n| {
                    n.duoqian_admins
                        .iter()
                        .copied()
                        .map(AccountId32::new)
                        .collect()
                }),
            ORG_PRB => CHINA_CH
                .iter()
                .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
                .map(|n| {
                    n.duoqian_admins
                        .iter()
                        .copied()
                        .map(AccountId32::new)
                        .collect()
                }),
            ORG_REN if institution == registered_subject_institution() => {
                let override_admins =
                    REGISTERED_ADMIN_LIST_OVERRIDE.with(|value| value.borrow().clone());
                Some(override_admins.unwrap_or_else(|| {
                    sp_std::vec![
                        registered_subject_admin(0),
                        registered_subject_admin(1),
                        registered_subject_admin(2),
                    ]
                }))
            }
            _ => None,
        }
    }

    fn is_pending_internal_admin(org: u8, institution: SubjectId, who: &AccountId32) -> bool {
        org == ORG_REN
            && institution == pending_subject_institution()
            && [pending_subject_admin(0), pending_subject_admin(1)]
                .iter()
                .any(|admin| admin == who)
    }

    fn get_pending_admin_list(
        org: u8,
        institution: SubjectId,
    ) -> Option<sp_std::vec::Vec<AccountId32>> {
        if org != ORG_REN || institution != pending_subject_institution() {
            return None;
        }
        Some(sp_std::vec![
            pending_subject_admin(0),
            pending_subject_admin(1)
        ])
    }
}

impl InternalThresholdProvider for TestInternalThresholdProvider {
    fn is_known_subject(org: u8, institution: SubjectId) -> bool {
        org == ORG_REN && institution == registered_subject_institution()
    }

    fn is_known_pending_subject(org: u8, institution: SubjectId) -> bool {
        org == ORG_REN && institution == pending_subject_institution()
    }

    fn pass_threshold(org: u8, institution: SubjectId) -> Option<u32> {
        if org == ORG_REN && institution == registered_subject_institution() {
            return REGISTERED_DUOQIAN_THRESHOLD.with(|value| Some(*value.borrow()));
        }
        // 中文注释：治理机构返回“毒化阈值”，用于证明治理投票不再依赖动态 Provider。
        if matches!(org, ORG_NRC | ORG_PRC | ORG_PRB) {
            return Some(1);
        }
        None
    }

    fn pending_pass_threshold(org: u8, institution: SubjectId) -> Option<u32> {
        if org != ORG_REN || institution != pending_subject_institution() {
            return None;
        }
        PENDING_DUOQIAN_THRESHOLD.with(|value| Some(*value.borrow()))
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
impl
    PopulationSnapshotVerifier<
        AccountId32,
        votingengine::pallet::VoteNonceOf<Test>,
        votingengine::pallet::VoteSignatureOf<Test>,
    > for TestPopulationSnapshotVerifier
{
    fn verify_population_snapshot(
        _who: &AccountId32,
        eligible_total: u64,
        nonce: &votingengine::pallet::VoteNonceOf<Test>,
        signature: &votingengine::pallet::VoteSignatureOf<Test>,
        province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        eligible_total > 0
            && !nonce.is_empty()
            && signature.as_slice() == b"snapshot-ok"
            && !province.is_empty()
    }
}

impl SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash> for TestSfidEligibility {
    fn is_eligible(binding_id: &<Test as frame_system::Config>::Hash, who: &AccountId32) -> bool {
        *binding_id == binding_id_ok() && who == &nrc_admin(0)
    }

    fn verify_and_consume_vote_credential(
        binding_id: &<Test as frame_system::Config>::Hash,
        who: &AccountId32,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
        province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        if !Self::is_eligible(binding_id, who)
            || signature != b"vote-ok"
            || nonce.is_empty()
            || province.is_empty()
        {
            return false;
        }
        let key = (proposal_id, binding_id.encode(), nonce.to_vec());
        USED_VOTE_NONCES.with(|set| {
            let mut set = set.borrow_mut();
            if set.contains(&key) {
                false
            } else {
                set.insert(key);
                true
            }
        })
    }

    fn cleanup_vote_credentials(proposal_id: u64) {
        USED_VOTE_NONCES.with(|set| {
            set.borrow_mut().retain(|(pid, _, _)| *pid != proposal_id);
        });
    }

    fn cleanup_vote_credentials_chunk(proposal_id: u64, limit: u32) -> VoteCredentialCleanup {
        let mut to_remove = Vec::new();
        USED_VOTE_NONCES.with(|set| {
            for key in set.borrow().iter() {
                if key.0 == proposal_id {
                    to_remove.push(key.clone());
                    if to_remove.len() >= limit as usize {
                        break;
                    }
                }
            }
        });

        let has_remaining = USED_VOTE_NONCES.with(|set| {
            let mut set = set.borrow_mut();
            for key in &to_remove {
                set.remove(key);
            }
            set.iter().any(|(pid, _, _)| *pid == proposal_id)
        });

        VoteCredentialCleanup {
            removed: to_remove.len() as u32,
            loops: to_remove.len() as u32,
            has_remaining,
        }
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
        // 先记日志,无论成功/失败都记 — 事务回滚会让日志外的状态回退,但
        // thread_local 不参与事务,通过对比"日志有/状态没变"即可验证回滚语义。
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
        USED_VOTE_NONCES.with(|set| set.borrow_mut().clear());
        TEST_NOW_SECS.with(|secs| *secs.borrow_mut() = DEFAULT_TEST_NOW_SECS);
        JOINT_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = false);
        JOINT_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = None);
        INTERNAL_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = false);
        INTERNAL_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = None);
        INTERNAL_CALLBACK_LOG.with(|log| log.borrow_mut().clear());
        INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow_mut().clear());
        REGISTERED_DUOQIAN_THRESHOLD.with(|value| *value.borrow_mut() = 3);
        PENDING_DUOQIAN_THRESHOLD.with(|value| *value.borrow_mut() = 2);
        REGISTERED_ADMIN_LIST_OVERRIDE.with(|value| *value.borrow_mut() = None);
        System::set_block_number(1);
    });
    ext
}

fn nrc_pid() -> SubjectId {
    subject_id_from_sfid_number(CHINA_CB[0].sfid_number)
        .expect("nrc id should be sfid_number bytes")
}

fn prc_pid() -> SubjectId {
    subject_id_from_sfid_number(CHINA_CB[1].sfid_number)
        .expect("prc id should be sfid_number bytes")
}

fn prb_pid() -> SubjectId {
    subject_id_from_sfid_number(CHINA_CH[0].sfid_number)
        .expect("prb id should be sfid_number bytes")
}

fn nrc_admin(index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CB[0].duoqian_admins[index])
}

fn all_prc_institutions() -> Vec<(SubjectId, AccountId32)> {
    CHINA_CB
        .iter()
        .skip(1)
        .map(|n| {
            (
                subject_id_from_sfid_number(n.sfid_number)
                    .expect("prc id should be sfid_number bytes"),
                AccountId32::new(n.duoqian_admins[0]),
            )
        })
        .collect()
}

fn all_prb_institutions() -> Vec<(SubjectId, AccountId32)> {
    CHINA_CH
        .iter()
        .map(|n| {
            (
                subject_id_from_sfid_number(n.sfid_number)
                    .expect("prb id should be sfid_number bytes"),
                AccountId32::new(n.duoqian_admins[0]),
            )
        })
        .collect()
}

fn prc_admin(index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CB[1].duoqian_admins[index])
}

fn prb_admin(index: usize) -> AccountId32 {
    AccountId32::new(CHINA_CH[0].duoqian_admins[index])
}

fn institution_admins(institution: SubjectId) -> Vec<AccountId32> {
    CHINA_CB
        .iter()
        .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
        .map(|n| {
            n.duoqian_admins
                .iter()
                .copied()
                .map(AccountId32::new)
                .collect()
        })
        .or_else(|| {
            CHINA_CH
                .iter()
                .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
                .map(|n| {
                    n.duoqian_admins
                        .iter()
                        .copied()
                        .map(AccountId32::new)
                        .collect()
                })
        })
        .expect("institution should have admins")
}

fn institution_threshold(institution: SubjectId) -> usize {
    if institution == nrc_pid() {
        return primitives::count_const::NRC_INTERNAL_THRESHOLD as usize;
    }
    if CHINA_CB
        .iter()
        .skip(1)
        .any(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
    {
        return primitives::count_const::PRC_INTERNAL_THRESHOLD as usize;
    }
    if CHINA_CH
        .iter()
        .any(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
    {
        return primitives::count_const::PRB_INTERNAL_THRESHOLD as usize;
    }
    panic!("unknown institution");
}

fn cast_joint_votes_until_finalized(proposal_id: u64, institution: SubjectId, approve: bool) {
    let admins = institution_admins(institution);
    let threshold = institution_threshold(institution);
    let required_votes = if approve {
        threshold
    } else {
        admins.len().saturating_sub(threshold).saturating_add(1)
    };
    for admin in admins.into_iter().take(required_votes) {
        assert_ok!(submit_joint_vote(admin, proposal_id, institution, approve));
    }
}

fn submit_joint_vote(
    who: AccountId32,
    proposal_id: u64,
    institution: SubjectId,
    approve: bool,
) -> DispatchResult {
    // 测试 helper 直调底层 do_joint_vote,绕过 extrinsic 签名包装。
    <joint_vote::Pallet<Test>>::do_joint_vote(who, proposal_id, institution, approve)
}

fn binding_id_ok() -> <Test as frame_system::Config>::Hash {
    <Test as frame_system::Config>::Hashing::hash(b"sfid-ok")
}

fn vote_nonce(input: &str) -> votingengine::pallet::VoteNonceOf<Test> {
    input
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("nonce should fit")
}

fn vote_sig_ok() -> votingengine::pallet::VoteSignatureOf<Test> {
    b"vote-ok"
        .to_vec()
        .try_into()
        .expect("signature should fit")
}

fn vote_sig_bad() -> votingengine::pallet::VoteSignatureOf<Test> {
    b"bad".to_vec().try_into().expect("signature should fit")
}

/// ADR-008 step3:测试用占位 province + signer_admin_pubkey,
/// `TestPopulationSnapshotVerifier` / `TestSfidEligibility` 仅做空字段非空检验,
/// 真实 sr25519 验签覆盖留 runtime 层。
fn province_ok() -> frame_support::BoundedVec<u8, frame_support::pallet_prelude::ConstU32<64>> {
    b"liaoning"
        .to_vec()
        .try_into()
        .expect("province should fit")
}

fn signer_admin_pubkey_ok() -> [u8; 32] {
    [7u8; 32]
}

fn snapshot_nonce_ok() -> votingengine::pallet::VoteNonceOf<Test> {
    b"snap-nonce"
        .to_vec()
        .try_into()
        .expect("snapshot nonce should fit")
}

fn snapshot_sig_ok() -> votingengine::pallet::VoteSignatureOf<Test> {
    b"snapshot-ok"
        .to_vec()
        .try_into()
        .expect("snapshot signature should fit")
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

fn set_test_now_secs(secs: u64) {
    TEST_NOW_SECS.with(|value| *value.borrow_mut() = secs);
}

fn mark_vote_nonce_used(
    proposal_id: u64,
    binding_id: <Test as frame_system::Config>::Hash,
    nonce: &str,
) {
    USED_VOTE_NONCES.with(|set| {
        set.borrow_mut()
            .insert((proposal_id, binding_id.encode(), nonce.as_bytes().to_vec()));
    });
}

fn has_used_vote_nonce(
    proposal_id: u64,
    binding_id: <Test as frame_system::Config>::Hash,
    nonce: &str,
) -> bool {
    USED_VOTE_NONCES.with(|set| {
        set.borrow()
            .contains(&(proposal_id, binding_id.encode(), nonce.as_bytes().to_vec()))
    })
}

fn create_internal_proposal_via_engine(who: AccountId32, org: u8, institution: SubjectId) -> u64 {
    <InternalVote as InternalVoteEngine<AccountId32>>::create_internal_proposal(
        who,
        org,
        institution,
    )
    .expect("internal proposal should be created")
}

fn create_pending_subject_proposal_via_engine(
    who: AccountId32,
    org: u8,
    institution: SubjectId,
) -> u64 {
    <InternalVote as InternalVoteEngine<AccountId32>>::create_pending_subject_internal_proposal(
        who,
        org,
        institution,
    )
    .expect("pending subject proposal should be created")
}

fn create_admin_set_mutation_proposal_via_engine(
    who: AccountId32,
    org: u8,
    institution: SubjectId,
) -> u64 {
    <InternalVote as InternalVoteEngine<AccountId32>>::create_admin_set_mutation_internal_proposal(
        who,
        org,
        institution,
    )
    .expect("admin-set mutation proposal should be created")
}

/// 测试辅助:走公开 `internal_vote` extrinsic 投票。
///
/// Phase 1 改造后,管理员投票只能通过公开 call(不再经 trait 转发),
/// 此函数包裹 `RuntimeOrigin::signed(who)` 让测试代码保持简洁。
fn cast_internal_vote_via_extrinsic(
    who: AccountId32,
    proposal_id: u64,
    approve: bool,
) -> DispatchResult {
    // 测试 helper 直调底层 do_internal_vote;extrinsic dispatch 的隐式 transactional
    // 语义需要手工 with_transaction 还原,否则 callback 返回 Err 时无法整体回滚票数与状态。
    frame_support::storage::with_transaction(
        || -> frame_support::storage::TransactionOutcome<DispatchResult> {
            match Pallet::<Test>::do_internal_vote(who, proposal_id, approve) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
            }
        },
    )
}

fn insert_citizen_proposal(proposal_id: u64, eligible_total: u64, end: u64) {
    Proposals::<Test>::insert(
        proposal_id,
        Proposal {
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_REFERENDUM,
            status: STATUS_VOTING,
            internal_org: None,
            internal_institution: None,
            start: System::block_number(),
            end,
            citizen_eligible_total: eligible_total,
        },
    );
}

fn cleanup_retention_blocks() -> u64 {
    90u64 * primitives::pow_const::BLOCKS_PER_DAY
}

fn full_cleanup_bucket(seed: u64) -> BoundedVec<u64, ConstU32<50>> {
    (0..50u64)
        .map(|index| 1_000_000 + seed.saturating_mul(50) + index)
        .collect::<Vec<_>>()
        .try_into()
        .expect("test cleanup bucket should fit capacity")
}

fn full_retry_deadline_bucket(seed: u64) -> BoundedVec<u64, ConstU32<128>> {
    (0..128u64)
        .map(|index| 2_000_000 + seed.saturating_mul(128) + index)
        .collect::<Vec<_>>()
        .try_into()
        .expect("test retry deadline bucket should fit capacity")
}

fn fill_cleanup_schedule_window(current_block: u64) {
    let base = current_block.saturating_add(cleanup_retention_blocks());
    for offset in 0..100u64 {
        CleanupQueue::<Test>::insert(base + offset, full_cleanup_bucket(offset));
    }
}

fn clear_cleanup_schedule_window(current_block: u64) {
    let base = current_block.saturating_add(cleanup_retention_blocks());
    for offset in 0..100u64 {
        CleanupQueue::<Test>::remove(base + offset);
    }
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
