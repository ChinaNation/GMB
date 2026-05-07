#![cfg(test)]

use super::*;
use codec::Encode;
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU128, ConstU32},
};
use frame_system as system;
use sp_core::{sr25519, Pair as PairT};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::{STATUS_EXECUTED, STATUS_REJECTED, STATUS_VOTING};

type Balance = u128;
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
    pub type Balances = pallet_balances;

    #[runtime::pallet_index(2)]
    pub type VotingEngine = votingengine;

    #[runtime::pallet_index(99)]
    pub type InternalVote = internal_vote;

    #[runtime::pallet_index(3)]
    pub type OrganizationManage = organization_manage;

    #[runtime::pallet_index(4)]
    pub type DuoqianTransfer = super;

    #[runtime::pallet_index(5)]
    pub type AdminsChange = admins_change;

    #[runtime::pallet_index(6)]
    pub type PersonalManage = personal_manage;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type AccountData = pallet_balances::AccountData<Balance>;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type MaxLocks = ConstU32<0>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = ConstU32<0>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
    type WeightInfo = ();
}

pub struct TestAddressValidator;
impl organization_manage::DuoqianAddressValidator<AccountId32> for TestAddressValidator {
    fn is_valid(address: &AccountId32) -> bool {
        address != &AccountId32::new([0u8; 32])
    }
}

pub struct TestReservedAddressChecker;
impl organization_manage::DuoqianReservedAddressChecker<AccountId32> for TestReservedAddressChecker {
    fn is_reserved(address: &AccountId32) -> bool {
        *address == AccountId32::new([0xAA; 32])
    }
}

pub struct TestSfidInstitutionVerifier;
impl
    organization_manage::SfidInstitutionVerifier<
        organization_manage::pallet::AccountNameOf<Test>,
        organization_manage::pallet::RegisterNonceOf<Test>,
        organization_manage::pallet::RegisterSignatureOf<Test>,
    > for TestSfidInstitutionVerifier
{
    fn verify_institution_registration(
        _sfid_number: &[u8],
        institution_name: &organization_manage::pallet::AccountNameOf<Test>,
        account_names: &[alloc::vec::Vec<u8>],
        nonce: &organization_manage::pallet::RegisterNonceOf<Test>,
        signature: &organization_manage::pallet::RegisterSignatureOf<Test>,
        province: &[u8],
        signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        !institution_name.is_empty()
            && !account_names.is_empty()
            && !nonce.is_empty()
            && !province.is_empty()
            && signer_admin_pubkey != &[0u8; 32]
            && signature.as_slice() == b"register-ok"
    }
}

pub struct TestSfidEligibility;
impl votingengine::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
    for TestSfidEligibility
{
    fn is_eligible(
        _binding_id: &<Test as frame_system::Config>::Hash,
        _who: &AccountId32,
    ) -> bool {
        true
    }

    fn verify_and_consume_vote_credential(
        _binding_id: &<Test as frame_system::Config>::Hash,
        _who: &AccountId32,
        _proposal_id: u64,
        _nonce: &[u8],
        _signature: &[u8],
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
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
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        true
    }
}

// 测试扩展:
// 原 TestInternalAdminProvider 只读 CHINA_CB/CHINA_CH 硬编码 admin(非真实 sr25519 公钥,无法签名)。
// 为支持 `internal_vote` 的可签名测试账户,新增 thread_local 覆盖层:
//   - EXTRA_ADMINS 按 (org, institution) 注入 sr25519 派生 admin 集合。
// NRC/PRC/PRB 的内部阈值是 votingengine 固定制度常量,测试必须注入足够管理员并投满该阈值。
// 若某 (org, institution) 在 thread_local 有注入,优先用;否则 fallback 到原硬编码逻辑。
thread_local! {
    static EXTRA_ADMINS: core::cell::RefCell<
        alloc::collections::BTreeMap<(u8, SubjectId), alloc::vec::Vec<AccountId32>>,
    > = core::cell::RefCell::new(alloc::collections::BTreeMap::new());
}

fn set_extra_admins(org: u8, institution: SubjectId, admins: Vec<AccountId32>) {
    EXTRA_ADMINS.with(|m| {
        m.borrow_mut().insert((org, institution), admins);
    });
}

fn get_extra_admins(org: u8, institution: SubjectId) -> Option<Vec<AccountId32>> {
    EXTRA_ADMINS.with(|m| m.borrow().get(&(org, institution)).cloned())
}

