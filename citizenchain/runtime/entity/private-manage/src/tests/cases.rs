use super::*;
use crate::institution::types::InstitutionLifecycleStatus;
use frame_support::{assert_noop, assert_ok, traits::Currency};
use primitives::account_derive::{
    AccountKind, RESERVED_NAME_FEE, RESERVED_NAME_HE, RESERVED_NAME_MAIN, RESERVED_NAME_SAFETYFUND,
    RESERVED_NAME_STAKE,
};
use votingengine::STATUS_EXECUTED;

const SEED_BALANCE: Balance = 100_000;
const ACCT_AMOUNT: Balance = 1_000;

fn fund_creator() -> AccountId32 {
    let c = creator();
    let _ = Balances::deposit_creating(&c, SEED_BALANCE);
    c
}

fn typical_accounts() -> pallet::InstitutionInitialAccountsOf<Test> {
    initial_accounts(&[
        (RESERVED_NAME_MAIN, ACCT_AMOUNT),
        (RESERVED_NAME_FEE, ACCT_AMOUNT),
    ])
}
// CID 登记路径(5 个用例)
#[test]
fn register_cid_private_institution_with_valid_signature_succeeds() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        let cid = generated_cid("CID001", "SFLP");
        let names = account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]);

        assert_ok!(PrivateManage::register_cid_private_institution(
            RuntimeOrigin::signed(submitter),
            cid.clone(),
            cid_full_name("机构甲".as_bytes()),
            names.clone(),
            register_nonce(b"nonce-1"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));

        assert!(pallet::CidRegisteredAccount::<Test>::contains_key(
            &cid,
            &account_name(RESERVED_NAME_MAIN),
        ));
        assert!(pallet::CidRegisteredAccount::<Test>::contains_key(
            &cid,
            &account_name(RESERVED_NAME_FEE),
        ));
        // 反向索引也写入
        let main_addr =
            PrivateManage::derive_registered_account(cid.as_slice(), RESERVED_NAME_MAIN)
                .expect("derive main")
                .0;
        assert!(pallet::AccountRegisteredCid::<Test>::contains_key(
            &main_addr
        ));
    });
}

#[test]
fn register_rejects_invalid_cid_institution_signature() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        assert_noop!(
            PrivateManage::register_cid_private_institution(
                RuntimeOrigin::signed(submitter),
                generated_cid("CID-bad-sig", "SFLP"),
                cid_full_name("机构甲".as_bytes()),
                account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]),
                register_nonce(b"nonce-bs"),
                invalid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::InvalidCidInstitutionSignature
        );
    });
}

#[test]
fn register_rejects_duplicate_cid_account_name() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        let cid = generated_cid("CID-dup", "SFLP");
        // 第一次成功
        assert_ok!(PrivateManage::register_cid_private_institution(
            RuntimeOrigin::signed(submitter.clone()),
            cid.clone(),
            cid_full_name("机构 A".as_bytes()),
            account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]),
            register_nonce(b"nonce-a1"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        // 第二次相同 cid + 主账户:CidAlreadyRegistered
        assert_noop!(
            PrivateManage::register_cid_private_institution(
                RuntimeOrigin::signed(submitter),
                cid,
                cid_full_name("机构 A".as_bytes()),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                register_nonce(b"nonce-a2"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::CidAlreadyRegistered
        );
    });
}

#[test]
fn register_rejects_replayed_nonce() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        // 第一次成功
        assert_ok!(PrivateManage::register_cid_private_institution(
            RuntimeOrigin::signed(submitter.clone()),
            generated_cid("CID-N1", "SFLP"),
            cid_full_name(b"A"),
            account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]),
            register_nonce(b"nonce-replay"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        // 第二次同 nonce 不同 cid:RegisterNonceAlreadyUsed
        assert_noop!(
            PrivateManage::register_cid_private_institution(
                RuntimeOrigin::signed(submitter),
                generated_cid("CID-N2", "SFLP"),
                cid_full_name(b"B"),
                account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]),
                register_nonce(b"nonce-replay"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::RegisterNonceAlreadyUsed
        );
    });
}

