//! sfid-system pallet 测试套件
//!
//! 覆盖范围:
//! - 旧 bind / unbind / 投票凭证测试(继承自老实现,语义不变)。
//! - ADR-008 Step 2a 4 个新 Pays::No extrinsic + ValidateUnsigned + helper。
//!
//! 命名约定保留任务卡列出的 12 条核心测试,验收清单可逐条对照。

use super::*;
use codec::Encode;
use frame_support::traits::EnsureOrigin;
use frame_support::{assert_noop, assert_ok, derive_impl, parameter_types};
use frame_system as system;
use frame_system::EnsureRoot;
use sp_core::sr25519;
use sp_core::Pair;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::Hash as HashTrait;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionSource};
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::{traits::IdentityLookup, BuildStorage};

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
    pub type SfidSystem = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
}

pub struct TestSfidVerifier;
impl
    SfidVerifier<u64, <Test as frame_system::Config>::Hash, NonceOf<Test>, SignatureOf<Test>>
    for TestSfidVerifier
{
    fn verify(_account: &u64, credential: &CredentialOf<Test>) -> bool {
        !credential.bind_nonce.is_empty() && credential.signature.as_slice() == b"bind-ok"
    }
}

pub struct TestSfidVoteVerifier;
impl
    SfidVoteVerifier<
        u64,
        <Test as frame_system::Config>::Hash,
        NonceOf<Test>,
        SignatureOf<Test>,
    > for TestSfidVoteVerifier
{
    fn verify_vote(
        _account: &u64,
        _binding_id: <Test as frame_system::Config>::Hash,
        _proposal_id: u64,
        _nonce: &NonceOf<Test>,
        signature: &SignatureOf<Test>,
    ) -> bool {
        signature.as_slice() == b"vote-ok"
    }
}

parameter_types! {
    pub const MaxCredentialNonceLength: u32 = 64;
    pub const MaxCredentialSignatureLength: u32 = 64;
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxCredentialNonceLength = MaxCredentialNonceLength;
    type MaxCredentialSignatureLength = MaxCredentialSignatureLength;
    type SfidVerifier = TestSfidVerifier;
    type SfidVoteVerifier = TestSfidVoteVerifier;
    type OnSfidBound = ();
    type UnbindOrigin = EnsureRoot<u64>;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("frame system genesis storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn binding_id(seed: &[u8]) -> <Test as frame_system::Config>::Hash {
    <Test as frame_system::Config>::Hashing::hash(seed)
}

fn nonce(input: &str) -> NonceOf<Test> {
    input
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("nonce should fit")
}

fn signature(input: &str) -> SignatureOf<Test> {
    input
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("signature should fit")
}

fn bind_credential(seed: &[u8], bind_nonce: &str, sig: &str) -> CredentialOf<Test> {
    BindCredential {
        binding_id: binding_id(seed),
        bind_nonce: nonce(bind_nonce),
        signature: signature(sig),
    }
}

// --- 老 bind / unbind / vote 测试(语义保留,unbind 改 Root origin) ---

#[test]
fn bind_succeeds_and_tracks_binding_id() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-a", "nonce-a", "bind-ok");

        assert_ok!(SfidSystem::bind_sfid(
            RuntimeOrigin::signed(1),
            credential.clone()
        ));

        assert_eq!(
            BindingIdToAccount::<Test>::get(credential.binding_id),
            Some(1)
        );
        assert_eq!(
            AccountToBindingId::<Test>::get(1),
            Some(credential.binding_id)
        );
        assert_eq!(BoundCount::<Test>::get(), 1);
    });
}

#[test]
fn bind_rejects_reused_bind_nonce() {
    new_test_ext().execute_with(|| {
        let first = bind_credential(b"binding-a", "same-nonce", "bind-ok");
        let second = bind_credential(b"binding-b", "same-nonce", "bind-ok");

        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), first));
        assert_noop!(
            SfidSystem::bind_sfid(RuntimeOrigin::signed(2), second),
            Error::<Test>::BindNonceAlreadyUsed
        );
    });
}

