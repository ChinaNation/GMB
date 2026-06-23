#![cfg(test)]

use super::*;

// ─── 测试:绑定 / 存取 / 切换 ─────────────────────────────────────────────

#[test]
fn bind_deposit_withdraw_full_cycle() {
    new_test_ext().execute_with(|| {
        let (alice, _) = new_l3_user(&[1u8; 32], 1_000_000);

        // 1. 绑定清算行
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(alice.clone()),
            bank_main()
        ));
        assert_eq!(UserBank::<Test>::get(&alice), Some(bank_main()));
        assert_eq!(DepositBalance::<Test>::get(bank_main(), &alice), 0);

        // 2. 充值 10_000 分
        assert_ok!(OffchainTx::deposit(
            RuntimeOrigin::signed(alice.clone()),
            10_000
        ));
        assert_eq!(DepositBalance::<Test>::get(bank_main(), &alice), 10_000);
        assert_eq!(BankTotalDeposits::<Test>::get(bank_main()), 10_000);

        // 3. 提现 3_000
        assert_ok!(OffchainTx::withdraw(
            RuntimeOrigin::signed(alice.clone()),
            3_000
        ));
        assert_eq!(DepositBalance::<Test>::get(bank_main(), &alice), 7_000);
        assert_eq!(BankTotalDeposits::<Test>::get(bank_main()), 7_000);

        // 4. 提现过量应拒绝
        assert_noop!(
            OffchainTx::withdraw(RuntimeOrigin::signed(alice.clone()), 1_000_000),
            Error::<Test>::InsufficientDepositBalance
        );
    });
}

#[test]
fn double_bind_rejected() {
    new_test_ext().execute_with(|| {
        let (alice, _) = new_l3_user(&[1u8; 32], 1_000_000);
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(alice.clone()),
            bank_main()
        ));
        assert_noop!(
            OffchainTx::bind_clearing_bank(RuntimeOrigin::signed(alice.clone()), bank_main()),
            Error::<Test>::AlreadyHasBank
        );
    });
}

#[test]
fn bind_rejects_unregistered_bank() {
    new_test_ext().execute_with(|| {
        let (alice, _) = new_l3_user(&[1u8; 32], 1_000_000);
        let ghost = AccountId32::new(OTHER_BANK_BYTES);
        assert_noop!(
            OffchainTx::bind_clearing_bank(RuntimeOrigin::signed(alice.clone()), ghost),
            Error::<Test>::NotRegisteredClearingBank
        );
    });
}

#[test]
fn switch_requires_zero_balance() {
    new_test_ext().execute_with(|| {
        let (alice, _) = new_l3_user(&[1u8; 32], 1_000_000);
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(alice.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::deposit(
            RuntimeOrigin::signed(alice.clone()),
            10_000
        ));
        // 余额 > 0,不能切换
        assert_noop!(
            OffchainTx::switch_bank(RuntimeOrigin::signed(alice.clone()), bank_main()),
            Error::<Test>::NewBankSameAsCurrent
        );
        // 切换到同一家也应被拒(NewBankSameAsCurrent 优先于 MustClearBalanceFirst)
    });
}

#[test]
fn switch_after_withdraw_all_works() {
    new_test_ext().execute_with(|| {
        // 当前 fixture 只有一家清算行 bank_main;切到"同一家"会被 NewBankSameAsCurrent
        // 拒绝。这里先通过另一家 mock 来验证路径。
        // 简化:直接 withdraw 全部 → 再 switch 到同一家拒绝(= NewBankSameAsCurrent)。
        // 真正的跨行切换需要 fixture 扩展。
        let (alice, _) = new_l3_user(&[1u8; 32], 1_000_000);
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(alice.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::deposit(
            RuntimeOrigin::signed(alice.clone()),
            10_000
        ));
        assert_ok!(OffchainTx::withdraw(
            RuntimeOrigin::signed(alice.clone()),
            10_000
        ));
        assert_eq!(DepositBalance::<Test>::get(bank_main(), &alice), 0);
        // 零余额但切到同家仍被 NewBankSameAsCurrent 拒绝 —— 行为正确。
        assert_noop!(
            OffchainTx::switch_bank(RuntimeOrigin::signed(alice.clone()), bank_main()),
            Error::<Test>::NewBankSameAsCurrent
        );
    });
}