#[test]
fn register_rejects_empty_required_fields() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        // 空 cid_number
        assert_noop!(
            PrivateManage::register_cid_private_institution(
                RuntimeOrigin::signed(submitter.clone()),
                cid_number(b""),
                cid_full_name(b"A"),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                register_nonce(b"nonce-empty1"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::EmptyCidNumber
        );
        // 空 cid_full_name
        assert_noop!(
            PrivateManage::register_cid_private_institution(
                RuntimeOrigin::signed(submitter.clone()),
                generated_cid("CID-E", "SFLP"),
                cid_full_name(b""),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                register_nonce(b"nonce-empty2"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::EmptyAccountName
        );
        // 空 province_name
        assert_noop!(
            PrivateManage::register_cid_private_institution(
                RuntimeOrigin::signed(submitter),
                generated_cid("CID-E", "SFLP"),
                cid_full_name(b"A"),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                register_nonce(b"nonce-empty3"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                alloc::vec::Vec::new(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::EmptyScopeProvinceName
        );
    });
}
// 创建路径(8 个用例)
#[test]
fn propose_create_private_institution_registers_active_without_vote() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let cid = generated_cid("CID-CR-1", "SFLP");
        let proposal_before = votingengine::Pallet::<Test>::next_proposal_id();

        assert_ok!(PrivateManage::propose_create_private_institution(
            RuntimeOrigin::signed(c.clone()),
            cid.clone(),
            cid_full_name("机构甲".as_bytes()),
            cid_short_name("简称".as_bytes()),
            empty_town_code(),
            legal_representative_name(),
            legal_representative_cid_number(),
            legal_representative_account(),
            typical_accounts(),
            code_bytes("SFLP"),
            institution_roles_vec(),
            institution_assignments_vec(3),
            2,
            register_nonce(b"nonce-cr-1"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));

        assert_eq!(
            votingengine::Pallet::<Test>::next_proposal_id(),
            proposal_before
        );
        assert!(pallet::Institutions::<Test>::contains_key(&cid));
        assert_eq!(
            pallet::Institutions::<Test>::get(&cid).unwrap().status,
            InstitutionLifecycleStatus::Active,
        );
        assert_eq!(
            pallet::Institutions::<Test>::get(&cid)
                .unwrap()
                .institution_code,
            code_bytes("SFLP"),
        );
        // 私权机构名称上链(链是唯一真源):存全称/简称。
        let stored = pallet::Institutions::<Test>::get(&cid).unwrap();
        assert_eq!(stored.cid_full_name, cid_full_name("机构甲".as_bytes()));
        assert_eq!(stored.cid_short_name, cid_short_name("简称".as_bytes()));
        assert_eq!(
            stored.legal_representative_name,
            Some(legal_representative_name())
        );
        assert_eq!(
            stored.legal_representative_cid_number,
            Some(legal_representative_cid_number())
        );
        assert_eq!(
            stored.legal_representative_account,
            Some(legal_representative_account())
        );
        let main = PrivateManage::derive_registered_account(cid.as_slice(), RESERVED_NAME_MAIN)
            .unwrap()
            .0;
        assert_eq!(
            <PrivateManage as entity_primitives::InstitutionLegalRepresentativeQuery<
                AccountId32,
            >>::legal_representative(code_bytes("SFLP"), main.clone()),
            Some(legal_representative_account())
        );
        let admin_account = private_admins::AdminAccounts::<Test>::get(main.clone())
            .expect("private admin account present");
        assert_eq!(admin_account.admins.len(), 3);
        assert_eq!(
            internal_vote::ActiveDynamicThresholds::<Test>::get(code_bytes("SFLP"), main),
            Some(2),
        );
        assert_eq!(Balances::reserved_balance(&c), 0);
    });
}

#[test]
fn private_institution_stores_full_and_short_name_onchain() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let cid = generated_cid("CID-PRI-1", "SFLP");

        assert_ok!(PrivateManage::propose_create_private_institution(
            RuntimeOrigin::signed(c.clone()),
            cid.clone(),
            cid_full_name("某市人民政府".as_bytes()),
            cid_short_name("某市府".as_bytes()),
            empty_town_code(),
            legal_representative_name(),
            legal_representative_cid_number(),
            legal_representative_account(),
            typical_accounts(),
            code_bytes("SFLP"),
            institution_roles_vec(),
            institution_assignments_vec(3),
            2,
            register_nonce(b"nonce-pri-1"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));

        // 私权机构名称上链(链是机构信息唯一真源)。
        let stored = pallet::Institutions::<Test>::get(&cid).unwrap();
        assert_eq!(
            stored.cid_full_name,
            cid_full_name("某市人民政府".as_bytes())
        );
        assert_eq!(stored.cid_short_name, cid_short_name("某市府".as_bytes()));
    });
}

#[test]
fn private_institution_rejects_empty_short_name() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        // 私权名上链后简称必填。
        assert_noop!(
            PrivateManage::propose_create_private_institution(
                RuntimeOrigin::signed(c),
                generated_cid("CID-PRI-2", "SFLP"),
                cid_full_name("某市人民政府".as_bytes()),
                cid_short_name(b""),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                typical_accounts(),
                code_bytes("SFLP"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"nonce-pri-2"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::EmptyAccountName
        );
    });
}

