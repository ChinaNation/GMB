use super::*;
use frame_support::{assert_noop, assert_ok};

use crate::{AccountKind, Error, RESERVED_NAME_FEE, RESERVED_NAME_MAIN};

const ACCOUNT_AMOUNT: Balance = 1_000;
const CREATOR_BALANCE: Balance = 100_000;
const CUSTOM_ACCOUNT_NAME: &[u8] = "专项账户".as_bytes();

fn fund_creator() {
    assert_ok!(Balances::force_set_balance(
        RuntimeOrigin::root(),
        creator(),
        CREATOR_BALANCE,
    ));
}

fn create_cgov(
    tag: &str,
    nonce: &[u8],
    accounts: pallet::InstitutionInitialAccountsOf<Test>,
    signature: pallet::RegisterSignatureOf<Test>,
) -> pallet::CidNumberOf<Test> {
    let cid = generated_cid(tag, "CGOV");
    assert_ok!(PublicManage::propose_create_public_institution(
        RuntimeOrigin::signed(creator()),
        cid.clone(),
        cid_full_name("测试公权机构".as_bytes()),
        cid_short_name("测试机构".as_bytes()),
        empty_town_code(),
        legal_representative_name(),
        legal_representative_cid_number(),
        legal_representative_account(),
        accounts,
        code_bytes("CGOV"),
        institution_roles_vec(),
        institution_assignments_vec(3),
        2,
        register_nonce(nonce),
        signature,
        b"REGISTRY-CID".to_vec(),
        signer_pubkey(),
        province_name(),
        Vec::new(),
    ));
    cid
}

fn create_cgov_with_custom(tag: &str) -> pallet::CidNumberOf<Test> {
    fund_creator();
    create_cgov(
        tag,
        tag.as_bytes(),
        initial_accounts(&[
            (RESERVED_NAME_MAIN, ACCOUNT_AMOUNT),
            (RESERVED_NAME_FEE, ACCOUNT_AMOUNT),
            (CUSTOM_ACCOUNT_NAME, ACCOUNT_AMOUNT),
        ]),
        valid_signature(),
    )
}

fn account_of(cid: &pallet::CidNumberOf<Test>, name: &[u8]) -> AccountId32 {
    pallet::InstitutionAccounts::<Test>::get(cid, account_name(name))
        .expect("institution account must exist")
        .address
}

#[test]
fn creation_uses_cid_as_identity_and_writes_all_account_indexes() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("create-cid-source");
        let main = account_of(&cid, RESERVED_NAME_MAIN);
        let fee = account_of(&cid, RESERVED_NAME_FEE);
        let custom = account_of(&cid, CUSTOM_ACCOUNT_NAME);

        assert_ne!(main, fee);
        assert_ne!(main, custom);
        assert_eq!(
            pallet::AccountRegisteredCid::<Test>::get(&main)
                .expect("reverse main account index")
                .cid_number,
            cid
        );
        assert_eq!(
            pallet::AccountRegisteredCid::<Test>::get(&fee)
                .expect("reverse fee account index")
                .cid_number,
            cid
        );

        let admins =
            public_admins::AdminAccounts::<Test>::get(&cid).expect("admins must be keyed by CID");
        assert_eq!(admins.admins.as_slice(), &[admin(0), admin(1), admin(2)]);
        assert!(PublicAdmins::is_institution_admin(
            code_bytes("CGOV"),
            cid.as_slice(),
            &admin(0),
        ));
        assert_eq!(
            internal_vote::ActiveInstitutionThresholds::<Test>::get(&cid),
            Some(2)
        );
        assert_eq!(
            <PublicManage as entity_primitives::InstitutionLegalRepresentativeQuery<
                AccountId32,
            >>::legal_representative(
                cid.as_slice(),
            ),
            Some(legal_representative_account())
        );
    });
}

#[test]
fn creation_accepts_zero_protocol_account_balances() {
    new_test_ext().execute_with(|| {
        fund_creator();
        let cid = create_cgov(
            "zero-balances",
            b"zero-balances",
            initial_accounts(&[(RESERVED_NAME_MAIN, 0), (RESERVED_NAME_FEE, 0)]),
            valid_signature(),
        );
        assert_eq!(
            Balances::free_balance(account_of(&cid, RESERVED_NAME_MAIN)),
            0
        );
        assert_eq!(
            Balances::free_balance(account_of(&cid, RESERVED_NAME_FEE)),
            0
        );
    });
}

