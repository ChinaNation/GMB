#![cfg(test)]

extern crate alloc;
use alloc::vec::Vec;

use super::*;
use crate::configs::is_nrc_admin;
use crate::configs::*;
use crate::ResolutionDestroy;
use frame_support::traits::{Contains, Currency, EnsureOrigin, FindAuthor};
use frame_support::{assert_noop, assert_ok};
use primitives::cid::china::china_cb::CHINA_CB;
use primitives::institution_asset::{InstitutionAsset, InstitutionAssetAction};
use primitives::multisig::ReservedAccountGuard;
use sp_core::{sr25519, Pair};
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
    let issuer_cid_number: public_manage::pallet::CidNumberOf<Runtime> = test_issuer_cid_number()
        .try_into()
        .expect("issuer cid fits");

    let main_pair = sr25519::Pair::from_string("//main-step3", None).expect("pair");
    let main_admin_pubkey = main_pair.public().0;
    let backup_pair = sr25519::Pair::from_string("//backup1-step3", None).expect("pair");
    let backup_admin_pubkey = backup_pair.public().0;

    let admin_accounts = vec![
        AccountId::new(main_admin_pubkey),
        AccountId::new(backup_admin_pubkey),
    ];
    public_admins::pallet::AdminAccounts::<Runtime>::insert(
        issuer_main_account.clone(),
        admin_primitives::InstitutionAdminAccount {
            institution_code: votingengine::types::PRC,
            cid_number: issuer_cid_number.clone(),
            // admins 只保存钱包账户；岗位、来源、任期归 entity。
            admins: admin_accounts
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .try_into()
                .expect("test admins should fit"),
            status: admin_primitives::AdminAccountStatus::Active,
        },
    );
    // runtime 管理员查询必须先由 entity 解析机构和主账户，再读取 admins 钱包集合。
    let main_name: public_manage::pallet::AccountNameOf<Runtime> =
        primitives::account_derive::RESERVED_NAME_MAIN
            .to_vec()
            .try_into()
            .expect("main name fits");
    public_manage::AccountRegisteredCid::<Runtime>::insert(
        issuer_main_account.clone(),
        entity_primitives::RegisteredInstitution {
            cid_number: issuer_cid_number.clone(),
            account_name: main_name.clone(),
        },
    );
    public_manage::CidRegisteredAccount::<Runtime>::insert(
        &issuer_cid_number,
        main_name,
        issuer_main_account.clone(),
    );
    public_manage::Institutions::<Runtime>::insert(
        &issuer_cid_number,
        entity_primitives::InstitutionInfo {
            cid_full_name: b"test registry"
                .to_vec()
                .try_into()
                .expect("full name fits"),
            cid_short_name: b"registry".to_vec().try_into().expect("short name fits"),
            town_code: Default::default(),
            legal_representative_name: None,
            legal_representative_cid_number: None,
            legal_representative_account: None,
            institution_code: votingengine::types::PRC,
            created_at: 0,
            status: entity_primitives::InstitutionLifecycleStatus::Active,
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
        admin_primitives::InstitutionAdminAccount {
            institution_code: admin_primitives::FRG,
            cid_number: Default::default(),
            admins: vec![registrar.clone()]
                .try_into()
                .expect("single registrar admin fits"),
            status: admin_primitives::AdminAccountStatus::Active,
        },
    );

    // 注册局省级授权由 entity 的“省专员岗位 + 有效任职”表达，不再建立虚拟管理员组。
    let cid_number: public_manage::pallet::CidNumberOf<Runtime> =
        b"FRG-TEST".to_vec().try_into().expect("test cid fits");
    let account_name: public_manage::pallet::AccountNameOf<Runtime> =
        b"main".to_vec().try_into().expect("account name fits");
    public_manage::AccountRegisteredCid::<Runtime>::insert(
        registrar_account.clone(),
        entity_primitives::RegisteredInstitution {
            cid_number: cid_number.clone(),
            account_name,
        },
    );
    public_manage::Institutions::<Runtime>::insert(
        &cid_number,
        entity_primitives::InstitutionInfo {
            cid_full_name: "联邦注册局测试机构"
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("full name fits"),
            cid_short_name: "注册局测试"
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("short name fits"),
            town_code: Default::default(),
            legal_representative_name: None,
            legal_representative_cid_number: None,
            legal_representative_account: None,
            institution_code: admin_primitives::FRG,
            created_at: 0,
            status: entity_primitives::InstitutionLifecycleStatus::Active,
        },
    );
    let province_code: primitives::cid::code::ProvinceCode =
        province_code.try_into().expect("province code fits");
    let role_code: public_manage::RoleCodeOf =
        primitives::governance_skeleton::province_commissioner_role_code(province_code)
            .try_into()
            .expect("role code fits");
    public_manage::InstitutionRoles::<Runtime>::insert(
        &cid_number,
        &role_code,
        entity_primitives::InstitutionRole {
            cid_number: cid_number.clone(),
            role_code: role_code.clone(),
            role_name: "测试省专员"
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("role name fits"),
            term_required: true,
            role_status: entity_primitives::InstitutionRoleStatus::Active,
        },
    );
    let assignments: public_manage::institution::role::RoleAssignmentsOf<Runtime> =
        vec![entity_primitives::InstitutionAdminAssignment {
            cid_number: cid_number.clone(),
            admin_account: registrar.clone(),
            role_code: role_code.clone(),
            term_start: 1,
            term_end: u32::MAX,
            assignment_source: entity_primitives::InstitutionAssignmentSource::Genesis,
            assignment_source_ref: Default::default(),
            assignment_status: entity_primitives::InstitutionAssignmentStatus::Active,
        }]
        .try_into()
        .expect("assignment fits");
    public_manage::InstitutionRoleAssignments::<Runtime>::insert(
        cid_number,
        role_code,
        assignments,
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