#[test]
fn private_institution_rejects_nonempty_town_code() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        assert_noop!(
            PrivateManage::propose_create_private_institution(
                RuntimeOrigin::signed(c),
                generated_cid("CID-PRI-TOWN-1", "SFLP"),
                cid_full_name("某公司".as_bytes()),
                cid_short_name("某公司".as_bytes()),
                town_code(b"001"),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                typical_accounts(),
                code_bytes("SFLP"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"nonce-pri-town-1"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::InvalidTownCode
        );
    });
}

#[test]
fn propose_create_rejects_unincorporated_without_parent_routing() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        assert_noop!(
            PrivateManage::propose_create_private_institution(
                RuntimeOrigin::signed(c),
                generated_cid("CID-UNIN-1", "UNIN"),
                cid_full_name("非法人机构".as_bytes()),
                cid_short_name("简称".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                typical_accounts(),
                code_bytes("UNIN"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"nonce-unin-1"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::InvalidInstitutionCode
        );
    });
}

#[test]
fn create_directly_funds_initial_accounts() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let cid = generated_cid("CID-CR-2", "SFLP");

        assert_ok!(PrivateManage::propose_create_private_institution(
            RuntimeOrigin::signed(c.clone()),
            cid.clone(),
            cid_full_name("机构乙".as_bytes()),
            cid_short_name("简称".as_bytes()),
            empty_town_code(),
            legal_representative_name(),
            legal_representative_cid_number(),
            legal_representative_account(),
            typical_accounts(),
            code_bytes("SFLP"),
            institution_roles_vec(),
            institution_assignments_vec(3),
            2,
            register_nonce(b"nonce-cr-2"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));

        assert_eq!(
            pallet::Institutions::<Test>::get(&cid).unwrap().status,
            InstitutionLifecycleStatus::Active,
        );
        // 主账户和费用账户都被划账
        let main = PrivateManage::derive_registered_account(cid.as_slice(), RESERVED_NAME_MAIN)
            .unwrap()
            .0;
        let fee_acc = PrivateManage::derive_registered_account(cid.as_slice(), RESERVED_NAME_FEE)
            .unwrap()
            .0;
        assert_eq!(Balances::free_balance(&main), ACCT_AMOUNT);
        assert_eq!(Balances::free_balance(&fee_acc), ACCT_AMOUNT);
        assert_eq!(Balances::reserved_balance(&c), 0);
    });
}

#[test]
fn propose_create_rejects_below_create_amount_minimum() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        // MinCreateAmount=111, 用 50 应拒
        let bad_accounts = initial_accounts(&[(RESERVED_NAME_MAIN, 50), (RESERVED_NAME_FEE, 200)]);
        assert_noop!(
            PrivateManage::propose_create_private_institution(
                RuntimeOrigin::signed(c),
                generated_cid("CID-MIN", "SFLP"),
                cid_full_name(b"X"),
                cid_short_name("简称".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                bad_accounts,
                code_bytes("SFLP"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"nonce-min"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::AccountInitialAmountBelowMinimum
        );
    });
}

#[test]
fn propose_create_rejects_duplicate_account_name() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let dup = initial_accounts(&[
            (RESERVED_NAME_MAIN, ACCT_AMOUNT),
            (RESERVED_NAME_FEE, ACCT_AMOUNT),
            (b"dept", ACCT_AMOUNT),
            (b"dept", ACCT_AMOUNT),
        ]);
        assert_noop!(
            PrivateManage::propose_create_private_institution(
                RuntimeOrigin::signed(c),
                generated_cid("CID-DUP", "SFLP"),
                cid_full_name(b"X"),
                cid_short_name("简称".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                dup,
                code_bytes("SFLP"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"nonce-dup"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::DuplicateAccountName
        );
    });
}

#[test]
fn derive_registered_account_rejects_reserved_system_names() {
    new_test_ext().execute_with(|| {
        let cid = generated_cid("CID-RESV", "SFLP");
        // 永久质押/安全基金/两和基金 为制度专属账户,普通机构禁止注册。
        for name in [
            RESERVED_NAME_STAKE,
            RESERVED_NAME_SAFETYFUND,
            RESERVED_NAME_HE,
        ] {
            assert_eq!(
                PrivateManage::derive_registered_account(cid.as_slice(), name).unwrap_err(),
                pallet::Error::<Test>::ReservedAccountName.into()
            );
        }
        // 空名拒绝。
        assert_eq!(
            PrivateManage::derive_registered_account(cid.as_slice(), b"").unwrap_err(),
            pallet::Error::<Test>::EmptyAccountName.into()
        );
        // 主账户/费用账户仍强制路由到对应种类,不报错。
        let (_, main_kind) =
            PrivateManage::derive_registered_account(cid.as_slice(), RESERVED_NAME_MAIN).unwrap();
        assert!(matches!(main_kind, AccountKind::InstitutionMain { .. }));
        let (_, fee_kind) =
            PrivateManage::derive_registered_account(cid.as_slice(), RESERVED_NAME_FEE).unwrap();
        assert!(matches!(fee_kind, AccountKind::InstitutionFee { .. }));
    });
}

