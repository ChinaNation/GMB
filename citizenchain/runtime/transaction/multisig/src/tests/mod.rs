#![cfg(test)]

use super::*;
use admin_primitives::AdminAccountQuery;
use codec::Encode;
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU128, ConstU32},
};
use frame_system as system;
use sp_core::{sr25519, Pair as PairT};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::types::{code_bytes, is_registered_multisig_code, InstitutionCode};
use votingengine::{STATUS_EXECUTED, STATUS_REJECTED, STATUS_VOTING};

// 测试用机构码:个人多签 / 私权法人,均属"注册多签动态账户"。
const PERSONAL_CODE: InstitutionCode = PMUL;
const PRIVATE_CODE: InstitutionCode = code_bytes("SFLP");

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
    pub type PublicManage = public_manage;

    #[runtime::pallet_index(4)]
    pub type MultisigTransfer = super;

    #[runtime::pallet_index(5)]
    pub type PublicAdmins = public_admins;

    #[runtime::pallet_index(6)]
    pub type PrivateAdmins = private_admins;

    #[runtime::pallet_index(7)]
    pub type PersonalManage = personal_manage;

    #[runtime::pallet_index(8)]
    pub type PersonalAdmins = personal_admins;
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
impl public_manage::AccountValidator<AccountId32> for TestAccountValidator {
    fn is_valid(address: &AccountId32) -> bool {
        address != &AccountId32::new([0u8; 32])
    }
}

pub struct TestReservedAccountChecker;
impl public_manage::ReservedAccountGuard<AccountId32> for TestReservedAccountChecker {
    fn is_reserved(address: &AccountId32) -> bool {
        *address == AccountId32::new([0xAA; 32])
    }
}

pub struct TestCidInstitutionVerifier;
impl
    public_manage::CidInstitutionVerifier<
        AccountId32,
        public_manage::pallet::AccountNameOf<Test>,
        public_manage::pallet::RegisterNonceOf<Test>,
        public_manage::pallet::RegisterSignatureOf<Test>,
    > for TestCidInstitutionVerifier
{
    fn verify_institution_registration(
        _cid_number: &[u8],
        cid_full_name: &public_manage::pallet::AccountNameOf<Test>,
        _cid_short_name: &[u8],
        account_names: &[alloc::vec::Vec<u8>],
        nonce: &public_manage::pallet::RegisterNonceOf<Test>,
        signature: &public_manage::pallet::RegisterSignatureOf<Test>,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        _scope_city_name: &[u8],
        _town_code: &[u8],
    ) -> bool {
        !cid_full_name.is_empty()
            && !account_names.is_empty()
            && !nonce.is_empty()
            && !scope_province_name.is_empty()
            && signer_pubkey != &[0u8; 32]
            && signature.as_slice() == b"register-ok"
    }

    fn verify_institution_deregistration(
        scope: u8,
        cid_number: &[u8],
        _account_name: &[u8],
        _target_account: &AccountId32,
        nonce: &public_manage::pallet::RegisterNonceOf<Test>,
        signature: &public_manage::pallet::RegisterSignatureOf<Test>,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        signer_pubkey: &[u8; 32],
    ) -> bool {
        scope <= public_manage::pallet::SCOPE_ACCOUNT
            && !cid_number.is_empty()
            && !nonce.is_empty()
            && signer_pubkey != &[0u8; 32]
            && signature.as_slice() == b"deregister-ok"
    }
}

pub struct TestCitizenIdentityReader;
impl votingengine::CitizenIdentityReader<AccountId32> for TestCitizenIdentityReader {
    fn can_vote(_who: &AccountId32, _scope: &votingengine::PopulationScope) -> bool {
        true
    }

    fn can_be_candidate(_who: &AccountId32, _scope: &votingengine::PopulationScope) -> bool {
        true
    }

    fn population_count(_scope: &votingengine::PopulationScope) -> u64 {
        100
    }
}

