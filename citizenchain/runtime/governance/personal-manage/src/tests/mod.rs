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
use std::cell::RefCell;
use votingengine::types::{InstitutionCode, PMUL};

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
    pub type PersonalManage = super;
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
thread_local! {
    static PROTECTED_ACCOUNT: RefCell<Option<AccountId32>> = const { RefCell::new(None) };
    static INSTITUTION_CAN_SPEND: RefCell<bool> = const { RefCell::new(true) };
}

impl primitives::multisig::ProtectedSourceChecker<AccountId32> for TestProtectedSourceChecker {
    fn is_protected(address: &AccountId32) -> bool {
        PROTECTED_ACCOUNT.with(|value| value.borrow().as_ref() == Some(address))
    }
}

pub struct TestInstitutionAsset;
impl institution_asset::InstitutionAsset<AccountId32> for TestInstitutionAsset {
    fn can_spend(
        _source: &AccountId32,
        _action: institution_asset::InstitutionAssetAction,
    ) -> bool {
        INSTITUTION_CAN_SPEND.with(|value| *value.borrow())
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

// ── Provider:仅支持 PMUL(个人多签),其他 institution_code 返回 None/false ──
//
// personal-manage 测试只走个人多签业务,固定治理 (NRC/PRC/PRB) 不参与;
// 因此 Provider 只需要从 admins-change 读 admins / count。

pub struct TestInternalAdminProvider;
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        if institution_code != PMUL {
            return false;
        }
        admins_change::Pallet::<Test>::is_active_account_admin(institution_code, institution, who)
    }

    fn get_admin_list(
        institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<alloc::vec::Vec<AccountId32>> {
        if institution_code != PMUL {
            return None;
        }
        admins_change::Pallet::<Test>::active_account_admins(institution_code, institution)
    }

    fn is_pending_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        institution_code == PMUL
            && admins_change::Pallet::<Test>::is_pending_account_admin_for_snapshot(
                institution_code,
                institution,
                who,
            )
    }

    fn get_pending_admin_list(
        institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<alloc::vec::Vec<AccountId32>> {
        if institution_code != PMUL {
            return None;
        }
        admins_change::Pallet::<Test>::pending_account_admins_for_snapshot(
            institution_code,
            institution,
        )
    }
}

pub struct TestInternalAdminsLenProvider;
impl votingengine::InternalAdminsLenProvider<AccountId32> for TestInternalAdminsLenProvider {
    fn admins_len(institution_code: InstitutionCode, institution: AccountId32) -> Option<u32> {
        if institution_code != PMUL {
            return None;
        }
        admins_change::Pallet::<Test>::active_account_admins_len(institution_code, institution)
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
    type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = TestInternalAdminsLenProvider;
    // 中文注释:机构多签上限=1989(同真实 runtime);全链创世测试含联邦注册局 215 管理员,须覆盖。
    // 个人多签上限是另一项 MaxPersonalAccountAdmins=64,不受此影响。
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
}

impl internal_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl admins_change::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    // 中文注释:机构多签上限=1989(同真实 runtime);全链创世测试含联邦注册局 215 管理员,须覆盖。
    // 个人多签上限是另一项 MaxPersonalAccountAdmins=64,不受此影响。
    type MaxAdminsPerInstitution = ConstU32<1989>;
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
    type FeeRouter = ();
    type MaxAccountNameLength = ConstU32<128>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<111>;
    type WeightInfo = ();
}

// ─── 测试 helper ────────────────────────────────────────────────────────

/// 从 (creator_seed, index) 派生 sr25519 keypair。
/// 同 (creator_seed, index) 每次返回相同 keypair,保证测试确定性。
pub fn derive_admin_pair(creator_seed: u8, index: u8) -> (AccountId32, sr25519::Pair) {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = creator_seed;
    seed_bytes[1] = index;
    seed_bytes[2] = 0xAB; // 区分本测试套和 multisig-transfer 的 seed 命名空间
    let pair = sr25519::Pair::from_seed(&seed_bytes);
    let account = AccountId32::new(pair.public().0);
    (account, pair)
}

pub fn admin(index: u8) -> AccountId32 {
    derive_admin_pair(1, index).0
}

pub fn creator() -> AccountId32 {
    admin(0)
}

pub fn beneficiary() -> AccountId32 {
    AccountId32::new([99u8; 32])
}

pub fn account_name(s: &[u8]) -> pallet::AccountNameOf<Test> {
    BoundedVec::try_from(s.to_vec()).expect("account name fits")
}

pub fn set_protected_account(address: Option<AccountId32>) {
    PROTECTED_ACCOUNT.with(|value| {
        *value.borrow_mut() = address;
    });
}

pub fn set_institution_can_spend(can_spend: bool) {
    INSTITUTION_CAN_SPEND.with(|value| {
        *value.borrow_mut() = can_spend;
    });
}

pub fn admins_vec(count: u8) -> pallet::AdminsOf<Test> {
    let v: alloc::vec::Vec<AccountId32> = (0..count).map(|i| admin(i)).collect();
    BoundedVec::try_from(v).expect("admins fit")
}

pub fn last_proposal_id() -> u64 {
    votingengine::Pallet::<Test>::next_proposal_id().saturating_sub(1)
}

/// 走投票引擎公开 `internal_vote` extrinsic,让前 n 个 admin 各投一张赞成票。
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

/// 走投票引擎让前 n 个 admin 各投一张反对票。
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

/// 直接灌已激活的个人多签账户 + admins-change 管理员账户,跳过 propose/vote 链路。
/// 用于关闭/资金边界测试,避免每个用例都重复一遍创建流程。
pub fn seed_active_multisig(
    account: &AccountId32,
    creator: &AccountId32,
    admins: &[AccountId32],
    initial_balance: Balance,
) {
    pallet::PersonalAccounts::<Test>::insert(
        account,
        types::PersonalAccount {
            creator: creator.clone(),
            account_name: account_name(b"seeded"),
            created_at: 1,
            status: types::PersonalStatus::Active,
        },
    );
    // admins-change 写 Active 管理员账户,让 propose_close 的 is_active_account_admin 通过。
    // 中文注释：普通业务阈值归 internal-vote 管，不再写入管理员主体。
    let account = account.clone();
    let admins_ac: admins_change::AdminsOf<Test> =
        BoundedVec::try_from(admins.to_vec()).expect("admins fit ac");
    let threshold = (admins.len() as u32 / 2).saturating_add(1);
    internal_vote::ActiveDynamicThresholds::<Test>::insert(PMUL, account.clone(), threshold);
    admins_change::AdminAccounts::<Test>::insert(
        account.clone(),
        admins_change::AdminAccount {
            institution_code: PMUL,
            kind: admins_change::AdminAccountKind::PersonalAccount,
            admins: admins_ac,
            creator: creator.clone(),
            created_at: 1,
            updated_at: 1,
            status: admins_change::AdminAccountStatus::Active,
        },
    );
    use frame_support::traits::Currency;
    let _ = Balances::deposit_creating(&account, initial_balance);
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");

    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| {
        System::set_block_number(1);
        set_protected_account(None);
        set_institution_can_spend(true);
    });
    ext
}

mod cases;