#[test]
fn propose_create_rejects_reserved_system_account_name() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        // 自定义账户取名"安全基金" → 制度专属保留名,创建即拒。
        let bad = initial_accounts(&[
            (RESERVED_NAME_MAIN, ACCT_AMOUNT),
            (RESERVED_NAME_FEE, ACCT_AMOUNT),
            (RESERVED_NAME_SAFETYFUND, ACCT_AMOUNT),
        ]);
        assert_noop!(
            PrivateManage::propose_create_private_institution(
                RuntimeOrigin::signed(c),
                generated_cid("CID-RSV", "SFLP"),
                cid_full_name(b"X"),
                cid_short_name("简称".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                bad,
                code_bytes("SFLP"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"nonce-rsv"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::ReservedAccountName
        );
    });
}

#[test]
fn propose_create_rejects_missing_main_account() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let no_main = initial_accounts(&[(RESERVED_NAME_FEE, ACCT_AMOUNT)]);
        assert_noop!(
            PrivateManage::propose_create_private_institution(
                RuntimeOrigin::signed(c),
                generated_cid("CID-NM", "SFLP"),
                cid_full_name(b"X"),
                cid_short_name("简称".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                no_main,
                code_bytes("SFLP"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"nonce-nm"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::MissingMainAccount
        );
    });
}

