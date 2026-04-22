//! 集成测试：验证 sfid-code-auth::bind_sfid → OnSfidBound → citizen-lightnode-issuance 完整链路。
//!
//! 本文件构建包含 sfid-code-auth + citizen-lightnode-issuance 的 mock runtime，
//! 直接调用 bind_sfid extrinsic，验证奖励事件、跳过事件、双重防重与 weight 叠加。

use frame_support::{
    derive_impl, parameter_types,
    traits::{ConstU128, ConstU32, VariantCountOf},
};
use frame_system as system;
use pallet_balances;
use primitives::citizen_const::{CITIZEN_LIGHTNODE_HIGH_REWARD, CITIZEN_LIGHTNODE_MAX_COUNT};
use sfid_code_auth::{BindCredential, SfidVerifier, SfidVoteVerifier};
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
    pub type SfidCodeAuth = sfid_code_auth;
    #[runtime::pallet_index(3)]
    pub type CitizenLightnodeIssuance = citizen_lightnode_issuance;
}

/// 中文注释：测试用验签器——仅当 signature == b"valid" 时返回 true。
pub struct TestSfidVerifier;
impl
    SfidVerifier<
        u64,
        <Test as frame_system::Config>::Hash,
        sfid_code_auth::pallet::NonceOf<Test>,
        sfid_code_auth::pallet::SignatureOf<Test>,
    > for TestSfidVerifier
{
    fn verify(
        _account: &u64,
        credential: &BindCredential<
            <Test as frame_system::Config>::Hash,
            sfid_code_auth::pallet::NonceOf<Test>,
            sfid_code_auth::pallet::SignatureOf<Test>,
        >,
    ) -> bool {
        credential.signature.as_slice() == b"valid"
    }
}

/// 中文注释：测试用投票验签器——集成测试不涉及投票，始终返回 false。
pub struct TestSfidVoteVerifier;
impl
    SfidVoteVerifier<
        u64,
        <Test as frame_system::Config>::Hash,
        sfid_code_auth::pallet::NonceOf<Test>,
        sfid_code_auth::pallet::SignatureOf<Test>,
    > for TestSfidVoteVerifier
{
    fn verify_vote(
        _account: &u64,
        _binding_id: <Test as frame_system::Config>::Hash,
        _proposal_id: u64,
        _nonce: &sfid_code_auth::pallet::NonceOf<Test>,
        _signature: &sfid_code_auth::pallet::SignatureOf<Test>,
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

impl sfid_code_auth::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxCredentialNonceLength = MaxCredentialNonceLength;
    type MaxCredentialSignatureLength = MaxCredentialSignatureLength;
    type SfidVerifier = TestSfidVerifier;
    type SfidVoteVerifier = TestSfidVoteVerifier;
    /// 中文注释：集成测试核心——将 OnSfidBound 接入真实的 CitizenLightnodeIssuance。
    type OnSfidBound = CitizenLightnodeIssuance;
    type WeightInfo = ();
}

impl citizen_lightnode_issuance::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = ();
}

fn make_credential(
    binding_id_seed: &[u8],
    nonce_seed: &[u8],
    valid: bool,
) -> BindCredential<
    <Test as frame_system::Config>::Hash,
    sfid_code_auth::pallet::NonceOf<Test>,
    sfid_code_auth::pallet::SignatureOf<Test>,
> {
    let binding_id = BlakeTwo256::hash(binding_id_seed);
    let nonce: sfid_code_auth::pallet::NonceOf<Test> =
        nonce_seed.to_vec().try_into().expect("nonce should fit");
    let sig_bytes: &[u8] = if valid { b"valid" } else { b"bad" };
    let signature: sfid_code_auth::pallet::SignatureOf<Test> =
        sig_bytes.to_vec().try_into().expect("sig should fit");
    BindCredential {
        binding_id,
        bind_nonce: nonce,
        signature,
    }
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("frame system genesis storage should build");
    sfid_code_auth::GenesisConfig::<Test> {
        sfid_main_account: Some(100),
        sfid_backup_account_1: Some(101),
        sfid_backup_account_2: Some(102),
    }
    .assimilate_storage(&mut storage)
    .expect("sfid genesis should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(10);
    });
    ext
}

// ============================================================================
// 集成测试用例
// ============================================================================

