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

pub struct TestAccountValidator;
impl organization_manage::DuoqianAccountValidator<AccountId32> for TestAccountValidator {
    fn is_valid(address: &AccountId32) -> bool {
        address != &AccountId32::new([0u8; 32])
    }
}

pub struct TestReservedAccountChecker;
impl organization_manage::DuoqianReservedAccountChecker<AccountId32>
    for TestReservedAccountChecker
{
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
        sfid_full_name: &organization_manage::pallet::AccountNameOf<Test>,
        account_names: &[alloc::vec::Vec<u8>],
        nonce: &organization_manage::pallet::RegisterNonceOf<Test>,
        signature: &organization_manage::pallet::RegisterSignatureOf<Test>,
        province_name: &[u8],
        signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        !sfid_full_name.is_empty()
            && !account_names.is_empty()
            && !nonce.is_empty()
            && !province_name.is_empty()
            && signer_admin_pubkey != &[0u8; 32]
            && signature.as_slice() == b"register-ok"
    }
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
//   - EXTRA_ADMINS 按 (org, institution AccountId) 注入 sr25519 派生 admin 集合。
// NRC/PRC/PRB 的内部阈值是 votingengine 固定制度常量,测试必须注入足够管理员并投满该阈值。
// 若某 (org, institution) 在 thread_local 有注入,优先用;否则 fallback 到原硬编码逻辑。
thread_local! {
    static EXTRA_ADMINS: core::cell::RefCell<
        alloc::collections::BTreeMap<(u8, AccountId32), alloc::vec::Vec<AccountId32>>,
    > = core::cell::RefCell::new(alloc::collections::BTreeMap::new());
}

fn set_extra_admins(org: u8, institution: AccountId32, admins: Vec<AccountId32>) {
    EXTRA_ADMINS.with(|m| {
        m.borrow_mut().insert((org, institution), admins);
    });
}

fn get_extra_admins(org: u8, institution: &AccountId32) -> Option<Vec<AccountId32>> {
    EXTRA_ADMINS.with(|m| m.borrow().get(&(org, institution.clone())).cloned())
}

pub struct TestInternalAdminProvider;
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(org: u8, institution: AccountId32, who: &AccountId32) -> bool {
        // 优先:测试注入的 sr25519 派生 admin
        if let Some(admins) = get_extra_admins(org, &institution) {
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
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false),
            ORG_PRB => CHINA_CH
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false),
            ORG_REN | ORG_PUP | ORG_OTH => {
                admins_change::Pallet::<Test>::is_active_account_admin(org, institution, who)
            }
            _ => false,
        }
    }

    fn get_admin_list(org: u8, institution: AccountId32) -> Option<Vec<AccountId32>> {
        if let Some(admins) = get_extra_admins(org, &institution) {
            return Some(admins);
        }
        match org {
            ORG_NRC | ORG_PRC => CHINA_CB
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| {
                    n.duoqian_admins
                        .iter()
                        .copied()
                        .map(AccountId32::new)
                        .collect()
                }),
            ORG_PRB => CHINA_CH
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| {
                    n.duoqian_admins
                        .iter()
                        .copied()
                        .map(AccountId32::new)
                        .collect()
                }),
            ORG_REN | ORG_PUP | ORG_OTH => {
                admins_change::Pallet::<Test>::active_account_admins(org, institution)
            }
            _ => None,
        }
    }
}