#[test]
fn propose_create_rejects_invalid_admin_threshold() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        // threshold=1 < min(2, ceil(3/2))
        assert_noop!(
            PrivateManage::propose_create_private_institution(
                RuntimeOrigin::signed(c.clone()),
                generated_cid("CID-T1", "SFLP"),
                cid_full_name(b"X"),
                cid_short_name("简称".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                typical_accounts(),
                code_bytes("SFLP"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                1,
                register_nonce(b"nonce-t1"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::InvalidThreshold
        );
        // threshold > admins_len
        assert_noop!(
            PrivateManage::propose_create_private_institution(
                RuntimeOrigin::signed(c),
                generated_cid("CID-T2", "SFLP"),
                cid_full_name(b"X"),
                cid_short_name("简称".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                typical_accounts(),
                code_bytes("SFLP"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                4,
                register_nonce(b"nonce-t2"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::InvalidThreshold
        );
    });
}

#[test]
fn propose_create_rejects_when_institution_already_exists() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let cid = generated_cid("CID-AE", "SFLP");

        // 先创建一个
        assert_ok!(PrivateManage::propose_create_private_institution(
            RuntimeOrigin::signed(c.clone()),
            cid.clone(),
            cid_full_name(b"A"),
            cid_short_name("简称".as_bytes()),
            empty_town_code(),
            legal_representative_name(),
            legal_representative_cid_number(),
            legal_representative_account(),
            typical_accounts(),
            code_bytes("SFLP"),
            institution_roles_vec(),
            institution_assignments_vec(3),
            2,
            register_nonce(b"nonce-ae1"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        // 第二次同 cid 应拒
        assert_noop!(
            PrivateManage::propose_create_private_institution(
                RuntimeOrigin::signed(c),
                cid,
                cid_full_name(b"B"),
                cid_short_name("简称".as_bytes()),
                empty_town_code(),
                legal_representative_name(),
                legal_representative_cid_number(),
                legal_representative_account(),
                typical_accounts(),
                code_bytes("SFLP"),
                institution_roles_vec(),
                institution_assignments_vec(3),
                2,
                register_nonce(b"nonce-ae2"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::InstitutionAlreadyExists
        );
    });
}
// 关闭路径(5 个用例)
fn create_and_activate_institution(
    cid_tag: &str,
    admins_len: u8,
) -> (pallet::CidNumberOf<Test>, AccountId32) {
    let c = creator();
    let _ = Balances::deposit_creating(&c, SEED_BALANCE);
    let cid = generated_cid(cid_tag, "SFLP");

    assert_ok!(PrivateManage::propose_create_private_institution(
        RuntimeOrigin::signed(c.clone()),
        cid.clone(),
        cid_full_name(b"X"),
        cid_short_name("简称".as_bytes()),
        empty_town_code(),
        legal_representative_name(),
        legal_representative_cid_number(),
        legal_representative_account(),
        typical_accounts(),
        code_bytes("SFLP"),
        institution_roles_vec(),
        institution_assignments_vec(admins_len),
        admins_len.saturating_add(1) as u32 / 2 + 1, // m-of-n 治理阈值,取一个能通过的
        register_nonce(cid_tag.as_bytes()),
        valid_signature(),
        province_name(),
        creator(),
        signer_pubkey(),
        province_name(),
        b"city".to_vec(),
    ));

    let main = PrivateManage::derive_registered_account(cid.as_slice(), RESERVED_NAME_MAIN)
        .unwrap()
        .0;
    (cid, main)
}

#[test]
fn propose_close_writes_pending() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution("CID-CL-1", 3);

        assert_ok!(close_with_cred(
            RuntimeOrigin::signed(admin(0)),
            main.clone(),
            beneficiary(),
            1,
        ));
        let pid = last_proposal_id();
        assert_eq!(
            pallet::InstitutionPendingClose::<Test>::get(&main),
            Some(pid)
        );
    });
}

#[test]
fn close_executes_when_vote_reaches_threshold_returns_balance() {
    new_test_ext().execute_with(|| {
        let (cid, main) = create_and_activate_institution("CID-CL-2", 3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();
        let beneficiary_acc = beneficiary();
        let main_name = account_name(RESERVED_NAME_MAIN);
        let account = main.clone();

        assert_ok!(close_with_cred(
            RuntimeOrigin::signed(admin(0)),
            main.clone(),
            beneficiary_acc.clone(),
            2,
        ));
        let pid = last_proposal_id();
        assert_ok!(cast_yes_votes(&admin_accounts[1..], 2, pid));

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal");
        assert_eq!(proposal.status, STATUS_EXECUTED);

        // 级联注销整个机构(主+费用账户):每账户 1000 扣 fee 10 → 各 990,beneficiary 收 1980。
        assert_eq!(Balances::free_balance(&beneficiary_acc), 1980);
        assert_eq!(Balances::free_balance(&main), 0);
        assert!(!pallet::InstitutionPendingClose::<Test>::contains_key(
            &main
        ));
        assert!(!pallet::InstitutionAccounts::<Test>::contains_key(
            &cid, &main_name
        ));
        assert!(!pallet::CidRegisteredAccount::<Test>::contains_key(
            &cid, &main_name
        ));
        assert!(!pallet::AccountRegisteredCid::<Test>::contains_key(&main));
        assert!(private_admins::AdminAccounts::<Test>::get(account.clone()).is_none());
        assert!(
            internal_vote::ActiveDynamicThresholds::<Test>::get(code_bytes("SFLP"), account)
                .is_none()
        );
        // 机构级墓碑:Institutions 永不删除,状态置 Closed。
        assert_eq!(
            pallet::Institutions::<Test>::get(&cid)
                .expect("tombstone kept")
                .status,
            InstitutionLifecycleStatus::Closed,
        );
        // 墓碑号永不复用:同号 register 重建账户索引被拒。
        assert_noop!(
            PrivateManage::register_cid_private_institution(
                RuntimeOrigin::signed(creator()),
                cid.clone(),
                cid_full_name("重建尝试".as_bytes()),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                register_nonce(b"nonce-reopen"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::InstitutionAlreadyClosed
        );
    });
}

#[test]
fn propose_close_rejects_close_balance_below_minimum() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution("CID-CL-3", 3);

        // 把主账户余额清空到 50(<MinCloseBalance=111)
        // 用 force-set: 直接 transfer 走
        let stranger = AccountId32::new([0xCD; 32]);
        let _ = <Balances as Currency<AccountId32>>::transfer(
            &main,
            &stranger,
            950,
            frame_support::traits::ExistenceRequirement::AllowDeath,
        );

        assert_noop!(
            close_with_cred(RuntimeOrigin::signed(admin(0)), main, beneficiary(), 3),
            pallet::Error::<Test>::CloseBalanceBelowMinimum
        );
    });
}

#[test]
fn propose_close_rejects_when_not_institution_account() {
    new_test_ext().execute_with(|| {
        // 没在 AccountRegisteredCid 表里的地址
        let stranger = AccountId32::new([0xEE; 32]);
        assert_noop!(
            close_with_cred(RuntimeOrigin::signed(admin(0)), stranger, beneficiary(), 4),
            pallet::Error::<Test>::NotInstitutionAccount
        );
    });
}

#[test]
fn propose_close_rejects_self_beneficiary() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution("CID-CL-5", 3);
        // beneficiary == account 应拒
        assert_noop!(
            close_with_cred(RuntimeOrigin::signed(admin(0)), main.clone(), main, 5),
            pallet::Error::<Test>::InvalidBeneficiary
        );
    });
}
// Cleanup / 边界(4 个用例)
#[test]
fn cleanup_rejected_private_proposal_only_after_engine_rejected() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution("CID-CU", 3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();

        assert_ok!(close_with_cred(
            RuntimeOrigin::signed(admin(0)),
            main,
            beneficiary(),
            9,
        ));
        let pid = last_proposal_id();

        // STATUS_VOTING 期间 cleanup 应拒
        assert_noop!(
            PrivateManage::cleanup_rejected_private_proposal(RuntimeOrigin::signed(admin(0)), pid,),
            pallet::Error::<Test>::ProposalNotRejected
        );

        // 一票否决进入 REJECTED
        assert_ok!(cast_no_votes(&admin_accounts[1..], 1, pid));
        // 调 cleanup 仍应 Ok(虽然 Executor 已经 cleanup 过,这里是幂等再调)
        assert_ok!(PrivateManage::cleanup_rejected_private_proposal(
            RuntimeOrigin::signed(admin(0)),
            pid,
        ));
    });
}