pub struct TestInternalAdminProvider;
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        // 优先:测试注入的 sr25519 派生 admin
        if let Some(admins) = get_extra_admins(institution_code, &institution) {
            return admins.iter().any(|a| a == who);
        }
        // Fallback:原硬编码 admin
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
            PMUL => personal_admins::Pallet::<Test>::is_active_account_admin(
                institution_code,
                institution,
                who,
            ),
            c if is_registered_multisig_code(&c) => {
                TestAdminAccountQuery::is_active_account_admin(institution_code, institution, who)
            }
            _ => false,
        }
    }

    fn get_admin_list(
        institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<Vec<AccountId32>> {
        if let Some(admins) = get_extra_admins(institution_code, &institution) {
            return Some(admins);
        }
        match institution_code {
            NRC | PRC => CHINA_CB
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| n.admins.iter().copied().map(AccountId32::new).collect()),
            PRB => CHINA_CH
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .map(|n| n.admins.iter().copied().map(AccountId32::new).collect()),
            PMUL => personal_admins::Pallet::<Test>::active_account_admins(
                institution_code,
                institution,
            ),
            c if is_registered_multisig_code(&c) => {
                TestAdminAccountQuery::active_account_admins(institution_code, institution)
            }
            _ => None,
        }
    }
}

pub struct TestInternalAdminsLenProvider;
impl votingengine::InternalAdminsLenProvider<AccountId32> for TestInternalAdminsLenProvider {
    fn admins_len(institution_code: InstitutionCode, institution: AccountId32) -> Option<u32> {
        match institution_code {
            NRC | PRC => CHINA_CB
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .and_then(|n| u32::try_from(n.admins.len()).ok()),
            PRB => CHINA_CH
                .iter()
                .find(|n| AccountId32::new(n.main_account) == institution)
                .and_then(|n| u32::try_from(n.admins.len()).ok()),
            PMUL => personal_admins::Pallet::<Test>::active_account_admins_len(
                institution_code,
                institution,
            ),
            c if is_registered_multisig_code(&c) => {
                TestAdminAccountQuery::active_account_admins_len(institution_code, institution)
            }
            _ => None,
        }
    }
}

