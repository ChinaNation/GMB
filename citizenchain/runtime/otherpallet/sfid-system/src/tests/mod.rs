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
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionSource};
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
impl SfidVerifier<u64, <Test as frame_system::Config>::Hash, NonceOf<Test>, SignatureOf<Test>>
    for TestSfidVerifier
{
    fn verify(_account: &u64, credential: &CredentialOf<Test>) -> bool {
        !credential.bind_nonce.is_empty() && credential.signature.as_slice() == b"bind-ok"
    }
}

pub struct TestSfidVoteVerifier;
impl SfidVoteVerifier<u64, <Test as frame_system::Config>::Hash, NonceOf<Test>, SignatureOf<Test>>
    for TestSfidVoteVerifier
{
    fn verify_vote(
        _account: &u64,
        _binding_id: <Test as frame_system::Config>::Hash,
        _proposal_id: u64,
        _nonce: &NonceOf<Test>,
        signature: &SignatureOf<Test>,
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
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
        // ADR-008 step3:每个 BindCredential 必带 (province_name, signer_admin_pubkey)。
        // 测试用占位值即可,sfid-system 自身不解析这两个字段(真实双层校验留 runtime verifier)。
        province_name: b"liaoning".to_vec().try_into().expect("province_name fits"),
        signer_admin_pubkey: [7u8; 32],
        signature: signature(sig),
    }
}

mod cases;