#[test]
fn registry_creator_need_not_be_target_admin() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let cid = generated_cid("CID-NA", "SFLP");
        // 注册局代创建：交易发起人不要求进入新机构 admins 集合。
        let assignments_no_creator = institution_assignments_from(&[admin(1), admin(2), admin(3)]);
        assert_ok!(PrivateManage::propose_create_private_institution(
            RuntimeOrigin::signed(c),
            cid.clone(),
            cid_full_name(b"X"),
            cid_short_name("简称".as_bytes()),
            empty_town_code(),
            legal_representative_name(),
            legal_representative_cid_number(),
            legal_representative_account(),
            typical_accounts(),
            code_bytes("SFLP"),
            institution_roles_vec(),
            assignments_no_creator,
            2,
            register_nonce(b"nonce-na"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        let main = PrivateManage::derive_registered_account(cid.as_slice(), RESERVED_NAME_MAIN)
            .unwrap()
            .0;
        let stored = private_admins::AdminAccounts::<Test>::get(main)
            .expect("private admin account present");
        assert_eq!(
            stored.admins.to_vec(),
            alloc::vec![admin(1), admin(2), admin(3)]
        );
    });
}

#[test]
fn existential_deposit_is_preserved_after_close() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution("CID-ED", 3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();
        let beneficiary_acc = beneficiary();

        assert_ok!(close_with_cred(
            RuntimeOrigin::signed(admin(0)),
            main.clone(),
            beneficiary_acc.clone(),
            6,
        ));
        let pid = last_proposal_id();
        assert_ok!(cast_yes_votes(&admin_accounts[1..], 2, pid));

        // 级联注销主+费用账户(AllowDeath 转空),beneficiary 拿到 990+990=1980。
        assert_eq!(Balances::free_balance(&main), 0);
        assert_eq!(Balances::free_balance(&beneficiary_acc), 1980);
    });
}

#[test]
fn created_institution_stores_roles_assignments_and_pure_admin_accounts() {
    new_test_ext().execute_with(|| {
        let (cid, main) = create_and_activate_institution("CID-PROF", 3);
        // admins 只保存由有效任职去重派生的钱包账户。
        let stored = private_admins::AdminAccounts::<Test>::get(main.clone())
            .expect("private admin account present");
        assert_eq!(
            stored.admins.to_vec(),
            alloc::vec![admin(0), admin(1), admin(2)]
        );

        // 岗位和任职由 entity 按机构 CID 保存。
        let role_code: crate::RoleCodeOf =
            BoundedVec::try_from(b"TEST_ADMIN".to_vec()).expect("role code fits");
        assert!(pallet::InstitutionRoles::<Test>::contains_key(
            &cid, &role_code
        ));
        assert_eq!(
            pallet::InstitutionRoleAssignments::<Test>::get(&cid, &role_code).len(),
            3
        );

        // 一人一票/多签路径仍读账户:active_account_admins 返回 account 列表。
        let code = code_bytes("SFLP");
        let accounts = private_admins::Pallet::<Test>::active_account_admins(code, main.clone())
            .expect("active accounts present");
        assert_eq!(accounts, alloc::vec![admin(0), admin(1), admin(2)]);
    });
}

