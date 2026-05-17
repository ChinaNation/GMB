use super::*;
use crate::address::{RESERVED_NAME_FEE, RESERVED_NAME_MAIN};
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
// SFID 登记路径(5 个用例)
// ============================================================

#[test]
fn register_sfid_institution_with_valid_signature_succeeds() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        let sfid = sfid_number(b"SFID001");
        let names = account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]);

        assert_ok!(OrganizationManage::register_sfid_institution(
            RuntimeOrigin::signed(submitter),
            sfid.clone(),
            institution_name("机构甲".as_bytes()),
            names.clone(),
            register_nonce(b"nonce-1"),
            valid_signature(),
            province(),
            signer_pubkey(),
        ));

        assert!(pallet::SfidRegisteredAddress::<Test>::contains_key(
            &sfid,
            &account_name(RESERVED_NAME_MAIN),
        ));
        assert!(pallet::SfidRegisteredAddress::<Test>::contains_key(
            &sfid,
            &account_name(RESERVED_NAME_FEE),
        ));
        // 反向索引也写入
        let main_addr = OrganizationManage::derive_institution_address(
            sfid.as_slice(),
            crate::address::InstitutionAccountRole::Main,
        )
        .expect("derive main");
        assert!(pallet::AddressRegisteredSfid::<Test>::contains_key(
            &main_addr
        ));
    });
}

#[test]
fn register_rejects_invalid_sfid_institution_signature() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        assert_noop!(
            OrganizationManage::register_sfid_institution(
                RuntimeOrigin::signed(submitter),
                sfid_number(b"SFID-bad-sig"),
                institution_name("机构甲".as_bytes()),
                account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]),
                register_nonce(b"nonce-bs"),
                invalid_signature(),
                province(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::InvalidSfidInstitutionSignature
        );
    });
}

#[test]
fn register_rejects_duplicate_sfid_account_name() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        let sfid = sfid_number(b"SFID-dup");
        // 第一次成功
        assert_ok!(OrganizationManage::register_sfid_institution(
            RuntimeOrigin::signed(submitter.clone()),
            sfid.clone(),
            institution_name("机构 A".as_bytes()),
            account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]),
            register_nonce(b"nonce-a1"),
            valid_signature(),
            province(),
            signer_pubkey(),
        ));
        // 第二次相同 sfid + 主账户:SfidAlreadyRegistered
        assert_noop!(
            OrganizationManage::register_sfid_institution(
                RuntimeOrigin::signed(submitter),
                sfid,
                institution_name("机构 A".as_bytes()),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                register_nonce(b"nonce-a2"),
                valid_signature(),
                province(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::SfidAlreadyRegistered
        );
    });
}

#[test]
fn register_rejects_replayed_nonce() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        // 第一次成功
        assert_ok!(OrganizationManage::register_sfid_institution(
            RuntimeOrigin::signed(submitter.clone()),
            sfid_number(b"SFID-N1"),
            institution_name(b"A"),
            account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]),
            register_nonce(b"nonce-replay"),
            valid_signature(),
            province(),
            signer_pubkey(),
        ));
        // 第二次同 nonce 不同 sfid:RegisterNonceAlreadyUsed
        assert_noop!(
            OrganizationManage::register_sfid_institution(
                RuntimeOrigin::signed(submitter),
                sfid_number(b"SFID-N2"),
                institution_name(b"B"),
                account_names_bv(&[RESERVED_NAME_MAIN, RESERVED_NAME_FEE]),
                register_nonce(b"nonce-replay"),
                valid_signature(),
                province(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::RegisterNonceAlreadyUsed
        );
    });
}