#[test]
fn bind_allows_account_rebinding_to_new_binding_id() {
    new_test_ext().execute_with(|| {
        let first = bind_credential(b"binding-a", "nonce-a", "bind-ok");
        let second = bind_credential(b"binding-b", "nonce-b", "bind-ok");

        assert_ok!(SfidSystem::bind_sfid(
            RuntimeOrigin::signed(1),
            first.clone()
        ));
        assert_ok!(SfidSystem::bind_sfid(
            RuntimeOrigin::signed(1),
            second.clone()
        ));

        assert!(BindingIdToAccount::<Test>::get(first.binding_id).is_none());
        assert_eq!(BindingIdToAccount::<Test>::get(second.binding_id), Some(1));
        assert_eq!(AccountToBindingId::<Test>::get(1), Some(second.binding_id));
        assert_eq!(BoundCount::<Test>::get(), 1);
    });
}

#[test]
fn vote_credential_is_consumed_once_per_proposal_and_binding_id() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-vote", "bind-nonce", "bind-ok");
        let bid = credential.binding_id;
        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential));

        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::is_eligible(&bid, &1));
        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            7,
            b"vote-nonce",
            b"vote-ok"
        ));
        assert!(!<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            7,
            b"vote-nonce",
            b"vote-ok"
        ));
    });
}

#[test]
fn vote_nonce_is_scoped_per_proposal_and_cannot_replay_within_same_proposal() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-replay", "bind-nonce", "bind-ok");
        let bid = credential.binding_id;
        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential));

        let proposal_a = 10u64;
        let proposal_b = 20u64;

        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            proposal_a,
            b"same-nonce",
            b"vote-ok"
        ));

        assert!(!<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            proposal_a,
            b"same-nonce",
            b"vote-ok"
        ));

        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            proposal_b,
            b"same-nonce",
            b"vote-ok"
        ));
    });
}

#[test]
fn bind_rejects_empty_nonce() {
    new_test_ext().execute_with(|| {
        let empty_credential = BindCredential {
            binding_id: binding_id(b"id-empty"),
            bind_nonce: Vec::<u8>::new().try_into().expect("empty vec fits"),
            signature: signature("bind-ok"),
        };
        assert_noop!(
            SfidSystem::bind_sfid(RuntimeOrigin::signed(1), empty_credential),
            Error::<Test>::EmptyBindNonce
        );
    });
}

#[test]
fn bind_rejects_invalid_signature() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"id-badsig", "nonce-badsig", "bad-sig");
        assert_noop!(
            SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential),
            Error::<Test>::InvalidSfidBindingSignature
        );
    });
}

#[test]
fn bind_rejects_binding_id_owned_by_another_account() {
    new_test_ext().execute_with(|| {
        let credential_1 = bind_credential(b"shared-id", "nonce-1", "bind-ok");
        let credential_2 = bind_credential(b"shared-id", "nonce-2", "bind-ok");

        assert_ok!(SfidSystem::bind_sfid(
            RuntimeOrigin::signed(1),
            credential_1
        ));
        assert_noop!(
            SfidSystem::bind_sfid(RuntimeOrigin::signed(2), credential_2),
            Error::<Test>::BindingIdAlreadyBoundToAnotherAccount
        );
    });
}

#[test]
fn bind_rejects_same_binding_id_already_bound() {
    new_test_ext().execute_with(|| {
        let credential_1 = bind_credential(b"dup-id", "nonce-dup-1", "bind-ok");
        let credential_2 = bind_credential(b"dup-id", "nonce-dup-2", "bind-ok");

        assert_ok!(SfidSystem::bind_sfid(
            RuntimeOrigin::signed(1),
            credential_1
        ));
        assert_noop!(
            SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential_2),
            Error::<Test>::SameBindingIdAlreadyBound
        );
    });
}

#[test]
fn unbind_by_root_origin_succeeds() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-unbind", "nonce-unbind", "bind-ok");
        let bid = credential.binding_id;
        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential));
        assert_eq!(BoundCount::<Test>::get(), 1);

        assert_ok!(SfidSystem::unbind_sfid(RuntimeOrigin::root(), 1));
        assert!(AccountToBindingId::<Test>::get(1).is_none());
        assert!(BindingIdToAccount::<Test>::get(bid).is_none());
        assert_eq!(BoundCount::<Test>::get(), 0);
    });
}

#[test]
fn unbind_rejects_non_root_origin() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-reject", "nonce-reject", "bind-ok");
        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential));

        assert!(SfidSystem::unbind_sfid(RuntimeOrigin::signed(1), 1).is_err());
        assert!(SfidSystem::unbind_sfid(RuntimeOrigin::signed(99), 1).is_err());
    });
}

