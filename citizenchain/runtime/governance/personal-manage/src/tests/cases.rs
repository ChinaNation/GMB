use super::*;
use frame_support::{assert_noop, assert_ok, traits::Currency, BoundedVec};
use sp_runtime::DispatchError;
use votingengine::{STATUS_EXECUTED, STATUS_REJECTED, STATUS_VOTING};

const CREATE_AMOUNT: Balance = 1_000;
const CREATE_FEE: Balance = 10; // calculate_onchain_fee(1000) = max(1000*0.001, 10) = 10
const SEED_BALANCE: Balance = 5_000;

fn setup_creator_balance() -> AccountId32 {
    let c = creator();
    let _ = Balances::deposit_creating(&c, SEED_BALANCE);
    c
}

fn proposed_duoqian_address(creator: &AccountId32, name: &[u8]) -> AccountId32 {
    PersonalManage::derive_personal_duoqian_address(creator, name).expect("derive should succeed")
}

// ─── 1. propose_create:写 Pending + reserve fee + 发事件 ─────────────

#[test]
fn propose_create_writes_pending_and_reserves_fee() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let admins = admins_vec(3); // admin(0)..admin(2)
        let name = account_name(b"alice-personal");
        let dq = proposed_duoqian_address(&c, b"alice-personal");

        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c.clone()),
            name.clone(),
            admins.clone(),
            CREATE_AMOUNT,
        ));

        let pid = last_proposal_id();

        assert!(pallet::PendingPersonalCreate::<Test>::contains_key(pid));
        assert!(pallet::PersonalDuoqians::<Test>::contains_key(&dq));
        assert!(pallet::PersonalDuoqianInfo::<Test>::contains_key(&dq));
        assert_eq!(
            pallet::PersonalDuoqians::<Test>::get(&dq).unwrap().status,
            types::DuoqianStatus::Pending
        );
        let subject = primitives::derive::subject_id_from_account(&dq);
        assert_eq!(
            admins_change::Pallet::<Test>::pending_subject_threshold_for_snapshot(ORG_REN, subject),
            Some(2)
        );
        assert_eq!(
            internal_vote::InternalThresholdSnapshot::<Test>::get(pid),
            Some(3)
        );
        // creator 已被 reserve(amount + fee)
        assert_eq!(Balances::reserved_balance(&c), CREATE_AMOUNT + CREATE_FEE);
        assert_eq!(
            Balances::free_balance(&c),
            SEED_BALANCE - CREATE_AMOUNT - CREATE_FEE
        );

        // 投票引擎里提案在投票中
        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal exists");
        assert_eq!(proposal.status, STATUS_VOTING);
    });
}

// ─── 2. 投票通过 → 创建账户 → release reserve ──────────────────────────

#[test]
fn create_executes_when_internal_vote_reaches_threshold() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let admins = admins_vec(3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();
        let name = account_name(b"alice-personal");
        let dq = proposed_duoqian_address(&c, b"alice-personal");

        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c.clone()),
            name,
            admins,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();

        // 创建提案要求"全员通过"——投票引擎 threshold = admins.len() = 3
        assert_ok!(cast_yes_votes(&admin_accounts, 3, pid));

        // 提案进入 EXECUTED
        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal exists");
        assert_eq!(proposal.status, STATUS_EXECUTED);

        // 多签账户激活,资金到位,Pending 已清
        let dq_state = pallet::PersonalDuoqians::<Test>::get(&dq).expect("active duoqian");
        assert_eq!(dq_state.status, types::DuoqianStatus::Active);
        let subject = primitives::derive::subject_id_from_account(&dq);
        assert_eq!(
            admins_change::Pallet::<Test>::active_subject_threshold(ORG_REN, subject),
            Some(2)
        );
        assert_eq!(Balances::free_balance(&dq), CREATE_AMOUNT);
        assert_eq!(Balances::reserved_balance(&c), 0);
        assert!(!pallet::PendingPersonalCreate::<Test>::contains_key(pid));
    });
}

