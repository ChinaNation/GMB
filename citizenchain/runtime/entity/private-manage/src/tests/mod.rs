#![cfg(test)]

extern crate alloc;

use super::*;
use admin_primitives::AdminAccountQuery;
use frame_support::{
    derive_impl,
    traits::{ConstU128, ConstU32, Hooks},
    BoundedVec,
};
use frame_system as system;
use sp_core::{sr25519, Pair as PairT};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};

std::thread_local! {
    /// 当前测试线程最近生成的机构 CID；岗位夹具必须与创建交易中的 CID 完全一致。
    static CURRENT_INSTITUTION_CID: std::cell::RefCell<Option<pallet::CidNumberOf<Test>>> = const {
        std::cell::RefCell::new(None)
    };
}
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
impl primitives::institution_asset::InstitutionAsset<AccountId32> for TestInstitutionAsset {
    fn can_spend(
        _source: &AccountId32,
        _action: primitives::institution_asset::InstitutionAssetAction,
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
        _cid_short_name: &[u8],
        account_names: &[alloc::vec::Vec<u8>],
        nonce: &crate::pallet::RegisterNonceOf<Test>,
        signature: &crate::pallet::RegisterSignatureOf<Test>,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        _scope_city_name: &[u8],
        _town_code: &[u8],
    ) -> bool {
        // account_names 可为空(改名 update_institution_info 无账户名);
        // 登记入口自身已在 verifier 前拒空账户名。
        let _ = account_names;
        !cid_number.is_empty()
            && !cid_full_name.is_empty()
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

/// 注册局权限 mock:本测试包只验证 private-manage 的生命周期行为,真实 FRG/CREG
/// 省市边界由 runtime 配置层测试覆盖。
pub struct TestRegistryAuthority;
impl crate::traits::RegistryAuthority<AccountId32> for TestRegistryAuthority {
    fn can_register_institution(
        _registrar: &AccountId32,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        _signer_pubkey: &[u8; 32],
        target_cid_number: &[u8],
        _target_institution_code: InstitutionCode,
        scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        !target_cid_number.is_empty() && !scope_province_name.is_empty()
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
    type MaxAutoFinalizeWeightPerBlock = votingengine::weights::BlockWeightFraction<Test, 4>;
    type MaxExecutionWeightPerBlock = votingengine::weights::BlockWeightFraction<Test, 4>;
    type MaxCleanupWeightPerBlock = votingengine::weights::BlockWeightFraction<Test, 8>;
    type MaxProposalsPerExpiry = ConstU32<128>;
    type MaxInternalProposalMutexBindings = ConstU32<256>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxCleanupStepsPerBlock = ConstU32<8>;
    type CleanupKeysPerStep = ConstU32<64>;
    type CitizenIdentityReader = TestCitizenIdentityReader;
    type JointVoteResultCallback = ();
    // 接 private-manage 的 InternalVoteExecutor (lib.rs 末尾导出)
    type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = TestInternalAdminsLenProvider;
    // 私权机构多签上限=1989(同真实 runtime);全链创世测试含联邦注册局 215 管理员,须覆盖。
    // 个人多签上限是另一项 MaxPersonalAccountAdmins=64,不受此影响。
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type MaxProposalDataLen = ConstU32<2048>;
    type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxCleanupActivationsPerBlock = ConstU32<50>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
    type TrackHandlers = (InternalVote, ());
    type LegislationVoteResultCallback = ();
    type ElectionVoteResultCallback = ();
}

impl internal_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl public_admins::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type InstitutionQuery = ();
}

impl private_admins::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type InstitutionQuery = ();
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
    type RegistryAuthority = TestRegistryAuthority;
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

/// 用 primitives::cid 真实生成器产 CID 号字节。
///
/// tag 与旧假号一一对应:同 tag 产同号(保留去重语义),不同 tag 产不同号。
pub fn generated_cid_bytes(tag: &str, institution: &str) -> alloc::vec::Vec<u8> {
    primitives::cid::generator::generate_cid_number(
        primitives::cid::generator::GenerateCidNumberInput {
            account_pubkey: tag,
            // 固定盈利策略机构码忽略 p1;可变/继承策略(如 UNIN)取非盈利。
            p1: "0",
            province_code: "GD",
            province_name: "广东省",
            city_code: "001",
            city_name: "荔湾市",
            year: "2026",
            institution,
        },
    )
    .expect("cid generates")
    .into_bytes()
}

/// 指定机构码的真 CID 号夹具。
pub fn generated_cid(tag: &str, institution: &str) -> pallet::CidNumberOf<Test> {
    let cid = BoundedVec::try_from(generated_cid_bytes(tag, institution)).expect("cid_number fits");
    CURRENT_INSTITUTION_CID.with(|current| *current.borrow_mut() = Some(cid.clone()));
    cid
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

pub fn empty_town_code() -> pallet::AccountNameOf<Test> {
    BoundedVec::new()
}

pub fn town_code(s: &[u8]) -> pallet::AccountNameOf<Test> {
    BoundedVec::try_from(s.to_vec()).expect("town_code fits")
}

pub fn legal_representative_name() -> pallet::AccountNameOf<Test> {
    cid_full_name("测试法人".as_bytes())
}

pub fn legal_representative_cid_number() -> pallet::CidNumberOf<Test> {
    cid_number(b"GD001-CTZN1-000000001-2026")
}

pub fn legal_representative_account() -> AccountId32 {
    admin(99)
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

fn current_institution_cid() -> pallet::CidNumberOf<Test> {
    CURRENT_INSTITUTION_CID.with(|current| {
        current
            .borrow()
            .clone()
            .expect("generated_cid must run before role fixtures")
    })
}

/// 构造一个机构自定义岗位。岗位属于 entity，不再放入 admins。
pub fn institution_roles_vec() -> crate::InstitutionRolesOf<Test> {
    let role = entity_primitives::InstitutionRole {
        cid_number: current_institution_cid(),
        role_code: BoundedVec::try_from(b"TEST_ADMIN".to_vec()).expect("role code fits"),
        role_name: account_name("管理员岗位".as_bytes()),
        term_required: false,
        role_status: entity_primitives::InstitutionRoleStatus::Active,
    };
    BoundedVec::try_from(alloc::vec![role]).expect("roles fit")
}

/// 从给定账户构造注册局来源的有效任职；admins 账户集合由这些任职去重派生。
pub fn institution_assignments_from(
    accounts: &[AccountId32],
) -> crate::InstitutionAdminAssignmentsOf<Test> {
    let cid_number = current_institution_cid();
    let assignments = accounts
        .iter()
        .cloned()
        .map(
            |admin_account| entity_primitives::InstitutionAdminAssignment {
                cid_number: cid_number.clone(),
                admin_account,
                role_code: BoundedVec::try_from(b"TEST_ADMIN".to_vec()).expect("role code fits"),
                term_start: 0,
                term_end: 0,
                assignment_source: entity_primitives::InstitutionAssignmentSource::Registry,
                assignment_source_ref: BoundedVec::new(),
                assignment_status: entity_primitives::InstitutionAssignmentStatus::Active,
            },
        )
        .collect::<alloc::vec::Vec<_>>();
    BoundedVec::try_from(assignments).expect("assignments fit")
}

pub fn institution_assignments_vec(count: u8) -> crate::InstitutionAdminAssignmentsOf<Test> {
    let accounts = (0..count).map(admin).collect::<alloc::vec::Vec<_>>();
    institution_assignments_from(&accounts)
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
    if VotingEngine::proposals(pid)
        .map(|proposal| proposal.status != STATUS_VOTING)
        .unwrap_or(false)
    {
        <VotingEngine as Hooks<u64>>::on_initialize(System::block_number());
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
    if VotingEngine::proposals(pid)
        .map(|proposal| proposal.status != STATUS_VOTING)
        .unwrap_or(false)
    {
        <VotingEngine as Hooks<u64>>::on_initialize(System::block_number());
    }
    Ok(())
}

/// 测试用:带一组通过 TestCidInstitutionVerifier 的注销凭证发起关闭。
/// `nonce_seed` 区分同一测试内多次调用的 nonce,避免 `UsedDeregisterNonce` 冲突。
pub fn close_with_cred(
    origin: RuntimeOrigin,
    admin_account: AccountId32,
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
        admin_account,
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