pub struct TestInternalAdminProvider;
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(org: u8, institution: SubjectId, who: &AccountId32) -> bool {
        // 优先:测试注入的 sr25519 派生 admin
        if let Some(admins) = get_extra_admins(org, institution) {
            return admins.iter().any(|a| a == who);
        }
        // Fallback:原硬编码 admin
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
                let Ok(account) = AccountId32::decode(&mut &institution[1..33]) else {
                    return false;
                };
                if let Some(duoqian) = personal_manage::PersonalDuoqians::<Test>::get(&account) {
                    duoqian.duoqian_admins.iter().any(|admin| admin == who)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn get_admin_list(org: u8, institution: SubjectId) -> Option<Vec<AccountId32>> {
        if let Some(admins) = get_extra_admins(org, institution) {
            return Some(admins);
        }
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
            ORG_REN => {
                let account = AccountId32::decode(&mut &institution[1..33]).ok()?;
                let duoqian = personal_manage::PersonalDuoqians::<Test>::get(&account)?;
                Some(duoqian.duoqian_admins.into_inner())
            }
            _ => None,
        }
    }
}

pub struct TestInternalAdminCountProvider;
impl votingengine::InternalAdminCountProvider for TestInternalAdminCountProvider {
    fn admin_count(org: u8, institution: SubjectId) -> Option<u32> {
        match org {
            ORG_NRC | ORG_PRC => CHINA_CB
                .iter()
                .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
                .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok()),
            ORG_PRB => CHINA_CH
                .iter()
                .find(|n| subject_id_from_sfid_number(n.sfid_number) == Some(institution))
                .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok()),
            ORG_REN => {
                let account = AccountId32::decode(&mut &institution[1..33]).ok()?;
                let duoqian = personal_manage::PersonalDuoqians::<Test>::get(&account)?;
                u32::try_from(duoqian.duoqian_admins.len()).ok()
            }
            _ => None,
        }
    }
}

pub struct TestInternalThresholdProvider;
impl votingengine::InternalThresholdProvider for TestInternalThresholdProvider {
    fn is_known_subject(org: u8, institution: SubjectId) -> bool {
        match org {
            ORG_REN => AccountId32::decode(&mut &institution[1..33])
                .ok()
                .and_then(|account| personal_manage::PersonalDuoqians::<Test>::get(&account))
                .is_some(),
            _ => false,
        }
    }

    fn pass_threshold(org: u8, institution: SubjectId) -> Option<u32> {
        match org {
            ORG_NRC | ORG_PRC | ORG_PRB => {
                votingengine::types::fixed_governance_pass_threshold(org)
            }
            ORG_REN => {
                let account = AccountId32::decode(&mut &institution[1..33]).ok()?;
                let duoqian = personal_manage::PersonalDuoqians::<Test>::get(&account)?;
                Some(duoqian.threshold)
            }
            _ => None,
        }
    }
}

thread_local! {
    static PROTECTED_ADDRESS: core::cell::RefCell<Option<AccountId32>> = core::cell::RefCell::new(None);
    static DENIED_SPEND_SOURCE: core::cell::RefCell<Option<AccountId32>> = core::cell::RefCell::new(None);
}

pub struct TestProtectedSourceChecker;
impl organization_manage::ProtectedSourceChecker<AccountId32> for TestProtectedSourceChecker {
    fn is_protected(address: &AccountId32) -> bool {
        PROTECTED_ADDRESS.with(|pa| pa.borrow().as_ref() == Some(address))
    }
}

pub struct TestInstitutionAsset;
impl institution_asset::InstitutionAsset<AccountId32> for TestInstitutionAsset {
    fn can_spend(
        source: &AccountId32,
        _action: institution_asset::InstitutionAssetAction,
    ) -> bool {
        DENIED_SPEND_SOURCE.with(|blocked| blocked.borrow().as_ref() != Some(source))
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
    type SfidEligibility = TestSfidEligibility;
    type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
    type JointVoteResultCallback = ();
    // Phase 2:挂上本模块 Executor,3 组业务提案通过后自动 try_execute_X。
    type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminCountProvider = TestInternalAdminCountProvider;
    type InternalThresholdProvider = TestInternalThresholdProvider;
    type MaxAdminsPerInstitution = ConstU32<64>;
    type MaxProposalDataLen = ConstU32<1024>;
    type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxCleanupQueueBucketLimit = ConstU32<50>;
    type MaxCleanupScheduleOffset = ConstU32<100>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
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

impl organization_manage::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type AddressValidator = TestAddressValidator;
    type ReservedAddressChecker = TestReservedAddressChecker;
    type ProtectedSourceChecker = TestProtectedSourceChecker;
    type InstitutionAsset = TestInstitutionAsset;
    type SfidInstitutionVerifier = TestSfidInstitutionVerifier;
    type FeeRouter = ();
    type MaxAdmins = ConstU32<10>;
    type MaxSfidNumberLength = ConstU32<47>;
    type MaxAccountNameLength = ConstU32<128>;
    type MaxRegisterNonceLength = ConstU32<64>;
    type MaxRegisterSignatureLength = ConstU32<64>;
    type MaxInstitutionAccounts = ConstU32<8>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<111>;
    type WeightInfo = ();
}

impl admins_change::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<64>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type WeightInfo = ();
}