pub struct TestInternalAdminCountProvider;
impl votingengine::InternalAdminCountProvider<AccountId32> for TestInternalAdminCountProvider {
    fn admin_count(org: u8, institution: AccountId32) -> Option<u32> {
        match org {
            ORG_NRC | ORG_PRC => CHINA_CB
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok()),
            ORG_PRB => CHINA_CH
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok()),
            ORG_REN | ORG_PUP | ORG_OTH => {
                admins_change::Pallet::<Test>::active_account_admin_count(org, institution)
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
    fn can_spend(source: &AccountId32, _action: institution_asset::InstitutionAssetAction) -> bool {
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
    // Phase 2:挂上本模块 Executor,3 组业务提案通过后自动走 callback 执行。
    type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminCountProvider = TestInternalAdminCountProvider;
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
    type AccountValidator = TestAccountValidator;
    type ReservedAccountChecker = TestReservedAccountChecker;
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
    type MaxPersonalAccountAdmins = ConstU32<64>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type WeightInfo = ();
}

impl personal_manage::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type AccountValidator = TestAccountValidator;
    type ReservedAccountChecker = TestReservedAccountChecker;
    type ProtectedSourceChecker = TestProtectedSourceChecker;
    type InstitutionAsset = TestInstitutionAsset;
    type FeeRouter = ();
    type MaxAccountNameLength = ConstU32<128>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<111>;
    type WeightInfo = ();
}

impl pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxRemarkLen = ConstU32<256>;
    type FeeRouter = ();
    // 中文注释:测试 mock 把个人多签生命周期灌进 personal-manage，
    // 管理员灌进 admins-change，动态阈值灌进 internal-vote；PersonalQuery 负责合并读取。
    // InstitutionQuery 走 organization-manage,用于覆盖 0x05 InstitutionAccount 账户级主体。
    type PersonalQuery = personal_manage::Pallet<Test>;
    type InstitutionQuery = organization_manage::Pallet<Test>;
    type WeightInfo = ();
}

/// 测试 helper:从 (org, institution AccountId, index) 派生 sr25519 keypair。
///
/// 同 (org, institution AccountId, index) 每次调用返回相同 keypair,保证测试确定性。
/// 公钥的 32 字节直接作为 AccountId32,满足 `pubkey_from_accountid` 的铁律。
fn derive_admin_pair(
    org: u8,
    institution: &AccountId32,
    index: u8,
) -> (AccountId32, sr25519::Pair) {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = org;
    seed_bytes[1] = index;
    // 后 30 字节由机构 AccountId 前 30 字节填充,保证不同机构的 seed 不同。
    let institution_bytes: &[u8] = institution.as_ref();
    seed_bytes[2..32].copy_from_slice(&institution_bytes[..30]);
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

fn nrc_pallet_id() -> AccountId32 {
    AccountId32::new(CHINA_CB[0].main_account)
}

fn prc_pallet_id() -> AccountId32 {
    AccountId32::new(CHINA_CB[1].main_account)
}

fn prb_pallet_id() -> AccountId32 {
    AccountId32::new(CHINA_CH[0].main_account)
}

fn institution_account(institution: &AccountId32) -> AccountId32 {
    institution.clone()
}

fn registered_duoqian_account() -> AccountId32 {
    AccountId32::new([0x55; 32])
}

fn registered_duoqian_admin(index: usize) -> AccountId32 {
    registered_duoqian_pair(index).0
}

/// 注册多签(ORG_REN)的 admin sr25519 keypair helper。
/// seed 按 (ORG_REN, registered_duoqian_account, index) 派生,保证确定性。
fn registered_duoqian_pair(index: usize) -> (AccountId32, sr25519::Pair) {
    derive_admin_pair(ORG_REN, &registered_duoqian_account(), index as u8)
}

fn registered_duoqian_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    (0..count)
        .map(|i| registered_duoqian_pair(i as usize))
        .collect()
}

fn registered_institution_account() -> AccountId32 {
    AccountId32::new([0x66; 32])
}

fn registered_institution_admin(index: usize) -> AccountId32 {
    registered_institution_pair(index).0
}

/// 机构账户(ORG_OTH / 0x05)的 admin sr25519 keypair helper。
fn registered_institution_pair(index: usize) -> (AccountId32, sr25519::Pair) {
    derive_admin_pair(ORG_OTH, &registered_institution_account(), index as u8)
}

fn registered_institution_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    (0..count)
        .map(|i| registered_institution_pair(i as usize))
        .collect()
}

fn test_sfid_number() -> organization_manage::SfidNumberOf<Test> {
    b"AH001-SCB0H-202605070-2026"
        .to_vec()
        .try_into()
        .expect("sfid number should fit")
}

