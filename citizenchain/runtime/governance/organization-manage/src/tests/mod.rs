#![cfg(test)]

extern crate alloc;

use super::*;
use frame_support::{
    derive_impl,
    traits::{ConstU128, ConstU32},
    BoundedVec,
};
use frame_system as system;
use sp_core::{sr25519, Pair as PairT};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::types::{is_registered_multisig_org, ORG_OTH};

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
    pub type AdminsChange = admins_change;

    #[runtime::pallet_index(5)]
    pub type OrganizationManage = super;
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
impl primitives::multisig::DuoqianAccountValidator<AccountId32> for TestAccountValidator {
    fn is_valid(address: &AccountId32) -> bool {
        address != &AccountId32::new([0u8; 32])
    }
}

pub struct TestReservedAccountChecker;
impl primitives::multisig::DuoqianReservedAccountChecker<AccountId32>
    for TestReservedAccountChecker
{
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

/// SFID 双层签名 mock:仅当 signature == b"register-ok"
/// 且 nonce/sfid_full_name/account_names/province_name/signer_admin_pubkey 都非空时通过。
pub struct TestSfidInstitutionVerifier;
impl
    crate::traits::SfidInstitutionVerifier<
        crate::pallet::AccountNameOf<Test>,
        crate::pallet::RegisterNonceOf<Test>,
        crate::pallet::RegisterSignatureOf<Test>,
    > for TestSfidInstitutionVerifier
{
    fn verify_institution_registration(
        sfid_number: &[u8],
        sfid_full_name: &crate::pallet::AccountNameOf<Test>,
        account_names: &[alloc::vec::Vec<u8>],
        nonce: &crate::pallet::RegisterNonceOf<Test>,
        signature: &crate::pallet::RegisterSignatureOf<Test>,
        province_name: &[u8],
        signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        !sfid_number.is_empty()
            && !sfid_full_name.is_empty()
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

// ── Provider:支持注册多签动态机构账户(ORG_REN/ORG_PUP/ORG_OTH) ──
//
// 机构账户 institution = 注册机构 main AccountId。
// 测试环境直接读 admins-change::AdminAccounts[institution] 的管理员列表。
// 中文注释：动态阈值由 internal-vote 保存，不再挂在管理员主体上。

pub struct TestInternalAdminProvider;
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(org: u8, institution: AccountId32, who: &AccountId32) -> bool {
        if !is_registered_multisig_org(org) {
            return false;
        }
        admins_change::AdminAccounts::<Test>::get(institution)
            .map(|s| s.admins.iter().any(|a| a == who))
            .unwrap_or(false)
    }

    fn get_admin_list(org: u8, institution: AccountId32) -> Option<alloc::vec::Vec<AccountId32>> {
        if !is_registered_multisig_org(org) {
            return None;
        }
        admins_change::AdminAccounts::<Test>::get(institution).map(|s| s.admins.into_inner())
    }
}

pub struct TestInternalAdminCountProvider;
impl votingengine::InternalAdminCountProvider<AccountId32> for TestInternalAdminCountProvider {
    fn admin_count(org: u8, institution: AccountId32) -> Option<u32> {
        if !is_registered_multisig_org(org) {
            return None;
        }
        admins_change::AdminAccounts::<Test>::get(institution).map(|s| s.admins.len() as u32)
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
    type SfidEligibility = TestSfidEligibility;
    type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
    type JointVoteResultCallback = ();
    // 接 organization-manage 的 InternalVoteExecutor (lib.rs 末尾导出)
    type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminCountProvider = TestInternalAdminCountProvider;
    type MaxAdminsPerInstitution = ConstU32<64>;
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
}

impl internal_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl admins_change::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<64>;
    type MaxPersonalAccountAdmins = ConstU32<64>;
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
    type SfidInstitutionVerifier = TestSfidInstitutionVerifier;
    type FeeRouter = ();
    type MaxAdmins = ConstU32<10>;
    type MaxSfidNumberLength = ConstU32<{ primitives::core_const::SFID_NUMBER_MAX_BYTES }>;
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
    seed_bytes[0] = 2; // organization-manage 命名空间(personal-manage 用 1)
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

pub fn sfid_number(s: &[u8]) -> pallet::SfidNumberOf<Test> {
    BoundedVec::try_from(s.to_vec()).expect("sfid_number fits")
}

pub fn sfid_full_name(s: &[u8]) -> pallet::AccountNameOf<Test> {
    BoundedVec::try_from(s.to_vec()).expect("sfid_full_name fits")
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

pub fn admins_vec(count: u8) -> pallet::DuoqianAdminsOf<Test> {
    let v: alloc::vec::Vec<AccountId32> = (0..count).map(|i| admin(i)).collect();
    BoundedVec::try_from(v).expect("admins fit")
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