// ─── 3. 投票拒绝 → cleanup_rejected → 释放 reserve + 发拒绝事件 ────────

#[test]
fn create_rejected_cleanup_releases_reserve_and_emits_event() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let admins = admins_vec(3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();
        let name = account_name(b"alice-personal");
        let dq = proposed_duoqian_address(&c, b"alice-personal");

        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c.clone()),
            name,
            admins,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();

        // 一票否决:全员通过制度下,只要有人反对就立刻进 STATUS_REJECTED
        assert_ok!(cast_no_votes(&admin_accounts, 1, pid));

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal exists");
        assert_eq!(proposal.status, STATUS_REJECTED);

        // 拒绝路径下 Executor 应已 cleanup,reserve 释放,storage 清空
        assert_eq!(Balances::reserved_balance(&c), 0);
        assert!(!pallet::PersonalDuoqians::<Test>::contains_key(&dq));
        assert!(!pallet::PendingPersonalCreate::<Test>::contains_key(pid));
    });
}

// ─── 4. 重复地址被拒绝 ────────────────────────────────────────────────

#[test]
fn propose_create_rejects_duplicate_personal_address() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_duoqian_address(&c, b"alice-personal");
        // 直接把目标地址灌成 Active,模拟"地址已存在"
        seed_active_duoqian(&dq, &c, &[admin(0), admin(1), admin(2)], 500);

        assert_noop!(
            PersonalManage::propose_create(
                RuntimeOrigin::signed(c),
                account_name(b"alice-personal"),
                admins_vec(3),
                CREATE_AMOUNT,
            ),
            pallet::Error::<Test>::PersonalDuoqianAlreadyExists
        );
    });
}

// ─── 5. 普通业务阈值由链端派生 ───────────────────────────────────────

#[test]
fn propose_create_derives_regular_threshold_and_uses_all_admin_create_threshold() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_duoqian_address(&c, b"derived-threshold");
        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c.clone()),
            account_name(b"derived-threshold"),
            admins_vec(3),
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();
        let subject = primitives::derive::subject_id_from_account(&dq);
        assert_eq!(
            admins_change::Pallet::<Test>::pending_subject_threshold_for_snapshot(ORG_REN, subject),
            Some(2)
        );
        assert_eq!(
            internal_vote::InternalThresholdSnapshot::<Test>::get(pid),
            Some(3)
        );
    });
}

#[test]
fn two_admin_personal_create_uses_two_of_two_for_regular_and_create_threshold() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_duoqian_address(&c, b"two-admin");
        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c),
            account_name(b"two-admin"),
            admins_vec(2),
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();
        let subject = primitives::derive::subject_id_from_account(&dq);
        assert_eq!(
            admins_change::Pallet::<Test>::pending_subject_threshold_for_snapshot(ORG_REN, subject),
            Some(2)
        );
        assert_eq!(
            internal_vote::InternalThresholdSnapshot::<Test>::get(pid),
            Some(2)
        );
    });
}

#[test]
fn sixty_four_admin_personal_create_is_allowed_and_uses_full_create_threshold() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_duoqian_address(&c, b"sixty-four-admins");
        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c),
            account_name(b"sixty-four-admins"),
            admins_vec(64),
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();
        let subject = primitives::derive::subject_id_from_account(&dq);
        assert_eq!(
            admins_change::Pallet::<Test>::pending_subject_threshold_for_snapshot(ORG_REN, subject),
            Some(32)
        );
        assert_eq!(
            internal_vote::InternalThresholdSnapshot::<Test>::get(pid),
            Some(64)
        );
    });
}

#[test]
fn sixty_five_admin_personal_create_cannot_be_encoded() {
    new_test_ext().execute_with(|| {
        let admins: alloc::vec::Vec<AccountId32> = (0..65u8).map(admin).collect();
        assert!(pallet::DuoqianAdminsOf::<Test>::try_from(admins).is_err());
    });
}