pub struct TestAdminAccountQuery;
impl admin_primitives::AdminAccountQuery<AccountId32> for TestAdminAccountQuery {
    fn active_admin_account_exists(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
    ) -> bool {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Test>::active_admin_account_exists(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Test>::active_admin_account_exists(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Test>::active_admin_account_exists(
                institution_code,
                admin_root_account_id,
            );
        }
        false
    }

    fn is_active_account_admin(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
        who: &AccountId32,
    ) -> bool {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Test>::is_active_account_admin(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Test>::is_active_account_admin(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Test>::is_active_account_admin(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        false
    }

    fn active_account_admins(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
    ) -> Option<Vec<AccountId32>> {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Test>::active_account_admins(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Test>::active_account_admins(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Test>::active_account_admins(
                institution_code,
                admin_root_account_id,
            );
        }
        None
    }

    fn active_account_admins_len(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
    ) -> Option<u32> {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Test>::active_account_admins_len(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Test>::active_account_admins_len(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Test>::active_account_admins_len(
                institution_code,
                admin_root_account_id,
            );
        }
        None
    }

    fn pending_account_exists_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
    ) -> bool {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Test>::pending_account_exists_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Test>::pending_account_exists_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Test>::pending_account_exists_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        false
    }

    fn is_pending_account_admin_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
        who: &AccountId32,
    ) -> bool {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Test>::is_pending_account_admin_for_snapshot(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Test>::is_pending_account_admin_for_snapshot(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Test>::is_pending_account_admin_for_snapshot(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        false
    }

    fn pending_account_admins_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
    ) -> Option<Vec<AccountId32>> {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Test>::pending_account_admins_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Test>::pending_account_admins_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Test>::pending_account_admins_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        None
    }

    fn pending_account_admins_len_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
    ) -> Option<u32> {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Test>::pending_account_admins_len_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Test>::pending_account_admins_len_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Test>::pending_account_admins_len_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        None
    }
}

thread_local! {
    static PROTECTED_ACCOUNT: core::cell::RefCell<Option<AccountId32>> = core::cell::RefCell::new(None);
    static DENIED_SPEND_SOURCE: core::cell::RefCell<Option<AccountId32>> = core::cell::RefCell::new(None);
    static EXTRA_ADMINS: core::cell::RefCell<
        std::collections::BTreeMap<(InstitutionCode, AccountId32), Vec<AccountId32>>,
    > = core::cell::RefCell::new(std::collections::BTreeMap::new());
}

/// 测试注入:按 (机构码, 机构账户) 注入 sr25519 派生 admin 集合。
/// `TestInternalAdminProvider` 优先读取,未注入时 fallback 到 CHINA_CB/CHINA_CH 硬编码。
fn set_extra_admins(code: InstitutionCode, institution: AccountId32, admins: Vec<AccountId32>) {
    EXTRA_ADMINS.with(|m| m.borrow_mut().insert((code, institution), admins));
}
fn get_extra_admins(code: InstitutionCode, institution: &AccountId32) -> Option<Vec<AccountId32>> {
    EXTRA_ADMINS.with(|m| m.borrow().get(&(code, institution.clone())).cloned())
}

pub struct TestProtectedSourceChecker;
impl public_manage::ProtectedSourceChecker<AccountId32> for TestProtectedSourceChecker {
    fn is_protected(address: &AccountId32) -> bool {
        PROTECTED_ACCOUNT.with(|pa| pa.borrow().as_ref() == Some(address))
    }
}

pub struct TestInstitutionAsset;
impl primitives::institution_asset::InstitutionAsset<AccountId32> for TestInstitutionAsset {
    fn can_spend(
        source: &AccountId32,
        _action: primitives::institution_asset::InstitutionAssetAction,
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
    type CitizenIdentityReader = TestCitizenIdentityReader;
    type JointVoteResultCallback = ();
    // 挂上本模块 Executor,3 组业务提案通过后自动走 callback 执行。
    type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = TestInternalAdminsLenProvider;
    // 与真实 runtime 一致(1989)。联邦注册局创世内置 215 管理员,mock 上限须覆盖。
    type MaxAdminsPerInstitution = ConstU32<1989>;
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
    type LegislationVoteResultCallback = ();
    type LegislationFinalizer = ();
    type LegislationCleanup = ();
    type ElectionVoteResultCallback = ();
    type ElectionFinalizer = ();
    type ElectionCleanup = ();
}

impl internal_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl public_manage::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type AccountValidator = TestAccountValidator;
    type ReservedAccountChecker = TestReservedAccountChecker;
    type ProtectedSourceChecker = TestProtectedSourceChecker;
    type InstitutionAsset = TestInstitutionAsset;
    type CidInstitutionVerifier = TestCidInstitutionVerifier;
    type AdminLifecycle = PublicAdmins;
    type SiblingInstitutionQuery = ();
    type RegistryAuthority = ();
    type AdminAccountQuery = TestAdminAccountQuery;
    type FeeRouter = ();
    type MaxAdmins = ConstU32<10>;
    type MaxCidNumberLength = ConstU32<47>;
    type MaxAccountNameLength = ConstU32<128>;
    type MaxRegisterNonceLength = ConstU32<64>;
    type MaxRegisterSignatureLength = ConstU32<64>;
    type MaxInstitutionAccounts = ConstU32<8>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<111>;
    type WeightInfo = ();
}

impl public_admins::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type InstitutionQuery = public_manage::Pallet<Test>;
    type WeightInfo = ();
}

impl private_admins::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type InstitutionQuery = public_manage::Pallet<Test>;
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
    type PersonalAdminLifecycle = personal_admins::Pallet<Test>;
    type PersonalAdminQuery = personal_admins::Pallet<Test>;
    type FeeRouter = ();
    type MaxAccountNameLength = ConstU32<128>;
    type MaxPersonalAccountAdmins = ConstU32<64>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<111>;
    type WeightInfo = ();
}

impl personal_admins::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type MaxPersonalAccountAdmins = ConstU32<64>;
    type WeightInfo = ();
}

impl pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type InstitutionAsset = TestInstitutionAsset;
    type ProtectedSourceChecker = TestProtectedSourceChecker;
    type MaxRemarkLen = ConstU32<256>;
    type FeeRouter = ();
    // 测试 mock 把个人多签生命周期灌进 personal-manage，
    // 个人多签管理员灌进 personal-admins，动态阈值灌进 internal-vote。
    // InstitutionQuery 走 public-manage,用于覆盖 0x05 InstitutionAccount 账户级主体。
    type PersonalQuery = personal_manage::Pallet<Test>;
    type InstitutionQuery = public_manage::Pallet<Test>;
    type WeightInfo = ();
}

/// 测试 helper:从 (institution_code, institution AccountId, index) 派生 sr25519 keypair。
///
/// 同 (institution_code, institution AccountId, index) 每次调用返回相同 keypair,保证测试确定性。
/// 公钥的 32 字节直接作为 AccountId32,满足 `pubkey_from_accountid` 的铁律。
fn derive_admin_pair(
    institution_code: InstitutionCode,
    institution: &AccountId32,
    index: u8,
) -> (AccountId32, sr25519::Pair) {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = institution_code[0];
    seed_bytes[1] = index;
    // 后 30 字节由机构 AccountId 前 30 字节填充,保证不同机构的 seed 不同。
    let institution_bytes: &[u8] = institution.as_ref();
    seed_bytes[2..32].copy_from_slice(&institution_bytes[..30]);
    let pair = sr25519::Pair::from_seed(&seed_bytes);
    let account = AccountId32::new(pair.public().0);
    (account, pair)
}

fn nrc_admin(index: usize) -> AccountId32 {
    derive_admin_pair(NRC, &nrc_pallet_id(), index as u8).0
}

fn prc_admin(index: usize) -> AccountId32 {
    derive_admin_pair(PRC, &prc_pallet_id(), index as u8).0
}

fn prb_admin(index: usize) -> AccountId32 {
    derive_admin_pair(PRB, &prb_pallet_id(), index as u8).0
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

fn registered_account() -> AccountId32 {
    AccountId32::new([0x55; 32])
}

fn registered_account_admin(index: usize) -> AccountId32 {
    registered_account_pair(index).0
}

/// 注册个人账户(PERSONAL_CODE)的 admin sr25519 keypair helper。
/// seed 按 (PERSONAL_CODE, registered_account, index) 派生,保证确定性。
fn registered_account_pair(index: usize) -> (AccountId32, sr25519::Pair) {
    derive_admin_pair(PERSONAL_CODE, &registered_account(), index as u8)
}

fn registered_account_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    (0..count)
        .map(|i| registered_account_pair(i as usize))
        .collect()
}

fn registered_institution_account() -> AccountId32 {
    AccountId32::new([0x66; 32])
}

fn registered_institution_admin(index: usize) -> AccountId32 {
    registered_institution_pair(index).0
}

/// 机构账户(PRIVATE_CODE / 0x05)的 admin sr25519 keypair helper。
fn registered_institution_pair(index: usize) -> (AccountId32, sr25519::Pair) {
    derive_admin_pair(PRIVATE_CODE, &registered_institution_account(), index as u8)
}

fn registered_institution_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    (0..count)
        .map(|i| registered_institution_pair(i as usize))
        .collect()
}

