//! 清算行节点对 wuminapp 的 RPC 接口(Step 1 骨架)。
//!
//! 中文注释:
//! - Step 1 仅暴露**只读查询**:余额、下一个 nonce、待上链笔数。
//! - Step 2 起启用 `offchain_submit_payment(intent, sig)`(扫码支付入口)
//!   和 WebSocket 推送(`offchain_subscribe_notifications`)。
//! - 本文件定义 **JSON-RPC trait 与纯 Rust 实现**,`citizenchain/node/src/core/rpc.rs`
//!   在 Step 2 起委托到这里。

#![allow(dead_code)]

use codec::{Decode, Encode};
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorCode, ErrorObjectOwned},
};
use sc_client_api::StorageProvider;
use serde::{Deserialize, Serialize};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::SaturatedConversion;
use sp_runtime::AccountId32;
use sp_storage::StorageKey;
use std::sync::Arc;

use super::ledger::{NodePaymentIntent, OffchainLedger};
use crate::core::service::FullClient;
use crate::offchain::settlement::packer::BatchSigner;

/// 扫码支付 pallet 在 runtime `construct_runtime!` 中的实例名。与
/// `reserve.rs` 保持一致,storage key 前缀计算 `twox_128(PALLET_NAME)` 依赖它。
const PALLET_NAME: &[u8] = b"OffchainTransaction";

/// 清算行节点暴露给 wuminapp 的查询 RPC。
///
/// 命名空间 `offchain`,方法名与扫码支付协议保持一致。
#[rpc(server, namespace = "offchain")]
pub trait OffchainClearingRpc {
    /// 查询 L3 在本清算行的**可用存款余额**(分)。
    ///
    /// 可用余额 = 链上 `DepositBalance` 同步值 - 本地已接受未上链扣款。
    #[method(name = "queryBalance")]
    fn query_balance(&self, user: AccountId32) -> RpcResult<u128>;

    /// 查询 L3 下一个应使用的 `nonce`(Step 2 扫码支付前调用)。
    ///
    /// wuminapp 本地保管 nonce 的同时,每次签名前问一次以防错位。
    #[method(name = "queryNextNonce")]
    fn query_next_nonce(&self, user: AccountId32) -> RpcResult<u64>;

    /// 查询本清算行待上链笔数(运维查看)。
    #[method(name = "queryPendingCount")]
    fn query_pending_count(&self) -> RpcResult<u64>;

    // ─── Step 2b 新增:扫码支付提交入口 ───

    /// 扫码支付提交入口。wuminapp 本地对 `PaymentIntent` 做 SCALE 编码后
    /// 用 L3 sr25519 私钥签名,把 hex 形式的 intent 和 64 字节签名一起提交。
    ///
    /// 节点侧:
    /// - 反序列化 intent
    /// - 验证 L3 sr25519 签名
    /// - 校验绑定清算行 / 费率 / nonce / 可用余额
    /// - 入账到本地 pending 列表(Step 2b-ii packer 再上链)
    /// - 返回 tx_id + 清算行 ACK 签名
    #[method(name = "submitPayment")]
    fn submit_payment(
        &self,
        intent_hex: String,
        payer_sig_hex: String,
    ) -> RpcResult<SubmitPaymentResp>;

    // ─── Step 2c-i 新增:wuminapp 扫码前置查询 ───

    /// 查询 L3 当前绑定的清算行主账户地址(对应链上 `UserBank[user]`)。
    ///
    /// wuminapp 在扫码付款前调用,以确定"本人付款方清算行"(`payer_bank`)。
    /// 未绑定返回 `None`,调用方据此提示用户先完成绑定流程。
    #[method(name = "queryUserBank")]
    fn query_user_bank(&self, user: AccountId32) -> RpcResult<Option<AccountId32>>;

    /// 查询指定清算行当前生效费率(对应链上 `L2FeeRateBp[bank]`)。
    ///
    /// 返回 `rate_bp`(万分之一)与 `min_fee_fen`(最低手续费,分)。
    /// wuminapp 据此在 UI 上展示费率与预计扣费,并本地预计算 `fee_amount`
    /// 以便构造 `PaymentIntent`。runtime `ValueQuery` 默认 0,本 RPC 同步
    /// 把 0 映射为"费率未设置"提示,调用方应拒绝提交。
    #[method(name = "queryFeeRate")]
    fn query_fee_rate(&self, bank: AccountId32) -> RpcResult<FeeRateResp>;
}

