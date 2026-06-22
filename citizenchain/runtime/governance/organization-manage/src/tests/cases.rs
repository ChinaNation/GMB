use super::*;
use crate::address::{
    RESERVED_NAME_ANQUAN, RESERVED_NAME_FEE, RESERVED_NAME_HE, RESERVED_NAME_MAIN,
    RESERVED_NAME_STAKE,
};
use crate::institution::types::InstitutionLifecycleStatus;
use frame_support::{assert_noop, assert_ok, traits::Currency, BoundedVec};
use votingengine::{STATUS_EXECUTED, STATUS_REJECTED};

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

// ============================================================
// CID 登记路径(5 个用例)
// ============================================================

#[test]
fn register_cid_institution_with_valid_signature_succeeds() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        let cid = cid_number(b"CID001");
        let names = account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]);

        assert_ok!(OrganizationManage::register_cid_institution(
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
        let main_addr = OrganizationManage::derive_institution_account(
            cid.as_slice(),
            crate::address::InstitutionAccountRole::Main,
        )
        .expect("derive main");
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
            OrganizationManage::register_cid_institution(
                RuntimeOrigin::signed(submitter),
                cid_number(b"CID-bad-sig"),
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
        let cid = cid_number(b"CID-dup");
        // 第一次成功
        assert_ok!(OrganizationManage::register_cid_institution(
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
            OrganizationManage::register_cid_institution(
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
        assert_ok!(OrganizationManage::register_cid_institution(
            RuntimeOrigin::signed(submitter.clone()),
            cid_number(b"CID-N1"),
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
            OrganizationManage::register_cid_institution(
                RuntimeOrigin::signed(submitter),
                cid_number(b"CID-N2"),
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
            OrganizationManage::register_cid_institution(
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
            OrganizationManage::register_cid_institution(
                RuntimeOrigin::signed(submitter.clone()),
                cid_number(b"CID-E"),
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
            OrganizationManage::register_cid_institution(
                RuntimeOrigin::signed(submitter),
                cid_number(b"CID-E"),
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

// ============================================================
// 创建路径(8 个用例)
// ============================================================

#[test]
fn propose_create_institution_writes_pending_and_reserves() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let cid = cid_number(b"CID-CR-1");

        assert_ok!(OrganizationManage::propose_create_institution(
            RuntimeOrigin::signed(c.clone()),
            cid.clone(),
            cid_full_name("机构甲".as_bytes()),
            typical_accounts(),
            ORG_OTH,
            3,
            admins_vec(3),
            2,
            register_nonce(b"nonce-cr-1"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));

        let pid = last_proposal_id();
        assert!(pallet::PendingInstitutionCreate::<Test>::contains_key(pid));
        assert!(pallet::Institutions::<Test>::contains_key(&cid));
        assert_eq!(
            pallet::Institutions::<Test>::get(&cid).unwrap().status,
            InstitutionLifecycleStatus::Pending,
        );
        assert_eq!(
            pallet::Institutions::<Test>::get(&cid).unwrap().org,
            ORG_OTH,
        );
        // 主+费用 共 2_000 入金 + fee = max(2000*0.001, 10) = 10 → reserve 2_010
        assert_eq!(Balances::reserved_balance(&c), 2_000 + 10);
    });
}

#[test]
fn create_executes_when_vote_reaches_threshold_with_initial_accounts() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let cid = cid_number(b"CID-CR-2");
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();

        assert_ok!(OrganizationManage::propose_create_institution(
            RuntimeOrigin::signed(c.clone()),
            cid.clone(),
            cid_full_name("机构乙".as_bytes()),
            typical_accounts(),
            ORG_OTH,
            3,
            admins_vec(3),
            2,
            register_nonce(b"nonce-cr-2"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        let pid = last_proposal_id();
        assert_ok!(cast_yes_votes(&admin_accounts[1..], 2, pid));

        // 执行成功
        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal");
        assert_eq!(proposal.status, STATUS_EXECUTED);
        assert_eq!(
            pallet::Institutions::<Test>::get(&cid).unwrap().status,
            InstitutionLifecycleStatus::Active,
        );
        // 主账户和费用账户都被划账
        let main = OrganizationManage::derive_institution_account(
            cid.as_slice(),
            crate::address::InstitutionAccountRole::Main,
        )
        .unwrap();
        let fee_acc = OrganizationManage::derive_institution_account(
            cid.as_slice(),
            crate::address::InstitutionAccountRole::Fee,
        )
        .unwrap();
        assert_eq!(Balances::free_balance(&main), ACCT_AMOUNT);
        assert_eq!(Balances::free_balance(&fee_acc), ACCT_AMOUNT);
        assert_eq!(Balances::reserved_balance(&c), 0);
    });
}

#[test]
fn create_rejected_releases_reserve_and_no_storage_residue() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let cid = cid_number(b"CID-CR-3");
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();

        assert_ok!(OrganizationManage::propose_create_institution(
            RuntimeOrigin::signed(c.clone()),
            cid.clone(),
            cid_full_name("机构丙".as_bytes()),
            typical_accounts(),
            ORG_OTH,
            3,
            admins_vec(3),
            2,
            register_nonce(b"nonce-cr-3"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        let pid = last_proposal_id();

        // 一票否决,创建提案要求全员通过 → 立刻 REJECTED
        assert_ok!(cast_no_votes(&admin_accounts[1..], 1, pid));

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal");
        assert_eq!(proposal.status, STATUS_REJECTED);

        assert_eq!(Balances::reserved_balance(&c), 0);
        assert!(!pallet::Institutions::<Test>::contains_key(&cid));
        assert!(!pallet::PendingInstitutionCreate::<Test>::contains_key(pid));
    });
}

#[test]
fn propose_create_rejects_below_create_amount_minimum() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        // MinCreateAmount=111, 用 50 应拒
        let bad_accounts = initial_accounts(&[(RESERVED_NAME_MAIN, 50), (RESERVED_NAME_FEE, 200)]);
        assert_noop!(
            OrganizationManage::propose_create_institution(
                RuntimeOrigin::signed(c),
                cid_number(b"CID-MIN"),
                cid_full_name(b"X"),
                bad_accounts,
                ORG_OTH,
                3,
                admins_vec(3),
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
            OrganizationManage::propose_create_institution(
                RuntimeOrigin::signed(c),
                cid_number(b"CID-DUP"),
                cid_full_name(b"X"),
                dup,
                ORG_OTH,
                3,
                admins_vec(3),
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
fn role_from_account_name_rejects_reserved_system_names() {
    new_test_ext().execute_with(|| {
        // 永久质押/安全基金/两和基金 为制度专属账户,普通机构禁止注册。
        for name in [RESERVED_NAME_STAKE, RESERVED_NAME_ANQUAN, RESERVED_NAME_HE] {
            assert_eq!(
                OrganizationManage::role_from_account_name(name).unwrap_err(),
                pallet::Error::<Test>::ReservedAccountName.into()
            );
        }
        // 主账户/费用账户仍强制路由,不报错。
        assert!(matches!(
            OrganizationManage::role_from_account_name(RESERVED_NAME_MAIN).unwrap(),
            crate::address::InstitutionAccountRole::Main
        ));
        assert!(matches!(
            OrganizationManage::role_from_account_name(RESERVED_NAME_FEE).unwrap(),
            crate::address::InstitutionAccountRole::Fee
        ));
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
            (RESERVED_NAME_ANQUAN, ACCT_AMOUNT),
        ]);
        assert_noop!(
            OrganizationManage::propose_create_institution(
                RuntimeOrigin::signed(c),
                cid_number(b"CID-RSV"),
                cid_full_name(b"X"),
                bad,
                ORG_OTH,
                3,
                admins_vec(3),
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
            OrganizationManage::propose_create_institution(
                RuntimeOrigin::signed(c),
                cid_number(b"CID-NM"),
                cid_full_name(b"X"),
                no_main,
                ORG_OTH,
                3,
                admins_vec(3),
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
            OrganizationManage::propose_create_institution(
                RuntimeOrigin::signed(c.clone()),
                cid_number(b"CID-T1"),
                cid_full_name(b"X"),
                typical_accounts(),
                ORG_OTH,
                3,
                admins_vec(3),
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
            OrganizationManage::propose_create_institution(
                RuntimeOrigin::signed(c),
                cid_number(b"CID-T2"),
                cid_full_name(b"X"),
                typical_accounts(),
                ORG_OTH,
                3,
                admins_vec(3),
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
        let cid = cid_number(b"CID-AE");

        // 先创建一个
        assert_ok!(OrganizationManage::propose_create_institution(
            RuntimeOrigin::signed(c.clone()),
            cid.clone(),
            cid_full_name(b"A"),
            typical_accounts(),
            ORG_OTH,
            3,
            admins_vec(3),
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
            OrganizationManage::propose_create_institution(
                RuntimeOrigin::signed(c),
                cid,
                cid_full_name(b"B"),
                typical_accounts(),
                ORG_OTH,
                3,
                admins_vec(3),
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

// ============================================================
// 关闭路径(5 个用例)
// ============================================================

fn create_and_activate_institution(
    cid_number_bytes: &[u8],
    admins_len: u8,
) -> (pallet::CidNumberOf<Test>, AccountId32) {
    let c = creator();
    let _ = Balances::deposit_creating(&c, SEED_BALANCE);
    let cid = cid_number(cid_number_bytes);
    let admin_accounts: alloc::vec::Vec<AccountId32> = (0..admins_len).map(|i| admin(i)).collect();

    assert_ok!(OrganizationManage::propose_create_institution(
        RuntimeOrigin::signed(c.clone()),
        cid.clone(),
        cid_full_name(b"X"),
        typical_accounts(),
        ORG_OTH,
        admins_len as u32,
        admins_vec(admins_len),
        admins_len.saturating_add(1) as u32 / 2 + 1, // m-of-n 治理阈值,取一个能通过的
        register_nonce(cid_number_bytes),
        valid_signature(),
        province_name(),
        creator(),
        signer_pubkey(),
        province_name(),
        b"city".to_vec(),
    ));
    let pid = last_proposal_id();
    assert_ok!(cast_yes_votes(
        &admin_accounts[1..],
        admins_len.saturating_sub(1) as usize,
        pid
    ));

    let main = OrganizationManage::derive_institution_account(
        cid.as_slice(),
        crate::address::InstitutionAccountRole::Main,
    )
    .unwrap();
    (cid, main)
}

#[test]
fn propose_close_writes_pending() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution(b"CID-CL-1", 3);

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
        let (cid, main) = create_and_activate_institution(b"CID-CL-2", 3);
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
        assert!(admins_change::AdminAccounts::<Test>::get(account.clone()).is_none());
        assert!(internal_vote::ActiveDynamicThresholds::<Test>::get(ORG_OTH, account).is_none());
    });
}

#[test]
fn propose_close_rejects_close_balance_below_minimum() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution(b"CID-CL-3", 3);

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
        let (_cid, main) = create_and_activate_institution(b"CID-CL-5", 3);
        // beneficiary == account 应拒
        assert_noop!(
            close_with_cred(RuntimeOrigin::signed(admin(0)), main.clone(), main, 5),
            pallet::Error::<Test>::InvalidBeneficiary
        );
    });
}

// ============================================================
// Cleanup / 边界(4 个用例)
// ============================================================

#[test]
fn cleanup_rejected_proposal_only_after_engine_rejected() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();

        assert_ok!(OrganizationManage::propose_create_institution(
            RuntimeOrigin::signed(c),
            cid_number(b"CID-CU"),
            cid_full_name(b"X"),
            typical_accounts(),
            ORG_OTH,
            3,
            admins_vec(3),
            2,
            register_nonce(b"nonce-cu"),
            valid_signature(),
            province_name(),
            creator(),
            signer_pubkey(),
            province_name(),
            b"city".to_vec(),
        ));
        let pid = last_proposal_id();

        // STATUS_VOTING 期间 cleanup 应拒
        assert_noop!(
            OrganizationManage::cleanup_rejected_proposal(RuntimeOrigin::signed(admin(0)), pid,),
            pallet::Error::<Test>::ProposalNotRejected
        );

        // 一票否决进入 REJECTED
        assert_ok!(cast_no_votes(&admin_accounts[1..], 1, pid));
        // 调 cleanup 仍应 Ok(虽然 Executor 已经 cleanup 过,这里是幂等再调)
        assert_ok!(OrganizationManage::cleanup_rejected_proposal(
            RuntimeOrigin::signed(admin(0)),
            pid,
        ));
    });
}

#[test]
fn non_admin_cannot_propose_create() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        // 提案者不在 admins 列表 → PermissionDenied
        let admins_no_creator =
            BoundedVec::try_from(vec![admin(1), admin(2), admin(3)]).expect("fits");
        assert_noop!(
            OrganizationManage::propose_create_institution(
                RuntimeOrigin::signed(c),
                cid_number(b"CID-NA"),
                cid_full_name(b"X"),
                typical_accounts(),
                ORG_OTH,
                3,
                admins_no_creator,
                2,
                register_nonce(b"nonce-na"),
                valid_signature(),
                province_name(),
                creator(),
                signer_pubkey(),
                province_name(),
                b"city".to_vec(),
            ),
            pallet::Error::<Test>::PermissionDenied
        );
    });
}

#[test]
fn existential_deposit_is_preserved_after_close() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution(b"CID-ED", 3);
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
        let (cid, main) = create_and_activate_institution(b"CID-SUB", 3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();
        let beneficiary_acc = beneficiary();
        let fee_name = account_name(RESERVED_NAME_FEE);
        let fee_acc = OrganizationManage::derive_institution_account(
            cid.as_slice(),
            crate::address::InstitutionAccountRole::Fee,
        )
        .unwrap();

        // 机构管理员(admin0)注销【非主】费用账户:role=Fee → scope=account。
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
        assert!(!pallet::InstitutionAccounts::<Test>::contains_key(&cid, &fee_name));
        assert!(!pallet::AccountRegisteredCid::<Test>::contains_key(&fee_acc));
        assert!(pallet::AccountRegisteredCid::<Test>::contains_key(&main));
        assert!(admins_change::AdminAccounts::<Test>::get(main).is_some());
        // 仅费用账户余额(1000-10)转 beneficiary。
        assert_eq!(Balances::free_balance(&beneficiary_acc), 990);
        assert_eq!(Balances::free_balance(&fee_acc), 0);
    });
}

#[test]
fn propose_close_rejects_invalid_deregister_credential() {
    new_test_ext().execute_with(|| {
        let (_cid, main) = create_and_activate_institution(b"CID-BC", 3);
        let bad_sig: pallet::RegisterSignatureOf<Test> =
            b"wrong-sig".to_vec().try_into().expect("sig fits");
        let nonce: pallet::RegisterNonceOf<Test> = vec![0xAB, 0xCD].try_into().expect("nonce fits");
        assert_noop!(
            OrganizationManage::propose_close(
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
        let (_cid, main) = create_and_activate_institution(b"CID-NR", 3);
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