impl personal_manage::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type AddressValidator = TestAddressValidator;
    type ReservedAddressChecker = TestReservedAddressChecker;
    type ProtectedSourceChecker = TestProtectedSourceChecker;
    type InstitutionAsset = TestInstitutionAsset;
    type FeeRouter = ();
    type MaxAdmins = ConstU32<10>;
    type MaxAccountNameLength = ConstU32<128>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<111>;
    type WeightInfo = ();
}

impl pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxRemarkLen = ConstU32<256>;
    type FeeRouter = ();
    // 中文注释:测试 mock 把所有"已注册"多签都灌进 personal-manage::PersonalDuoqians 表,
    // 因此 PersonalQuery 走 personal_manage::Pallet<Test> 命中;
    // InstitutionQuery 走单元桩 ()(测试 fixture 不构造 SFID 注册路径)。
    type PersonalQuery = personal_manage::Pallet<Test>;
    type InstitutionQuery = ();
    type WeightInfo = ();
}

/// 测试 helper:从 (org, institution, index) 派生 sr25519 keypair。
///
/// 同 (org, institution, index) 每次调用返回相同 keypair,保证测试确定性。
/// 公钥的 32 字节直接作为 AccountId32,满足 `pubkey_from_accountid` 的铁律。
fn derive_admin_pair(
    org: u8,
    institution: &SubjectId,
    index: u8,
) -> (AccountId32, sr25519::Pair) {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = org;
    seed_bytes[1] = index;
    // 后 30 字节由 institution_pallet_id 前 30 字节填充,保证不同机构的 seed 不同
    seed_bytes[2..32].copy_from_slice(&institution[..30]);
    let pair = sr25519::Pair::from_seed(&seed_bytes);
    let account = AccountId32::new(pair.public().0);
    (account, pair)
}

fn nrc_admin(index: usize) -> AccountId32 {
    derive_admin_pair(ORG_NRC, &nrc_pallet_id(), index as u8).0
}

fn prc_admin(index: usize) -> AccountId32 {
    derive_admin_pair(ORG_PRC, &prc_pallet_id(), index as u8).0
}

fn prb_admin(index: usize) -> AccountId32 {
    derive_admin_pair(ORG_PRB, &prb_pallet_id(), index as u8).0
}

// 统一状态机整改:业务模块不再持有独立 vote/finalize call,投票统一走
// `InternalVote::cast`;`cast_transfer_votes_n` 直接用 admin 账户逐个投票。

fn nrc_pallet_id() -> SubjectId {
    subject_id_from_sfid_number(CHINA_CB[0].sfid_number).expect("nrc id should be valid")
}

fn prc_pallet_id() -> SubjectId {
    subject_id_from_sfid_number(CHINA_CB[1].sfid_number).expect("prc id should be valid")
}

fn prb_pallet_id() -> SubjectId {
    subject_id_from_sfid_number(CHINA_CH[0].sfid_number).expect("prb id should be valid")
}

fn institution_account(institution: SubjectId) -> AccountId32 {
    let raw =
        subject_pallet_address(institution).expect("institution pallet address must exist");
    AccountId32::new(raw)
}

fn registered_duoqian_account() -> AccountId32 {
    AccountId32::new([0x55; 32])
}

fn registered_duoqian_institution() -> SubjectId {
    primitives::derive::subject_id_from_account(&registered_duoqian_account())
}

fn registered_duoqian_admin(index: usize) -> AccountId32 {
    registered_duoqian_pair(index).0
}

/// 注册多签(ORG_REN)的 admin sr25519 keypair helper。
/// seed 按 (ORG_REN, registered_duoqian_institution, index) 派生,保证确定性。
fn registered_duoqian_pair(index: usize) -> (AccountId32, sr25519::Pair) {
    derive_admin_pair(ORG_REN, &registered_duoqian_institution(), index as u8)
}

fn registered_duoqian_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    (0..count)
        .map(|i| registered_duoqian_pair(i as usize))
        .collect()
}

/// 收款人：使用一个不是管理员也不是机构的普通地址
fn beneficiary() -> AccountId32 {
    AccountId32::new([99u8; 32])
}

/// 获取最近一次 create_internal_proposal 分配的 proposal_id。
fn last_proposal_id() -> u64 {
    votingengine::Pallet::<Test>::next_proposal_id().saturating_sub(1)
}