/// 扫码支付提交响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitPaymentResp {
    /// 本笔支付 tx_id 的 hex(含 `0x` 前缀)。
    pub tx_id: String,
    /// 清算行 ACK 签名的 hex(64 字节 = 128 hex,含 `0x` 前缀)。
    pub l2_ack_sig: String,
    /// 节点接受本笔的 UNIX 秒时间戳。
    pub accepted_at: u64,
}

/// 清算行费率查询响应(`offchain_queryFeeRate`)。
///
/// 与 runtime `fee_config::calc_fee` 口径对齐:fee = `max(amount * rate_bp / 10_000, min_fee_fen)`,
/// 四舍五入规则同 runtime。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeRateResp {
    /// 清算行当前生效费率(万分之一)。runtime 未配置时为 0,调用方应拒绝提交。
    pub rate_bp: u32,
    /// 最低手续费(分),与 runtime `settlement::MIN_FEE_FEN` 常量一致(当前 1)。
    pub min_fee_fen: u128,
}

/// 清算行 RPC 的具体实现。`derive(Clone)` 让 service.rs 能把 `Arc<Self>` 里
/// 的内容 owned 地传给 `into_rpc`;内部字段只有 Arc,clone 廉价。
#[derive(Clone)]
pub struct OffchainClearingRpcImpl {
    ledger: Arc<OffchainLedger>,
    /// 用于读取链上 storage(`UserBank` / `DepositBalance` / `L3PaymentNonce` / `L2FeeRateBp`)。
    client: Arc<FullClient>,
    /// 当前节点归属的收款方清算行主账户,用于拒绝错路由支付。
    bank_main: AccountId32,
    /// 复用清算行管理员签名器生成 L2 ACK。
    ack_signer: Arc<dyn BatchSigner>,
}

impl OffchainClearingRpcImpl {
    pub fn new(
        ledger: Arc<OffchainLedger>,
        client: Arc<FullClient>,
        bank_main: AccountId32,
        ack_signer: Arc<dyn BatchSigner>,
    ) -> Self {
        Self {
            ledger,
            client,
            bank_main,
            ack_signer,
        }
    }

    fn read_user_bank(&self, user: &AccountId32) -> RpcResult<Option<AccountId32>> {
        let best = self.client.info().best_hash;
        let key = user_bank_storage_key(user);
        let raw = self
            .client
            .storage(best, &key)
            .map_err(|e| rpc_err(ErrorCode::InternalError, format!("storage 读取失败:{e}")))?;
        match raw {
            None => Ok(None),
            Some(data) => AccountId32::decode(&mut &data.0[..])
                .map(Some)
                .map_err(|e| {
                    rpc_err(
                        ErrorCode::InternalError,
                        format!("AccountId32 解码失败:{e}"),
                    )
                }),
        }
    }

    fn read_fee_rate(&self, bank: &AccountId32) -> RpcResult<u32> {
        let best = self.client.info().best_hash;
        let key = l2_fee_rate_bp_storage_key(bank);
        let raw = self
            .client
            .storage(best, &key)
            .map_err(|e| rpc_err(ErrorCode::InternalError, format!("storage 读取失败:{e}")))?;
        match raw {
            None => Ok(0),
            Some(data) => u32::decode(&mut &data.0[..])
                .map_err(|e| rpc_err(ErrorCode::InternalError, format!("u32 解码失败:{e}"))),
        }
    }

    fn read_l3_payment_nonce(&self, user: &AccountId32) -> RpcResult<u64> {
        let best = self.client.info().best_hash;
        let key = l3_payment_nonce_storage_key(user);
        let raw = self
            .client
            .storage(best, &key)
            .map_err(|e| rpc_err(ErrorCode::InternalError, format!("storage 读取失败:{e}")))?;
        match raw {
            None => Ok(0),
            Some(data) => u64::decode(&mut &data.0[..])
                .map_err(|e| rpc_err(ErrorCode::InternalError, format!("u64 解码失败:{e}"))),
        }
    }

    fn read_deposit_balance(&self, bank: &AccountId32, user: &AccountId32) -> RpcResult<u128> {
        let best = self.client.info().best_hash;
        let key = deposit_balance_storage_key(bank, user);
        let raw = self
            .client
            .storage(best, &key)
            .map_err(|e| rpc_err(ErrorCode::InternalError, format!("storage 读取失败:{e}")))?;
        match raw {
            None => Ok(0),
            Some(data) => u128::decode(&mut &data.0[..])
                .map_err(|e| rpc_err(ErrorCode::InternalError, format!("u128 解码失败:{e}"))),
        }
    }
}