#[test]
fn unbind_rejects_unbound_target() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            SfidSystem::unbind_sfid(RuntimeOrigin::root(), 99),
            Error::<Test>::NotBound
        );
    });
}

#[test]
fn cleanup_vote_credentials_removes_nonces() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-cleanup", "nonce-cleanup", "bind-ok");
        let bid = credential.binding_id;
        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential));

        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            42,
            b"vote-nonce-c",
            b"vote-ok"
        ));

        <Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::cleanup_vote_credentials(42);

        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            42,
            b"vote-nonce-c",
            b"vote-ok"
        ));
    });
}

// ============================================================================
// ADR-008 Step 2a 新增测试(必加 ≥ 12 条)
// ============================================================================

const PROVINCE: &[u8] = b"liaoning";
const OTHER_PROVINCE: &[u8] = b"jilin";

fn province_bounded(p: &[u8]) -> ProvinceBound {
    p.to_vec().try_into().expect("province fits")
}

fn make_nonce(seed: u8) -> ShengNonce {
    let mut n = [0u8; 32];
    n[0] = seed;
    n
}

fn fresh_keypair(seed_phrase: &str) -> (sr25519::Pair, [u8; 32]) {
    let pair = sr25519::Pair::from_string(&format!("//{}", seed_phrase), None)
        .expect("pair from seed");
    let public = pair.public().0;
    (pair, public)
}

fn build_add_backup_payload(
    province: &[u8],
    slot: Slot,
    new_pubkey: &[u8; 32],
    nonce: &ShengNonce,
) -> [u8; 32] {
    let payload = (
        crate::ADD_BACKUP_DOMAIN,
        province,
        slot,
        new_pubkey,
        nonce,
    );
    blake2_256(&payload.encode())
}

fn build_remove_backup_payload(
    province: &[u8],
    slot: Slot,
    nonce: &ShengNonce,
) -> [u8; 32] {
    let payload = (crate::REMOVE_BACKUP_DOMAIN, province, slot, nonce);
    blake2_256(&payload.encode())
}

fn build_activate_payload(
    province: &[u8],
    admin_pubkey: &[u8; 32],
    signing_pubkey: &[u8; 32],
    nonce: &ShengNonce,
) -> [u8; 32] {
    let payload = (
        crate::ACTIVATE_DOMAIN,
        province,
        admin_pubkey,
        signing_pubkey,
        nonce,
    );
    blake2_256(&payload.encode())
}

fn build_rotate_payload(
    province: &[u8],
    admin_pubkey: &[u8; 32],
    new_signing_pubkey: &[u8; 32],
    nonce: &ShengNonce,
) -> [u8; 32] {
    let payload = (
        crate::ROTATE_DOMAIN,
        province,
        admin_pubkey,
        new_signing_pubkey,
        nonce,
    );
    blake2_256(&payload.encode())
}

fn sign_msg(pair: &sr25519::Pair, msg: &[u8; 32]) -> [u8; 64] {
    pair.sign(msg).0
}

fn activate_main(
    province: &[u8],
    admin_pair: &sr25519::Pair,
    admin_pubkey: &[u8; 32],
    signing_pubkey: &[u8; 32],
    nonce: ShengNonce,
) {
    let payload = build_activate_payload(province, admin_pubkey, signing_pubkey, &nonce);
    let sig = sign_msg(admin_pair, &payload);
    assert_ok!(SfidSystem::activate_sheng_signing_pubkey(
        RuntimeOrigin::none(),
        province.to_vec(),
        *admin_pubkey,
        *signing_pubkey,
        nonce,
        sig,
    ));
}

#[test]
fn activate_first_come_first_serve_on_empty_main() {
    new_test_ext().execute_with(|| {
        let (pair, pubkey) = fresh_keypair("alice-main");
        let signing_key = [9u8; 32];
        activate_main(PROVINCE, &pair, &pubkey, &signing_key, make_nonce(1));

        assert_eq!(
            ShengAdmins::<Test>::get(province_bounded(PROVINCE), Slot::Main),
            Some(pubkey)
        );
        assert_eq!(
            ShengSigningPubkey::<Test>::get(province_bounded(PROVINCE), pubkey),
            Some(signing_key)
        );
    });
}

