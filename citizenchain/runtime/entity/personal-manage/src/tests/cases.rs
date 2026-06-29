use super::*;
use codec::Encode;
use frame_support::{assert_noop, assert_ok, traits::Currency, BoundedVec};
use sp_runtime::DispatchError;
use votingengine::{STATUS_EXECUTED, STATUS_EXECUTION_FAILED, STATUS_REJECTED, STATUS_VOTING};

const CREATE_AMOUNT: Balance = 1_000;
const CREATE_FEE: Balance = 10; // calculate_onchain_fee(1000) = max(1000*0.001, 10) = 10
const SEED_BALANCE: Balance = 5_000;

fn setup_creator_balance() -> AccountId32 {
    let c = creator();
    let _ = Balances::deposit_creating(&c, SEED_BALANCE);
    c
}

fn proposed_account(creator: &AccountId32, name: &[u8]) -> AccountId32 {
    PersonalManage::derive_personal_account(creator, name).expect("derive should succeed")
}

fn create_rejected_event_count(pid: u64) -> usize {
    System::events()
        .iter()
        .filter(|record| {
            matches!(
                &record.event,
                RuntimeEvent::PersonalManage(pallet::Event::PersonalCreateRejected {
                    proposal_id,
                    ..
                }) if *proposal_id == pid
            )
        })
        .count()
}

fn create_failed_event_count(pid: u64) -> usize {
    System::events()
        .iter()
        .filter(|record| {
            matches!(
                &record.event,
                RuntimeEvent::PersonalManage(pallet::Event::CreateExecutionFailed {
                    proposal_id,
                    ..
                }) if *proposal_id == pid
            )
        })
        .count()
}

fn close_failed_event_count(pid: u64) -> usize {
    System::events()
        .iter()
        .filter(|record| {
            matches!(
                &record.event,
                RuntimeEvent::PersonalManage(pallet::Event::CloseExecutionFailed {
                    proposal_id,
                    ..
                }) if *proposal_id == pid
            )
        })
        .count()
}

fn overwrite_create_proposal_fee(pid: u64, fee: Balance) {
    let mut action = pallet::PendingPersonalCreate::<Test>::get(pid).expect("pending action");
    action.fee = fee;
    let mut data = alloc::vec::Vec::from(crate::MODULE_TAG);
    data.push(crate::ACTION_CREATE);
    data.extend_from_slice(&action.encode());
    let bounded: BoundedVec<u8, <Test as votingengine::Config>::MaxProposalDataLen> =
        BoundedVec::try_from(data).expect("proposal data fits");
    votingengine::ProposalData::<Test>::insert(pid, bounded);
}

// ─── 1. propose_create:写 Pending + reserve fee + 发事件 ─────────────

#[test]
fn propose_create_writes_pending_and_reserves_fee() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let admins = admins_vec(3); // admin(0)..admin(2)
        let name = account_name(b"alice-personal");
        let dq = proposed_account(&c, b"alice-personal");

        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c.clone()),
            name.clone(),
            admins.clone(),
            2,
            CREATE_AMOUNT,
        ));

        let pid = last_proposal_id();

        assert!(pallet::PendingPersonalCreate::<Test>::contains_key(pid));
        assert!(pallet::PersonalAccounts::<Test>::contains_key(&dq));
        let pending_action = pallet::PendingPersonalCreate::<Test>::get(pid).unwrap();
        assert_eq!(pending_action.fee, CREATE_FEE);
        let pending_account = pallet::PersonalAccounts::<Test>::get(&dq).unwrap();
        assert_eq!(pending_account.status, types::PersonalStatus::Pending);
        assert_eq!(pending_account.account_name, name);
        let account = dq.clone();
        assert_eq!(
            internal_vote::PendingDynamicThresholds::<Test>::get(PMUL, account),
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
        let dq = proposed_account(&c, b"alice-personal");

        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c.clone()),
            name,
            admins,
            2,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();

        // 创建提案要求"全员通过"——投票引擎 threshold = admins.len() = 3
        assert_ok!(cast_yes_votes(&admin_accounts[1..], 2, pid));

        // 提案进入 EXECUTED
        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal exists");
        assert_eq!(proposal.status, STATUS_EXECUTED);

        // 多签账户激活,资金到位,Pending 已清
        let dq_state = pallet::PersonalAccounts::<Test>::get(&dq).expect("active multisig");
        assert_eq!(dq_state.status, types::PersonalStatus::Active);
        let account = dq.clone();
        assert_eq!(
            internal_vote::ActiveDynamicThresholds::<Test>::get(PMUL, account),
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
        let dq = proposed_account(&c, b"alice-personal");

        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c.clone()),
            name,
            admins,
            2,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();

        // 一票否决:全员通过制度下,只要有人反对就立刻进 STATUS_REJECTED
        assert_ok!(cast_no_votes(&admin_accounts[1..], 1, pid));

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal exists");
        assert_eq!(proposal.status, STATUS_REJECTED);

        // 拒绝路径下 Executor 应已 cleanup,reserve 释放,storage 清空
        assert_eq!(Balances::reserved_balance(&c), 0);
        assert!(!pallet::PersonalAccounts::<Test>::contains_key(&dq));
        assert!(!pallet::PendingPersonalCreate::<Test>::contains_key(pid));
        assert_eq!(create_rejected_event_count(pid), 1);
    });
}

