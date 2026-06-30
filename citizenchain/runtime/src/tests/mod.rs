#![cfg(test)]

extern crate alloc;
use alloc::vec::Vec;

use super::*;
use crate::configs::is_nrc_admin;
use crate::configs::*;
use crate::ResolutionDestro;
use cid_system::{CidVerifier, CidVoteVerifier};
use frame_support::assert_ok;
use frame_support::traits::{Contains, Currency, EnsureOrigin, FindAuthor};
use institution_asset::{InstitutionAsset, InstitutionAssetAction};
use primitives::cid::china::china_cb::CHINA_CB;
use primitives::multisig::ReservedAccountGuard;
use sp_core::{sr25519, Pair};
use sp_io::hashing::blake2_256;
use sp_runtime::{traits::Hash as HashT, traits::IdentifyAccount, BuildStorage, MultiSigner};
use votingengine::{
    CidEligibility, InternalAdminProvider, JointVoteResultCallback, PopulationSnapshotVerifier,
};

fn new_test_ext() -> sp_io::TestExternalities {
    let storage = crate::RuntimeGenesisConfig::default()
        .build_storage()
        .expect("runtime test storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

fn setup_step3_test_admins() -> (sr25519::Pair, [u8; 32], sr25519::Pair, [u8; 32], Vec<u8>) {
    let province_name: Vec<u8> = b"liaoning".to_vec();
    let issuer_main_account = test_issuer_main_account();

    let main_pair = sr25519::Pair::from_string("//main-step3", None).expect("pair");
    let main_admin_pubkey = main_pair.public().0;
    let backup_pair = sr25519::Pair::from_string("//backup1-step3", None).expect("pair");
    let backup_admin_pubkey = backup_pair.public().0;

    let admin_accounts = vec![
        AccountId::new(main_admin_pubkey),
        AccountId::new(backup_admin_pubkey),
    ];
    public_admins::pallet::AdminAccounts::<Runtime>::insert(
        issuer_main_account,
        admin_primitives::AdminAccount {
            institution_code: votingengine::types::PRC,
            kind: admin_primitives::AdminAccountKind::PublicInstitution,
            // 固定治理机构管理员集合存 AdminProfile(来源 Genesis、逐人 meta 暂空)。
            admins: admin_accounts
                .iter()
                .cloned()
                .map(|account| admin_primitives::AdminProfile {
                    account,
                    admin_cid_number: Default::default(),
                    name: Default::default(),
                    admin_role: Default::default(),
                    term_start: 0,
                    term_end: 0,
                    source: admin_primitives::AdminSource::Genesis,
                })
                .collect::<Vec<_>>()
                .try_into()
                .expect("test admins should fit"),
            creator: AccountId::new(main_admin_pubkey),
            created_at: Default::default(),
            updated_at: Default::default(),
            status: admin_primitives::AdminAccountStatus::Active,
        },
    );

    (
        main_pair,
        main_admin_pubkey,
        backup_pair,
        backup_admin_pubkey,
        province_name,
    )
}

fn test_issuer_cid_number() -> Vec<u8> {
    b"CB-TEST-ISSUER".to_vec()
}

fn test_issuer_main_account() -> AccountId {
    AccountId::new([77u8; 32])
}

fn test_scope_city_name() -> Vec<u8> {
    b"shenyang".to_vec()
}

fn build_bind_credential(
    signing_pair: &sr25519::Pair,
    signer_pubkey: &[u8; 32],
    scope_province_name: &[u8],
    account: &AccountId,
    binding_id: Hash,
    bind_nonce: &cid_system::pallet::NonceOf<Runtime>,
) -> cid_system::pallet::CredentialOf<Runtime> {
    let issuer_cid_number = test_issuer_cid_number();
    let issuer_main_account = test_issuer_main_account();
    let scope_city_name = test_scope_city_name();
    let payload = (
        primitives::core_const::GMB,
        primitives::core_const::OP_SIGN_BIND,
        frame_system::Pallet::<Runtime>::block_hash(0),
        account,
        binding_id,
        bind_nonce.as_slice(),
        issuer_cid_number.as_slice(),
        &issuer_main_account,
        signer_pubkey,
        scope_province_name,
        scope_city_name.as_slice(),
    );
    let msg = blake2_256(&payload.encode());
    let sig = signing_pair.sign(&msg);
    let signature: cid_system::pallet::SignatureOf<Runtime> =
        sig.0.to_vec().try_into().expect("signature fits");
    cid_system::BindCredential {
        binding_id,
        bind_nonce: bind_nonce.clone(),
        issuer_cid_number: issuer_cid_number.try_into().expect("issuer cid fits"),
        issuer_main_account,
        signer_pubkey: *signer_pubkey,
        scope_province_name: scope_province_name
            .to_vec()
            .try_into()
            .expect("scope province fits"),
        scope_city_name: scope_city_name.try_into().expect("scope city fits"),
        signature,
    }
}

fn build_vote_signature(
    signing_pair: &sr25519::Pair,
    signer_pubkey: &[u8; 32],
    scope_province_name: &[u8],
    account: &AccountId,
    binding_id: Hash,
    proposal_id: u64,
    vote_nonce: &cid_system::pallet::NonceOf<Runtime>,
) -> cid_system::pallet::SignatureOf<Runtime> {
    let issuer_cid_number = test_issuer_cid_number();
    let issuer_main_account = test_issuer_main_account();
    let scope_city_name = test_scope_city_name();
    let payload = (
        primitives::core_const::GMB,
        primitives::core_const::OP_SIGN_VOTE,
        frame_system::Pallet::<Runtime>::block_hash(0),
        account,
        binding_id,
        proposal_id,
        vote_nonce.as_slice(),
        issuer_cid_number.as_slice(),
        &issuer_main_account,
        signer_pubkey,
        scope_province_name,
        scope_city_name.as_slice(),
    );
    let msg = blake2_256(&payload.encode());
    signing_pair
        .sign(&msg)
        .0
        .to_vec()
        .try_into()
        .expect("signature fits")
}

fn build_pop_signature(
    signing_pair: &sr25519::Pair,
    signer_pubkey: &[u8; 32],
    scope_province_name: &[u8],
    who: &AccountId,
    eligible_total: u64,
    pop_nonce: &votingengine::pallet::VoteNonceOf<Runtime>,
) -> votingengine::pallet::VoteSignatureOf<Runtime> {
    let issuer_cid_number = test_issuer_cid_number();
    let issuer_main_account = test_issuer_main_account();
    let scope_city_name = test_scope_city_name();
    let payload = (
        primitives::core_const::GMB,
        primitives::core_const::OP_SIGN_POP,
        frame_system::Pallet::<Runtime>::block_hash(0),
        who,
        eligible_total,
        pop_nonce.as_slice(),
        issuer_cid_number.as_slice(),
        &issuer_main_account,
        signer_pubkey,
        scope_province_name,
        scope_city_name.as_slice(),
    );
    let msg = blake2_256(&payload.encode());
    signing_pair
        .sign(&msg)
        .0
        .to_vec()
        .try_into()
        .expect("signature fits")
}

fn stake_account() -> AccountId {
    AccountId::new(primitives::cid::china::china_ch::CHINA_CH[0].stake_account)
}

fn reserved_main_account() -> AccountId {
    AccountId::new(primitives::cid::china::china_cb::CHINA_CB[1].main_account)
}

fn reserved_fee_account() -> AccountId {
    AccountId::new(primitives::cid::china::china_ch::CHINA_CH[0].fee_account)
}

fn ordinary_account() -> AccountId {
    AccountId::new([99u8; 32])
}

mod cases;