#[test]
fn activate_existing_admin_writes_signing_pubkey() {
    new_test_ext().execute_with(|| {
        // 1. Main 已激活
        let (main_pair, main_pubkey) = fresh_keypair("alice-main");
        let main_signing = [9u8; 32];
        activate_main(PROVINCE, &main_pair, &main_pubkey, &main_signing, make_nonce(1));

        // 2. Main 添加 backup1
        let (b1_pair, b1_pubkey) = fresh_keypair("bob-backup1");
        let nonce = make_nonce(2);
        let payload = build_add_backup_payload(PROVINCE, Slot::Backup1, &b1_pubkey, &nonce);
        let sig = sign_msg(&main_pair, &payload);
        assert_ok!(SfidSystem::add_sheng_admin_backup(
            RuntimeOrigin::none(),
            PROVINCE.to_vec(),
            Slot::Backup1,
            b1_pubkey,
            nonce,
            sig,
        ));

        // 3. backup1 自己 activate 自己的 signing pubkey
        let b1_signing = [11u8; 32];
        let nonce2 = make_nonce(3);
        let payload2 =
            build_activate_payload(PROVINCE, &b1_pubkey, &b1_signing, &nonce2);
        let sig2 = sign_msg(&b1_pair, &payload2);
        assert_ok!(SfidSystem::activate_sheng_signing_pubkey(
            RuntimeOrigin::none(),
            PROVINCE.to_vec(),
            b1_pubkey,
            b1_signing,
            nonce2,
            sig2,
        ));

        assert_eq!(
            ShengSigningPubkey::<Test>::get(province_bounded(PROVINCE), b1_pubkey),
            Some(b1_signing)
        );
        // Main 的 signing 仍然存在,与 backup1 互不影响
        assert_eq!(
            ShengSigningPubkey::<Test>::get(province_bounded(PROVINCE), main_pubkey),
            Some(main_signing)
        );
    });
}

#[test]
fn activate_unknown_admin_rejected_when_main_filled() {
    new_test_ext().execute_with(|| {
        let (main_pair, main_pubkey) = fresh_keypair("alice-main");
        activate_main(PROVINCE, &main_pair, &main_pubkey, &[9u8; 32], make_nonce(1));

        // 不在花名册的随机 admin 想 activate
        let (stranger_pair, stranger_pubkey) = fresh_keypair("eve-stranger");
        let signing = [12u8; 32];
        let nonce = make_nonce(2);
        let payload =
            build_activate_payload(PROVINCE, &stranger_pubkey, &signing, &nonce);
        let sig = sign_msg(&stranger_pair, &payload);

        assert_noop!(
            SfidSystem::activate_sheng_signing_pubkey(
                RuntimeOrigin::none(),
                PROVINCE.to_vec(),
                stranger_pubkey,
                signing,
                nonce,
                sig,
            ),
            Error::<Test>::Sheng3TierAdminNotInRoster
        );
    });
}

#[test]
fn add_backup_signed_by_main_succeeds() {
    new_test_ext().execute_with(|| {
        let (main_pair, main_pubkey) = fresh_keypair("alice-main");
        activate_main(PROVINCE, &main_pair, &main_pubkey, &[9u8; 32], make_nonce(1));

        let (_b1_pair, b1_pubkey) = fresh_keypair("bob-backup1");
        let nonce = make_nonce(2);
        let payload = build_add_backup_payload(PROVINCE, Slot::Backup1, &b1_pubkey, &nonce);
        let sig = sign_msg(&main_pair, &payload);
        assert_ok!(SfidSystem::add_sheng_admin_backup(
            RuntimeOrigin::none(),
            PROVINCE.to_vec(),
            Slot::Backup1,
            b1_pubkey,
            nonce,
            sig,
        ));
        assert_eq!(
            ShengAdmins::<Test>::get(province_bounded(PROVINCE), Slot::Backup1),
            Some(b1_pubkey)
        );
    });
}