// ─── 4. 重复账户被拒绝 ────────────────────────────────────────────────

#[test]
fn propose_create_rejects_duplicate_personal_account() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_account(&c, b"alice-personal");
        // 直接把目标地址灌成 Active,模拟"地址已存在"
        seed_active_multisig(&dq, &c, &[admin(0), admin(1), admin(2)], 500);

        assert_noop!(
            PersonalManage::propose_create(
                RuntimeOrigin::signed(c),
                account_name(b"alice-personal"),
                admins_vec(3),
                2,
                CREATE_AMOUNT,
            ),
            pallet::Error::<Test>::PersonalAlreadyExists
        );
    });
}

// ─── 5. 普通业务阈值由用户传入，投票引擎统一校验保存 ───────────────────

#[test]
fn propose_create_stores_regular_threshold_and_uses_all_admin_create_threshold() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_account(&c, b"derived-threshold");
        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c.clone()),
            account_name(b"derived-threshold"),
            admins_vec(3),
            2,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();
        let account = dq.clone();
        assert_eq!(
            internal_vote::PendingDynamicThresholds::<Test>::get(PMUL, account),
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
        let dq = proposed_account(&c, b"two-admin");
        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c),
            account_name(b"two-admin"),
            admins_vec(2),
            2,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();
        let account = dq.clone();
        assert_eq!(
            internal_vote::PendingDynamicThresholds::<Test>::get(PMUL, account),
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
        let dq = proposed_account(&c, b"sixty-four-admins");
        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c),
            account_name(b"sixty-four-admins"),
            admins_vec(64),
            33,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();
        let account = dq.clone();
        assert_eq!(
            internal_vote::PendingDynamicThresholds::<Test>::get(PMUL, account),
            Some(33)
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
        assert!(pallet::AdminsOf::<Test>::try_from(admins).is_err());
    });
}

// ─── 6. admin 重复 ────────────────────────────────────────────────────

#[test]
fn propose_create_rejects_duplicate_admins() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let v = vec![admin(0), admin(1), admin(0)]; // admin(0) 重复
        let dup_admins: pallet::AdminsOf<Test> = BoundedVec::try_from(v).expect("fits");

        assert_noop!(
            PersonalManage::propose_create(
                RuntimeOrigin::signed(c),
                account_name(b"dup"),
                dup_admins,
                2,
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
                2,
                100, // 100 < 111
            ),
            pallet::Error::<Test>::CreateAmountBelowMinimum
        );
    });
}

#[test]
fn propose_create_rejects_reserved_and_protected_accounts() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let protected = proposed_account(&c, b"protected-target");
        set_protected_account(Some(protected));

        assert_noop!(
            PersonalManage::propose_create(
                RuntimeOrigin::signed(c.clone()),
                account_name(b"protected-target"),
                admins_vec(3),
                2,
                CREATE_AMOUNT,
            ),
            pallet::Error::<Test>::ProtectedSource
        );

        set_protected_account(Some(c.clone()));
        assert_noop!(
            PersonalManage::propose_create(
                RuntimeOrigin::signed(c),
                account_name(b"protected-creator"),
                admins_vec(3),
                2,
                CREATE_AMOUNT,
            ),
            pallet::Error::<Test>::ProtectedSource
        );
    });
}

