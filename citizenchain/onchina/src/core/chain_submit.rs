//! 链交易组装与提交(ADR-031 D7)。
//!
//! onchina 侧唯一的 extrinsic 组装+提交通路:QR 仍只签不提交(冷钱包边界不变),
//! 管理员扫码回签后由本模块「重建签名材料 → 本地 sr25519 验签 → system_dryRun
//! 拒 Future/Stale → author_submitExtrinsic → 轮询 nonce 消费(InBestBlock 代理)」。
//! 签名材料构建与 node/src/governance/signing.rs 同源同规则:runtime 类型拼
//! SignedPayload,immortal era + 显式 nonce(PoW 推链三件套)。

use citizenchain as runtime;
use codec::{Decode, Encode};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sp_core::crypto::Ss58Codec;
use sp_runtime::generic::Era;
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::MultiSigner;

use super::chain_url::chain_http_url;

const RPC_TIMEOUT_SECS: u64 = 10;
/// 等交易进块上限:创世期 30 秒出块 × 3 块 + 余量。
const WAIT_INCLUSION_SECS: u64 = 95;
/// nonce 消费轮询间隔。
const WAIT_POLL_INTERVAL_SECS: u64 = 3;

/// 链交易签名材料:与冷钱包侧 `SignedPayload::using_encoded` 规则逐字节一致。
pub(crate) struct SigningMaterial {
    pub call: runtime::RuntimeCall,
    pub tx_ext: runtime::TxExtension,
    /// QR `b.d` 承载的完整 SignedPayload SCALE 字节。
    pub payload: Vec<u8>,
    /// sr25519 实际签名输入(payload 超 256B 时 Substrate 规则改签 blake2_256)。
    pub signing_bytes: Vec<u8>,
}

/// prepare 阶段产物:随会话持久化,submit 阶段重建校验。
pub(crate) struct PreparedChainSign {
    pub nonce: u32,
    /// 给 QR 的完整签名载荷字节。
    pub payload: Vec<u8>,
    /// sha256(签名输入) hex,submit 阶段重建校验防 runtime 漂移。
    pub signing_hash_hex: String,
}

async fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    let url = chain_http_url()?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(RPC_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("build rpc client failed: {e}"))?;
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    let resp: Value = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("rpc {method} request failed: {e}"))?
        .json()
        .await
        .map_err(|e| format!("rpc {method} response decode failed: {e}"))?;
    if let Some(err) = resp.get("error") {
        return Err(format!("rpc {method} error: {err}"));
    }
    resp.get("result")
        .cloned()
        .ok_or_else(|| format!("rpc {method} missing result"))
}

pub(crate) async fn fetch_runtime_version() -> Result<(u32, u32), String> {
    let result = rpc_post("state_getRuntimeVersion", Value::Array(vec![])).await?;
    let spec = result
        .get("specVersion")
        .and_then(|v| v.as_u64())
        .ok_or("runtime version missing specVersion")?;
    let tx = result
        .get("transactionVersion")
        .and_then(|v| v.as_u64())
        .ok_or("runtime version missing transactionVersion")?;
    Ok((spec as u32, tx as u32))
}

pub(crate) async fn fetch_genesis_hash() -> Result<[u8; 32], String> {
    let result = rpc_post("chain_getBlockHash", Value::Array(vec![Value::from(0)])).await?;
    let text = result.as_str().ok_or("genesis hash malformed")?;
    let bytes = hex::decode(text.strip_prefix("0x").unwrap_or(text))
        .map_err(|e| format!("genesis hash decode failed: {e}"))?;
    <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| "genesis hash must be 32 bytes".to_string())
}

fn pubkey_to_ss58(pubkey_hex: &str) -> Result<String, String> {
    let raw = hex::decode(pubkey_hex.trim_start_matches("0x"))
        .map_err(|e| format!("pubkey decode failed: {e}"))?;
    let pk = <[u8; 32]>::try_from(raw.as_slice()).map_err(|_| "pubkey must be 32 bytes")?;
    Ok(sp_core::sr25519::Public::from_raw(pk)
        .to_ss58check_with_version(sp_core::crypto::Ss58AddressFormat::custom(2027)))
}

/// 实时读链上 nonce(死规则 P-SIGN-001:nonce 只来自链,不缓存不自增)。
pub(crate) async fn fetch_nonce(pubkey_hex: &str) -> Result<u32, String> {
    let ss58 = pubkey_to_ss58(pubkey_hex)?;
    let result = rpc_post(
        "system_accountNextIndex",
        Value::Array(vec![Value::from(ss58)]),
    )
    .await?;
    result
        .as_u64()
        .map(|v| v as u32)
        .ok_or_else(|| "accountNextIndex malformed".to_string())
}

fn decode_runtime_call(call_data: &[u8]) -> Result<runtime::RuntimeCall, String> {
    let mut input = call_data;
    let call = runtime::RuntimeCall::decode(&mut input)
        .map_err(|e| format!("call_data 不是当前 runtime 可解码调用: {e}"))?;
    if !input.is_empty() {
        return Err(format!("call_data 存在 {} 字节尾随数据", input.len()));
    }
    Ok(call)
}

