#![cfg(test)]

extern crate alloc;

use super::*;
use admin_primitives::InstitutionAdminQuery as _;
use frame_support::{
    derive_impl,
    traits::{ConstU128, ConstU32, Currency, ExistenceRequirement, Hooks, WithdrawReasons},
    BoundedVec,
};
use frame_system as system;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::types::InstitutionCode;

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
    pub type PrivateAdmins = private_admins;

    #[runtime::pallet_index(5)]
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
    type ExistentialDeposit = ConstU128<100>;
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

/// 测试查询仍委托 private-manage 的 CID/账户双索引；只注入创世注册局的
/// 出资账户归属，避免把管理员钱包误当成生产机构账户真源。
pub struct TestInstitutionQuery;
impl entity_primitives::InstitutionMultisigQuery<AccountId32> for TestInstitutionQuery {
    fn lookup_institution_account(cid_number: &[u8], account_name: &[u8]) -> Option<AccountId32> {
        <PrivateManage as entity_primitives::InstitutionMultisigQuery<AccountId32>>::lookup_institution_account(
            cid_number,
            account_name,
        )
    }

    fn account_belongs_to(cid_number: &[u8], addr: &AccountId32) -> bool {
        (cid_number == b"GD001-FRG00-000000001-2026" && addr == &registry_funding_account())
            || <PrivateManage as entity_primitives::InstitutionMultisigQuery<AccountId32>>::account_belongs_to(
                cid_number,
                addr,
            )
    }

    fn lookup_cid(addr: &AccountId32) -> Option<alloc::vec::Vec<u8>> {
        (addr == &registry_funding_account())
            .then(|| b"GD001-FRG00-000000001-2026".to_vec())
            .or_else(|| {
                <PrivateManage as entity_primitives::InstitutionMultisigQuery<AccountId32>>::lookup_cid(addr)
            })
    }

    fn lookup_org(addr: &AccountId32) -> Option<InstitutionCode> {
        <PrivateManage as entity_primitives::InstitutionMultisigQuery<AccountId32>>::lookup_org(
            addr,
        )
    }

    fn lookup_admin_config(
        addr: &AccountId32,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<AccountId32>> {
        <PrivateManage as entity_primitives::InstitutionMultisigQuery<AccountId32>>::lookup_admin_config(addr)
    }

    fn account_exists(addr: &AccountId32) -> bool {
        addr == &registry_funding_account()
            || <PrivateManage as entity_primitives::InstitutionMultisigQuery<AccountId32>>::account_exists(addr)
    }
}

/// 回调执行测试按生产链上费公式从明确的机构费用账户扣款。
pub struct TestOnchainFeeCharger;
impl primitives::fee_policy::OnchainFeeCharger<AccountId32, Balance> for TestOnchainFeeCharger {
    fn charge(
        payer: &AccountId32,
        transaction_amount: Balance,
    ) -> Result<Balance, sp_runtime::DispatchError> {
        let fee = primitives::fee_policy::calculate_onchain_fee(transaction_amount);
        let imbalance = Balances::withdraw(
            payer,
            fee,
            WithdrawReasons::FEE,
            ExistenceRequirement::KeepAlive,
        )?;
        drop(imbalance);
        Ok(fee)
    }
}

/// 测试凭证只认可固定签名字节，并强制 actor CID 与签名管理员公钥存在。
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
        _account_names: &[alloc::vec::Vec<u8>],
        nonce: &crate::pallet::RegisterNonceOf<Test>,
        signature: &crate::pallet::RegisterSignatureOf<Test>,
        actor_cid_number: &[u8],
        credential_signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        _scope_city_name: &[u8],
        _town_code: &[u8],
    ) -> bool {
        !cid_number.is_empty()
            && !cid_full_name.is_empty()
            && !nonce.is_empty()
            && !actor_cid_number.is_empty()
            && credential_signer_pubkey != &[0u8; 32]
            && !scope_province_name.is_empty()
            && signature.as_slice() == b"register-ok"
    }

    fn verify_institution_account_close(
        cid_number: &[u8],
        account_name: &[u8],
        _target_account: &AccountId32,
        nonce: &crate::pallet::RegisterNonceOf<Test>,
        signature: &crate::pallet::RegisterSignatureOf<Test>,
        credential_issuer_cid_number: &[u8],
        credential_signer_pubkey: &[u8; 32],
    ) -> bool {
        !cid_number.is_empty()
            && !account_name.is_empty()
            && !nonce.is_empty()
            && !credential_issuer_cid_number.is_empty()
            && credential_signer_pubkey != &[0u8; 32]
            && signature.as_slice() == b"deregister-ok"
    }
}

