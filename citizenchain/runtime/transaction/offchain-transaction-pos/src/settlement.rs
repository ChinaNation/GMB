//! 扫码支付清算体系:清算行批次上链 settlement 路径。
//!
//! 中文注释:
//! - 同行清算(payer_bank == recipient_bank):仅 `DepositBalance` 轧差 + 手续费从主账户到费用账户
//! - 跨行清算(payer_bank != recipient_bank):
//!     主账户 → 收款方主账户(本金)
//!   + 主账户 → 收款方费用账户(fee)
//!   + 双方 `DepositBalance` / `BankTotalDeposits` 同步
//! - 每条 `OffchainBatchItemV2` 必经:
//!     1. L3 签名验证(`sr25519_verify` 对 `PaymentIntent::signing_hash`)
//!     2. nonce 单调递增(`nonce::consume_nonce`)
//!     3. 费率正确性(按 **收款方** 清算行 `L2FeeRateBp`,最低 1 分)
//!     4. 偿付自动保护(`solvency::ensure_can_debit`)
//!     5. 防重放(`ProcessedOffchainTx` 不命中)
//! - 手续费**全部归收款方清算行的费用账户**,无省储行分成。
//!
//! Step 2(2026-04-27, ADR-007)修订:**收款方主导清算**模型。
//! - `submit_offchain_batch_v2` 的 `institution_main` 现在 = 收款方清算行主账户
//! - 同一批次所有 item 的 `recipient_bank` 必须 == `institution_main`(原为 `payer_bank`)
//! - 提交者 = 收款方清算行的某个激活管理员
//! - 链上 gas 由 `RuntimeFeePayerExtractor` 从 `fee_account_of(institution_main)` 扣
//!
//! 节点 packer 收齐多笔 → 提交 `submit_offchain_batch_v2` 走到这里。

use codec::{Decode, Encode};
use frame_support::{
    ensure,
    traits::{Currency, ExistenceRequirement::KeepAlive},
};
use sp_core::sr25519::{Public as Sr25519Public, Signature as Sr25519Signature};
use sp_io::crypto::sr25519_verify;
use sp_runtime::{traits::SaturatedConversion, DispatchResult};

use crate::batch_item::OffchainBatchItemV2;
use crate::{
    bank_check::{self, SfidAccountQuery},
    fee_config, nonce, solvency, BankTotalDeposits, Config, DepositBalance, Error, Event, Pallet,
    ProcessedOffchainTx, ProcessedOffchainTxAt,
};
use frame_system::pallet_prelude::BlockNumberFor;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// `OFFCHAIN_MIN_FEE_FEN`:单笔最低手续费(分),与 `primitives::core_const::OFFCHAIN_MIN_FEE` 对齐。
pub const MIN_FEE_FEN: u128 = 1;

/// 计算本笔按收款方清算行费率应收的手续费(分),最低 1 分。
fn calc_fee(transfer_amount: u128, rate_bp: u32) -> Result<u128, &'static str> {
    let numerator = transfer_amount
        .checked_mul(rate_bp as u128)
        .ok_or("fee overflow")?;
    let quotient = numerator / 10_000;
    let remainder = numerator % 10_000;
    let rounded = if remainder >= 5_000 {
        quotient + 1
    } else {
        quotient
    };
    Ok(core::cmp::max(rounded, MIN_FEE_FEN))
}