fn build_tx_extension(nonce: u32) -> runtime::TxExtension {
    (
        frame_system::AuthorizeCall::<runtime::Runtime>::new(),
        frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
        runtime::CheckNonStakeSender,
        frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<runtime::Runtime>::new(),
        frame_system::CheckGenesis::<runtime::Runtime>::new(),
        frame_system::CheckEra::<runtime::Runtime>::from(Era::Immortal),
        frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
        frame_system::CheckWeight::<runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
        frame_metadata_hash_extension::CheckMetadataHash::<runtime::Runtime>::new(false),
        frame_system::WeightReclaim::<runtime::Runtime>::new(),
    )
}

pub(crate) fn build_signing_material(
    call_data: &[u8],
    genesis_hash: &[u8; 32],
    nonce: u32,
    spec_version: u32,
    tx_version: u32,
) -> Result<SigningMaterial, String> {
    let call = decode_runtime_call(call_data)?;
    let tx_ext = build_tx_extension(nonce);
    let genesis_hash = sp_core::H256::from_slice(genesis_hash);
    let raw_payload = runtime::SignedPayload::from_raw(
        call.clone(),
        tx_ext.clone(),
        (
            (),
            (),
            (),
            spec_version,
            tx_version,
            genesis_hash,
            genesis_hash, // CheckEra: immortal → block_hash(0) = genesis_hash。
            (),
            (),
            (),
            None,
            (),
        ),
    );
    Ok(SigningMaterial {
        call,
        tx_ext,
        payload: raw_payload.encode(),
        signing_bytes: raw_payload.using_encoded(|payload| payload.to_vec()),
    })
}

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

/// prepare:实时取 nonce/版本/创世哈希,构建冷签载荷与校验哈希。
pub(crate) async fn prepare_signing(
    call_data: &[u8],
    signer_pubkey_hex: &str,
) -> Result<PreparedChainSign, String> {
    let nonce = fetch_nonce(signer_pubkey_hex).await?;
    let (spec_version, tx_version) = fetch_runtime_version().await?;
    let genesis_hash = fetch_genesis_hash().await?;
    let material =
        build_signing_material(call_data, &genesis_hash, nonce, spec_version, tx_version)?;
    Ok(PreparedChainSign {
        nonce,
        signing_hash_hex: sha256_hex(&material.signing_bytes),
        payload: material.payload,
    })
}

/// dry-run 拒绝原因(与 node 端同口径:Future 给人话,其余留技术原因)。
fn dry_run_reject_message(result_bytes: &[u8], raw_hex: &str) -> String {
    if result_bytes.starts_with(&[0x01, 0x00, 0x02]) {
        return "上一笔交易尚未出块,请稍候再试".to_string();
    }
    if result_bytes.starts_with(&[0x01, 0x00, 0x01]) {
        return "交易 nonce 已过期(Stale),请重新发起".to_string();
    }
    format!("交易预检被拒绝: 0x{raw_hex}")
}

/// submit:重建材料校验哈希 → 本地验签 → dry-run → 提交,返回交易哈希。
pub(crate) async fn assemble_and_submit(
    call_data: &[u8],
    signer_pubkey_hex: &str,
    signature_hex: &str,
    nonce: u32,
    expected_signing_hash_hex: &str,
) -> Result<String, String> {
    let (spec_version, tx_version) = fetch_runtime_version().await?;
    let genesis_hash = fetch_genesis_hash().await?;
    let material =
        build_signing_material(call_data, &genesis_hash, nonce, spec_version, tx_version)?;
    // 会话期间 runtime 版本/创世哈希不得漂移,否则签名对不上载荷。
    if sha256_hex(&material.signing_bytes) != expected_signing_hash_hex {
        return Err("签名载荷与会话不一致(runtime 版本或创世哈希已变化),请重新发起".into());
    }

    let pk_raw = hex::decode(signer_pubkey_hex.trim_start_matches("0x"))
        .map_err(|e| format!("公钥解码失败: {e}"))?;
    let public = sp_core::sr25519::Public::from_raw(
        <[u8; 32]>::try_from(pk_raw.as_slice()).map_err(|_| "公钥必须 32 字节")?,
    );
    let sig_raw = hex::decode(signature_hex.trim_start_matches("0x"))
        .map_err(|e| format!("签名解码失败: {e}"))?;
    let signature = sp_core::sr25519::Signature::from_raw(
        <[u8; 64]>::try_from(sig_raw.as_slice()).map_err(|_| "签名必须 64 字节")?,
    );
    if !sp_core::sr25519::Pair::verify(&signature, &material.signing_bytes, &public) {
        return Err("sr25519 本地验签失败,拒绝提交".to_string());
    }

    let account = MultiSigner::from(public).into_account();
    let extrinsic = runtime::UncheckedExtrinsic::new_signed(
        material.call,
        sp_runtime::MultiAddress::Id(account),
        runtime::Signature::Sr25519(signature),
        material.tx_ext,
    );
    let extrinsic_hex = format!("0x{}", hex::encode(extrinsic.encode()));

    // dry-run:Future/Stale 提交后只会"看似成功永不上链",必须先拒;
    // dry-run RPC 本身不可用时保持可用性兜底继续提交,由交易池终审。
    match rpc_post(
        "system_dryRun",
        Value::Array(vec![Value::from(extrinsic_hex.clone())]),
    )
    .await
    {
        Ok(v) => {
            let s = v.as_str().unwrap_or("");
            let raw = s.strip_prefix("0x").unwrap_or(s);
            let bytes = hex::decode(raw)
                .map_err(|e| format!("dry-run 结果异常,拒绝提交: {e} (raw: {s})"))?;
            if bytes.is_empty() {
                return Err("dry-run 返回空结果,拒绝提交".to_string());
            }
            if bytes[0] != 0x00 {
                return Err(dry_run_reject_message(&bytes, raw));
            }
            if bytes.len() > 1 && bytes[1] != 0x00 {
                return Err(format!("交易执行会失败: DispatchError (hex: {s})"));
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "system_dryRun 不可用,跳过预检继续提交");
        }
    }

    let result = rpc_post(
        "author_submitExtrinsic",
        Value::Array(vec![Value::from(extrinsic_hex)]),
    )
    .await?;
    result
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("author_submitExtrinsic 返回非字符串: {result}"))
}