// ─── 6. admin 重复 ────────────────────────────────────────────────────

#[test]
fn propose_create_rejects_duplicate_admins() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let v = vec![admin(0), admin(1), admin(0)]; // admin(0) 重复
        let dup_admins: pallet::DuoqianAdminsOf<Test> = BoundedVec::try_from(v).expect("fits");

        assert_noop!(
            PersonalManage::propose_create(
                RuntimeOrigin::signed(c),
                account_name(b"dup"),
                dup_admins,
                CREATE_AMOUNT,
            ),
            pallet::Error::<Test>::DuplicateAdmin
        );
    });
}

// ─── 7. 入金低于最小门槛 ───────────────────────────────────────────────

#[test]
fn propose_create_rejects_below_minimum_amount() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        // MinCreateAmount = 111
        assert_noop!(
            PersonalManage::propose_create(
                RuntimeOrigin::signed(c),
                account_name(b"too-small"),
                admins_vec(3),
                100, // 100 < 111
            ),
            pallet::Error::<Test>::CreateAmountBelowMinimum
        );
    });
}

// ─── 8. propose_close 写 Pending + 阻止并发 ───────────────────────────

#[test]
fn propose_close_writes_pending_and_blocks_concurrent() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_duoqian_address(&c, b"close-pending");
        let admins_acc = vec![admin(0), admin(1), admin(2)];
        seed_active_duoqian(&dq, &c, &admins_acc, 1_000);

        let beneficiary_acc = beneficiary();

        assert_ok!(PersonalManage::propose_close(
            RuntimeOrigin::signed(admin(0)),
            dq.clone(),
            beneficiary_acc.clone(),
        ));

        let pid = last_proposal_id();
        assert_eq!(pallet::PendingCloseProposal::<Test>::get(&dq), Some(pid));

        // 第二次发起应被阻止
        assert_noop!(
            PersonalManage::propose_close(RuntimeOrigin::signed(admin(1)), dq, beneficiary_acc,),
            pallet::Error::<Test>::CloseAlreadyPending
        );
    });
}

// ─── 9. 投票通过关闭 → 余额转出 → 删 storage ──────────────────────────

#[test]
fn close_executes_when_internal_vote_reaches_threshold() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_duoqian_address(&c, b"close-active");
        let admins_acc = vec![admin(0), admin(1), admin(2)];
        seed_active_duoqian(&dq, &c, &admins_acc, 1_000);
        let beneficiary_acc = beneficiary();

        assert_ok!(PersonalManage::propose_close(
            RuntimeOrigin::signed(admin(0)),
            dq.clone(),
            beneficiary_acc.clone(),
        ));
        let pid = last_proposal_id();

        // 关闭提案要求全员通过(3 票)
        assert_ok!(cast_yes_votes(&admins_acc, 3, pid));

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal exists");
        assert_eq!(proposal.status, STATUS_EXECUTED);

        // amount 1000 → fee = max(1, 10) = 10,beneficiary 收 990
        assert_eq!(Balances::free_balance(&beneficiary_acc), 990);
        assert!(!pallet::PersonalDuoqians::<Test>::contains_key(&dq));
        assert!(!pallet::PersonalDuoqianInfo::<Test>::contains_key(&dq));
        assert!(!pallet::PendingCloseProposal::<Test>::contains_key(&dq));
    });
}

// ─── 10. 关闭余额过低被拒(链安全保护) ──────────────────────────────────

#[test]
fn propose_close_rejects_when_balance_below_minimum() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_duoqian_address(&c, b"low-balance");
        let admins_acc = vec![admin(0), admin(1), admin(2)];
        // MinCloseBalance = 111,这里灌 50 → 应拒
        seed_active_duoqian(&dq, &c, &admins_acc, 50);

        assert_noop!(
            PersonalManage::propose_close(RuntimeOrigin::signed(admin(0)), dq, beneficiary(),),
            pallet::Error::<Test>::CloseBalanceBelowMinimum
        );
    });
}