#[test]
fn creation_rejects_nonzero_account_balance_below_ed() {
    new_test_ext().execute_with(|| {
        fund_creator();
        let cid = generated_cid("below-ed", "CGOV");
        assert_noop!(
            PublicManage::propose_create_public_institution(
                RuntimeOrigin::signed(creator()),
                cid,
                cid_full_name("低余额机构".as_bytes()),
                cid_short_name("低余额".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                initial_accounts(&[(RESERVED_NAME_MAIN, 1), (RESERVED_NAME_FEE, ACCOUNT_AMOUNT)]),
                code_bytes("CGOV"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"below-ed"),
                valid_signature(),
                b"REGISTRY-CID".to_vec(),
                signer_pubkey(),
                province_name(),
                Vec::new(),
            ),
            Error::<Test>::AccountInitialAmountBelowMinimum
        );
    });
}

#[test]
fn creation_requires_complete_protocol_account_set() {
    new_test_ext().execute_with(|| {
        fund_creator();
        let cid = generated_cid("missing-fee", "CGOV");
        assert_noop!(
            PublicManage::propose_create_public_institution(
                RuntimeOrigin::signed(creator()),
                cid,
                cid_full_name("缺费用账户机构".as_bytes()),
                cid_short_name("缺费用账户".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                initial_accounts(&[(RESERVED_NAME_MAIN, ACCOUNT_AMOUNT)]),
                code_bytes("CGOV"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"missing-fee"),
                valid_signature(),
                b"REGISTRY-CID".to_vec(),
                signer_pubkey(),
                province_name(),
                Vec::new(),
            ),
            Error::<Test>::MissingMainAccount
        );
    });
}

#[test]
fn creation_rejects_invalid_signature_without_partial_state() {
    new_test_ext().execute_with(|| {
        fund_creator();
        let cid = generated_cid("invalid-signature", "CGOV");
        assert_noop!(
            PublicManage::propose_create_public_institution(
                RuntimeOrigin::signed(creator()),
                cid.clone(),
                cid_full_name("无效签名机构".as_bytes()),
                cid_short_name("无效签名".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                initial_accounts(&[
                    (RESERVED_NAME_MAIN, ACCOUNT_AMOUNT),
                    (RESERVED_NAME_FEE, ACCOUNT_AMOUNT),
                ]),
                code_bytes("CGOV"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"invalid-signature"),
                invalid_signature(),
                b"REGISTRY-CID".to_vec(),
                signer_pubkey(),
                province_name(),
                Vec::new(),
            ),
            Error::<Test>::InvalidCidInstitutionSignature
        );
        assert!(!pallet::Institutions::<Test>::contains_key(&cid));
        assert!(!public_admins::AdminAccounts::<Test>::contains_key(&cid));
    });
}

#[test]
fn creation_rejects_duplicate_cid_and_replayed_nonce() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("duplicate-cid");
        assert_noop!(
            PublicManage::propose_create_public_institution(
                RuntimeOrigin::signed(creator()),
                cid,
                cid_full_name("重复机构".as_bytes()),
                cid_short_name("重复".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                initial_accounts(&[
                    (RESERVED_NAME_MAIN, ACCOUNT_AMOUNT),
                    (RESERVED_NAME_FEE, ACCOUNT_AMOUNT),
                ]),
                code_bytes("CGOV"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"duplicate-cid"),
                valid_signature(),
                b"REGISTRY-CID".to_vec(),
                signer_pubkey(),
                province_name(),
                Vec::new(),
            ),
            Error::<Test>::InstitutionAlreadyExists
        );
    });
}

#[test]
fn update_info_and_add_account_keep_cid_as_only_entity_key() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("maintain-cid");
        assert_ok!(PublicManage::update_institution_info(
            RuntimeOrigin::signed(creator()),
            cid.clone(),
            cid_full_name("更新后的机构全称".as_bytes()),
            cid_short_name("更新简称".as_bytes()),
            register_nonce(b"rename"),
            valid_signature(),
            b"REGISTRY-CID".to_vec(),
            signer_pubkey(),
            province_name(),
            Vec::new(),
        ));
        let updated = pallet::Institutions::<Test>::get(&cid).expect("institution remains");
        assert_eq!(
            updated.cid_full_name.as_slice(),
            "更新后的机构全称".as_bytes()
        );

        let added_name = "新增账户".as_bytes();
        assert_ok!(PublicManage::add_institution_account(
            RuntimeOrigin::signed(creator()),
            cid.clone(),
            account_names_bv(&[added_name]),
            register_nonce(b"add-account"),
            valid_signature(),
            b"REGISTRY-CID".to_vec(),
            signer_pubkey(),
            province_name(),
            Vec::new(),
        ));
        let added = account_of(&cid, added_name);
        assert_eq!(
            pallet::AccountRegisteredCid::<Test>::get(&added)
                .expect("new reverse index")
                .cid_number,
            cid
        );
        assert_eq!(Balances::free_balance(added), 0);
    });
}

#[test]
fn add_account_rejects_protocol_names_and_duplicate_custom_names() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("add-invalid");
        assert_noop!(
            PublicManage::add_institution_account(
                RuntimeOrigin::signed(creator()),
                cid.clone(),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                register_nonce(b"add-main"),
                valid_signature(),
                b"REGISTRY-CID".to_vec(),
                signer_pubkey(),
                province_name(),
                Vec::new(),
            ),
            Error::<Test>::ReservedAccountName
        );
        assert_noop!(
            PublicManage::add_institution_account(
                RuntimeOrigin::signed(creator()),
                cid,
                account_names_bv(&["重复账户".as_bytes(), "重复账户".as_bytes()]),
                register_nonce(b"add-duplicate"),
                valid_signature(),
                b"REGISTRY-CID".to_vec(),
                signer_pubkey(),
                province_name(),
                Vec::new(),
            ),
            Error::<Test>::DuplicateAccountName
        );
    });
}

