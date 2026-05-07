#![cfg(test)]

extern crate alloc;
use alloc::vec::Vec;

use super::*;
use crate::configs::is_nrc_admin;
use crate::configs::*;
use crate::ResolutionDestro;
use frame_support::assert_ok;
use frame_support::traits::{Contains, Currency, EnsureOrigin, FindAuthor};
use organization_manage::DuoqianReservedAddressChecker;
use primitives::china::china_cb::CHINA_CB;
use primitives::derive::subject_id_from_sfid_number;
use sfid_system::{SfidVerifier, SfidVoteVerifier};
use sp_core::{sr25519, Pair};
use sp_io::hashing::blake2_256;
use sp_runtime::{traits::Hash as HashT, traits::IdentifyAccount, BuildStorage, MultiSigner};
use votingengine::{
    InternalAdminProvider, JointVoteResultCallback, PopulationSnapshotVerifier, SfidEligibility,
};
use institution_asset::{InstitutionAsset, InstitutionAssetAction};

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
    let province: Vec<u8> = b"liaoning".to_vec();
    let bounded: sfid_system::pallet::ProvinceBound =
        province.clone().try_into().expect("province fits");

    let main_pair = sr25519::Pair::from_string("//main-step3", None).expect("pair");
    let main_signing_pair =
        sr25519::Pair::from_string("//main-signing-step3", None).expect("pair");
    let main_admin_pubkey = main_pair.public().0;
    let main_signing_pubkey = main_signing_pair.public().0;

    let backup_pair = sr25519::Pair::from_string("//backup1-step3", None).expect("pair");
    let backup_signing_pair =
        sr25519::Pair::from_string("//backup1-signing-step3", None).expect("pair");
    let backup_admin_pubkey = backup_pair.public().0;
    let backup_signing_pubkey = backup_signing_pair.public().0;

    sfid_system::pallet::ShengAdmins::<Runtime>::insert(
        &bounded,
        sfid_system::Slot::Main,
        main_admin_pubkey,
    );
    sfid_system::pallet::ShengAdmins::<Runtime>::insert(
        &bounded,
        sfid_system::Slot::Backup1,
        backup_admin_pubkey,
    );
    sfid_system::pallet::ShengSigningPubkey::<Runtime>::insert(
        &bounded,
        main_admin_pubkey,
        main_signing_pubkey,
    );
    sfid_system::pallet::ShengSigningPubkey::<Runtime>::insert(
        &bounded,
        backup_admin_pubkey,
        backup_signing_pubkey,
    );

    (
        main_signing_pair,
        main_admin_pubkey,
        backup_signing_pair,
        backup_admin_pubkey,
        province,
    )
}

fn build_bind_credential(
    signing_pair: &sr25519::Pair,
    signer_admin_pubkey: &[u8; 32],
    province: &[u8],
    account: &AccountId,
    binding_id: Hash,
    bind_nonce: &sfid_system::pallet::NonceOf<Runtime>,
) -> sfid_system::pallet::CredentialOf<Runtime> {
    let payload = (
        primitives::core_const::DUOQIAN_DOMAIN,
        primitives::core_const::OP_SIGN_BIND,
        frame_system::Pallet::<Runtime>::block_hash(0),
        account,
        binding_id,
        bind_nonce.as_slice(),
        province,
        signer_admin_pubkey,
    );
    let msg = blake2_256(&payload.encode());
    let sig = signing_pair.sign(&msg);
    let signature: sfid_system::pallet::SignatureOf<Runtime> =
        sig.0.to_vec().try_into().expect("signature fits");
    sfid_system::BindCredential {
        binding_id,
        bind_nonce: bind_nonce.clone(),
        province: province.to_vec().try_into().expect("province fits"),
        signer_admin_pubkey: *signer_admin_pubkey,
        signature,
    }
}

fn build_vote_signature(
    signing_pair: &sr25519::Pair,
    signer_admin_pubkey: &[u8; 32],
    province: &[u8],
    account: &AccountId,
    binding_id: Hash,
    proposal_id: u64,
    vote_nonce: &sfid_system::pallet::NonceOf<Runtime>,
) -> sfid_system::pallet::SignatureOf<Runtime> {
    let payload = (
        primitives::core_const::DUOQIAN_DOMAIN,
        primitives::core_const::OP_SIGN_VOTE,
        frame_system::Pallet::<Runtime>::block_hash(0),
        account,
        binding_id,
        proposal_id,
        vote_nonce.as_slice(),
        province,
        signer_admin_pubkey,
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
    signer_admin_pubkey: &[u8; 32],
    province: &[u8],
    who: &AccountId,
    eligible_total: u64,
    pop_nonce: &votingengine::pallet::VoteNonceOf<Runtime>,
) -> votingengine::pallet::VoteSignatureOf<Runtime> {
    let payload = (
        primitives::core_const::DUOQIAN_DOMAIN,
        primitives::core_const::OP_SIGN_POP,
        frame_system::Pallet::<Runtime>::block_hash(0),
        who,
        eligible_total,
        pop_nonce.as_slice(),
        province,
        signer_admin_pubkey,
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
    AccountId::new(primitives::china::china_ch::CHINA_CH[0].stake_address)
}

fn reserved_main_account() -> AccountId {
    AccountId::new(primitives::china::china_cb::CHINA_CB[1].main_address)
}

fn reserved_fee_account() -> AccountId {
    AccountId::new(primitives::china::china_ch::CHINA_CH[0].fee_address)
}

fn ordinary_account() -> AccountId {
    AccountId::new([99u8; 32])
}

mod cases;