/// 返回 (org, institution) 对应的前 `count` 个 sr25519 admin keypair。
fn admin_pairs(
    org: u8,
    institution: SubjectId,
    count: u8,
) -> Vec<(AccountId32, sr25519::Pair)> {
    (0..count)
        .map(|i| derive_admin_pair(org, &institution, i))
        .collect()
}

fn nrc_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    admin_pairs(ORG_NRC, nrc_pallet_id(), count)
}

fn prc_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    admin_pairs(ORG_PRC, prc_pallet_id(), count)
}

fn prb_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    admin_pairs(ORG_PRB, prb_pallet_id(), count)
}

fn nrc_pass_count() -> usize {
    primitives::count_const::NRC_INTERNAL_THRESHOLD as usize
}

fn prc_pass_count() -> usize {
    primitives::count_const::PRC_INTERNAL_THRESHOLD as usize
}

fn prb_pass_count() -> usize {
    primitives::count_const::PRB_INTERNAL_THRESHOLD as usize
}

fn nrc_pass_pairs() -> Vec<(AccountId32, sr25519::Pair)> {
    nrc_pairs(primitives::count_const::NRC_INTERNAL_THRESHOLD as u8)
}

fn prc_pass_pairs() -> Vec<(AccountId32, sr25519::Pair)> {
    prc_pairs(primitives::count_const::PRC_INTERNAL_THRESHOLD as u8)
}

fn prb_pass_pairs() -> Vec<(AccountId32, sr25519::Pair)> {
    prb_pairs(primitives::count_const::PRB_INTERNAL_THRESHOLD as u8)
}

/// Phase 2 测试辅助:走投票引擎公开 `internal_vote` extrinsic,
/// 让 `pairs` 前 `n` 个成员各投一张赞成票。
///
/// 替代旧的聚合签名 helper——业务模块不再持有独立 finalize call,
/// 通过后由 [`InternalVoteExecutor`] 自动触发 `try_execute_transfer`。
/// 多余参数(`_org` / `_institution` / `_from` / `_to` /
/// `_amount` / `_remark` / `_proposer`)保留占位,让调用端旧语义透明迁移。
fn cast_transfer_votes_n(
    pairs: &[(AccountId32, sr25519::Pair)],
    n: usize,
    pid: u64,
    _org: u8,
    _institution: SubjectId,
    _from: AccountId32,
    _to: AccountId32,
    _amount: Balance,
    _remark: &[u8],
    _proposer: AccountId32,
) -> frame_support::dispatch::DispatchResult {
    for (admin, _pair) in pairs.iter().take(n) {
        <internal_vote::Pallet<Test>>::do_internal_vote(admin.clone(), pid, true)?;
        if VotingEngine::proposals(pid)
            .map(|proposal| proposal.status != STATUS_VOTING)
            .unwrap_or(true)
        {
            break;
        }
    }
    Ok(())
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");

    let balances = vec![
        (institution_account(nrc_pallet_id()), 10_000),
        (institution_account(prc_pallet_id()), 10_000),
        (institution_account(prb_pallet_id()), 10_000),
    ];
    pallet_balances::GenesisConfig::<Test> {
        balances,
        ..Default::default()
    }
    .assimilate_storage(&mut storage)
    .expect("balances should assimilate");
    admins_change::GenesisConfig::<Test>::default()
        .assimilate_storage(&mut storage)
        .expect("admins-change genesis should assimilate");

    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| {
        // 为 3 种固定治理 org 注入 sr25519 派生 admin。
        // 注入数量必须覆盖 votingengine 的固定制度阈值,保证投票测试走真实状态机。
        // Provider 的 is_internal_admin / get_admin_list 会优先读 thread_local 注入,
        // 未注入时 fallback 到 CHINA_CB / CHINA_CH 硬编码。
        let nrc = nrc_pallet_id();
        let prc = prc_pallet_id();
        let prb = prb_pallet_id();
        let dq = registered_duoqian_institution();
        let nrc_accts: Vec<AccountId32> =
            nrc_pass_pairs().into_iter().map(|(a, _)| a).collect();
        let prc_accts: Vec<AccountId32> =
            prc_pass_pairs().into_iter().map(|(a, _)| a).collect();
        let prb_accts: Vec<AccountId32> =
            prb_pass_pairs().into_iter().map(|(a, _)| a).collect();
        set_extra_admins(ORG_NRC, nrc, nrc_accts);
        set_extra_admins(ORG_PRC, prc, prc_accts);
        set_extra_admins(ORG_PRB, prb, prb_accts);
        // ORG_REN 的 admin / threshold 直接从 personal_manage::PersonalDuoqians 读
        // (B 阶段拆分后 mirror 表删除,改走 PersonalMultisigQuery trait)。
        // 测试需要时显式写入 PersonalDuoqians(见 `registered_duoqian_admin` 路径)。
        let _ = dq;
    });
    ext
}

mod cases;