/// 清算行批次上链的完整执行。
///
/// Step 2(2026-04-27, ADR-007)修订:**收款方主导清算**。
///
/// [`submitter`] 提交该批次的清算行多签管理员(必须是 institution_main 即收款方机构的激活管理员)
/// [`institution_main`] 批次归属的清算行主账户地址(= **收款方**清算行)
/// [`batch`] SCALE 编码过的 V2 批次数据(已在 extrinsic 入口完成 BoundedVec 长度校验)
///
/// 偿付预检按 **付款方清算行** 做(每个 payer_bank 各自统计扣减总额),因为
/// 跨行支付的链上 Currency 流出来自付款方主账户。同一批次内可能有多个 payer_bank。
pub fn execute_clearing_bank_batch<T: Config>(
    submitter: &T::AccountId,
    institution_main: &T::AccountId,
    batch: &[OffchainBatchItemV2<T::AccountId, BlockNumberFor<T>>],
) -> DispatchResult {
    // 批次级校验:提交方必须是 institution_main(收款方清算行)的激活管理员
    ensure!(
        T::SfidAccountQuery::is_admin_of(institution_main, submitter),
        Error::<T>::UnauthorizedAdmin
    );

    let now = frame_system::Pallet::<T>::block_number();

    // 按付款方清算行分组统计扣款总额(同行只扣 fee 不流出主账户;跨行扣 transfer+fee)。
    // 用 BTreeMap 保证迭代顺序确定,与 saturating_add 一起避免重入风险。
    let mut projected_debits: sp_std::collections::btree_map::BTreeMap<T::AccountId, u128> =
        Default::default();

    for item in batch.iter() {
        // recipient_bank 必须是本批次的 institution_main(收款方主导)
        ensure!(
            &item.recipient_bank == institution_main,
            Error::<T>::InstitutionMismatch
        );
        ensure!(item.transfer_amount > 0, Error::<T>::InvalidTransferAmount);
        ensure!(
            item.payer != item.recipient,
            Error::<T>::SelfTransferNotAllowed
        );
        ensure!(now <= item.expires_at, Error::<T>::ExpiredIntent);

        // 付款方清算行必须合法(跨行时必要;同行时即 institution_main 自身,已知合法)
        if item.payer_bank != item.recipient_bank {
            bank_check::ensure_can_be_bound::<T>(&item.payer_bank)?;
        }

        // 费率校验(按收款方清算行 = institution_main)
        let rate_bp = fee_config::current_rate_bp::<T>(&item.recipient_bank);
        ensure!(rate_bp > 0, Error::<T>::L2FeeRateNotConfigured);
        let expected_fee = calc_fee(item.transfer_amount, rate_bp)
            .map_err(|_| Error::<T>::TransferAmountTooLarge)?;
        ensure!(
            item.fee_amount == expected_fee,
            Error::<T>::InvalidFeeAmount
        );

        // 统计付款方清算行即将扣减的总额(用于偿付预检)
        let item_debit = if item.payer_bank == item.recipient_bank {
            // 同行:fee 走 fee_account 但本金内部轧差(主账户净流出 = fee)
            item.fee_amount
        } else {
            // 跨行:本金 + fee 都从付款方主账户流出
            item.transfer_amount.saturating_add(item.fee_amount)
        };
        let entry = projected_debits.entry(item.payer_bank.clone()).or_insert(0);
        *entry = entry.saturating_add(item_debit);
    }

    // 按付款方清算行做偿付预检
    let mut total_batch_debit: u128 = 0;
    for (payer_bank, debit) in projected_debits.iter() {
        solvency::ensure_can_debit::<T>(payer_bank, *debit)?;
        total_batch_debit = total_batch_debit.saturating_add(*debit);
    }

    // 逐笔执行
    for item in batch.iter() {
        execute_single_item::<T>(item, now)?;
    }

    Pallet::<T>::deposit_event(Event::<T>::ClearingBankBatchSettled {
        bank: institution_main.clone(),
        submitter: submitter.clone(),
        item_count: batch.len() as u32,
        total_debit: total_batch_debit,
    });
    Ok(())
}