/// 等交易进块:immortal + 显式 nonce 下,accountNextIndex 越过提交 nonce
/// 即已被打包(InBestBlock 可靠代理);超时视为失败上抛。
pub(crate) async fn wait_nonce_consumed(
    pubkey_hex: &str,
    submitted_nonce: u32,
) -> Result<(), String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(WAIT_INCLUSION_SECS);
    loop {
        let current = fetch_nonce(pubkey_hex).await?;
        if current > submitted_nonce {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!(
                "等待交易进块超时({WAIT_INCLUSION_SECS} 秒):nonce {submitted_nonce} 尚未消费"
            ));
        }
        tokio::time::sleep(std::time::Duration::from_secs(WAIT_POLL_INTERVAL_SECS)).await;
    }
}

/// 交易进块后回查块高:从链头回溯比对交易哈希(blake2_256(extrinsic 字节))。
/// 提交路径同步回写(ADR-031 D8),不依赖后台 indexer。
pub(crate) async fn find_extrinsic_block(tx_hash_hex: &str) -> Result<Option<u64>, String> {
    let target = hex::decode(tx_hash_hex.trim_start_matches("0x"))
        .map_err(|e| format!("tx hash decode failed: {e}"))?;
    let mut hash: String = rpc_post("chain_getBlockHash", Value::Array(vec![]))
        .await?
        .as_str()
        .ok_or("head hash malformed")?
        .to_string();
    for _ in 0..20 {
        let block = rpc_post(
            "chain_getBlock",
            Value::Array(vec![Value::from(hash.clone())]),
        )
        .await?;
        let header = &block["block"]["header"];
        let number = header["number"]
            .as_str()
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .ok_or("block number malformed")?;
        if let Some(exts) = block["block"]["extrinsics"].as_array() {
            for ext in exts {
                let Some(ext_hex) = ext.as_str() else {
                    continue;
                };
                let Ok(bytes) = hex::decode(ext_hex.trim_start_matches("0x")) else {
                    continue;
                };
                if sp_core::hashing::blake2_256(&bytes)[..] == target[..] {
                    return Ok(Some(number));
                }
            }
        }
        if number == 0 {
            break;
        }
        hash = header["parentHash"]
            .as_str()
            .ok_or("parent hash malformed")?
            .to_string();
    }
    Ok(None)
}

pub(crate) use sp_core::Pair as _;

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;

    /// 材料构建离线自洽:同输入同产物,call 解码回等值,>256B 载荷签名输入为 blake2。
    #[test]
    fn signing_material_roundtrip_and_hash_rule() {
        let call = runtime::RuntimeCall::System(frame_system::Call::remark {
            remark: vec![7u8; 8],
        });
        let call_data = call.encode();
        let genesis = [9u8; 32];
        let m = build_signing_material(&call_data, &genesis, 5, 1, 1).expect("material");
        assert_eq!(m.call.encode(), call_data);
        // 小载荷:签名输入 == payload 本体。
        assert!(m.payload.len() <= 256);
        assert_eq!(m.signing_bytes, m.payload);

        let big_call = runtime::RuntimeCall::System(frame_system::Call::remark {
            remark: vec![7u8; 400],
        });
        let big = build_signing_material(&big_call.encode(), &genesis, 5, 1, 1).expect("material");
        // 大载荷:SignedPayload 的 Encode 规则内置 >256B 改签 blake2_256,
        // encode() 与 using_encoded 同值 —— payload 即最终签名输入(32 字节哈希)。
        assert_eq!(big.signing_bytes.len(), 32);
        assert_eq!(big.payload, big.signing_bytes);
    }

    #[test]
    fn tail_data_in_call_is_rejected() {
        let call = runtime::RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
        let mut data = call.encode();
        data.push(0xff);
        assert!(decode_runtime_call(&data).is_err());
    }
}