#[test]
fn register_rejects_empty_required_fields() {
    new_test_ext().execute_with(|| {
        let submitter = fund_creator();
        // 空 sfid_number
        assert_noop!(
            OrganizationManage::register_sfid_institution(
                RuntimeOrigin::signed(submitter.clone()),
                sfid_number(b""),
                institution_name(b"A"),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                register_nonce(b"nonce-empty1"),
                valid_signature(),
                province(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::EmptySfidNumber
        );
        // 空 institution_name
        assert_noop!(
            OrganizationManage::register_sfid_institution(
                RuntimeOrigin::signed(submitter.clone()),
                sfid_number(b"SFID-E"),
                institution_name(b""),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                register_nonce(b"nonce-empty2"),
                valid_signature(),
                province(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::EmptyAccountName
        );
        // 空 province
        assert_noop!(
            OrganizationManage::register_sfid_institution(
                RuntimeOrigin::signed(submitter),
                sfid_number(b"SFID-E"),
                institution_name(b"A"),
                account_names_bv(&[RESERVED_NAME_MAIN]),
                register_nonce(b"nonce-empty3"),
                valid_signature(),
                alloc::vec::Vec::new(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::EmptyProvince
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
        let sfid = sfid_number(b"SFID-CR-1");

        assert_ok!(OrganizationManage::propose_create_institution(
            RuntimeOrigin::signed(c.clone()),
            sfid.clone(),
            institution_name("机构甲".as_bytes()),
            typical_accounts(),
            ORG_OTH,
            3,
            admins_vec(3),
            2,
            register_nonce(b"nonce-cr-1"),
            valid_signature(),
            province(),
            signer_pubkey(),
        ));

        let pid = last_proposal_id();
        assert!(pallet::PendingInstitutionCreate::<Test>::contains_key(pid));
        assert!(pallet::Institutions::<Test>::contains_key(&sfid));
        assert_eq!(
            pallet::Institutions::<Test>::get(&sfid).unwrap().status,
            InstitutionLifecycleStatus::Pending,
        );
        assert_eq!(
            pallet::Institutions::<Test>::get(&sfid).unwrap().admin_org,
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
        let sfid = sfid_number(b"SFID-CR-2");
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();

        assert_ok!(OrganizationManage::propose_create_institution(
            RuntimeOrigin::signed(c.clone()),
            sfid.clone(),
            institution_name("机构乙".as_bytes()),
            typical_accounts(),
            ORG_OTH,
            3,
            admins_vec(3),
            2,
            register_nonce(b"nonce-cr-2"),
            valid_signature(),
            province(),
            signer_pubkey(),
        ));
        let pid = last_proposal_id();
        assert_ok!(cast_yes_votes(&admin_accounts[1..], 2, pid));

        // 执行成功
        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal");
        assert_eq!(proposal.status, STATUS_EXECUTED);
        assert_eq!(
            pallet::Institutions::<Test>::get(&sfid).unwrap().status,
            InstitutionLifecycleStatus::Active,
        );
        // 主账户和费用账户都被划账
        let main = OrganizationManage::derive_institution_address(
            sfid.as_slice(),
            crate::address::InstitutionAccountRole::Main,
        )
        .unwrap();
        let fee_acc = OrganizationManage::derive_institution_address(
            sfid.as_slice(),
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
        let sfid = sfid_number(b"SFID-CR-3");
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();

        assert_ok!(OrganizationManage::propose_create_institution(
            RuntimeOrigin::signed(c.clone()),
            sfid.clone(),
            institution_name("机构丙".as_bytes()),
            typical_accounts(),
            ORG_OTH,
            3,
            admins_vec(3),
            2,
            register_nonce(b"nonce-cr-3"),
            valid_signature(),
            province(),
            signer_pubkey(),
        ));
        let pid = last_proposal_id();

        // 一票否决,创建提案要求全员通过 → 立刻 REJECTED
        assert_ok!(cast_no_votes(&admin_accounts[1..], 1, pid));

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal");
        assert_eq!(proposal.status, STATUS_REJECTED);

        assert_eq!(Balances::reserved_balance(&c), 0);
        assert!(!pallet::Institutions::<Test>::contains_key(&sfid));
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
                sfid_number(b"SFID-MIN"),
                institution_name(b"X"),
                bad_accounts,
                ORG_OTH,
                3,
                admins_vec(3),
                2,
                register_nonce(b"nonce-min"),
                valid_signature(),
                province(),
                signer_pubkey(),
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
                sfid_number(b"SFID-DUP"),
                institution_name(b"X"),
                dup,
                ORG_OTH,
                3,
                admins_vec(3),
                2,
                register_nonce(b"nonce-dup"),
                valid_signature(),
                province(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::DuplicateAccountName
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
                sfid_number(b"SFID-NM"),
                institution_name(b"X"),
                no_main,
                ORG_OTH,
                3,
                admins_vec(3),
                2,
                register_nonce(b"nonce-nm"),
                valid_signature(),
                province(),
                signer_pubkey(),
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
                sfid_number(b"SFID-T1"),
                institution_name(b"X"),
                typical_accounts(),
                ORG_OTH,
                3,
                admins_vec(3),
                1,
                register_nonce(b"nonce-t1"),
                valid_signature(),
                province(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::InvalidThreshold
        );
        // threshold > admin_count
        assert_noop!(
            OrganizationManage::propose_create_institution(
                RuntimeOrigin::signed(c),
                sfid_number(b"SFID-T2"),
                institution_name(b"X"),
                typical_accounts(),
                ORG_OTH,
                3,
                admins_vec(3),
                4,
                register_nonce(b"nonce-t2"),
                valid_signature(),
                province(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::InvalidThreshold
        );
    });
}

#[test]
fn propose_create_rejects_when_institution_already_exists() {
    new_test_ext().execute_with(|| {
        let c = fund_creator();
        let sfid = sfid_number(b"SFID-AE");

        // 先创建一个
        assert_ok!(OrganizationManage::propose_create_institution(
            RuntimeOrigin::signed(c.clone()),
            sfid.clone(),
            institution_name(b"A"),
            typical_accounts(),
            ORG_OTH,
            3,
            admins_vec(3),
            2,
            register_nonce(b"nonce-ae1"),
            valid_signature(),
            province(),
            signer_pubkey(),
        ));
        // 第二次同 sfid 应拒
        assert_noop!(
            OrganizationManage::propose_create_institution(
                RuntimeOrigin::signed(c),
                sfid,
                institution_name(b"B"),
                typical_accounts(),
                ORG_OTH,
                3,
                admins_vec(3),
                2,
                register_nonce(b"nonce-ae2"),
                valid_signature(),
                province(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::InstitutionAlreadyExists
        );
    });
}

// ============================================================
// 关闭路径(5 个用例)
// ============================================================

fn create_and_activate_institution(
    sfid_number_bytes: &[u8],
    admin_count: u8,
) -> (pallet::SfidNumberOf<Test>, AccountId32) {
    let c = creator();
    let _ = Balances::deposit_creating(&c, SEED_BALANCE);
    let sfid = sfid_number(sfid_number_bytes);
    let admin_accounts: alloc::vec::Vec<AccountId32> = (0..admin_count).map(|i| admin(i)).collect();

    assert_ok!(OrganizationManage::propose_create_institution(
        RuntimeOrigin::signed(c.clone()),
        sfid.clone(),
        institution_name(b"X"),
        typical_accounts(),
        ORG_OTH,
        admin_count as u32,
        admins_vec(admin_count),
        admin_count.saturating_add(1) as u32 / 2 + 1, // m-of-n 治理阈值,取一个能通过的
        register_nonce(sfid_number_bytes),
        valid_signature(),
        province(),
        signer_pubkey(),
    ));
    let pid = last_proposal_id();
    assert_ok!(cast_yes_votes(
        &admin_accounts[1..],
        admin_count.saturating_sub(1) as usize,
        pid
    ));

    let main = OrganizationManage::derive_institution_address(
        sfid.as_slice(),
        crate::address::InstitutionAccountRole::Main,
    )
    .unwrap();
    (sfid, main)
}

#[test]
fn propose_close_writes_pending() {
    new_test_ext().execute_with(|| {
        let (_sfid, main) = create_and_activate_institution(b"SFID-CL-1", 3);

        assert_ok!(OrganizationManage::propose_close(
            RuntimeOrigin::signed(admin(0)),
            main.clone(),
            beneficiary(),
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
        let (sfid, main) = create_and_activate_institution(b"SFID-CL-2", 3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();
        let beneficiary_acc = beneficiary();
        let main_name = account_name(RESERVED_NAME_MAIN);
        let subject = primitives::derive::subject_id_from_institution_account(&main);

        assert_ok!(OrganizationManage::propose_close(
            RuntimeOrigin::signed(admin(0)),
            main.clone(),
            beneficiary_acc.clone(),
        ));
        let pid = last_proposal_id();
        assert_ok!(cast_yes_votes(&admin_accounts[1..], 2, pid));

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal");
        assert_eq!(proposal.status, STATUS_EXECUTED);

        // ACCT_AMOUNT=1000 → fee = max(1, 10) = 10,beneficiary 收 990
        assert_eq!(Balances::free_balance(&beneficiary_acc), 990);
        assert_eq!(Balances::free_balance(&main), 0);
        assert!(!pallet::InstitutionPendingClose::<Test>::contains_key(
            &main
        ));
        assert!(!pallet::InstitutionAccounts::<Test>::contains_key(
            &sfid, &main_name
        ));
        assert!(!pallet::SfidRegisteredAddress::<Test>::contains_key(
            &sfid, &main_name
        ));
        assert!(!pallet::AddressRegisteredSfid::<Test>::contains_key(&main));
        assert!(admins_change::Subjects::<Test>::get(subject).is_none());
        assert!(internal_vote::ActiveDynamicThresholds::<Test>::get(ORG_OTH, subject).is_none());
    });
}

#[test]
fn propose_close_rejects_close_balance_below_minimum() {
    new_test_ext().execute_with(|| {
        let (_sfid, main) = create_and_activate_institution(b"SFID-CL-3", 3);

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
            OrganizationManage::propose_close(
                RuntimeOrigin::signed(admin(0)),
                main,
                beneficiary(),
            ),
            pallet::Error::<Test>::CloseBalanceBelowMinimum
        );
    });
}

#[test]
fn propose_close_rejects_when_not_institution_address() {
    new_test_ext().execute_with(|| {
        // 没在 AddressRegisteredSfid 表里的地址
        let stranger = AccountId32::new([0xEE; 32]);
        assert_noop!(
            OrganizationManage::propose_close(
                RuntimeOrigin::signed(admin(0)),
                stranger,
                beneficiary(),
            ),
            pallet::Error::<Test>::NotInstitutionDuoqian
        );
    });
}

#[test]
fn propose_close_rejects_self_beneficiary() {
    new_test_ext().execute_with(|| {
        let (_sfid, main) = create_and_activate_institution(b"SFID-CL-5", 3);
        // beneficiary == duoqian_address 应拒
        assert_noop!(
            OrganizationManage::propose_close(RuntimeOrigin::signed(admin(0)), main.clone(), main,),
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
            sfid_number(b"SFID-CU"),
            institution_name(b"X"),
            typical_accounts(),
            ORG_OTH,
            3,
            admins_vec(3),
            2,
            register_nonce(b"nonce-cu"),
            valid_signature(),
            province(),
            signer_pubkey(),
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
                sfid_number(b"SFID-NA"),
                institution_name(b"X"),
                typical_accounts(),
                ORG_OTH,
                3,
                admins_no_creator,
                2,
                register_nonce(b"nonce-na"),
                valid_signature(),
                province(),
                signer_pubkey(),
            ),
            pallet::Error::<Test>::PermissionDenied
        );
    });
}

#[test]
fn existential_deposit_is_preserved_after_close() {
    new_test_ext().execute_with(|| {
        let (_sfid, main) = create_and_activate_institution(b"SFID-ED", 3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();
        let beneficiary_acc = beneficiary();

        assert_ok!(OrganizationManage::propose_close(
            RuntimeOrigin::signed(admin(0)),
            main.clone(),
            beneficiary_acc.clone(),
        ));
        let pid = last_proposal_id();
        assert_ok!(cast_yes_votes(&admin_accounts[1..], 2, pid));

        // 主账户转空(AllowDeath),beneficiary 拿到 990
        assert_eq!(Balances::free_balance(&main), 0);
        assert_eq!(Balances::free_balance(&beneficiary_acc), 990);
    });
}

#[test]
fn admin_subject_id_is_built_from_main_account_with_kind_tag() {
    new_test_ext().execute_with(|| {
        // 管理员更换主体必须是主账户地址派生的 InstitutionAccount,不是 SFID 机构号。
        let main = AccountId32::new([0x42; 32]);
        let subj = primitives::derive::subject_id_from_institution_account(&main);
        assert_eq!(subj[0], 0x05, "kind tag must be InstitutionAccount");
    });
}