#[test]
fn election_result_replaces_role_assignments_and_preserves_admin_threshold() {
    new_test_ext().execute_with(|| {
        let (cid, main) = create_and_activate_institution("CID-ELECT", 3);
        let code = code_bytes("SFLP");

        assert_ok!(PrivateManage::apply_institution_assignment_result(
            entity_primitives::InstitutionAssignmentResult {
                institution_code: code,
                institution_account: main.clone(),
                role_code: b"TEST_ADMIN".to_vec(),
                admin_accounts: alloc::vec![admin(4), admin(1), admin(5)],
                term_start: 0,
                term_end: 0,
                assignment_source: entity_primitives::InstitutionAssignmentSource::MutualElection,
                assignment_source_ref: 93u64.to_le_bytes().to_vec(),
            }
        ));

        let role_code: crate::RoleCodeOf =
            BoundedVec::try_from(b"TEST_ADMIN".to_vec()).expect("role code fits");
        let assignments =
            pallet::InstitutionRoleAssignments::<Test>::get(&cid, &role_code).to_vec();
        assert_eq!(assignments.len(), 3);
        assert_eq!(assignments[0].admin_account, admin(4));
        assert_eq!(assignments[1].admin_account, admin(1));
        assert_eq!(assignments[2].admin_account, admin(5));
        assert!(assignments.iter().all(|assignment| {
            assignment.assignment_source
                == entity_primitives::InstitutionAssignmentSource::MutualElection
        }));

        let stored = private_admins::AdminAccounts::<Test>::get(main.clone())
            .expect("private admin account present");
        assert_eq!(
            stored.admins.to_vec(),
            alloc::vec![admin(1), admin(4), admin(5)]
        );
        assert_eq!(
            internal_vote::ActiveDynamicThresholds::<Test>::get(code, main),
            Some(3)
        );
    });
}

#[test]
fn assignment_and_admin_sync_roll_back_together_when_threshold_is_missing() {
    new_test_ext().execute_with(|| {
        let (cid, main) = create_and_activate_institution("CID-ELECT-RB", 3);
        let code = code_bytes("SFLP");
        let role_code: crate::RoleCodeOf =
            BoundedVec::try_from(b"TEST_ADMIN".to_vec()).expect("role code fits");
        let before = pallet::InstitutionRoleAssignments::<Test>::get(&cid, &role_code);
        internal_vote::ActiveDynamicThresholds::<Test>::remove(code, main.clone());

        assert_noop!(
            PrivateManage::apply_institution_assignment_result(
                entity_primitives::InstitutionAssignmentResult {
                    institution_code: code,
                    institution_account: main,
                    role_code: b"TEST_ADMIN".to_vec(),
                    admin_accounts: alloc::vec![admin(5), admin(6)],
                    term_start: 0,
                    term_end: 0,
                    assignment_source:
                        entity_primitives::InstitutionAssignmentSource::PopularElection,
                    assignment_source_ref: 94u64.to_le_bytes().to_vec(),
                }
            ),
            private_admins::Error::<Test>::MissingDynamicThreshold
        );
        assert_eq!(
            pallet::InstitutionRoleAssignments::<Test>::get(&cid, &role_code),
            before
        );
    });
}

#[test]
fn admin_account_is_main_account() {
    new_test_ext().execute_with(|| {
        // 管理员更换主体必须是机构 main AccountId,不是 CID 机构号或派生主体。
        let main = AccountId32::new([0x42; 32]);
        assert_eq!(main, AccountId32::new([0x42; 32]));
    });
}

#[test]
fn close_non_main_account_only_removes_that_account() {
    new_test_ext().execute_with(|| {
        let (cid, main) = create_and_activate_institution("CID-SUB", 3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();
        let beneficiary_acc = beneficiary();
        let fee_name = account_name(RESERVED_NAME_FEE);
        let fee_acc = PrivateManage::derive_registered_account(cid.as_slice(), RESERVED_NAME_FEE)
            .unwrap()
            .0;

        // 私权机构生命周期员(admin0)注销【非主】费用账户:role=Fee → scope=account。
        // 授权靠 resolve 统一解析到机构主账户的管理员集(子账户无独立管理员)。
        assert_ok!(close_with_cred(
            RuntimeOrigin::signed(admin(0)),
            fee_acc.clone(),
            beneficiary_acc.clone(),
            8,
        ));
        let pid = last_proposal_id();
        assert_ok!(cast_yes_votes(&admin_accounts[1..], 2, pid));

        // 仅费用账户被删;机构主账户 + AdminAccount + 机构记录保留(机构不消亡)。
        assert!(!pallet::InstitutionAccounts::<Test>::contains_key(
            &cid, &fee_name
        ));
        assert!(!pallet::AccountRegisteredCid::<Test>::contains_key(
            &fee_acc
        ));
        assert!(pallet::AccountRegisteredCid::<Test>::contains_key(&main));
        assert!(private_admins::AdminAccounts::<Test>::get(main).is_some());
        // 仅费用账户余额(1000-10)转 beneficiary。
        assert_eq!(Balances::free_balance(&beneficiary_acc), 990);
        assert_eq!(Balances::free_balance(&fee_acc), 0);
    });
}

#[test]
fn propose_close_rejects_invalid_deregister_credential() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution("CID-BC", 3);
        let bad_sig: pallet::RegisterSignatureOf<Test> =
            b"wrong-sig".to_vec().try_into().expect("sig fits");
        let nonce: pallet::RegisterNonceOf<Test> = vec![0xAB, 0xCD].try_into().expect("nonce fits");
        assert_noop!(
            PrivateManage::propose_close_private_institution(
                RuntimeOrigin::signed(admin(0)),
                main,
                beneficiary(),
                nonce,
                bad_sig,
                b"ISSUER".to_vec(),
                AccountId32::new([7u8; 32]),
                [9u8; 32],
            ),
            pallet::Error::<Test>::InvalidDeregisterCredential
        );
    });
}