/// 这里只验证 pallet 边界；注册局辖区规则由 runtime 配置层测试负责。
pub struct TestRegistryAuthority;
impl crate::traits::RegistryAuthority<AccountId32> for TestRegistryAuthority {
    fn can_register_institution(
        _registrar: &AccountId32,
        actor_cid_number: &[u8],
        credential_signer_pubkey: &[u8; 32],
        target_cid_number: &[u8],
        _target_institution_code: InstitutionCode,
        scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        !actor_cid_number.is_empty()
            && credential_signer_pubkey != &[0u8; 32]
            && !target_cid_number.is_empty()
            && !scope_province_name.is_empty()
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

/// 投票引擎只按机构 CID 查询管理员；机构账户不参与授权寻址。
pub struct TestInternalAdminProvider;
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_institution_admin(
        institution_code: InstitutionCode,
        cid_number: &[u8],
        who: &AccountId32,
    ) -> bool {
        PrivateAdmins::is_institution_admin(institution_code, cid_number, who)
    }

    fn get_institution_admins(
        institution_code: InstitutionCode,
        cid_number: &[u8],
    ) -> Option<alloc::vec::Vec<AccountId32>> {
        PrivateAdmins::institution_admins(institution_code, cid_number)
    }
}

pub struct TestInternalAdminsLenProvider;
impl votingengine::InternalAdminsLenProvider<AccountId32> for TestInternalAdminsLenProvider {
    fn institution_admins_len(institution_code: InstitutionCode, cid_number: &[u8]) -> Option<u32> {
        PrivateAdmins::institution_admins_len(institution_code, cid_number)
    }

    fn personal_admins_len(_personal_account: AccountId32) -> Option<u32> {
        None
    }
}

pub struct TestTimeProvider;
impl frame_support::traits::UnixTime for TestTimeProvider {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(1_782_864_000)
    }
}

impl votingengine::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type MaxAutoFinalizePerBlock = ConstU32<64>;
    type MaxAutoFinalizeWeightPerBlock = votingengine::BlockWeightFraction<Test, 4>;
    type MaxExecutionWeightPerBlock = votingengine::BlockWeightFraction<Test, 4>;
    type MaxCleanupWeightPerBlock = votingengine::BlockWeightFraction<Test, 8>;
    type MaxProposalsPerExpiry = ConstU32<128>;
    type MaxInternalProposalMutexBindings = ConstU32<256>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxCleanupStepsPerBlock = ConstU32<8>;
    type CleanupKeysPerStep = ConstU32<64>;
    type CitizenIdentityReader = TestCitizenIdentityReader;
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = TestInternalAdminsLenProvider;
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

impl private_admins::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
}

impl pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type AdminLifecycle = PrivateAdmins;
    type SiblingInstitutionQuery = ();
    type InstitutionAdminQuery = PrivateAdmins;
    type AccountValidator = TestAccountValidator;
    type ReservedAccountChecker = TestReservedAccountChecker;
    type ProtectedSourceChecker = TestProtectedSourceChecker;
    type InstitutionAsset = TestInstitutionAsset;
    type InstitutionQuery = TestInstitutionQuery;
    type OnchainFeeCharger = TestOnchainFeeCharger;
    type CidInstitutionVerifier = TestCidInstitutionVerifier;
    type RegistryAuthority = TestRegistryAuthority;
    type MaxAdmins = ConstU32<10>;
    type MaxCidNumberLength = ConstU32<{ primitives::core_const::CID_NUMBER_MAX_BYTES }>;
    type MaxAccountNameLength = ConstU32<128>;
    type MaxRegisterNonceLength = ConstU32<64>;
    type MaxRegisterSignatureLength = ConstU32<64>;
    type MaxInstitutionAccounts = ConstU32<8>;
    type WeightInfo = ();
}

pub fn admin(index: u8) -> AccountId32 {
    let mut bytes = [0u8; 32];
    bytes[0] = 2;
    bytes[1] = index;
    bytes[2] = 0xAB;
    AccountId32::new(bytes)
}

pub fn registrar() -> AccountId32 {
    admin(0)
}

/// 注册局无私钥机构账户；管理员只签名，机构创建本金从此账户支出。
pub fn registry_funding_account() -> AccountId32 {
    AccountId32::new([0x32; 32])
}

pub fn beneficiary() -> AccountId32 {
    AccountId32::new([99u8; 32])
}

pub fn generated_cid(tag: &str, institution: &str) -> pallet::CidNumberOf<Test> {
    let number = primitives::cid::generator::generate_cid_number(
        primitives::cid::generator::GenerateCidNumberInput {
            account_pubkey: tag,
            p1: "0",
            province_code: "GD",
            province_name: "广东省",
            city_code: "001",
            city_name: "荔湾市",
            year: "2026",
            institution,
        },
    )
    .expect("CID 夹具必须符合真实生成规则");
    number.into_bytes().try_into().expect("CID 长度必须受界")
}