impl OffchainClearingRpcServer for OffchainClearingRpcImpl {
    fn query_balance(&self, user: AccountId32) -> RpcResult<u128> {
        Ok(self.ledger.available_balance(&user))
    }

    fn query_next_nonce(&self, user: AccountId32) -> RpcResult<u64> {
        let chain_next = self.read_l3_payment_nonce(&user)?.saturating_add(1);
        Ok(chain_next.max(self.ledger.next_nonce(&user)))
    }

    fn query_pending_count(&self) -> RpcResult<u64> {
        Ok(self.ledger.pending_count() as u64)
    }

    fn submit_payment(
        &self,
        intent_hex: String,
        payer_sig_hex: String,
    ) -> RpcResult<SubmitPaymentResp> {
        // 1. 解析 intent SCALE hex → NodePaymentIntent
        let intent_bytes = decode_hex(&intent_hex)
            .map_err(|e| rpc_err(ErrorCode::InvalidParams, format!("intent_hex 解析失败:{e}")))?;
        let intent = NodePaymentIntent::decode(&mut &intent_bytes[..]).map_err(|e| {
            rpc_err(
                ErrorCode::InvalidParams,
                format!("PaymentIntent SCALE 反序列化失败:{e}"),
            )
        })?;

        // 2. 解析签名 hex → 64 字节
        let sig_bytes = decode_hex(&payer_sig_hex)
            .map_err(|e| rpc_err(ErrorCode::InvalidParams, format!("sig_hex 解析失败:{e}")))?;
        if sig_bytes.len() != 64 {
            return Err(rpc_err(
                ErrorCode::InvalidParams,
                format!("payer_sig 必须 64 字节,实际 {}", sig_bytes.len()),
            ))?;
        }
        let mut payer_sig = [0u8; 64];
        payer_sig.copy_from_slice(&sig_bytes);

        // 3. 链上状态早拒:错路由、绑定漂移、费率不一致都不进入 pending。
        if intent.recipient_bank != self.bank_main {
            return Err(rpc_err(
                ErrorCode::InvalidParams,
                "recipient_bank 不属于当前清算行节点",
            ))?;
        }
        let payer_bank = self.read_user_bank(&intent.payer)?;
        if payer_bank.as_ref() != Some(&intent.payer_bank) {
            return Err(rpc_err(
                ErrorCode::InvalidParams,
                "payer_bank 与链上 UserBank[payer] 不一致",
            ))?;
        }
        let recipient_bank = self.read_user_bank(&intent.recipient)?;
        if recipient_bank.as_ref() != Some(&intent.recipient_bank) {
            return Err(rpc_err(
                ErrorCode::InvalidParams,
                "recipient_bank 与链上 UserBank[recipient] 不一致",
            ))?;
        }
        let rate_bp = self.read_fee_rate(&intent.recipient_bank)?;
        if rate_bp == 0 {
            return Err(rpc_err(ErrorCode::InvalidParams, "收款方清算行费率未配置"))?;
        }
        let expected_fee = calc_fee(intent.amount, rate_bp)
            .map_err(|e| rpc_err(ErrorCode::InvalidParams, format!("手续费计算失败:{e}")))?;
        if intent.fee != expected_fee {
            return Err(rpc_err(
                ErrorCode::InvalidParams,
                format!(
                    "手续费不匹配:expected={expected_fee}, actual={}",
                    intent.fee
                ),
            ))?;
        }

        let current_block: u32 = self.client.info().best_number.saturated_into();
        let accepted_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let ack_message = l2_ack_signing_message(&self.bank_main, &intent, &payer_sig, accepted_at);
        let l2_ack = self
            .ack_signer
            .sign_batch(&ack_message)
            .map_err(|e| rpc_err(ErrorCode::InternalError, format!("L2 ACK 签名失败:{e}")))?;

        // 4. 调 ledger 执行 L3 签名、nonce、余额校验并写入 pending。
        //
        // 中文注释:收款方主导清算后,跨行支付会提交到收款方清算节点。此时付款方
        // 不属于本地 ledger,必须读取链上权威 `DepositBalance/L3PaymentNonce`
        // 做校验,不能把付款方写成本地 ghost 账户。
        let cross_bank = intent.payer_bank != self.bank_main;
        let (chain_confirmed, chain_nonce) = if cross_bank {
            (
                Some(self.read_deposit_balance(&intent.payer_bank, &intent.payer)?),
                Some(self.read_l3_payment_nonce(&intent.payer)?),
            )
        } else {
            (None, None)
        };
        let (tx_id, l2_ack) = self
            .ledger
            .accept_payment_with_chain_state(
                intent,
                payer_sig,
                Some(current_block),
                l2_ack,
                accepted_at,
                &self.bank_main,
                chain_confirmed,
                chain_nonce,
            )
            .map_err(|e| rpc_err(ErrorCode::InvalidParams, e))?;

        Ok(SubmitPaymentResp {
            tx_id: format!("0x{}", encode_hex(tx_id.as_bytes())),
            l2_ack_sig: format!("0x{}", encode_hex(&l2_ack)),
            accepted_at,
        })
    }