fn test_account_name() -> organization_manage::AccountNameOf<Test> {
    b"main"
        .to_vec()
        .try_into()
        .expect("account name should fit")
}

fn insert_active_registered_institution_account(
    account: &AccountId32,
    admins: admins_change::pallet::AdminsOf<Test>,
) {
    let sfid_number = test_sfid_number();
    let account_name = test_account_name();
    let institution_admins: organization_manage::pallet::DuoqianAdminsOf<Test> = admins
        .clone()
        .into_inner()
        .try_into()
        .expect("institution admins should fit organization-manage MaxAdmins");
    organization_manage::AccountRegisteredSfid::<Test>::insert(
        account,
        organization_manage::RegisteredInstitution {
            sfid_number: sfid_number.clone(),
            account_name: account_name.clone(),
        },
    );
    organization_manage::Institutions::<Test>::insert(
        &sfid_number,
        organization_manage::InstitutionInfo {
            sfid_full_name: test_account_name(),
            main_account: account.clone(),
            fee_account: AccountId32::new([0x67; 32]),
            admin_org: ORG_OTH,
            admin_count: institution_admins.len() as u32,
            threshold: 2,
            duoqian_admins: institution_admins,
            creator: registered_institution_admin(0),
            created_at: 1,
            status: organization_manage::InstitutionLifecycleStatus::Active,
            account_count: 1,
        },
    );
    organization_manage::InstitutionAccounts::<Test>::insert(
        &sfid_number,
        &account_name,
        organization_manage::InstitutionAccountInfo {
            address: account.clone(),
            initial_balance: 0,
            status: organization_manage::InstitutionLifecycleStatus::Active,
            is_default: true,
            created_at: 1,
        },
    );
    admins_change::AdminAccounts::<Test>::insert(
        account.clone(),
        admins_change::AdminAccount {
            org: ORG_OTH,
            kind: admins_change::AdminAccountKind::InstitutionAccount,
            admins,
            creator: registered_institution_admin(0),
            created_at: 1,
            updated_at: 1,
            status: admins_change::AdminAccountStatus::Active,
        },
    );
    internal_vote::ActiveDynamicThresholds::<Test>::insert(ORG_OTH, account.clone(), 2);
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
fn admin_pairs(org: u8, institution: AccountId32, count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
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

/// 测试辅助:走投票引擎公开 `internal_vote` extrinsic,
/// 让 `pairs` 前 `n` 个成员各投一张赞成票。
///
/// 中文注释：发起人已在创建提案事务中自动赞成，调用方只传剩余补票人。
fn cast_transfer_votes_n(
    pairs: &[(AccountId32, sr25519::Pair)],
    n: usize,
    pid: u64,
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
        (institution_account(&nrc_pallet_id()), 10_000),
        (institution_account(&prc_pallet_id()), 10_000),
        (institution_account(&prb_pallet_id()), 10_000),
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
        let dq = registered_duoqian_account();
        let nrc_accts: Vec<AccountId32> = nrc_pass_pairs().into_iter().map(|(a, _)| a).collect();
        let prc_accts: Vec<AccountId32> = prc_pass_pairs().into_iter().map(|(a, _)| a).collect();
        let prb_accts: Vec<AccountId32> = prb_pass_pairs().into_iter().map(|(a, _)| a).collect();
        set_extra_admins(ORG_NRC, nrc, nrc_accts);
        set_extra_admins(ORG_PRC, prc, prc_accts);
        set_extra_admins(ORG_PRB, prb, prb_accts);
        // ORG_REN/ORG_PUP/ORG_OTH 的 admin 从 admins-change 读；
        // 中文注释：动态阈值真源在 internal-vote::ActiveDynamicThresholds。
        // personal-manage / organization-manage 只保存账户生命周期状态和 org 归属。
        // 测试需要时显式写入 PersonalDuoqians + admins-change AdminAccounts。
        let _ = dq;
    });
    ext
}

mod cases;
