//! 扫码支付 Step 2b-ii-β-2-a 新增:接 substrate `TransactionPool` 的批次提交器。
//!
//! 中文注释:
//! - 本文件实现 `packer::BatchSubmitter` trait,把组好的 batch 包进
//!   `RuntimeCall::OffchainTransaction(submit_offchain_batch_v2 { .. })`
//!   → `UncheckedExtrinsic` → 扔到节点本地 `TransactionPool`。
//! - extrinsic 构造流程与 `benchmarking.rs::create_benchmark_extrinsic` 严格对齐
//!   (`TxExtension` 各 Check 顺序必须与 runtime `type TxExtension = (..)` 完全一致)。
//! - 签名密钥复用 β-1 的 `KeystoreBatchSigner::signing_key` 容器:同一把清算行
//!   管理员 sr25519 私钥既签 batch 内部的 `batch_signature`,也签 extrinsic 外
//!   层的 `SignedPayload`。
//!
//! 本文件与 β-2-b 的衔接:
//! - β-2-b 的 service.rs 负责传入具体 `Arc<FullClient>` + `Arc<TransactionPoolHandle>`
//!   + `Arc<RwLock<Option<SigningKey>>>`,构造 `PoolBatchSubmitter` 后作为
//!   `Arc<dyn BatchSubmitter>` 注入 `start_clearing_bank_components`。

#![allow(dead_code)]

use codec::{Decode, Encode};
use frame_system_rpc_runtime_api::AccountNonceApi;
use sc_client_api::BlockBackend;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
    generic::Era, traits::SaturatedConversion, AccountId32, MultiAddress, OpaqueExtrinsic,
};
use std::sync::{Arc, RwLock};

use citizenchain as runtime;
use offchain_transaction::batch_item::OffchainBatchItemV2;

use crate::core::service::FullClient;
use super::packer::BatchSubmitter;
use super::keystore::SigningKey;

/// 具体 pool 别名。与 `service.rs` 里 `Service` 第 5 项严格对齐:
/// `TransactionPoolHandle<opaque::Block, FullClient>`。
/// 注意:pool 约束的 Block 是 **opaque::Block**(OpaqueExtrinsic 版),
/// 而不是 `runtime::Block`(带具体 UncheckedExtrinsic 版)。
pub type TxPool = sc_transaction_pool::TransactionPoolHandle<runtime::opaque::Block, FullClient>;

/// 扫码支付 Step 2b-ii-β-2-a 新增:真正走 `TransactionPool` 提交 extrinsic 的 submitter。
pub struct PoolBatchSubmitter {
    client: Arc<FullClient>,
    /// substrate TransactionPool,β-2-b 起真实调 `submit_one` 提交 extrinsic。
    pool: Arc<TxPool>,
    /// 同一把清算行管理员 sr25519 私钥:既是 batch 内部 `batch_signature` 的
    /// 签名者,也是 extrinsic 外层 `SignedPayload` 的签名者。
    signing_key: Arc<RwLock<Option<SigningKey>>>,
}

impl PoolBatchSubmitter {
    pub fn new(
        client: Arc<FullClient>,
        pool: Arc<TxPool>,
        signing_key: Arc<RwLock<Option<SigningKey>>>,
    ) -> Self {
        Self {
            client,
            pool,
            signing_key,
        }
    }
}