    fn query_user_bank(&self, user: AccountId32) -> RpcResult<Option<AccountId32>> {
        self.read_user_bank(&user)
    }

    fn query_fee_rate(&self, bank: AccountId32) -> RpcResult<FeeRateResp> {
        let rate_bp = self.read_fee_rate(&bank)?;
        Ok(FeeRateResp {
            rate_bp,
            min_fee_fen: MIN_FEE_FEN,
        })
    }
}

/// 与 runtime `settlement::MIN_FEE_FEN` 常量逐字节一致的最低手续费。改动必须同改 runtime。
const MIN_FEE_FEN: u128 = 1;

/// 清算行 ACK 签名域,供 wuminapp 验证"节点确实接受了该支付意图"。
const L2_ACK_SIGNING_DOMAIN: &[u8] = b"GMB_L2_ACK_V1";

/// 与 runtime `settlement::calc_fee` 保持一致:按万分比四舍五入,最低 1 分。
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
    Ok(std::cmp::max(rounded, MIN_FEE_FEN))
}

/// 构造 L2 ACK 签名消息:
/// `blake2_256(DOMAIN || bank_main || SCALE(intent) || payer_sig || accepted_at_le)`。
fn l2_ack_signing_message(
    bank_main: &AccountId32,
    intent: &NodePaymentIntent,
    payer_sig: &[u8; 64],
    accepted_at: u64,
) -> [u8; 32] {
    let intent_bytes = intent.encode();
    let mut data =
        Vec::with_capacity(L2_ACK_SIGNING_DOMAIN.len() + 32 + intent_bytes.len() + 64 + 8);
    data.extend_from_slice(L2_ACK_SIGNING_DOMAIN);
    data.extend_from_slice(bank_main.as_ref());
    data.extend_from_slice(&intent_bytes);
    data.extend_from_slice(payer_sig);
    data.extend_from_slice(&accepted_at.to_le_bytes());
    sp_io::hashing::blake2_256(&data)
}

/// 构造 `UserBank[user]` 的 storage key(`StorageMap<_, Blake2_128Concat, AccountId, AccountId, OptionQuery>`)。
fn user_bank_storage_key(user: &AccountId32) -> StorageKey {
    let encoded = user.encode();
    let mut k = Vec::with_capacity(16 + 16 + 16 + encoded.len());
    k.extend_from_slice(&sp_io::hashing::twox_128(PALLET_NAME));
    k.extend_from_slice(&sp_io::hashing::twox_128(b"UserBank"));
    k.extend_from_slice(&sp_io::hashing::blake2_128(&encoded));
    k.extend_from_slice(&encoded);
    StorageKey(k)
}

/// 构造 `L2FeeRateBp[bank]` 的 storage key(`StorageMap<_, Blake2_128Concat, AccountId, u32, ValueQuery>`)。
fn l2_fee_rate_bp_storage_key(bank: &AccountId32) -> StorageKey {
    let encoded = bank.encode();
    let mut k = Vec::with_capacity(16 + 16 + 16 + encoded.len());
    k.extend_from_slice(&sp_io::hashing::twox_128(PALLET_NAME));
    k.extend_from_slice(&sp_io::hashing::twox_128(b"L2FeeRateBp"));
    k.extend_from_slice(&sp_io::hashing::blake2_128(&encoded));
    k.extend_from_slice(&encoded);
    StorageKey(k)
}