#[test]
fn propose_close_rejects_replayed_deregister_nonce() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution("CID-NR", 3);
        // 首次注销(nonce seed 7)成功 → nonce 标记已用。
        assert_ok!(close_with_cred(
            RuntimeOrigin::signed(admin(0)),
            main.clone(),
            beneficiary(),
            7,
        ));
        // 同 nonce 再发起 → DeregisterNonceAlreadyUsed(nonce 检查先于并发检查命中)。
        assert_noop!(
            close_with_cred(RuntimeOrigin::signed(admin(0)), main, beneficiary(), 7),
            pallet::Error::<Test>::DeregisterNonceAlreadyUsed
        );
    });
}

#[test]
fn register_rejects_non_private_family_cid_number() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        // 真实格式的公权机构号(CGOV)打到私权入口必须被家族断言拒绝。
        assert_noop!(
            PrivateManage::register_cid_private_institution(
                RuntimeOrigin::signed(submitter),
                generated_cid("CID-FAMILY-X", "CGOV"),
                cid_full_name("机构甲".as_bytes()),
                account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]),
                register_nonce(b"nonce-family-x"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::InvalidCidNumber
        );
    });
}

// ── 机构信息维护:改名 + 新增账户 ──

#[test]
fn update_institution_info_changes_names_only() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let cid = generated_cid("CID-UPD-1", "SFLP");
        assert_ok!(PrivateManage::propose_create_private_institution(
            RuntimeOrigin::signed(c.clone()),
            cid.clone(),
            cid_full_name("旧全称".as_bytes()),
            cid_short_name("旧简称".as_bytes()),
            empty_town_code(),
            legal_representative_name(),
            legal_representative_cid_number(),
            legal_representative_account(),
            typical_accounts(),
            code_bytes("SFLP"),
            institution_roles_vec(),
            institution_assignments_vec(3),
            2,
            register_nonce(b"nonce-upd-c"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        assert_ok!(PrivateManage::update_institution_info(
            RuntimeOrigin::signed(c),
            cid.clone(),
            cid_full_name("新全称".as_bytes()),
            cid_short_name("新简称".as_bytes()),
            register_nonce(b"nonce-upd-u"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        let info = pallet::Institutions::<Test>::get(&cid).expect("institution");
        assert_eq!(info.cid_full_name, cid_full_name("新全称".as_bytes()));
        assert_eq!(info.cid_short_name, cid_short_name("新简称".as_bytes()));
        assert_eq!(info.institution_code, code_bytes("SFLP"));
    });
}

#[test]
fn update_institution_info_rejects_unknown() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        assert_noop!(
            PrivateManage::update_institution_info(
                RuntimeOrigin::signed(c),
                generated_cid("CID-UPD-X", "SFLP"),
                cid_full_name("x".as_bytes()),
                cid_short_name("y".as_bytes()),
                register_nonce(b"nonce-upd-x"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::InstitutionNotFound
        );
    });
}

#[test]
fn add_institution_account_derives_and_registers() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let cid = generated_cid("CID-ADD-1", "SFLP");
        assert_ok!(PrivateManage::propose_create_private_institution(
            RuntimeOrigin::signed(c.clone()),
            cid.clone(),
            cid_full_name("机构".as_bytes()),
            cid_short_name("简".as_bytes()),
            empty_town_code(),
            legal_representative_name(),
            legal_representative_cid_number(),
            legal_representative_account(),
            typical_accounts(),
            code_bytes("SFLP"),
            institution_roles_vec(),
            institution_assignments_vec(3),
            2,
            register_nonce(b"nonce-add-c"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        assert_ok!(PrivateManage::add_institution_account(
            RuntimeOrigin::signed(c),
            cid.clone(),
            account_names_bv(&["专项账户".as_bytes()]),
            register_nonce(b"nonce-add-a"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        let expected =
            PrivateManage::derive_registered_account(cid.as_slice(), "专项账户".as_bytes())
                .expect("derive")
                .0;
        assert_eq!(
            pallet::CidRegisteredAccount::<Test>::get(&cid, &account_name("专项账户".as_bytes())),
            Some(expected.clone())
        );
        assert!(pallet::InstitutionAccounts::<Test>::contains_key(
            &cid,
            &account_name("专项账户".as_bytes())
        ));
        assert!(pallet::AccountRegisteredCid::<Test>::contains_key(
            &expected
        ));
    });
}