/// 单笔 item 的验证 + 分账。
fn execute_single_item<T: Config>(
    item: &OffchainBatchItemV2<T::AccountId, BlockNumberFor<T>>,
    now: BlockNumberFor<T>,
) -> DispatchResult {
    // 1. 验 L3 签名
    let intent = item.to_intent();
    let msg = intent.signing_hash();
    let payer_pk = pubkey_from_accountid::<T>(&item.payer)?;
    let sig = Sr25519Signature::try_from(&item.payer_sig[..])
        .map_err(|_| Error::<T>::InvalidL3Signature)?;
    ensure!(
        sr25519_verify(&sig, &msg, &payer_pk),
        Error::<T>::InvalidL3Signature
    );

    // 2. 消费 nonce
    nonce::consume_nonce::<T>(&item.payer, item.payer_nonce)?;

    // 3. 防重放(t2 从付款方清算行 sfid_id 取前 2 字节作为 shard key;本步
    //    为了兼容现有 ProcessedOffchainTx 双 map 结构,用固定 t2 或 0)
    //    Step 3 再细化清算行级别的防重放分桶。
    //
    //    `item.tx_id` 是 `H256`,链上 Storage 用 `T::Hash`(Substrate 默认等于
    //    H256)。通过 SCALE 编解码跨类型转换,与 frame_system 默认配置兼容。
    let t2 = t2_from_bank::<T>(&item.payer_bank).unwrap_or([b'L', b'2']);
    let tx_hash: T::Hash = T::Hash::decode(&mut &item.tx_id.as_bytes()[..])
        .map_err(|_| Error::<T>::InvalidL3Signature)?;
    ensure!(
        !ProcessedOffchainTx::<T>::contains_key(t2, tx_hash),
        Error::<T>::TxAlreadyProcessed
    );

    // 4. 分账
    let payer_bank = &item.payer_bank;
    let recipient_bank = &item.recipient_bank;
    let fee_account = bank_check::fee_account_of::<T>(recipient_bank)?;

    // 付款方 L3 存款校验
    let payer_balance = DepositBalance::<T>::get(payer_bank, &item.payer);
    let total_debit = item.transfer_amount.saturating_add(item.fee_amount);
    ensure!(
        payer_balance >= total_debit,
        Error::<T>::InsufficientDepositBalance
    );

    if payer_bank == recipient_bank {
        // 同行:本金在 L2 内部轧差,只 fee 从主账户流出
        DepositBalance::<T>::mutate(payer_bank, &item.payer, |b| {
            *b = b.saturating_sub(total_debit);
        });
        DepositBalance::<T>::mutate(payer_bank, &item.recipient, |b| {
            *b = b.saturating_add(item.transfer_amount);
        });
        // 同行时 BankTotalDeposits 下降 fee 部分(手续费流出 L2)
        BankTotalDeposits::<T>::mutate(payer_bank, |t| {
            *t = t.saturating_sub(item.fee_amount);
        });
        let fee_bal: BalanceOf<T> = item.fee_amount.saturated_into();
        T::Currency::transfer(payer_bank, &fee_account, fee_bal, KeepAlive)?;
    } else {
        // 跨行:本金跨行转 + fee 转到收款方费用账户
        let transfer_bal: BalanceOf<T> = item.transfer_amount.saturated_into();
        let fee_bal: BalanceOf<T> = item.fee_amount.saturated_into();
        T::Currency::transfer(payer_bank, recipient_bank, transfer_bal, KeepAlive)?;
        T::Currency::transfer(payer_bank, &fee_account, fee_bal, KeepAlive)?;

        DepositBalance::<T>::mutate(payer_bank, &item.payer, |b| {
            *b = b.saturating_sub(total_debit);
        });
        DepositBalance::<T>::mutate(recipient_bank, &item.recipient, |b| {
            *b = b.saturating_add(item.transfer_amount);
        });
        BankTotalDeposits::<T>::mutate(payer_bank, |t| {
            *t = t.saturating_sub(total_debit);
        });
        BankTotalDeposits::<T>::mutate(recipient_bank, |t| {
            *t = t.saturating_add(item.transfer_amount);
        });
    }

    // 5. 防重放标记 + 事件(tx_hash 已在本函数开头从 item.tx_id 解码)
    ProcessedOffchainTx::<T>::insert(t2, tx_hash, true);
    ProcessedOffchainTxAt::<T>::insert(t2, tx_hash, now);

    Pallet::<T>::deposit_event(Event::<T>::PaymentSettled {
        tx_id: tx_hash,
        payer: item.payer.clone(),
        payer_bank: item.payer_bank.clone(),
        recipient: item.recipient.clone(),
        recipient_bank: item.recipient_bank.clone(),
        transfer_amount: item.transfer_amount,
        fee_amount: item.fee_amount,
    });
    Ok(())
}

/// 把 `T::AccountId` 编码取前 32 字节作为 sr25519 公钥。
fn pubkey_from_accountid<T: Config>(acc: &T::AccountId) -> Result<Sr25519Public, Error<T>> {
    let encoded = acc.encode();
    if encoded.len() < 32 {
        return Err(Error::<T>::InvalidL3Signature);
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&encoded[..32]);
    Ok(Sr25519Public::from_raw(arr))
}

/// 从清算行主账户反查 t2(SFID 第二段前 2 字符),失败时返回 None。
fn t2_from_bank<T: Config>(bank_main: &T::AccountId) -> Option<[u8; 2]> {
    let info = T::SfidAccountQuery::account_info(bank_main)?;
    let sfid_bytes = info.0;
    // SFID 格式 A3-R5-T2P1C1-N9-D8,第二段 R5 前 2 字符是省编码,本步用它做 shard
    // 近似 t2。例如 "SFR-GD-SZ01-...." → ['G','D']
    if sfid_bytes.len() < 6 {
        return None;
    }
    // 跳过 "SFR-"(4 字节)取后面 2 字节
    let t2_slice = sfid_bytes.get(4..6)?;
    Some([t2_slice[0], t2_slice[1]])
}
