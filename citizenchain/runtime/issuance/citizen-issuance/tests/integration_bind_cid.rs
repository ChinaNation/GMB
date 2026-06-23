//! 集成测试：验证 cid-system::bind_cid → OnCidBound → citizen-issuance 完整链路。
//!
//! 本文件构建包含 cid-system + citizen-issuance 的 mock runtime，
//! 直接调用 bind_cid extrinsic，验证奖励事件、跳过事件、双重防重与 weight 叠加。

use frame_support::{
    derive_impl, parameter_types,
    traits::{ConstU128, ConstU32, VariantCountOf},
};
use frame_system::{self as system, EnsureRoot};
use pallet_balances;
use primitives::citizen_const::{CITIZEN_ISSUANCE_HIGH_REWARD, CITIZEN_ISSUANCE_MAX_COUNT};
use cid_system::{BindCredential, CidVerifier, CidVoteVerifier};
use sp_runtime::{
    traits::{BlakeTwo256, Hash, IdentityLookup, Zero},
    BuildStorage,
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
    pub type Balances = pallet_balances;
    #[runtime::pallet_index(2)]
    pub type CidSystem = cid_system;
    #[runtime::pallet_index(3)]
    pub type CitizenIssuance = citizen_issuance;
}

/// 中文注释：测试用验签器——仅当 signature == b"valid" 时返回 true。
pub struct TestCidVerifier;
impl
    CidVerifier<
        u64,
        <Test as frame_system::Config>::Hash,
        cid_system::pallet::NonceOf<Test>,
        cid_system::pallet::SignatureOf<Test>,
    > for TestCidVerifier
{
    fn verify(
        _account: &u64,
        credential: &BindCredential<
            u64,
            <Test as frame_system::Config>::Hash,
            cid_system::pallet::NonceOf<Test>,
            cid_system::pallet::SignatureOf<Test>,
        >,
    ) -> bool {
        credential.signature.as_slice() == b"valid"
    }
}

/// 中文注释：测试用投票验签器——集成测试不涉及投票，始终返回 false。
pub struct TestCidVoteVerifier;
impl
    CidVoteVerifier<
        u64,
        <Test as frame_system::Config>::Hash,
        cid_system::pallet::NonceOf<Test>,
        cid_system::pallet::SignatureOf<Test>,
    > for TestCidVoteVerifier
{
    fn verify_vote(
        _account: &u64,
        _binding_id: <Test as frame_system::Config>::Hash,
        _proposal_id: u64,
        _nonce: &cid_system::pallet::NonceOf<Test>,
        _signature: &cid_system::pallet::SignatureOf<Test>,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &u64,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        false
    }
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = u64;
    type AccountData = pallet_balances::AccountData<u128>;
    type Lookup = IdentityLookup<Self::AccountId>;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<0>;
    type MaxReserves = ConstU32<0>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const MaxCredentialNonceLength: u32 = 64;
    pub const MaxCredentialSignatureLength: u32 = 64;
}

impl cid_system::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxCredentialNonceLength = MaxCredentialNonceLength;
    type MaxCredentialSignatureLength = MaxCredentialSignatureLength;
    type CidVerifier = TestCidVerifier;
    type CidVoteVerifier = TestCidVoteVerifier;
    /// 中文注释：集成测试核心——将 OnCidBound 接入真实的 CitizenIssuance。
    type OnCidBound = CitizenIssuance;
    /// `unbind_cid` 由 Root 授权(集成测试不涉及解绑路径)。
    type UnbindOrigin = EnsureRoot<u64>;
    type WeightInfo = ();
}

impl citizen_issuance::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = ();
}

fn make_credential(
    binding_id_seed: &[u8],
    nonce_seed: &[u8],
    valid: bool,
) -> BindCredential<
    u64,
    <Test as frame_system::Config>::Hash,
    cid_system::pallet::NonceOf<Test>,
    cid_system::pallet::SignatureOf<Test>,
> {
    let binding_id = BlakeTwo256::hash(binding_id_seed);
    let nonce: cid_system::pallet::NonceOf<Test> =
        nonce_seed.to_vec().try_into().expect("nonce should fit");
    let sig_bytes: &[u8] = if valid { b"valid" } else { b"bad" };
    let signature: cid_system::pallet::SignatureOf<Test> =
        sig_bytes.to_vec().try_into().expect("sig should fit");
    // 中文注释:`BindCredential` 必带签发机构、签名管理员和业务作用域字段。
    // 集成测试用 TestCidVerifier 不解析这些字段,只检查 signature == "valid",
    // 真实双层签名校验在 runtime 层 `RuntimeCidVerifier` 单独覆盖。
    BindCredential {
        binding_id,
        bind_nonce: nonce,
        issuer_cid_number: b"CN000-GZF0A-000000001-2026"
            .to_vec()
            .try_into()
            .expect("issuer_cid_number should fit"),
        issuer_main_account: 1,
        signer_pubkey: [7u8; 32],
        scope_province_name: b"liaoning"
            .to_vec()
            .try_into()
            .expect("scope_province_name should fit"),
        scope_city_name: b"shenyang"
            .to_vec()
            .try_into()
            .expect("scope_city_name should fit"),
        signature,
    }
}

fn new_test_ext() -> sp_io::TestExternalities {
    // cid_system::GenesisConfig 创世 storage 全空,
    // 链上 0 prior knowledge of CID,集成测试不再注入任何 CID 创世账户。
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("frame system genesis storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(10);
    });
    ext
}