#[test]
fn add_backup_unauthorized_signature_rejected() {
    new_test_ext().execute_with(|| {
        let (main_pair, main_pubkey) = fresh_keypair("alice-main");
        activate_main(PROVINCE, &main_pair, &main_pubkey, &[9u8; 32], make_nonce(1));

        // 使用非 Main 的私钥伪造签名
        let (forger_pair, _forger_pubkey) = fresh_keypair("mallory");
        let (_b_pair, b_pubkey) = fresh_keypair("victim-backup");
        let nonce = make_nonce(2);
        let payload = build_add_backup_payload(PROVINCE, Slot::Backup1, &b_pubkey, &nonce);
        let bad_sig = sign_msg(&forger_pair, &payload);

        assert_noop!(
            SfidSystem::add_sheng_admin_backup(
                RuntimeOrigin::none(),
                PROVINCE.to_vec(),
                Slot::Backup1,
                b_pubkey,
                nonce,
                bad_sig,
            ),
            Error::<Test>::Sheng3TierSignatureInvalid
        );
    });
}

#[test]
fn remove_backup_cascades_to_signing_pubkey() {
    new_test_ext().execute_with(|| {
        let (main_pair, main_pubkey) = fresh_keypair("alice-main");
        activate_main(PROVINCE, &main_pair, &main_pubkey, &[9u8; 32], make_nonce(1));

        let (b1_pair, b1_pubkey) = fresh_keypair("bob-backup1");
        // add backup1
        {
            let nonce = make_nonce(2);
            let payload = build_add_backup_payload(PROVINCE, Slot::Backup1, &b1_pubkey, &nonce);
            let sig = sign_msg(&main_pair, &payload);
            assert_ok!(SfidSystem::add_sheng_admin_backup(
                RuntimeOrigin::none(),
                PROVINCE.to_vec(),
                Slot::Backup1,
                b1_pubkey,
                nonce,
                sig,
            ));
        }
        // backup1 activate own signing
        let b1_signing = [11u8; 32];
        {
            let nonce = make_nonce(3);
            let payload = build_activate_payload(PROVINCE, &b1_pubkey, &b1_signing, &nonce);
            let sig = sign_msg(&b1_pair, &payload);
            assert_ok!(SfidSystem::activate_sheng_signing_pubkey(
                RuntimeOrigin::none(),
                PROVINCE.to_vec(),
                b1_pubkey,
                b1_signing,
                nonce,
                sig,
            ));
        }
        assert_eq!(
            ShengSigningPubkey::<Test>::get(province_bounded(PROVINCE), b1_pubkey),
            Some(b1_signing)
        );

        // Main remove backup1
        let nonce = make_nonce(4);
        let payload = build_remove_backup_payload(PROVINCE, Slot::Backup1, &nonce);
        let sig = sign_msg(&main_pair, &payload);
        assert_ok!(SfidSystem::remove_sheng_admin_backup(
            RuntimeOrigin::none(),
            PROVINCE.to_vec(),
            Slot::Backup1,
            nonce,
            sig,
        ));

        // backup1 admin 槽 + signing pubkey 都被清
        assert!(
            ShengAdmins::<Test>::get(province_bounded(PROVINCE), Slot::Backup1).is_none()
        );
        assert!(
            ShengSigningPubkey::<Test>::get(province_bounded(PROVINCE), b1_pubkey).is_none()
        );
    });
}

#[test]
fn rotate_signing_pubkey_replaces_value() {
    new_test_ext().execute_with(|| {
        let (main_pair, main_pubkey) = fresh_keypair("alice-main");
        let old_signing = [9u8; 32];
        activate_main(PROVINCE, &main_pair, &main_pubkey, &old_signing, make_nonce(1));

        let new_signing = [99u8; 32];
        let nonce = make_nonce(2);
        let payload =
            build_rotate_payload(PROVINCE, &main_pubkey, &new_signing, &nonce);
        let sig = sign_msg(&main_pair, &payload);
        assert_ok!(SfidSystem::rotate_sheng_signing_pubkey(
            RuntimeOrigin::none(),
            PROVINCE.to_vec(),
            main_pubkey,
            new_signing,
            nonce,
            sig,
        ));
        assert_eq!(
            ShengSigningPubkey::<Test>::get(province_bounded(PROVINCE), main_pubkey),
            Some(new_signing)
        );
    });
}