/// 构造 `L3PaymentNonce[user]` 的 storage key(`StorageMap<_, Blake2_128Concat, AccountId, u64, ValueQuery>`)。
fn l3_payment_nonce_storage_key(user: &AccountId32) -> StorageKey {
    let encoded = user.encode();
    let mut k = Vec::with_capacity(16 + 16 + 16 + encoded.len());
    k.extend_from_slice(&sp_io::hashing::twox_128(PALLET_NAME));
    k.extend_from_slice(&sp_io::hashing::twox_128(b"L3PaymentNonce"));
    k.extend_from_slice(&sp_io::hashing::blake2_128(&encoded));
    k.extend_from_slice(&encoded);
    StorageKey(k)
}

/// 构造 `DepositBalance[bank][user]` 的 storage key。
///
/// runtime 定义为 `StorageDoubleMap<_, Blake2_128Concat, AccountId, Blake2_128Concat,
/// AccountId, u128, ValueQuery>`,因此两级 key 都是 `blake2_128(account) || account`。
fn deposit_balance_storage_key(bank: &AccountId32, user: &AccountId32) -> StorageKey {
    let bank_encoded = bank.encode();
    let user_encoded = user.encode();
    let mut k = Vec::with_capacity(16 + 16 + 16 + bank_encoded.len() + 16 + user_encoded.len());
    k.extend_from_slice(&sp_io::hashing::twox_128(PALLET_NAME));
    k.extend_from_slice(&sp_io::hashing::twox_128(b"DepositBalance"));
    k.extend_from_slice(&sp_io::hashing::blake2_128(&bank_encoded));
    k.extend_from_slice(&bank_encoded);
    k.extend_from_slice(&sp_io::hashing::blake2_128(&user_encoded));
    k.extend_from_slice(&user_encoded);
    StorageKey(k)
}

// ─── 内部工具 ───