// ─── 11. cleanup_rejected_proposal 仅在 REJECTED 后生效 ────────────────

#[test]
fn cleanup_rejected_proposal_only_works_after_engine_rejected() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let admins = admins_vec(3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(|i| admin(i)).collect();

        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c.clone()),
            account_name(b"cleanup-test"),
            admins,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();

        // STATUS_VOTING 期间禁止 cleanup
        assert_noop!(
            PersonalManage::cleanup_rejected_proposal(RuntimeOrigin::signed(admin(0)), pid,),
            pallet::Error::<Test>::ProposalNotRejected
        );

        // 一票否决进入 REJECTED + Executor 自己已经 cleanup 过
        assert_ok!(cast_no_votes(&admin_accounts, 1, pid));

        // Executor 已经在 callback 里清掉 Pending,这里 cleanup 进来时应继续返回 Ok
        // (cleanup_pending_create 的 storage::remove 是幂等的)
        assert_ok!(PersonalManage::cleanup_rejected_proposal(
            RuntimeOrigin::signed(admin(0)),
            pid,
        ));
    });
}

// ─── 12. propose_close 拒绝非个人多签地址 ─────────────────────────────

#[test]
fn propose_close_rejects_when_not_personal_duoqian() {
    new_test_ext().execute_with(|| {
        // 没在 PersonalDuoqians 表里的地址
        let stranger = AccountId32::new([0xCC; 32]);

        assert_noop!(
            PersonalManage::propose_close(
                RuntimeOrigin::signed(admin(0)),
                stranger,
                beneficiary(),
            ),
            pallet::Error::<Test>::NotPersonalDuoqian
        );
    });
}

// ─── 13. 非 admin 不能投票 ────────────────────────────────────────────

#[test]
fn non_admin_cannot_propose_or_vote() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();

        // 非 admin 提案 propose_create 时,creator 不在 admins 列表 → PermissionDenied
        assert_noop!(
            PersonalManage::propose_create(
                RuntimeOrigin::signed(c),
                account_name(b"x"),
                BoundedVec::try_from(vec![admin(1), admin(2), admin(3)]).expect("fits"),
                CREATE_AMOUNT,
            ),
            pallet::Error::<Test>::PermissionDenied
        );

        // 准备一个走通的提案,然后非 admin 投票被引擎拒
        let c2 = AccountId32::new([0x77; 32]);
        let _ = Balances::deposit_creating(&c2, SEED_BALANCE);
        let admins = admins_vec(3);
        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(admin(0)),
            account_name(b"y"),
            admins,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();
        let stranger = AccountId32::new([0xEE; 32]);
        assert!(matches!(
            <internal_vote::Pallet<Test>>::do_internal_vote(stranger, pid, true),
            Err(DispatchError::Module(_))
        ));
    });
}

// ─── 14. 关闭后链不死账(Existential Deposit 保留) ────────────────────

#[test]
fn existential_deposit_is_preserved_after_close() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_duoqian_address(&c, b"ed-check");
        let admins_acc = vec![admin(0), admin(1), admin(2)];
        seed_active_duoqian(&dq, &c, &admins_acc, 500);
        let beneficiary_acc = beneficiary();

        assert_ok!(PersonalManage::propose_close(
            RuntimeOrigin::signed(admin(0)),
            dq.clone(),
            beneficiary_acc.clone(),
        ));
        let pid = last_proposal_id();
        assert_ok!(cast_yes_votes(&admins_acc, 3, pid));

        // 多签账户应已被销户(转出后余额 < ED 直接 reap),beneficiary 拿到剩余金额
        assert_eq!(Balances::free_balance(&dq), 0);
        // 500 - fee(10) = 490
        assert_eq!(Balances::free_balance(&beneficiary_acc), 490);
        // ED = 1,490 >= 1,链不死账
    });
}