#[test]
fn rotate_unknown_admin_rejected() {
    new_test_ext().execute_with(|| {
        let (main_pair, main_pubkey) = fresh_keypair("alice-main");
        activate_main(PROVINCE, &main_pair, &main_pubkey, &[9u8; 32], make_nonce(1));

        // 不在花名册的随机 admin 想 rotate
        let (stranger_pair, stranger_pubkey) = fresh_keypair("eve");
        let new_signing = [88u8; 32];
        let nonce = make_nonce(2);
        let payload =
            build_rotate_payload(PROVINCE, &stranger_pubkey, &new_signing, &nonce);
        let sig = sign_msg(&stranger_pair, &payload);
        assert_noop!(
            SfidSystem::rotate_sheng_signing_pubkey(
                RuntimeOrigin::none(),
                PROVINCE.to_vec(),
                stranger_pubkey,
                new_signing,
                nonce,
                sig,
            ),
            Error::<Test>::Sheng3TierAdminNotInRoster
        );
    });
}

#[test]
fn pays_no_zero_balance_account_succeeds() {
    // 关键测试:Pays::No + ensure_none 路径下,无需任何 signed account / 余额,
    // SFID 后端零余额账户也能成功调用。
    new_test_ext().execute_with(|| {
        let (pair, pubkey) = fresh_keypair("zero-balance-admin");
        let signing = [42u8; 32];
        let nonce = make_nonce(7);
        let payload = build_activate_payload(PROVINCE, &pubkey, &signing, &nonce);
        let sig = sign_msg(&pair, &payload);

        // Origin::none() 不需要任何账户余额。
        let result = SfidSystem::activate_sheng_signing_pubkey(
            RuntimeOrigin::none(),
            PROVINCE.to_vec(),
            pubkey,
            signing,
            nonce,
            sig,
        );
        assert_ok!(result);
        assert_eq!(
            ShengAdmins::<Test>::get(province_bounded(PROVINCE), Slot::Main),
            Some(pubkey)
        );
    });
}

#[test]
fn cross_province_admin_cannot_modify_other_province() {
    new_test_ext().execute_with(|| {
        // province A:Main = alice
        let (alice_pair, alice_pubkey) = fresh_keypair("alice-main-A");
        activate_main(PROVINCE, &alice_pair, &alice_pubkey, &[9u8; 32], make_nonce(1));

        // province B:Main = bob
        let (bob_pair, bob_pubkey) = fresh_keypair("bob-main-B");
        activate_main(
            OTHER_PROVINCE,
            &bob_pair,
            &bob_pubkey,
            &[10u8; 32],
            make_nonce(2),
        );

        // alice 想给 province B 加 backup → 应被拒绝(B 的 Main 是 bob,alice 签名验不过)
        let (_attacker_pair, attacker_pubkey) = fresh_keypair("attacker");
        let nonce = make_nonce(3);
        let payload =
            build_add_backup_payload(OTHER_PROVINCE, Slot::Backup1, &attacker_pubkey, &nonce);
        let bad_sig = sign_msg(&alice_pair, &payload);

        assert_noop!(
            SfidSystem::add_sheng_admin_backup(
                RuntimeOrigin::none(),
                OTHER_PROVINCE.to_vec(),
                Slot::Backup1,
                attacker_pubkey,
                nonce,
                bad_sig,
            ),
            Error::<Test>::Sheng3TierSignatureInvalid
        );

        // OTHER_PROVINCE 的 Backup1 仍为空
        assert!(
            ShengAdmins::<Test>::get(province_bounded(OTHER_PROVINCE), Slot::Backup1).is_none()
        );
    });
}

#[test]
fn nonce_replay_rejected() {
    new_test_ext().execute_with(|| {
        let (pair, pubkey) = fresh_keypair("alice-main");
        let signing = [9u8; 32];
        let nonce_a = make_nonce(50);
        let payload = build_activate_payload(PROVINCE, &pubkey, &signing, &nonce_a);
        let sig = sign_msg(&pair, &payload);

        // 第一次成功
        assert_ok!(SfidSystem::activate_sheng_signing_pubkey(
            RuntimeOrigin::none(),
            PROVINCE.to_vec(),
            pubkey,
            signing,
            nonce_a,
            sig,
        ));

        // 同 nonce 第二次 reject(防重放)
        // 重新构造一笔不同业务但同 nonce 的 rotate(签 rotate payload)
        let new_signing = [55u8; 32];
        let payload_r = build_rotate_payload(PROVINCE, &pubkey, &new_signing, &nonce_a);
        let sig_r = sign_msg(&pair, &payload_r);
        assert_noop!(
            SfidSystem::rotate_sheng_signing_pubkey(
                RuntimeOrigin::none(),
                PROVINCE.to_vec(),
                pubkey,
                new_signing,
                nonce_a,
                sig_r,
            ),
            Error::<Test>::Sheng3TierNonceUsed
        );
    });
}