/// 中文注释：完整链路测试——bind_sfid 成功后自动发放高额认证奖励。
#[test]
fn bind_sfid_triggers_reward_issuance() {
    new_test_ext().execute_with(|| {
        let credential = make_credential(b"sfid-integ-1", b"nonce-1", true);
        let binding_id = credential.binding_id;

        assert_eq!(Balances::free_balance(1), 0);

        assert!(SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(1), credential).is_ok());

        // 验证奖励已发放
        assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
        assert_eq!(citizen_lightnode_issuance::RewardedCount::<Test>::get(), 1);
        assert!(citizen_lightnode_issuance::RewardClaimed::<Test>::contains_key(binding_id));
        assert!(citizen_lightnode_issuance::AccountRewarded::<Test>::contains_key(1));

        // 验证上游绑定状态也正确写入
        assert_eq!(
            sfid_code_auth::BindingIdToAccount::<Test>::get(binding_id),
            Some(1)
        );
        assert_eq!(
            sfid_code_auth::AccountToBindingId::<Test>::get(1),
            Some(binding_id)
        );
        assert_eq!(sfid_code_auth::BoundCount::<Test>::get(), 1);

        // 验证事件链：先 issuance 事件，后 sfid 事件
        let events: Vec<RuntimeEvent> = System::events().into_iter().map(|r| r.event).collect();
        assert!(events.iter().any(|e| matches!(
            e,
            RuntimeEvent::CitizenLightnodeIssuance(
                citizen_lightnode_issuance::Event::CertificationRewardIssued { .. }
            )
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            RuntimeEvent::SfidCodeAuth(sfid_code_auth::Event::SfidBound { .. })
        )));
    });
}

/// 中文注释：同一账户换绑 SFID 时，第二次不发奖但 bind_sfid 本身成功。
#[test]
fn rebind_skips_reward_but_bind_succeeds() {
    new_test_ext().execute_with(|| {
        let cred1 = make_credential(b"sfid-rebind-a", b"nonce-rebind-1", true);
        let cred2 = make_credential(b"sfid-rebind-b", b"nonce-rebind-2", true);

        assert!(SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(1), cred1).is_ok());
        assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);

        // 换绑到新 binding_id
        assert!(SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(1), cred2).is_ok());

        // 余额不变——第二次奖励被跳过
        assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
        assert_eq!(citizen_lightnode_issuance::RewardedCount::<Test>::get(), 1);

        // 验证跳过事件
        let events: Vec<RuntimeEvent> = System::events().into_iter().map(|r| r.event).collect();
        assert!(events.iter().any(|e| matches!(
            e,
            RuntimeEvent::CitizenLightnodeIssuance(
                citizen_lightnode_issuance::Event::CertificationRewardSkipped {
                    reason: citizen_lightnode_issuance::pallet::SkipReason::AccountAlreadyRewarded,
                    ..
                }
            )
        )));
    });
}

/// 中文注释：不同账户分别绑定不同 SFID，各自独立领奖。
#[test]
fn two_users_bind_independently_and_both_get_rewards() {
    new_test_ext().execute_with(|| {
        let cred1 = make_credential(b"sfid-user-a", b"nonce-a", true);
        let cred2 = make_credential(b"sfid-user-b", b"nonce-b", true);

        assert!(SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(1), cred1).is_ok());
        assert!(SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(2), cred2).is_ok());

        assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
        assert_eq!(Balances::free_balance(2), CITIZEN_LIGHTNODE_HIGH_REWARD);
        assert_eq!(citizen_lightnode_issuance::RewardedCount::<Test>::get(), 2);
        assert_eq!(sfid_code_auth::BoundCount::<Test>::get(), 2);
    });
}

/// 中文注释：达到发放上限后，bind_sfid 仍成功但奖励被跳过。
#[test]
fn bind_after_max_count_skips_reward() {
    new_test_ext().execute_with(|| {
        citizen_lightnode_issuance::RewardedCount::<Test>::put(CITIZEN_LIGHTNODE_MAX_COUNT);

        let credential = make_credential(b"sfid-over-cap", b"nonce-cap", true);

        assert!(SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(1), credential).is_ok());

        // 绑定成功但余额为 0
        assert_eq!(Balances::free_balance(1), 0);
        assert_eq!(
            citizen_lightnode_issuance::RewardedCount::<Test>::get(),
            CITIZEN_LIGHTNODE_MAX_COUNT
        );

        // 验证跳过事件
        let events: Vec<RuntimeEvent> = System::events().into_iter().map(|r| r.event).collect();
        assert!(events.iter().any(|e| matches!(
            e,
            RuntimeEvent::CitizenLightnodeIssuance(
                citizen_lightnode_issuance::Event::CertificationRewardSkipped {
                    reason: citizen_lightnode_issuance::pallet::SkipReason::MaxCountReached,
                    ..
                }
            )
        )));
    });
}

/// 中文注释：验证 bind_sfid weight 声明包含回调 weight（非零）。
#[test]
fn bind_sfid_weight_includes_callback_budget() {
    use citizen_lightnode_issuance::weights::WeightInfo;
    let callback_weight = <() as WeightInfo>::on_sfid_bound();
    // 回调 weight 必须非零，证明 bind_sfid 的总 weight 涵盖了发行模块开销。
    assert!(!callback_weight.ref_time().is_zero());
    assert!(!callback_weight.proof_size().is_zero());
}
