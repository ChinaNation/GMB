#![cfg(test)]

extern crate alloc;

use super::*;
use admin_primitives::AdminAccountQuery;
use frame_support::{
    derive_impl,
    traits::{ConstU128, ConstU32},
    BoundedVec,
};
use frame_system as system;
use sp_core::{sr25519, Pair as PairT};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::types::{code_bytes, is_registered_multisig_code, InstitutionCode};

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

    #[runtime::pallet_index(3)]
    pub type InternalVote = internal_vote;

    #[runtime::pallet_index(4)]
    pub type PublicAdmins = public_admins;

    #[runtime::pallet_index(5)]
    pub type PrivateAdmins = private_admins;

    #[runtime::pallet_index(6)]
    pub type PrivateManage = super;
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

// ─── Trait mock 实现 ─────────────────────────────────────────────────────

pub struct TestAccountValidator;
impl primitives::multisig::AccountValidator<AccountId32> for TestAccountValidator {
    fn is_valid(address: &AccountId32) -> bool {
        address != &AccountId32::new([0u8; 32])
    }
}

pub struct TestReservedAccountChecker;
impl primitives::multisig::ReservedAccountGuard<AccountId32> for TestReservedAccountChecker {
    fn is_reserved(address: &AccountId32) -> bool {
        *address == AccountId32::new([0xAA; 32])
    }
}

pub struct TestProtectedSourceChecker;
impl primitives::multisig::ProtectedSourceChecker<AccountId32> for TestProtectedSourceChecker {
    fn is_protected(_address: &AccountId32) -> bool {
        false
    }
}

pub struct TestInstitutionAsset;
impl institution_asset::InstitutionAsset<AccountId32> for TestInstitutionAsset {
    fn can_spend(
        _source: &AccountId32,
        _action: institution_asset::InstitutionAssetAction,
    ) -> bool {
        true
    }
}

/// CID 双层签名 mock:仅当 signature == b"register-ok"
/// 且 nonce/cid_full_name/account_names/issuer/scope 字段都非空时通过。
pub struct TestCidInstitutionVerifier;
impl
    crate::traits::CidInstitutionVerifier<
        AccountId32,
        crate::pallet::AccountNameOf<Test>,
        crate::pallet::RegisterNonceOf<Test>,
        crate::pallet::RegisterSignatureOf<Test>,
    > for TestCidInstitutionVerifier
{
    fn verify_institution_registration(
        cid_number: &[u8],
        cid_full_name: &crate::pallet::AccountNameOf<Test>,
        account_names: &[alloc::vec::Vec<u8>],
        nonce: &crate::pallet::RegisterNonceOf<Test>,
        signature: &crate::pallet::RegisterSignatureOf<Test>,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        !cid_number.is_empty()
            && !cid_full_name.is_empty()
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
        nonce: &crate::pallet::RegisterNonceOf<Test>,
        signature: &crate::pallet::RegisterSignatureOf<Test>,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        signer_pubkey: &[u8; 32],
    ) -> bool {
        scope <= crate::pallet::SCOPE_ACCOUNT
            && !cid_number.is_empty()
            && !nonce.is_empty()
            && signer_pubkey != &[0u8; 32]
            && signature.as_slice() == b"deregister-ok"
    }
}

pub struct TestCidEligibility;
impl votingengine::CidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
    for TestCidEligibility
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
        _issuer_cid_number: &[u8],
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
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        true
    }
}

// ── Provider:支持注册多签动态机构账户(PMUL/公权法人/私权法人) ──
//
// 机构账户 institution = 注册机构 main AccountId。
// 测试环境直接读 public-admins / private-admins 的管理员列表。
// 中文注释：动态阈值由 internal-vote 保存，不再挂在管理员主体上。

pub struct TestInternalAdminProvider;
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        if !is_registered_multisig_code(&institution_code) {
            return false;
        }
        TestAdminAccountQuery::is_active_account_admin(institution_code, institution, who)
    }

    fn get_admin_list(
        institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<alloc::vec::Vec<AccountId32>> {
        if !is_registered_multisig_code(&institution_code) {
            return None;
        }
        TestAdminAccountQuery::active_account_admins(institution_code, institution)
    }
}

pub struct TestInternalAdminsLenProvider;
impl votingengine::InternalAdminsLenProvider<AccountId32> for TestInternalAdminsLenProvider {
    fn admins_len(institution_code: InstitutionCode, institution: AccountId32) -> Option<u32> {
        if !is_registered_multisig_code(&institution_code) {
            return None;
        }
        TestAdminAccountQuery::active_account_admins_len(institution_code, institution)
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
        false
    }

    fn active_account_admins(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
    ) -> Option<alloc::vec::Vec<AccountId32>> {
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
        false
    }

    fn pending_account_admins_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
    ) -> Option<alloc::vec::Vec<AccountId32>> {
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
        None
    }

    fn legal_representative(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId32,
    ) -> Option<AccountId32> {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Test>::legal_representative(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Test>::legal_representative(
                institution_code,
                admin_root_account_id,
            );
        }
        None
    }
}