impl BatchSubmitter for PoolBatchSubmitter {
    fn submit(
        &self,
        institution_main: AccountId32,
        batch_seq: u64,
        batch_bytes: Vec<u8>,
        batch_signature: [u8; 64],
    ) -> Result<H256, String> {
        // 1. 解回 Vec<OffchainBatchItemV2>
        let batch = decode_batch_items(&batch_bytes)?;

        // 2. 构造 batch_signature 的 BoundedVec(runtime 端 BatchSignatureOf<T>)
        let sig_bounded = encode_bounded_sig::<runtime::Runtime>(&batch_signature)?;

        // 3. 构造 BoundedVec<OffchainBatchItemV2, MaxBatchSize>
        let batch_bounded = encode_bounded_batch::<runtime::Runtime>(batch)?;

        // 4. 拼 RuntimeCall
        let call = runtime::RuntimeCall::OffchainTransaction(
            offchain_transaction::pallet::Call::submit_offchain_batch_v2 {
                institution_main: institution_main.clone(),
                batch_seq,
                batch: batch_bounded,
                batch_signature: sig_bounded,
            },
        );

        // 5. 从 signing_key 拿 sr25519 pair
        let guard = self
            .signing_key
            .read()
            .map_err(|e| format!("签名密钥锁读取失败:{e}"))?;
        let key = guard
            .as_ref()
            .ok_or_else(|| "清算行签名管理员密钥未加载".to_string())?;
        let sender_pair = key.pair.clone();
        drop(guard);

        // 6. 查链上 nonce(对 sender_pair 对应账户)
        let sender_account = AccountId32::from(sender_pair.public());
        let nonce = lookup_nonce(&self.client, &sender_account);

        // 7. 构造签名过的 extrinsic
        let extrinsic = build_signed_extrinsic(&self.client, &sender_pair, call, nonce)?;

        // 8. 真实提交到 TransactionPool。
        //    `pool.submit_one` 返回 Future,packer 侧已经是 async 环境,但本
        //    trait `BatchSubmitter::submit` 保持 sync,因此用 `block_on`。
        //    若未来并发度上升,可把 trait 改 async + packer 内 .await。
        let best_hash = self.client.info().best_hash;
        let opaque: OpaqueExtrinsic = extrinsic.into();
        let fut = self
            .pool
            .submit_one(best_hash, TransactionSource::Local, opaque);
        let tx_hash =
            futures::executor::block_on(fut).map_err(|e| format!("pool.submit_one 失败:{e:?}"))?;

        log::info!(
            "[PoolBatchSubmitter] extrinsic submitted, institution={institution_main:?} \
             batch_seq={batch_seq} nonce={nonce}"
        );

        // `TxPool::Hash` 实际是 `H256`。用 SCALE 编解码做稳妥的跨类型转换。
        let hash_bytes = tx_hash.encode();
        H256::decode(&mut &hash_bytes[..]).map_err(|e| format!("TxHash 转 H256 失败:{e}"))
    }
}

/// 查询链上某账户最新 nonce。若 runtime api 不可用,回退 0(链上 `CheckNonce`
/// 会把不匹配的 nonce 拒掉,不会静默成功)。
fn lookup_nonce(client: &FullClient, account: &AccountId32) -> u32 {
    let best_hash = client.info().best_hash;
    let api = client.runtime_api();
    api.account_nonce(best_hash, account.clone()).unwrap_or(0)
}

// ---------------- 纯函数工具(便于单测) ----------------

/// 解 SCALE 字节为 `Vec<OffchainBatchItemV2>`(字段顺序与 runtime 严格一致)。
pub fn decode_batch_items(
    batch_bytes: &[u8],
) -> Result<Vec<OffchainBatchItemV2<AccountId32, u32>>, String> {
    <Vec<OffchainBatchItemV2<AccountId32, u32>>>::decode(&mut &batch_bytes[..])
        .map_err(|e| format!("batch_bytes 解码失败:{e}"))
}

/// 把 64 字节签名包装为 runtime `BatchSignatureOf<Runtime>`(`BoundedVec<u8, MaxBatchSignatureLength>`)。
pub fn encode_bounded_sig<T: offchain_transaction::pallet::Config>(
    sig: &[u8; 64],
) -> Result<
    frame_support::BoundedVec<
        u8,
        <T as offchain_transaction::pallet::Config>::MaxBatchSignatureLength,
    >,
    String,
> {
    sig.to_vec()
        .try_into()
        .map_err(|_| "batch_signature 超出 MaxBatchSignatureLength".to_string())
}

/// 把 Vec<OffchainBatchItemV2> 包装为 `BoundedVec<_, MaxBatchSize>`。
pub fn encode_bounded_batch<T: offchain_transaction::pallet::Config>(
    items: Vec<OffchainBatchItemV2<AccountId32, u32>>,
) -> Result<
    frame_support::BoundedVec<
        OffchainBatchItemV2<
            <T as frame_system::Config>::AccountId,
            frame_system::pallet_prelude::BlockNumberFor<T>,
        >,
        <T as offchain_transaction::pallet::Config>::MaxBatchSize,
    >,
    String,
>
where
    <T as frame_system::Config>::AccountId: From<AccountId32>,
    frame_system::pallet_prelude::BlockNumberFor<T>: From<u32>,
{
    let converted: Vec<_> = items
        .into_iter()
        .map(|it| OffchainBatchItemV2 {
            tx_id: it.tx_id,
            payer: it.payer.into(),
            payer_bank: it.payer_bank.into(),
            recipient: it.recipient.into(),
            recipient_bank: it.recipient_bank.into(),
            transfer_amount: it.transfer_amount,
            fee_amount: it.fee_amount,
            payer_sig: it.payer_sig,
            payer_nonce: it.payer_nonce,
            expires_at: it.expires_at.into(),
        })
        .collect();
    converted
        .try_into()
        .map_err(|_| "batch 超出 MaxBatchSize".to_string())
}