fn test_cid_number() -> public_manage::CidNumberOf<Test> {
    b"AH001-SCB0H-202605070-2026"
        .to_vec()
        .try_into()
        .expect("cid number should fit")
}

fn test_account_name() -> public_manage::AccountNameOf<Test> {
    b"main"
        .to_vec()
        .try_into()
        .expect("account name should fit")
}

fn insert_active_registered_institution_account(
    account: &AccountId32,
    admins: private_admins::pallet::AdminsOf<Test>,
) {
    let cid_number = test_cid_number();
    let account_name = test_account_name();
    public_manage::AccountRegisteredCid::<Test>::insert(
        account,
        public_manage::RegisteredInstitution {
            cid_number: cid_number.clone(),
            account_name: account_name.clone(),
        },
    );
    public_manage::Institutions::<Test>::insert(
        &cid_number,
        public_manage::InstitutionInfo {
            // 本测试只关心账户反查,机构名称可为空;town_code 非镇行政区为空。
            cid_full_name: Default::default(),
            cid_short_name: Default::default(),
            town_code: Default::default(),
            legal_representative_name: None,
            legal_representative_cid_number: None,
            legal_representative_account: None,
            institution_code: PRIVATE_CODE,
            created_at: 1,
            status: public_manage::InstitutionLifecycleStatus::Active,
        },
    );
    public_manage::InstitutionAccounts::<Test>::insert(
        &cid_number,
        &account_name,
        public_manage::InstitutionAccountInfo {
            address: account.clone(),
            initial_balance: 0,
            status: public_manage::InstitutionLifecycleStatus::Active,
            is_default: true,
            created_at: 1,
        },
    );
    private_admins::AdminAccounts::<Test>::insert(
        account.clone(),
        admin_primitives::AdminAccount {
            cid_number: Default::default(),
            institution_code: PRIVATE_CODE,
            kind: admin_primitives::AdminAccountKind::PrivateInstitution,
            // 机构管理员集合存 AdminProfile(测试种子用空 meta、来源 Registry)。
            admins: admins
                .iter()
                .cloned()
                .map(|account| admin_primitives::AdminProfile {
                    admin_account: account,
                    admin_cid_number: Default::default(),
                    admin_name: Default::default(),
                    role_code: Default::default(),
                    role_name: Default::default(),
                    term_start: 0,
                    term_end: 0,
                    admin_source: admin_primitives::AdminSource::Registry,
                    admin_source_ref: Default::default(),
                })
                .collect::<Vec<_>>()
                .try_into()
                .expect("institution profiles should fit"),
            creator: registered_institution_admin(0),
            created_at: 1,
            updated_at: 1,
            status: admin_primitives::AdminAccountStatus::Active,
        },
    );
    internal_vote::ActiveDynamicThresholds::<Test>::insert(PRIVATE_CODE, account.clone(), 2);
}