pub fn account_name(value: &[u8]) -> pallet::AccountNameOf<Test> {
    value.to_vec().try_into().expect("账户名必须受界")
}

pub fn register_nonce(value: &[u8]) -> pallet::RegisterNonceOf<Test> {
    value.to_vec().try_into().expect("nonce 必须受界")
}

pub fn valid_signature() -> pallet::RegisterSignatureOf<Test> {
    b"register-ok".to_vec().try_into().expect("签名必须受界")
}

pub fn close_signature() -> pallet::RegisterSignatureOf<Test> {
    b"deregister-ok".to_vec().try_into().expect("签名必须受界")
}

pub fn initial_accounts(items: &[(&[u8], Balance)]) -> pallet::InstitutionInitialAccountsOf<Test> {
    items
        .iter()
        .map(|(name, amount)| crate::InstitutionInitialAccount {
            account_name: account_name(name),
            amount: *amount,
        })
        .collect::<alloc::vec::Vec<_>>()
        .try_into()
        .expect("初始账户列表必须受界")
}

pub fn roles(cid_number: &pallet::CidNumberOf<Test>) -> crate::InstitutionRolesOf<Test> {
    alloc::vec![entity_primitives::InstitutionRole {
        cid_number: cid_number.clone(),
        role_code: b"ADMIN".to_vec().try_into().expect("岗位码必须受界"),
        role_name: account_name("管理员".as_bytes()),
        term_required: false,
        role_status: entity_primitives::InstitutionRoleStatus::Active,
    }]
    .try_into()
    .expect("岗位列表必须受界")
}

pub fn assignments(
    cid_number: &pallet::CidNumberOf<Test>,
    admins: &[AccountId32],
) -> crate::InstitutionAdminAssignmentsOf<Test> {
    admins
        .iter()
        .cloned()
        .map(
            |admin_account| entity_primitives::InstitutionAdminAssignment {
                cid_number: cid_number.clone(),
                admin_account,
                role_code: b"ADMIN".to_vec().try_into().expect("岗位码必须受界"),
                term_start: 0,
                term_end: 0,
                assignment_source: entity_primitives::InstitutionAssignmentSource::Registry,
                assignment_source_ref: BoundedVec::new(),
                assignment_status: entity_primitives::InstitutionAssignmentStatus::Active,
            },
        )
        .collect::<alloc::vec::Vec<_>>()
        .try_into()
        .expect("任职列表必须受界")
}

pub fn create_institution(
    cid_number: pallet::CidNumberOf<Test>,
    institution_code: InstitutionCode,
    accounts: pallet::InstitutionInitialAccountsOf<Test>,
) -> sp_runtime::DispatchResult {
    let target_admins = [admin(1), admin(2)];
    let funding_account = accounts
        .iter()
        .any(|account| account.amount > 0)
        .then(registry_funding_account);
    PrivateManage::propose_create_private_institution(
        RuntimeOrigin::signed(registrar()),
        cid_number.clone(),
        account_name("测试私权机构".as_bytes()),
        account_name("测试机构".as_bytes()),
        BoundedVec::new(),
        account_name("测试法人".as_bytes()),
        b"GD001-CTZN1-000000001-2026"
            .to_vec()
            .try_into()
            .expect("法人 CID 必须受界"),
        admin(9),
        accounts,
        funding_account,
        institution_code,
        roles(&cid_number),
        assignments(&cid_number, &target_admins),
        2,
        register_nonce(cid_number.as_slice()),
        valid_signature(),
        b"GD001-FRG00-000000001-2026".to_vec(),
        [7u8; 32],
        "广东省".as_bytes().to_vec(),
        "荔湾市".as_bytes().to_vec(),
    )
}

pub fn account_of(cid_number: &pallet::CidNumberOf<Test>, name: &[u8]) -> AccountId32 {
    PrivateManage::derive_institution_account(cid_number.as_slice(), name)
        .expect("机构账户必须可派生")
        .0
}

pub fn cast_yes_votes(proposal_id: u64) -> sp_runtime::DispatchResult {
    // 创建提案时发起人 admin(1) 已由引擎自动投赞成票，只需第二名管理员补票。
    <internal_vote::Pallet<Test>>::do_internal_vote(admin(2), proposal_id, true)?;
    <VotingEngine as Hooks<u64>>::on_initialize(System::block_number());
    Ok(())
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("测试存储必须构建成功");
    pallet_balances::GenesisConfig::<Test> {
        balances: alloc::vec![
            (registry_funding_account(), 1_000_000),
            (beneficiary(), 100)
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut storage)
    .expect("测试余额必须写入创世");
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

mod cases;