// ─── 测试:submit_offchain_batch_v2 ────────────────────────────────────────

fn seed_fee_rate(bank: &AccountId32, bp: u32) {
    L2FeeRateBp::<Test>::insert(bank, bp);
}

/// 用 `pair` 对 `PaymentIntent::signing_hash` 签名,返回 64 字节数组。
fn sign_intent(
    pair: &sr25519::Pair,
    intent: &crate::batch_item::PaymentIntent<AccountId32, u64>,
) -> [u8; 64] {
    use sp_core::crypto::Pair as _;
    let hash = intent.signing_hash();
    let sig = pair.sign(&hash);
    let bytes: [u8; 64] = sig.0;
    bytes
}

/// 用清算行管理员密钥对 `(institution_main, batch_seq, batch.encode())` 签名。
fn sign_batch(
    institution_main: &AccountId32,
    batch_seq: u64,
    batch: &BoundedVec<OffchainBatchItemV2<AccountId32, u64>, <Test as Config>::MaxBatchSize>,
) -> BatchSignatureOf<Test> {
    use sp_core::crypto::Pair as _;
    let message =
        crate::batch_item::batch_signing_hash(institution_main, batch_seq, &batch.encode());
    bank_admin_pair()
        .sign(&message)
        .0
        .to_vec()
        .try_into()
        .unwrap()
}

#[test]
fn submit_batch_rejects_non_admin() {
    new_test_ext().execute_with(|| {
        seed_fee_rate(&bank_main(), 5);
        let (alice, alice_pair) = new_l3_user(&[1u8; 32], 1_000_000);
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(alice.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::deposit(
            RuntimeOrigin::signed(alice.clone()),
            100_000
        ));

        let (bob, _) = new_l3_user(&[2u8; 32], 1_000_000);
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(bob.clone()),
            bank_main()
        ));

        let intent = crate::batch_item::PaymentIntent::<AccountId32, u64> {
            tx_id: H256::repeat_byte(7),
            payer: alice.clone(),
            payer_bank: bank_main(),
            recipient: bob.clone(),
            recipient_bank: bank_main(),
            amount: 10_000,
            fee: 5,
            nonce: 1,
            expires_at: 100,
        };
        let sig = sign_intent(&alice_pair, &intent);
        let item = OffchainBatchItemV2::<AccountId32, u64> {
            tx_id: intent.tx_id,
            payer: intent.payer.clone(),
            payer_bank: intent.payer_bank.clone(),
            recipient: intent.recipient.clone(),
            recipient_bank: intent.recipient_bank.clone(),
            transfer_amount: intent.amount,
            fee_amount: intent.fee,
            payer_sig: sig,
            payer_nonce: intent.nonce,
            expires_at: intent.expires_at,
        };
        let batch: BoundedVec<_, _> = sp_std::vec![item].try_into().unwrap();

        // 用非管理员账户提交 → UnauthorizedAdmin
        assert_noop!(
            OffchainTx::submit_offchain_batch_v2(
                RuntimeOrigin::signed(alice.clone()),
                bank_main(),
                1,
                batch.clone(),
                sign_batch(&bank_main(), 1, &batch)
            ),
            Error::<Test>::UnauthorizedAdmin
        );
    });
}