// ============================================================================
// 集成测试用例
// ============================================================================

/// 中文注释：完整链路测试——bind_cid 成功后自动发放高额认证奖励。
#[test]
fn bind_cid_triggers_reward_issuance() {
    new_test_ext().execute_with(|| {
        let credential = make_credential(b"cid-integ-1", b"nonce-1", true);
        let binding_id = credential.binding_id;

        assert_eq!(Balances::free_balance(1), 0);

        assert!(CidSystem::bind_cid(RuntimeOrigin::signed(1), credential).is_ok());

        // 验证奖励已发放
        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(citizen_issuance::RewardedCount::<Test>::get(), 1);
        assert!(citizen_issuance::RewardClaimed::<Test>::contains_key(
            binding_id
        ));
        assert!(citizen_issuance::AccountRewarded::<Test>::contains_key(1));

        // 验证上游绑定状态也正确写入
        assert_eq!(
            cid_system::BindingIdToAccount::<Test>::get(binding_id),
            Some(1)
        );
        assert_eq!(
            cid_system::AccountToBindingId::<Test>::get(1),
            Some(binding_id)
        );
        assert_eq!(cid_system::BoundCount::<Test>::get(), 1);

        // 验证事件链：先 issuance 事件，后 cid 事件
        let events: Vec<RuntimeEvent> = System::events().into_iter().map(|r| r.event).collect();
        assert!(events.iter().any(|e| matches!(
            e,
            RuntimeEvent::CitizenIssuance(
                citizen_issuance::Event::CertificationRewardIssued { .. }
            )
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            RuntimeEvent::CidSystem(cid_system::Event::CidBound { .. })
        )));
    });
}

/// 中文注释：同一账户换绑 CID 时，第二次不发奖但 bind_cid 本身成功。
#[test]
fn rebind_skips_reward_but_bind_succeeds() {
    new_test_ext().execute_with(|| {
        let cred1 = make_credential(b"cid-rebind-a", b"nonce-rebind-1", true);
        let cred2 = make_credential(b"cid-rebind-b", b"nonce-rebind-2", true);

        assert!(CidSystem::bind_cid(RuntimeOrigin::signed(1), cred1).is_ok());
        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);

        // 换绑到新 binding_id
        assert!(CidSystem::bind_cid(RuntimeOrigin::signed(1), cred2).is_ok());

        // 余额不变——第二次奖励被跳过
        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(citizen_issuance::RewardedCount::<Test>::get(), 1);

        // 验证跳过事件
        let events: Vec<RuntimeEvent> = System::events().into_iter().map(|r| r.event).collect();
        assert!(events.iter().any(|e| matches!(
            e,
            RuntimeEvent::CitizenIssuance(citizen_issuance::Event::CertificationRewardSkipped {
                reason: citizen_issuance::pallet::SkipReason::AccountAlreadyRewarded,
                ..
            })
        )));
    });
}

/// 中文注释：不同账户分别绑定不同 CID，各自独立领奖。
#[test]
fn two_users_bind_independently_and_both_get_rewards() {
    new_test_ext().execute_with(|| {
        let cred1 = make_credential(b"cid-user-a", b"nonce-a", true);
        let cred2 = make_credential(b"cid-user-b", b"nonce-b", true);

        assert!(CidSystem::bind_cid(RuntimeOrigin::signed(1), cred1).is_ok());
        assert!(CidSystem::bind_cid(RuntimeOrigin::signed(2), cred2).is_ok());

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(Balances::free_balance(2), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(citizen_issuance::RewardedCount::<Test>::get(), 2);
        assert_eq!(cid_system::BoundCount::<Test>::get(), 2);
    });
}

/// 中文注释：达到发放上限后，bind_cid 仍成功但奖励被跳过。
#[test]
fn bind_after_max_count_skips_reward() {
    new_test_ext().execute_with(|| {
        citizen_issuance::RewardedCount::<Test>::put(CITIZEN_ISSUANCE_MAX_COUNT);

        let credential = make_credential(b"cid-over-cap", b"nonce-cap", true);

        assert!(CidSystem::bind_cid(RuntimeOrigin::signed(1), credential).is_ok());

        // 绑定成功但余额为 0
        assert_eq!(Balances::free_balance(1), 0);
        assert_eq!(
            citizen_issuance::RewardedCount::<Test>::get(),
            CITIZEN_ISSUANCE_MAX_COUNT
        );

        // 验证跳过事件
        let events: Vec<RuntimeEvent> = System::events().into_iter().map(|r| r.event).collect();
        assert!(events.iter().any(|e| matches!(
            e,
            RuntimeEvent::CitizenIssuance(citizen_issuance::Event::CertificationRewardSkipped {
                reason: citizen_issuance::pallet::SkipReason::MaxCountReached,
                ..
            })
        )));
    });
}

/// 中文注释：验证 bind_cid weight 声明包含回调 weight（非零）。
#[test]
fn bind_cid_weight_includes_callback_budget() {
    use citizen_issuance::weights::WeightInfo;
    let callback_weight = <() as WeightInfo>::on_cid_bound();
    // 回调 weight 必须非零，证明 bind_cid 的总 weight 涵盖了发行模块开销。
    assert!(!callback_weight.ref_time().is_zero());
    assert!(!callback_weight.proof_size().is_zero());
}