pub struct TestTimeProvider;
impl frame_support::traits::UnixTime for TestTimeProvider {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(1_782_864_000) // 2026-07-01
    }
}

// ─── Pallet Config 实现 ─────────────────────────────────────────────────

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
    type CidEligibility = TestCidEligibility;
    type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
    type JointVoteResultCallback = ();
    // 接 private-manage 的 InternalVoteExecutor (lib.rs 末尾导出)
    type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = TestInternalAdminsLenProvider;
    // 中文注释:私权机构多签上限=1989(同真实 runtime);全链创世测试含联邦注册局 215 管理员,须覆盖。
    // 个人多签上限是另一项 MaxPersonalAccountAdmins=64,不受此影响。
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type MaxProposalDataLen = ConstU32<2048>;
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

impl public_admins::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type WeightInfo = ();
}

impl private_admins::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type WeightInfo = ();
}

impl pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type AccountValidator = TestAccountValidator;
    type ReservedAccountChecker = TestReservedAccountChecker;
    type ProtectedSourceChecker = TestProtectedSourceChecker;
    type InstitutionAsset = TestInstitutionAsset;
    type CidInstitutionVerifier = TestCidInstitutionVerifier;
    type AdminLifecycle = PrivateAdmins;
    type SiblingInstitutionQuery = ();
    type AdminAccountQuery = TestAdminAccountQuery;
    type FeeRouter = ();
    type MaxAdmins = ConstU32<10>;
    type MaxCidNumberLength = ConstU32<{ primitives::core_const::CID_NUMBER_MAX_BYTES }>;
    type MaxAccountNameLength = ConstU32<128>;
    type MaxRegisterNonceLength = ConstU32<64>;
    type MaxRegisterSignatureLength = ConstU32<64>;
    type MaxInstitutionAccounts = ConstU32<8>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<111>;
    type WeightInfo = ();
}

// ─── 测试 helper ────────────────────────────────────────────────────────

/// 派生 sr25519 admin 账户。seed 区分本测试套命名空间。
pub fn derive_admin_pair(index: u8) -> (AccountId32, sr25519::Pair) {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = 2; // private-manage 命名空间(personal-admins 用 1)
    seed_bytes[1] = index;
    seed_bytes[2] = 0xAB;
    let pair = sr25519::Pair::from_seed(&seed_bytes);
    (AccountId32::new(pair.public().0), pair)
}

pub fn admin(index: u8) -> AccountId32 {
    derive_admin_pair(index).0
}

pub fn creator() -> AccountId32 {
    admin(0)
}

pub fn beneficiary() -> AccountId32 {
    AccountId32::new([99u8; 32])
}

pub fn cid_number(s: &[u8]) -> pallet::CidNumberOf<Test> {
    BoundedVec::try_from(s.to_vec()).expect("cid_number fits")
}

pub fn cid_full_name(s: &[u8]) -> pallet::AccountNameOf<Test> {
    BoundedVec::try_from(s.to_vec()).expect("cid_full_name fits")
}

pub fn cid_short_name(s: &[u8]) -> pallet::AccountNameOf<Test> {
    BoundedVec::try_from(s.to_vec()).expect("cid_short_name fits")
}

pub fn account_name(s: &[u8]) -> pallet::AccountNameOf<Test> {
    BoundedVec::try_from(s.to_vec()).expect("account_name fits")
}

pub fn register_nonce(s: &[u8]) -> pallet::RegisterNonceOf<Test> {
    BoundedVec::try_from(s.to_vec()).expect("register_nonce fits")
}

pub fn valid_signature() -> pallet::RegisterSignatureOf<Test> {
    BoundedVec::try_from(b"register-ok".to_vec()).expect("sig fits")
}

pub fn invalid_signature() -> pallet::RegisterSignatureOf<Test> {
    BoundedVec::try_from(b"bad-signature".to_vec()).expect("sig fits")
}

pub fn signer_pubkey() -> [u8; 32] {
    [7u8; 32]
}

pub fn province_name() -> alloc::vec::Vec<u8> {
    b"liaoning".to_vec()
}

/// 构造机构创建用的管理员资料集合(account = admin(i),空元数据,来源 Registry)。
pub fn admin_profiles_vec(count: u8) -> pallet::AdminProfilesOf<Test> {
    let accounts: alloc::vec::Vec<AccountId32> = (0..count).map(|i| admin(i)).collect();
    admin_profiles_from(&accounts)
}

/// 从给定账户列表构造空元数据(来源 Registry)的管理员资料集合。
pub fn admin_profiles_from(accounts: &[AccountId32]) -> pallet::AdminProfilesOf<Test> {
    let v: alloc::vec::Vec<admin_primitives::AdminProfile<AccountId32>> = accounts
        .iter()
        .cloned()
        .map(|account| admin_primitives::AdminProfile {
            account,
            admin_cid_number: BoundedVec::new(),
            name: BoundedVec::new(),
            admin_role: BoundedVec::new(),
            term_start: 0,
            term_end: 0,
            source: admin_primitives::AdminSource::Registry,
        })
        .collect();
    BoundedVec::try_from(v).expect("admin profiles fit")
}