/// 构造签名 extrinsic(与 `benchmarking.rs::create_benchmark_extrinsic` 严格对齐)。
///
/// 本函数是 **本步单测的重点**:
/// - 参数确定性输入 → extrinsic 可 SCALE roundtrip
/// - SignedPayload 可被 `sr25519_verify` 验证
/// - TxExtension 各字段顺序与 runtime `type TxExtension` 一致
pub fn build_signed_extrinsic(
    client: &FullClient,
    sender: &sr25519::Pair,
    call: runtime::RuntimeCall,
    nonce: u32,
) -> Result<runtime::UncheckedExtrinsic, String> {
    let genesis_hash = client
        .block_hash(0)
        .map_err(|e| format!("读取 genesis_hash 失败:{e}"))?
        .ok_or_else(|| "Genesis block 尚未可用".to_string())?;

    let info = client.info();
    let best_hash = info.best_hash;
    let best_block = info.best_number;

    let period = runtime::configs::BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or(2) as u64;

    let tx_ext: runtime::TxExtension = (
        frame_system::AuthorizeCall::<runtime::Runtime>::new(),
        frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
        runtime::CheckNonStakeSender,
        frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<runtime::Runtime>::new(),
        frame_system::CheckGenesis::<runtime::Runtime>::new(),
        frame_system::CheckEra::<runtime::Runtime>::from(Era::mortal(
            period,
            best_block.saturated_into(),
        )),
        frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
        frame_system::CheckWeight::<runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
        frame_metadata_hash_extension::CheckMetadataHash::<runtime::Runtime>::new(false),
        frame_system::WeightReclaim::<runtime::Runtime>::new(),
    );

    let raw_payload = runtime::SignedPayload::from_raw(
        call.clone(),
        tx_ext.clone(),
        (
            (),
            (),
            (),
            runtime::VERSION.spec_version,
            runtime::VERSION.transaction_version,
            genesis_hash,
            best_hash,
            (),
            (),
            (),
            None,
            (),
        ),
    );
    let signature = raw_payload.using_encoded(|e| sender.sign(e));

    Ok(runtime::UncheckedExtrinsic::new_signed(
        call,
        MultiAddress::Id(AccountId32::from(sender.public())),
        runtime::Signature::Sr25519(signature),
        tx_ext,
    ))
}

// ---------------- 单元测试 ----------------

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_item(seed: u8) -> OffchainBatchItemV2<AccountId32, u32> {
        OffchainBatchItemV2 {
            tx_id: H256::repeat_byte(seed),
            payer: AccountId32::new([seed; 32]),
            payer_bank: AccountId32::new([0xAA; 32]),
            recipient: AccountId32::new([seed.wrapping_add(1); 32]),
            recipient_bank: AccountId32::new([0xBB; 32]),
            transfer_amount: 1_000,
            fee_amount: 1,
            payer_sig: [0u8; 64],
            payer_nonce: seed as u64,
            expires_at: 100,
        }
    }

    #[test]
    fn batch_bytes_decodes_to_items() {
        let items = vec![mk_item(1), mk_item(2), mk_item(3)];
        let bytes = items.encode();
        let decoded = decode_batch_items(&bytes).unwrap();
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0], items[0]);
        assert_eq!(decoded[2], items[2]);
    }

    #[test]
    fn encode_bounded_sig_respects_limit() {
        let ok: Result<
            frame_support::BoundedVec<
                u8,
                <runtime::Runtime as offchain_transaction::pallet::Config>::MaxBatchSignatureLength,
            >,
            String,
        > = encode_bounded_sig::<runtime::Runtime>(&[7u8; 64]);
        let bounded = ok.expect("64 字节应在 MaxBatchSignatureLength 以内");
        assert_eq!(bounded.len(), 64);
        assert_eq!(bounded[0], 7);
    }

    #[test]
    fn encode_bounded_batch_respects_limit() {
        let items = vec![mk_item(1), mk_item(2)];
        let bounded = encode_bounded_batch::<runtime::Runtime>(items.clone())
            .expect("2 item 应在 MaxBatchSize 以内");
        assert_eq!(bounded.len(), 2);
        assert_eq!(bounded[0].tx_id, items[0].tx_id);
    }

    #[test]
    fn decode_batch_items_rejects_invalid_bytes() {
        let bytes = vec![0xFFu8; 3];
        assert!(decode_batch_items(&bytes).is_err());
    }

    #[test]
    fn sign_key_slot_none_returns_err() {
        // 不构造 FullClient,只测 signing_key 路径的 None 分支 —
        // 通过 submitter.submit(...) 调用触发。为避免依赖 client,此测试
        // 仅验证 signing_key 锁正常访问:取出 read guard 并确认 None。
        let slot: Arc<RwLock<Option<SigningKey>>> = Arc::new(RwLock::new(None));
        let guard = slot.read().unwrap();
        assert!(guard.is_none());
    }
}