#[test]
fn derive_account_distinguishes_protocol_and_custom_account_kinds() {
    new_test_ext().execute_with(|| {
        let cid = generated_cid("derive-kinds", "CGOV");
        let (_, main_kind) =
            PublicManage::derive_institution_account(cid.as_slice(), RESERVED_NAME_MAIN).unwrap();
        let (_, fee_kind) =
            PublicManage::derive_institution_account(cid.as_slice(), RESERVED_NAME_FEE).unwrap();
        let (_, custom_kind) =
            PublicManage::derive_institution_account(cid.as_slice(), CUSTOM_ACCOUNT_NAME).unwrap();
        assert!(matches!(main_kind, AccountKind::InstitutionMain { .. }));
        assert!(matches!(fee_kind, AccountKind::InstitutionFee { .. }));
        assert!(matches!(custom_kind, AccountKind::InstitutionNamed { .. }));
    });
}

#[test]
fn close_requires_matching_actor_cid_and_an_institution_admin() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("close-auth");
        let custom = account_of(&cid, CUSTOM_ACCOUNT_NAME);
        let other_cid = generated_cid("close-auth-other", "CGOV");
        assert_noop!(
            close_with_cred(
                RuntimeOrigin::signed(admin(0)),
                other_cid,
                custom.clone(),
                beneficiary(),
                1,
            ),
            Error::<Test>::NotInstitutionAccount
        );
        assert_noop!(
            close_with_cred(
                RuntimeOrigin::signed(admin(9)),
                cid,
                custom,
                beneficiary(),
                2,
            ),
            Error::<Test>::PermissionDenied
        );
    });
}

#[test]
fn protocol_accounts_cannot_be_closed() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("close-protocol");
        for account in [
            account_of(&cid, RESERVED_NAME_MAIN),
            account_of(&cid, RESERVED_NAME_FEE),
        ] {
            assert_noop!(
                close_with_cred(
                    RuntimeOrigin::signed(admin(0)),
                    cid.clone(),
                    account,
                    beneficiary(),
                    3,
                ),
                Error::<Test>::CannotCloseProtectedInstitution
            );
        }
    });
}

#[test]
fn approved_close_removes_only_custom_account() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("close-custom");
        let main = account_of(&cid, RESERVED_NAME_MAIN);
        let fee = account_of(&cid, RESERVED_NAME_FEE);
        let custom = account_of(&cid, CUSTOM_ACCOUNT_NAME);
        let beneficiary_account = beneficiary();

        assert_ok!(close_with_cred(
            RuntimeOrigin::signed(admin(0)),
            cid.clone(),
            custom.clone(),
            beneficiary_account.clone(),
            4,
        ));
        let proposal_id = last_proposal_id();
        assert_ok!(cast_yes_votes(&[admin(1), admin(2)], 2, proposal_id));

        assert!(!pallet::AccountRegisteredCid::<Test>::contains_key(&custom));
        assert!(!pallet::InstitutionAccounts::<Test>::contains_key(
            &cid,
            account_name(CUSTOM_ACCOUNT_NAME),
        ));
        assert!(pallet::AccountRegisteredCid::<Test>::contains_key(&main));
        assert!(pallet::AccountRegisteredCid::<Test>::contains_key(&fee));
        assert!(pallet::Institutions::<Test>::contains_key(&cid));
        assert!(public_admins::AdminAccounts::<Test>::contains_key(&cid));
        assert!(Balances::free_balance(beneficiary_account) > 0);
    });
}

#[test]
fn close_rejects_invalid_credential_and_nonce_replay() {
    new_test_ext().execute_with(|| {
        let cid = create_cgov_with_custom("close-credential");
        let custom = account_of(&cid, CUSTOM_ACCOUNT_NAME);
        let nonce = register_nonce(b"bad-close");
        assert_noop!(
            PublicManage::propose_close_public_institution(
                RuntimeOrigin::signed(admin(0)),
                cid.clone(),
                custom.clone(),
                beneficiary(),
                nonce,
                invalid_signature(),
                b"REGISTRY-CID".to_vec(),
                signer_pubkey(),
            ),
            Error::<Test>::InvalidDeregisterCredential
        );
        assert_ok!(close_with_cred(
            RuntimeOrigin::signed(admin(0)),
            cid.clone(),
            custom.clone(),
            beneficiary(),
            5,
        ));
        assert_noop!(
            close_with_cred(
                RuntimeOrigin::signed(admin(0)),
                cid,
                custom,
                beneficiary(),
                5,
            ),
            Error::<Test>::DeregisterNonceAlreadyUsed
        );
    });
}