// ─── 8. propose_close 写 Pending + 阻止并发 ───────────────────────────

#[test]
fn propose_close_writes_pending_and_blocks_concurrent() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_account(&c, b"close-pending");
        let admins_acc = vec![admin(0), admin(1), admin(2)];
        seed_active_multisig(&dq, &c, &admins_acc, 1_000);

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
        let dq = proposed_account(&c, b"close-active");
        let admins_acc = vec![admin(0), admin(1), admin(2)];
        seed_active_multisig(&dq, &c, &admins_acc, 1_000);
        let beneficiary_acc = beneficiary();

        assert_ok!(PersonalManage::propose_close(
            RuntimeOrigin::signed(admin(0)),
            dq.clone(),
            beneficiary_acc.clone(),
        ));
        let pid = last_proposal_id();

        // 关闭提案要求全员通过(3 票)
        assert_ok!(cast_yes_votes(&admins_acc[1..], 2, pid));

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal exists");
        assert_eq!(proposal.status, STATUS_EXECUTED);

        // amount 1000 → fee = max(1, 10) = 10,beneficiary 收 990
        assert_eq!(Balances::free_balance(&beneficiary_acc), 990);
        assert_eq!(Balances::free_balance(&dq), 0);
        let account = dq.clone();
        assert!(!pallet::PersonalAccounts::<Test>::contains_key(&dq));
        assert!(!pallet::PendingCloseProposal::<Test>::contains_key(&dq));
        assert!(personal_admins::AdminAccounts::<Test>::get(account.clone()).is_none());
        assert!(internal_vote::ActiveDynamicThresholds::<Test>::get(PMUL, account).is_none());

        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c),
            account_name(b"close-active"),
            admins_vec(3),
            2,
            CREATE_AMOUNT,
        ));
    });
}

// ─── 10. 关闭余额过低被拒(链安全保护) ──────────────────────────────────

#[test]
fn propose_close_rejects_when_balance_below_minimum() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_account(&c, b"low-balance");
        let admins_acc = vec![admin(0), admin(1), admin(2)];
        // MinCloseBalance = 111,这里灌 50 → 应拒
        seed_active_multisig(&dq, &c, &admins_acc, 50);

        assert_noop!(
            PersonalManage::propose_close(RuntimeOrigin::signed(admin(0)), dq, beneficiary(),),
            pallet::Error::<Test>::CloseBalanceBelowMinimum
        );
    });
}

#[test]
fn propose_close_rejects_reserved_and_protected_beneficiary() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_account(&c, b"close-protected");
        let admins_acc = vec![admin(0), admin(1), admin(2)];
        seed_active_multisig(&dq, &c, &admins_acc, 1_000);

        assert_noop!(
            PersonalManage::propose_close(
                RuntimeOrigin::signed(admin(0)),
                dq.clone(),
                AccountId32::new([0xAA; 32]),
            ),
            pallet::Error::<Test>::InvalidBeneficiary
        );

        let protected = beneficiary();
        set_protected_account(Some(protected.clone()));
        assert_noop!(
            PersonalManage::propose_close(RuntimeOrigin::signed(admin(0)), dq, protected,),
            pallet::Error::<Test>::InvalidBeneficiary
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
            2,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();

        // STATUS_VOTING 期间禁止 cleanup
        assert_noop!(
            PersonalManage::cleanup_rejected_proposal(RuntimeOrigin::signed(admin(0)), pid,),
            pallet::Error::<Test>::ProposalNotRejected
        );

        // 一票否决进入 REJECTED + Executor 自己已经 cleanup 过
        assert_ok!(cast_no_votes(&admin_accounts[1..], 1, pid));
        assert_eq!(create_rejected_event_count(pid), 1);

        // Executor 已经在 callback 里清掉 Pending,这里 cleanup 进来时应继续返回 Ok,
        // 但不能重复发 PersonalCreateRejected。
        assert_ok!(PersonalManage::cleanup_rejected_proposal(
            RuntimeOrigin::signed(admin(0)),
            pid,
        ));
        assert_eq!(create_rejected_event_count(pid), 1);
    });
}

// ─── 12. 创建执行失败 → 终态清理 reserve + pending + 失败事件 ───────────