#[test]
fn signature_with_wrong_payload_rejected() {
    // 攻击者用对 nonce_a 的有效签名,试图以 nonce_b 进入 → ValidateUnsigned 端必须查 BadProof
    new_test_ext().execute_with(|| {
        let (pair, pubkey) = fresh_keypair("alice-main");
        let signing = [9u8; 32];
        let nonce_a = make_nonce(60);
        let nonce_b = make_nonce(61);

        let payload_a = build_activate_payload(PROVINCE, &pubkey, &signing, &nonce_a);
        let sig_a = sign_msg(&pair, &payload_a);

        // 把 nonce_a 的签名挪给 nonce_b 提交 → 链上会按 nonce_b 算 payload,验签必败
        assert_noop!(
            SfidSystem::activate_sheng_signing_pubkey(
                RuntimeOrigin::none(),
                PROVINCE.to_vec(),
                pubkey,
                signing,
                nonce_b,
                sig_a,
            ),
            Error::<Test>::Sheng3TierSignatureInvalid
        );
    });
}

// --- ValidateUnsigned 单独测试(bonus,但常用) ---

#[test]
fn validate_unsigned_rejects_unknown_call_path() {
    new_test_ext().execute_with(|| {
        // bind_sfid 是 signed call,不应通过 ValidateUnsigned 入口
        let credential = bind_credential(b"bind-x", "n", "bind-ok");
        let call = Call::<Test>::bind_sfid { credential };
        let result =
            <Pallet<Test> as ValidateUnsigned>::validate_unsigned(TransactionSource::External, &call);
        match result {
            Err(sp_runtime::transaction_validity::TransactionValidityError::Invalid(
                InvalidTransaction::Call,
            )) => {}
            other => panic!("expected InvalidTransaction::Call, got {:?}", other),
        }
    });
}

#[test]
fn validate_unsigned_passes_for_valid_activate() {
    new_test_ext().execute_with(|| {
        let (pair, pubkey) = fresh_keypair("alice-main");
        let signing = [9u8; 32];
        let nonce_v = make_nonce(70);
        let payload = build_activate_payload(PROVINCE, &pubkey, &signing, &nonce_v);
        let sig = sign_msg(&pair, &payload);

        let call = Call::<Test>::activate_sheng_signing_pubkey {
            province: PROVINCE.to_vec(),
            admin_pubkey: pubkey,
            signing_pubkey: signing,
            nonce: nonce_v,
            sig,
        };
        let result =
            <Pallet<Test> as ValidateUnsigned>::validate_unsigned(TransactionSource::External, &call);
        assert!(result.is_ok(), "validate_unsigned should pass: {:?}", result);
    });
}

#[test]
fn helpers_is_sheng_admin_and_main_work() {
    new_test_ext().execute_with(|| {
        let (main_pair, main_pubkey) = fresh_keypair("alice-main");
        activate_main(PROVINCE, &main_pair, &main_pubkey, &[9u8; 32], make_nonce(1));

        assert_eq!(
            SfidSystem::is_sheng_admin(PROVINCE, &main_pubkey),
            Some(Slot::Main)
        );
        assert!(SfidSystem::is_sheng_main(PROVINCE, &main_pubkey));

        let (_bp, bogus) = fresh_keypair("bogus");
        assert!(SfidSystem::is_sheng_admin(PROVINCE, &bogus).is_none());
        assert!(!SfidSystem::is_sheng_main(PROVINCE, &bogus));

        assert_eq!(
            SfidSystem::sheng_signing_pubkey_for_admin(PROVINCE, &main_pubkey),
            Some([9u8; 32])
        );
    });
}

#[test]
fn unbind_origin_is_root_in_test_runtime() {
    // sanity:确认 Test runtime 的 UnbindOrigin = Root
    new_test_ext().execute_with(|| {
        assert!(<EnsureRoot<u64> as EnsureOrigin<RuntimeOrigin>>::try_origin(
            RuntimeOrigin::root()
        )
        .is_ok());
    });
}