/// 解析 hex(支持 `0x` 前缀),与现有 wuminapp / offchain 客户端风格一致。
fn decode_hex(input: &str) -> Result<Vec<u8>, String> {
    let text = input.strip_prefix("0x").unwrap_or(input);
    if text.is_empty() {
        return Ok(Vec::new());
    }
    if text.len() % 2 != 0 {
        return Err("hex 长度必须偶数".to_string());
    }
    let mut out = Vec::with_capacity(text.len() / 2);
    for i in (0..text.len()).step_by(2) {
        let byte =
            u8::from_str_radix(&text[i..i + 2], 16).map_err(|e| format!("非法 hex 字节: {e}"))?;
        out.push(byte);
    }
    Ok(out)
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

/// 通用错误构造器(Step 2 起 submit_payment 会用)。
pub(crate) fn rpc_err<T: Into<String>>(code: ErrorCode, msg: T) -> ErrorObjectOwned {
    ErrorObjectOwned::owned::<()>(code.code(), msg.into(), None)
}

#[cfg(test)]
mod tests {
    // RPC impl 实测需要 `Arc<FullClient>`,单元测试构造 FullClient 的代价
    // 不值当;这里只验证:(a) storage key 布局与 runtime 哈希器(Blake2_128Concat)
    // 一致;(b) hex 编解码 roundtrip。功能回归靠集成测试(Step 2b-iii-b 起
    // 的 dev-chain smoke-test)覆盖。

    use super::*;

    fn acc(b: u8) -> AccountId32 {
        AccountId32::new([b; 32])
    }

    #[test]
    fn user_bank_storage_key_layout() {
        let user = acc(0x11);
        let encoded = user.encode();
        let key = user_bank_storage_key(&user);
        assert_eq!(key.0.len(), 16 + 16 + 16 + encoded.len());
        assert_eq!(&key.0[..16], &sp_io::hashing::twox_128(PALLET_NAME));
        assert_eq!(&key.0[16..32], &sp_io::hashing::twox_128(b"UserBank"));
        assert_eq!(&key.0[32..48], &sp_io::hashing::blake2_128(&encoded));
        assert_eq!(&key.0[48..], &encoded[..]);
    }

    #[test]
    fn l2_fee_rate_bp_storage_key_layout() {
        let bank = acc(0x22);
        let encoded = bank.encode();
        let key = l2_fee_rate_bp_storage_key(&bank);
        assert_eq!(key.0.len(), 16 + 16 + 16 + encoded.len());
        assert_eq!(&key.0[..16], &sp_io::hashing::twox_128(PALLET_NAME));
        assert_eq!(&key.0[16..32], &sp_io::hashing::twox_128(b"L2FeeRateBp"));
        assert_eq!(&key.0[32..48], &sp_io::hashing::blake2_128(&encoded));
        assert_eq!(&key.0[48..], &encoded[..]);
    }

    #[test]
    fn l3_payment_nonce_storage_key_layout() {
        let user = acc(0x33);
        let encoded = user.encode();
        let key = l3_payment_nonce_storage_key(&user);
        assert_eq!(key.0.len(), 16 + 16 + 16 + encoded.len());
        assert_eq!(&key.0[..16], &sp_io::hashing::twox_128(PALLET_NAME));
        assert_eq!(&key.0[16..32], &sp_io::hashing::twox_128(b"L3PaymentNonce"));
        assert_eq!(&key.0[32..48], &sp_io::hashing::blake2_128(&encoded));
        assert_eq!(&key.0[48..], &encoded[..]);
    }

    #[test]
    fn deposit_balance_storage_key_layout() {
        let bank = acc(0x44);
        let user = acc(0x55);
        let bank_encoded = bank.encode();
        let user_encoded = user.encode();
        let key = deposit_balance_storage_key(&bank, &user);
        assert_eq!(
            key.0.len(),
            16 + 16 + 16 + bank_encoded.len() + 16 + user_encoded.len()
        );
        assert_eq!(&key.0[..16], &sp_io::hashing::twox_128(PALLET_NAME));
        assert_eq!(&key.0[16..32], &sp_io::hashing::twox_128(b"DepositBalance"));
        assert_eq!(&key.0[32..48], &sp_io::hashing::blake2_128(&bank_encoded));
        assert_eq!(&key.0[48..80], &bank_encoded[..]);
        assert_eq!(&key.0[80..96], &sp_io::hashing::blake2_128(&user_encoded));
        assert_eq!(&key.0[96..], &user_encoded[..]);
    }

    #[test]
    fn storage_keys_distinct_per_account() {
        assert_ne!(
            user_bank_storage_key(&acc(1)).0,
            user_bank_storage_key(&acc(2)).0
        );
        assert_ne!(
            l2_fee_rate_bp_storage_key(&acc(1)).0,
            l2_fee_rate_bp_storage_key(&acc(2)).0,
        );
        assert_ne!(
            l3_payment_nonce_storage_key(&acc(1)).0,
            l3_payment_nonce_storage_key(&acc(2)).0,
        );
        assert_ne!(
            deposit_balance_storage_key(&acc(1), &acc(2)).0,
            deposit_balance_storage_key(&acc(2), &acc(1)).0,
        );
    }

    #[test]
    fn hex_codec_roundtrip() {
        let bytes = vec![0x00, 0xab, 0xff, 0x5c];
        let hex = encode_hex(&bytes);
        assert_eq!(hex, "00abff5c");
        assert_eq!(decode_hex(&format!("0x{hex}")).unwrap(), bytes);
        assert_eq!(decode_hex(&hex).unwrap(), bytes);
    }

    #[test]
    fn hex_decode_rejects_odd_length() {
        assert!(decode_hex("0xabc").is_err());
    }

    #[test]
    fn calc_fee_matches_runtime_rounding() {
        assert_eq!(calc_fee(10_000, 5).unwrap(), 5);
        assert_eq!(calc_fee(1, 1).unwrap(), MIN_FEE_FEN);
        assert_eq!(calc_fee(15_000, 1).unwrap(), 2);
        assert!(calc_fee(u128::MAX, 10_000).is_err());
    }

    #[test]
    fn l2_ack_signing_message_changes_with_inputs() {
        let intent = NodePaymentIntent {
            tx_id: sp_core::H256::repeat_byte(1),
            payer: acc(1),
            payer_bank: acc(2),
            recipient: acc(3),
            recipient_bank: acc(4),
            amount: 10_000,
            fee: 5,
            nonce: 1,
            expires_at: 100,
        };
        let sig = [9u8; 64];
        let h1 = l2_ack_signing_message(&acc(0xAA), &intent, &sig, 1_717_000_000);
        let h2 = l2_ack_signing_message(&acc(0xAA), &intent, &sig, 1_717_000_000);
        assert_eq!(h1, h2);
        assert_ne!(
            h1,
            l2_ack_signing_message(&acc(0xAB), &intent, &sig, 1_717_000_000)
        );
        assert_ne!(
            h1,
            l2_ack_signing_message(&acc(0xAA), &intent, &[8u8; 64], 1_717_000_000)
        );
        assert_ne!(
            h1,
            l2_ack_signing_message(&acc(0xAA), &intent, &sig, 1_717_000_001)
        );
    }
}
