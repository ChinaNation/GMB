#![cfg(test)]

extern crate alloc;
use alloc::vec::Vec;

use super::*;
use crate::configs::is_nrc_admin;
use crate::configs::*;
use crate::ResolutionDestro;
use frame_support::traits::{Contains, Currency, EnsureOrigin, FindAuthor};
use frame_support::{assert_noop, assert_ok};
use institution_asset::{InstitutionAsset, InstitutionAssetAction};
use primitives::cid::china::china_cb::CHINA_CB;
use primitives::multisig::ReservedAccountGuard;
use sp_core::{sr25519, Pair};
use sp_io::hashing::blake2_256;
use sp_runtime::{traits::Hash as HashT, traits::IdentifyAccount, BuildStorage, MultiSigner};
use votingengine::{CitizenIdentityReader, InternalAdminProvider, JointVoteResultCallback};

fn new_test_ext() -> sp_io::TestExternalities {
    let storage = crate::RuntimeGenesisConfig::default()
        .build_storage()
        .expect("runtime test storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
        // 链上时间 = 2026-07-02 00:00 UTC,使公民身份夹具护照(20260630-20360630)
        // 落在投票资格的护照有效期窗口内。
        pallet_timestamp::Pallet::<crate::Runtime>::set_timestamp(1_782_950_400_000);
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
            cid_number: Default::default(),
            // 固定治理机构管理员集合存 AdminProfile(来源 Genesis、逐人 meta 暂空)。
            admins: admin_accounts
                .iter()
                .cloned()
                .map(|account| admin_primitives::AdminProfile {
                    admin_account: account,
                    admin_cid_number: Default::default(),
                    admin_name: Default::default(),
                    role_code: Default::default(),
                    role_name: Default::default(),
                    term_start: 0,
                    term_end: 0,
                    admin_source: admin_primitives::AdminSource::Genesis,
                    admin_source_ref: Default::default(),
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

fn test_area_code(bytes: &[u8]) -> citizen_identity::AreaCodeBound {
    bytes.to_vec().try_into().expect("area code fits")
}

fn test_cid_number(bytes: &[u8]) -> citizen_identity::CidNumberBound {
    bytes.to_vec().try_into().expect("cid number fits")
}

/// 按 tag 生成真实规则 CID 号(格式/校验和/机构码全合规)。
fn real_cid_number(tag: &str, institution: &str, p1: &str) -> Vec<u8> {
    primitives::cid::generator::generate_cid_number(
        primitives::cid::generator::GenerateCidNumberInput {
            account_pubkey: tag,
            p1,
            province_code: "GD",
            province_name: "广东省",
            city_code: "001",
            city_name: "荔湾市",
            year: "2026",
            institution,
        },
    )
    .expect("cid should generate")
    .into_bytes()
}

fn build_voting_identity_payload(
    wallet_account: AccountId,
    cid_number: &[u8],
    province_code: &[u8],
    city_code: &[u8],
    town_code: &[u8],
) -> citizen_identity::VotingIdentityPayload<AccountId> {
    citizen_identity::VotingIdentityPayload {
        cid_number: test_cid_number(cid_number),
        wallet_account,
        citizen_age_years: 18,
        passport_valid_from: 20260630,
        passport_valid_until: 20360630,
        citizen_status: citizen_identity::CitizenStatus::Normal,
        residence_province_code: test_area_code(province_code),
        residence_city_code: test_area_code(city_code),
        residence_town_code: test_area_code(town_code),
    }
}

fn sign_citizen_identity_payload(
    wallet_pair: &sr25519::Pair,
    payload: &impl codec::Encode,
) -> citizen_identity::pallet::SignatureOf<Runtime> {
    let msg = primitives::sign::signing_message(
        primitives::sign::OP_SIGN_CITIZEN_IDENTITY,
        &payload.encode(),
    );
    wallet_pair
        .sign(&msg)
        .0
        .to_vec()
        .try_into()
        .expect("citizen identity signature fits")
}

fn setup_frg_citizen_identity_admin(province_code: &[u8]) -> (sr25519::Pair, AccountId, AccountId) {
    let registrar_pair =
        sr25519::Pair::from_string("//frg-citizen-identity-admin", None).expect("registrar pair");
    let registrar = AccountId::new(registrar_pair.public().0);
    let registrar_account = AccountId::new([88u8; 32]);
    public_admins::pallet::AdminAccounts::<Runtime>::insert(
        registrar_account.clone(),
        admin_primitives::AdminAccount {
            institution_code: admin_primitives::FRG,
            kind: admin_primitives::AdminAccountKind::PublicInstitution,
            cid_number: Default::default(),
            admins: vec![admin_primitives::AdminProfile {
                admin_account: registrar.clone(),
                admin_cid_number: Default::default(),
                admin_name: Default::default(),
                role_code: Default::default(),
                role_name: Default::default(),
                term_start: 0,
                term_end: 0,
                admin_source: admin_primitives::AdminSource::Genesis,
                admin_source_ref: Default::default(),
            }]
            .try_into()
            .expect("single registrar admin fits"),
            creator: registrar.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
            status: admin_primitives::AdminAccountStatus::Active,
        },
    );
    let province_code: primitives::cid::code::ProvinceCode =
        province_code.try_into().expect("province code fits");
    public_admins::FederalRegistryProvinceGroupAccounts::<Runtime>::insert(
        registrar_account.clone(),
        province_code,
    );
    (registrar_pair, registrar, registrar_account)
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