#[test]
fn create_execution_failed_terminal_cleans_pending_and_emits_once() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let admins = admins_vec(3);
        let admin_accounts: alloc::vec::Vec<AccountId32> = (0..3u8).map(admin).collect();
        let dq = proposed_account(&c, b"exec-fail-create");

        assert_ok!(PersonalManage::propose_create(
            RuntimeOrigin::signed(c.clone()),
            account_name(b"exec-fail-create"),
            admins,
            2,
            CREATE_AMOUNT,
        ));
        let pid = last_proposal_id();

        // 中文注释:模拟 fee_policy 在投票期变更后 ProposalData 中记录的快照费更高。
        // execute_create 只能按快照释放 reserve,因此会进入执行失败终态;
        // 终态回调随后必须按同一快照清理 Pending 和 reserve。
        overwrite_create_proposal_fee(pid, CREATE_FEE + 1);

        assert_ok!(cast_yes_votes(&admin_accounts[1..], 2, pid));

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal exists");
        assert_eq!(proposal.status, STATUS_EXECUTION_FAILED);
        assert_eq!(Balances::reserved_balance(&c), 0);
        assert!(!pallet::PendingPersonalCreate::<Test>::contains_key(pid));
        assert!(!pallet::PersonalAccounts::<Test>::contains_key(&dq));
        assert_eq!(create_failed_event_count(pid), 1);
    });
}

// ─── 13. 关闭执行失败 → 只清 PendingCloseProposal + 失败事件 ───────────

#[test]
fn close_execution_failed_terminal_keeps_account_and_clears_pending() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_account(&c, b"exec-fail-close");
        let admins_acc = vec![admin(0), admin(1), admin(2)];
        seed_active_multisig(&dq, &c, &admins_acc, 1_000);

        assert_ok!(PersonalManage::propose_close(
            RuntimeOrigin::signed(admin(0)),
            dq.clone(),
            beneficiary(),
        ));
        let pid = last_proposal_id();
        set_institution_can_spend(false);

        assert_ok!(cast_yes_votes(&admins_acc[1..], 2, pid));

        let proposal = votingengine::Pallet::<Test>::proposals(pid).expect("proposal exists");
        assert_eq!(proposal.status, STATUS_EXECUTION_FAILED);
        assert!(pallet::PersonalAccounts::<Test>::contains_key(&dq));
        assert!(!pallet::PendingCloseProposal::<Test>::contains_key(&dq));
        assert_eq!(close_failed_event_count(pid), 1);
    });
}

// ─── 14. propose_close 拒绝非个人多签账户 ─────────────────────────────

#[test]
fn propose_close_rejects_when_not_personal_account() {
    new_test_ext().execute_with(|| {
        // 没在 PersonalAccounts 表里的地址
        let stranger = AccountId32::new([0xCC; 32]);

        assert_noop!(
            PersonalManage::propose_close(
                RuntimeOrigin::signed(admin(0)),
                stranger,
                beneficiary(),
            ),
            pallet::Error::<Test>::NotPersonalAccount
        );
    });
}

// ─── 15. 非 admin 不能投票 ────────────────────────────────────────────

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
                2,
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
            2,
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

// ─── 17. 关闭后链不死账(Existential Deposit 保留) ────────────────────

#[test]
fn existential_deposit_is_preserved_after_close() {
    new_test_ext().execute_with(|| {
        let c = setup_creator_balance();
        let dq = proposed_account(&c, b"ed-check");
        let admins_acc = vec![admin(0), admin(1), admin(2)];
        seed_active_multisig(&dq, &c, &admins_acc, 500);
        let beneficiary_acc = beneficiary();

        assert_ok!(PersonalManage::propose_close(
            RuntimeOrigin::signed(admin(0)),
            dq.clone(),
            beneficiary_acc.clone(),
        ));
        let pid = last_proposal_id();
        assert_ok!(cast_yes_votes(&admins_acc[1..], 2, pid));

        // 多签账户应已被销户(转出后余额 < ED 直接 reap),beneficiary 拿到剩余金额
        assert_eq!(Balances::free_balance(&dq), 0);
        // 500 - fee(10) = 490
        assert_eq!(Balances::free_balance(&beneficiary_acc), 490);
        // ED = 1,490 >= 1,链不死账
    });
}