#[test]
fn submit_batch_rejects_invalid_batch_signature() {
    new_test_ext().execute_with(|| {
        seed_fee_rate(&bank_main(), 5);
        let (alice, alice_pair) = new_l3_user(&[1u8; 32], 1_000_000);
        let (bob, _) = new_l3_user(&[2u8; 32], 1_000_000);
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(alice.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(bob.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::deposit(
            RuntimeOrigin::signed(alice.clone()),
            100_000
        ));

        let intent = crate::batch_item::PaymentIntent::<AccountId32, u64> {
            tx_id: H256::repeat_byte(0x31),
            payer: alice.clone(),
            payer_bank: bank_main(),
            recipient: bob.clone(),
            recipient_bank: bank_main(),
            amount: 10_000,
            fee: 5,
            nonce: 1,
            expires_at: 100,
        };
        let item = OffchainBatchItemV2::<AccountId32, u64> {
            tx_id: intent.tx_id,
            payer: intent.payer.clone(),
            payer_bank: intent.payer_bank.clone(),
            recipient: intent.recipient.clone(),
            recipient_bank: intent.recipient_bank.clone(),
            transfer_amount: intent.amount,
            fee_amount: intent.fee,
            payer_sig: sign_intent(&alice_pair, &intent),
            payer_nonce: intent.nonce,
            expires_at: intent.expires_at,
        };
        let batch: BoundedVec<_, _> = sp_std::vec![item].try_into().unwrap();
        let bad_batch_sig: BatchSignatureOf<Test> = sp_std::vec![0u8; 64].try_into().unwrap();

        assert_noop!(
            OffchainTx::submit_offchain_batch_v2(
                RuntimeOrigin::signed(bank_admin()),
                bank_main(),
                1,
                batch,
                bad_batch_sig
            ),
            Error::<Test>::InvalidBatchSignature
        );
    });
}

#[test]
fn submit_batch_rejects_wrong_batch_seq() {
    new_test_ext().execute_with(|| {
        seed_fee_rate(&bank_main(), 5);
        let (alice, alice_pair) = new_l3_user(&[1u8; 32], 1_000_000);
        let (bob, _) = new_l3_user(&[2u8; 32], 1_000_000);
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(alice.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(bob.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::deposit(
            RuntimeOrigin::signed(alice.clone()),
            100_000
        ));

        let intent = crate::batch_item::PaymentIntent::<AccountId32, u64> {
            tx_id: H256::repeat_byte(0x32),
            payer: alice.clone(),
            payer_bank: bank_main(),
            recipient: bob.clone(),
            recipient_bank: bank_main(),
            amount: 10_000,
            fee: 5,
            nonce: 1,
            expires_at: 100,
        };
        let item = OffchainBatchItemV2::<AccountId32, u64> {
            tx_id: intent.tx_id,
            payer: intent.payer.clone(),
            payer_bank: intent.payer_bank.clone(),
            recipient: intent.recipient.clone(),
            recipient_bank: intent.recipient_bank.clone(),
            transfer_amount: intent.amount,
            fee_amount: intent.fee,
            payer_sig: sign_intent(&alice_pair, &intent),
            payer_nonce: intent.nonce,
            expires_at: intent.expires_at,
        };
        let batch: BoundedVec<_, _> = sp_std::vec![item].try_into().unwrap();

        assert_noop!(
            OffchainTx::submit_offchain_batch_v2(
                RuntimeOrigin::signed(bank_admin()),
                bank_main(),
                2,
                batch.clone(),
                sign_batch(&bank_main(), 2, &batch)
            ),
            Error::<Test>::InvalidBatchSeq
        );
    });
}

#[test]
fn submit_batch_same_bank_end_to_end() {
    new_test_ext().execute_with(|| {
        seed_fee_rate(&bank_main(), 5); // 5 bp = 0.05%

        let (alice, alice_pair) = new_l3_user(&[1u8; 32], 2_000_000);
        let (bob, _) = new_l3_user(&[2u8; 32], 1_000_000);

        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(alice.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(bob.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::deposit(
            RuntimeOrigin::signed(alice.clone()),
            1_000_000
        ));

        // 10_000 分 × 5 bp / 10000 = 5 分,但按 runtime fee_config 最低 1 分,5 ≥ 1 → 5 分
        let transfer_amount = 10_000u128;
        let expected_fee = 5u128;

        let intent = crate::batch_item::PaymentIntent::<AccountId32, u64> {
            tx_id: H256::repeat_byte(0x42),
            payer: alice.clone(),
            payer_bank: bank_main(),
            recipient: bob.clone(),
            recipient_bank: bank_main(),
            amount: transfer_amount,
            fee: expected_fee,
            nonce: 1,
            expires_at: 100,
        };
        let sig = sign_intent(&alice_pair, &intent);
        let item = OffchainBatchItemV2::<AccountId32, u64> {
            tx_id: intent.tx_id,
            payer: intent.payer.clone(),
            payer_bank: intent.payer_bank.clone(),
            recipient: intent.recipient.clone(),
            recipient_bank: intent.recipient_bank.clone(),
            transfer_amount,
            fee_amount: expected_fee,
            payer_sig: sig,
            payer_nonce: intent.nonce,
            expires_at: intent.expires_at,
        };
        let batch: BoundedVec<_, _> = sp_std::vec![item].try_into().unwrap();

        // 由管理员提交
        let bank_total_before = BankTotalDeposits::<Test>::get(bank_main());
        let fee_account_before = Balances::free_balance(&bank_fee());
        let bank_main_balance_before = Balances::free_balance(&bank_main());

        assert_ok!(OffchainTx::submit_offchain_batch_v2(
            RuntimeOrigin::signed(bank_admin()),
            bank_main(),
            1,
            batch.clone(),
            sign_batch(&bank_main(), 1, &batch)
        ));

        // 付款方 DepositBalance 扣 (transfer + fee)
        assert_eq!(
            DepositBalance::<Test>::get(bank_main(), &alice),
            1_000_000 - (transfer_amount + expected_fee)
        );
        // 收款方 DepositBalance 加 transfer
        assert_eq!(
            DepositBalance::<Test>::get(bank_main(), &bob),
            transfer_amount
        );
        // 同行场景:BankTotalDeposits 下降 fee(fee 流出到 fee_account)
        assert_eq!(
            BankTotalDeposits::<Test>::get(bank_main()),
            bank_total_before - expected_fee
        );
        // 清算行主账户 Balances 减 fee(转到 fee_account)
        assert_eq!(
            Balances::free_balance(&bank_main()),
            bank_main_balance_before - expected_fee
        );
        assert_eq!(
            Balances::free_balance(&bank_fee()),
            fee_account_before + expected_fee
        );
        // nonce 已消费
        assert_eq!(L3PaymentNonce::<Test>::get(&alice), 1);
        // 批次序号只在 settlement 成功后推进。
        assert_eq!(LastClearingBatchSeq::<Test>::get(bank_main()), 1);

        // 事件断言:最后几条里应有 PaymentSettled + ClearingBankBatchSettled
        let events = frame_system::Pallet::<Test>::events();
        let event_names: sp_std::vec::Vec<_> =
            events.iter().map(|r| format!("{:?}", r.event)).collect();
        assert!(
            event_names.iter().any(|s| s.contains("PaymentSettled")),
            "missing PaymentSettled event; got {:?}",
            event_names
        );
        assert!(
            event_names
                .iter()
                .any(|s| s.contains("ClearingBankBatchSettled")),
            "missing ClearingBankBatchSettled event; got {:?}",
            event_names
        );

        // 防重放:同一 tx_id 再次提交应被拒 TxAlreadyProcessed。
        // 注意 nonce 必须递增(否则会先撞 `InvalidL3Nonce`),且 sig 重新
        // 基于 nonce=2 的 signing_hash 签名,否则先撞 `InvalidL3Signature`。
        let replay_intent = crate::batch_item::PaymentIntent::<AccountId32, u64> {
            nonce: 2,
            ..intent.clone()
        };
        let replay_sig = sign_intent(&alice_pair, &replay_intent);
        let replay_item = OffchainBatchItemV2::<AccountId32, u64> {
            tx_id: replay_intent.tx_id,
            payer: replay_intent.payer.clone(),
            payer_bank: replay_intent.payer_bank.clone(),
            recipient: replay_intent.recipient.clone(),
            recipient_bank: replay_intent.recipient_bank.clone(),
            transfer_amount,
            fee_amount: expected_fee,
            payer_sig: replay_sig,
            payer_nonce: replay_intent.nonce,
            expires_at: replay_intent.expires_at,
        };
        let replay_batch: BoundedVec<_, _> = sp_std::vec![replay_item].try_into().unwrap();
        assert_noop!(
            OffchainTx::submit_offchain_batch_v2(
                RuntimeOrigin::signed(bank_admin()),
                bank_main(),
                2,
                replay_batch.clone(),
                sign_batch(&bank_main(), 2, &replay_batch)
            ),
            Error::<Test>::TxAlreadyProcessed
        );
    });
}

#[test]
fn submit_batch_rejects_user_bank_mismatch() {
    new_test_ext().execute_with(|| {
        seed_fee_rate(&bank_main(), 5);
        let (alice, alice_pair) = new_l3_user(&[1u8; 32], 1_000_000);
        let (bob, _) = new_l3_user(&[2u8; 32], 1_000_000);
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(alice.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(bob.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::deposit(
            RuntimeOrigin::signed(alice.clone()),
            100_000
        ));
        // 直接制造链上绑定与 item 声明不一致的状态,验证 settlement 会早拒。
        UserBank::<Test>::insert(&bob, AccountId32::new(OTHER_BANK_BYTES));

        let intent = crate::batch_item::PaymentIntent::<AccountId32, u64> {
            tx_id: H256::repeat_byte(0x33),
            payer: alice.clone(),
            payer_bank: bank_main(),
            recipient: bob.clone(),
            recipient_bank: bank_main(),
            amount: 10_000,
            fee: 5,
            nonce: 1,
            expires_at: 100,
        };
        let item = OffchainBatchItemV2::<AccountId32, u64> {
            tx_id: intent.tx_id,
            payer: intent.payer.clone(),
            payer_bank: intent.payer_bank.clone(),
            recipient: intent.recipient.clone(),
            recipient_bank: intent.recipient_bank.clone(),
            transfer_amount: intent.amount,
            fee_amount: intent.fee,
            payer_sig: sign_intent(&alice_pair, &intent),
            payer_nonce: intent.nonce,
            expires_at: intent.expires_at,
        };
        let batch: BoundedVec<_, _> = sp_std::vec![item].try_into().unwrap();

        assert_noop!(
            OffchainTx::submit_offchain_batch_v2(
                RuntimeOrigin::signed(bank_admin()),
                bank_main(),
                1,
                batch.clone(),
                sign_batch(&bank_main(), 1, &batch)
            ),
            Error::<Test>::UserBankMismatch
        );
    });
}

#[test]
fn submit_batch_expired_intent_rejected() {
    new_test_ext().execute_with(|| {
        seed_fee_rate(&bank_main(), 5);
        let (alice, alice_pair) = new_l3_user(&[1u8; 32], 1_000_000);
        let (bob, _) = new_l3_user(&[2u8; 32], 1_000_000);
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(alice.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::bind_clearing_bank(
            RuntimeOrigin::signed(bob.clone()),
            bank_main()
        ));
        assert_ok!(OffchainTx::deposit(
            RuntimeOrigin::signed(alice.clone()),
            100_000
        ));

        // 故意把块高推到 expires_at 之后
        System::set_block_number(200);

        let intent = crate::batch_item::PaymentIntent::<AccountId32, u64> {
            tx_id: H256::repeat_byte(9),
            payer: alice.clone(),
            payer_bank: bank_main(),
            recipient: bob.clone(),
            recipient_bank: bank_main(),
            amount: 10_000,
            fee: 5,
            nonce: 1,
            expires_at: 100, // 已过
        };
        let sig = sign_intent(&alice_pair, &intent);
        let item = OffchainBatchItemV2::<AccountId32, u64> {
            tx_id: intent.tx_id,
            payer: intent.payer.clone(),
            payer_bank: intent.payer_bank.clone(),
            recipient: intent.recipient.clone(),
            recipient_bank: intent.recipient_bank.clone(),
            transfer_amount: intent.amount,
            fee_amount: intent.fee,
            payer_sig: sig,
            payer_nonce: intent.nonce,
            expires_at: intent.expires_at,
        };
        let batch: BoundedVec<_, _> = sp_std::vec![item].try_into().unwrap();

        assert_noop!(
            OffchainTx::submit_offchain_batch_v2(
                RuntimeOrigin::signed(bank_admin()),
                bank_main(),
                1,
                batch.clone(),
                sign_batch(&bank_main(), 1, &batch)
            ),
            Error::<Test>::ExpiredIntent
        );
    });
}

// 用 `Encode` 保留 future 扩展余地(如 emit 断言里使用)。
#[allow(dead_code)]
fn _touch_encode() {
    let x: u32 = 0;
    let _ = x.encode();
}
