//! cid-system pallet 测试套件。
//!
//! 中文注释:本模块当前只负责 CID 绑定/解绑和公民投票凭证消费。
//! 签发管理员集合不在本 pallet 内维护,由 runtime verifier 对接 admins 模块。

use super::*;
use frame_support::{derive_impl, parameter_types};
use frame_system as system;
use frame_system::EnsureRoot;
use sp_runtime::{traits::Hash, traits::IdentityLookup, BuildStorage};

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
    pub type CidSystem = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
}

pub struct TestCidVerifier;
impl CidVerifier<u64, <Test as frame_system::Config>::Hash, NonceOf<Test>, SignatureOf<Test>>
    for TestCidVerifier
{
    fn verify(_account: &u64, credential: &CredentialOf<Test>) -> bool {
        !credential.issuer_cid_number.is_empty()
            && !credential.scope_province_name.is_empty()
            && credential.signature.as_slice() == b"bind-ok"
    }
}

pub struct TestCidVoteVerifier;
impl CidVoteVerifier<u64, <Test as frame_system::Config>::Hash, NonceOf<Test>, SignatureOf<Test>>
    for TestCidVoteVerifier
{
    fn verify_vote(
        _account: &u64,
        _binding_id: <Test as frame_system::Config>::Hash,
        _proposal_id: u64,
        _nonce: &NonceOf<Test>,
        signature: &SignatureOf<Test>,
        issuer_cid_number: &[u8],
        _issuer_main_account: &u64,
        _signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        !issuer_cid_number.is_empty()
            && !scope_province_name.is_empty()
            && signature.as_slice() == b"vote-ok"
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
    type CidVerifier = TestCidVerifier;
    type CidVoteVerifier = TestCidVoteVerifier;
    type OnCidBound = ();
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
        issuer_cid_number: b"CID-ISSUER".to_vec().try_into().expect("issuer cid fits"),
        issuer_main_account: 99,
        signer_pubkey: [7u8; 32],
        scope_province_name: b"liaoning".to_vec().try_into().expect("scope fits"),
        scope_city_name: b"shenyang".to_vec().try_into().expect("scope fits"),
        signature: signature(sig),
    }
}

mod cases;