/// 构造带非空 姓名/职务/任期/实名CID 的管理员资料集合(3 人,末位留空元数据)。
///
/// 中文注释:专供验证 profile 经机构创建提案 `CreateInstitutionAction` 的 SCALE 往返不丢字段。
pub fn admin_profiles_with_meta() -> pallet::AdminProfilesOf<Test> {
    let mut v: alloc::vec::Vec<admin_primitives::AdminProfile<AccountId32>> =
        alloc::vec::Vec::new();
    v.push(admin_primitives::AdminProfile {
        account: admin(0),
        admin_cid_number: BoundedVec::try_from(b"LN001-AAAAA-000000001-2026".to_vec())
            .expect("cid fits"),
        name: BoundedVec::try_from("张三".as_bytes().to_vec()).expect("name fits"),
        admin_role: BoundedVec::try_from("董事长".as_bytes().to_vec()).expect("admin_role fits"),
        term_start: 20_100,
        term_end: 21_561,
        source: admin_primitives::AdminSource::MutualElection,
    });
    v.push(admin_primitives::AdminProfile {
        account: admin(1),
        admin_cid_number: BoundedVec::try_from(b"LN001-BBBBB-000000002-2026".to_vec())
            .expect("cid fits"),
        name: BoundedVec::try_from("李四".as_bytes().to_vec()).expect("name fits"),
        admin_role: BoundedVec::try_from("董事".as_bytes().to_vec()).expect("admin_role fits"),
        term_start: 20_100,
        term_end: 21_561,
        source: admin_primitives::AdminSource::MutualElection,
    });
    v.push(admin_primitives::AdminProfile {
        account: admin(2),
        admin_cid_number: BoundedVec::new(),
        name: BoundedVec::new(),
        admin_role: BoundedVec::new(),
        term_start: 0,
        term_end: 0,
        source: admin_primitives::AdminSource::Registry,
    });
    BoundedVec::try_from(v).expect("admin profiles fit")
}

pub fn account_names_bv(names: &[&[u8]]) -> pallet::InstitutionAccountNamesOf<Test> {
    let v: alloc::vec::Vec<pallet::AccountNameOf<Test>> =
        names.iter().map(|n| account_name(n)).collect();
    BoundedVec::try_from(v).expect("account names fit")
}

pub fn initial_accounts(items: &[(&[u8], Balance)]) -> pallet::InstitutionInitialAccountsOf<Test> {
    let v: alloc::vec::Vec<pallet::InstitutionInitialAccountOf<Test>> = items
        .iter()
        .map(
            |(n, amt)| crate::institution::types::InstitutionInitialAccount {
                account_name: account_name(n),
                amount: *amt,
            },
        )
        .collect();
    BoundedVec::try_from(v).expect("initial accounts fit")
}

pub fn last_proposal_id() -> u64 {
    votingengine::Pallet::<Test>::next_proposal_id().saturating_sub(1)
}

pub fn cast_yes_votes(admins: &[AccountId32], n: usize, pid: u64) -> sp_runtime::DispatchResult {
    use votingengine::STATUS_VOTING;
    for who in admins.iter().take(n) {
        <internal_vote::Pallet<Test>>::do_internal_vote(who.clone(), pid, true)?;
        if VotingEngine::proposals(pid)
            .map(|p| p.status != STATUS_VOTING)
            .unwrap_or(true)
        {
            break;
        }
    }
    Ok(())
}

pub fn cast_no_votes(admins: &[AccountId32], n: usize, pid: u64) -> sp_runtime::DispatchResult {
    use votingengine::STATUS_VOTING;
    for who in admins.iter().take(n) {
        <internal_vote::Pallet<Test>>::do_internal_vote(who.clone(), pid, false)?;
        if VotingEngine::proposals(pid)
            .map(|p| p.status != STATUS_VOTING)
            .unwrap_or(true)
        {
            break;
        }
    }
    Ok(())
}

/// 测试用:带一组通过 TestCidInstitutionVerifier 的注销凭证发起关闭。
/// `nonce_seed` 区分同一测试内多次调用的 nonce,避免 `UsedDeregisterNonce` 冲突。
pub fn close_with_cred(
    origin: RuntimeOrigin,
    account: AccountId32,
    beneficiary: AccountId32,
    nonce_seed: u8,
) -> sp_runtime::DispatchResult {
    let nonce: crate::pallet::RegisterNonceOf<Test> =
        vec![nonce_seed, 0xDE].try_into().expect("nonce fits bound");
    let signature: crate::pallet::RegisterSignatureOf<Test> = b"deregister-ok"
        .to_vec()
        .try_into()
        .expect("sig fits bound");
    PrivateManage::propose_close_private_institution(
        origin,
        account,
        beneficiary,
        nonce,
        signature,
        b"ISSUER-CID".to_vec(),
        AccountId32::new([7u8; 32]),
        [9u8; 32],
    )
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

mod cases;