/// 收款人：使用一个不是管理员也不是机构的普通地址
fn beneficiary() -> AccountId32 {
    AccountId32::new([99u8; 32])
}

/// 获取最近一次 create_internal_proposal 分配的 proposal_id。
fn last_proposal_id() -> u64 {
    votingengine::Pallet::<Test>::next_proposal_id().saturating_sub(1)
}

/// 返回 (institution_code, institution) 对应的前 `count` 个 sr25519 admin keypair。
fn admin_pairs(
    institution_code: InstitutionCode,
    institution: AccountId32,
    count: u8,
) -> Vec<(AccountId32, sr25519::Pair)> {
    (0..count)
        .map(|i| derive_admin_pair(institution_code, &institution, i))
        .collect()
}

fn nrc_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    admin_pairs(NRC, nrc_pallet_id(), count)
}

fn prc_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    admin_pairs(PRC, prc_pallet_id(), count)
}

fn prb_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
    admin_pairs(PRB, prb_pallet_id(), count)
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
/// 发起人已在创建提案事务中自动赞成，调用方只传剩余补票人。
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
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| {
        // 为储备治理三档注入 sr25519 派生 admin。
        // 注入数量必须覆盖 votingengine 的固定制度阈值,保证投票测试走真实状态机。
        // Provider 的 is_internal_admin / get_admin_list 会优先读 thread_local 注入,
        // 未注入时 fallback 到 CHINA_CB / CHINA_CH 硬编码。
        let nrc = nrc_pallet_id();
        let prc = prc_pallet_id();
        let prb = prb_pallet_id();
        let dq = registered_account();
        let nrc_accts: Vec<AccountId32> = nrc_pass_pairs().into_iter().map(|(a, _)| a).collect();
        let prc_accts: Vec<AccountId32> = prc_pass_pairs().into_iter().map(|(a, _)| a).collect();
        let prb_accts: Vec<AccountId32> = prb_pass_pairs().into_iter().map(|(a, _)| a).collect();
        set_extra_admins(NRC, nrc, nrc_accts);
        set_extra_admins(PRC, prc, prc_accts);
        set_extra_admins(PRB, prb, prb_accts);
        // PERSONAL_CODE/PUBLIC_CODE/PRIVATE_CODE 的 admin 从 personal/public/private-admins 读；
        // 动态阈值真源在 internal-vote::ActiveDynamicThresholds。
        // personal-manage / public-manage 只保存账户生命周期状态和 org 归属。
        // 测试需要时显式写入 PersonalAccounts + 对应管理员表。
        let _ = dq;
    });
    ext
}

mod cases;
