//! 链交易组装与提交(ADR-031 D7)。
//!
//! OnChina 侧唯一的 extrinsic 组装+提交通路：CitizenWallet 只签名一次并显示响应二维码，
//! OnChina 回扫后由本模块「重建签名材料 → 本地 sr25519 验签 → system_dryRun
//! 拒 Future/Stale → author_submitExtrinsic → 轮询 nonce 消费(InBestBlock 代理)」。
//! 签名材料构建统一调用 `chain-signing`,避免 OnChina 和 node 各自拼 payload。

use serde_json::{json, Value};
use sp_core::crypto::Ss58Codec;

use super::chain_url::chain_http_url;

const RPC_TIMEOUT_SECS: u64 = 10;
/// 客户端等待确认的观察窗口，不是 PoW 最晚出块期限；窗口结束后交易仍可能继续等待进块。
const WAIT_CONFIRMATION_OBSERVATION_SECS: u64 = 20 * 60;
/// nonce 消费轮询间隔。
const WAIT_POLL_INTERVAL_SECS: u64 = 3;

/// prepare 阶段产物:随会话持久化,submit 阶段重建校验。
pub(crate) struct PreparedChainSign {
    pub nonce: u32,
    /// 给 QR `b.d` 的完整审阅载荷字节；不得用 32 字节签名哈希替代。
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

fn public_key_to_ss58(public_key: &str) -> Result<String, String> {
    let public = chain_signing::parse_sr25519_public_key(public_key)?;
    Ok(public.to_ss58check_with_version(sp_core::crypto::Ss58AddressFormat::custom(2027)))
}

/// 实时读链上 nonce(死规则 P-SIGN-001:nonce 只来自链,不缓存不自增)。
pub(crate) async fn fetch_nonce(public_key: &str) -> Result<u32, String> {
    let ss58 = public_key_to_ss58(public_key)?;
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

/// prepare:实时取 nonce/版本/创世哈希,构建 QR 审阅载荷与签名校验哈希。
pub(crate) async fn prepare_signing(
    call_data: &[u8],
    signer_public_key: &str,
) -> Result<PreparedChainSign, String> {
    let nonce = fetch_nonce(signer_public_key).await?;
    let (spec_version, tx_version) = fetch_runtime_version().await?;
    let genesis_hash = fetch_genesis_hash().await?;
    let material = chain_signing::build_signing_material(
        call_data,
        &genesis_hash,
        nonce,
        spec_version,
        tx_version,
    )?;
    Ok(PreparedChainSign {
        nonce,
        signing_hash_hex: chain_signing::sha256_hex(&material.signing_bytes),
        payload: material.payload,
    })
}

/// submit:重建材料校验哈希 → 本地验签 → dry-run → 提交,返回交易哈希。
pub(crate) async fn assemble_and_submit(
    call_data: &[u8],
    signer_public_key: &str,
    signature_hex: &str,
    nonce: u32,
    expected_signing_hash_hex: &str,
) -> Result<String, String> {
    let (spec_version, tx_version) = fetch_runtime_version().await?;
    let genesis_hash = fetch_genesis_hash().await?;
    let material = chain_signing::build_signing_material(
        call_data,
        &genesis_hash,
        nonce,
        spec_version,
        tx_version,
    )?;
    // 会话期间 runtime 版本/创世哈希不得漂移,否则签名对不上载荷。
    if chain_signing::sha256_hex(&material.signing_bytes) != expected_signing_hash_hex {
        return Err("签名载荷与会话不一致(runtime 版本或创世哈希已变化),请重新发起".into());
    }

    let public = chain_signing::parse_sr25519_public_key(signer_public_key)?;
    let signature = chain_signing::parse_sr25519_signature_hex(signature_hex)?;
    if !chain_signing::verify_signature(&material, &signature, &public) {
        return Err("sr25519 本地验签失败,拒绝提交".to_string());
    }

    let extrinsic = chain_signing::assemble_signed_extrinsic(material, public, signature);
    let extrinsic_hex = chain_signing::signed_extrinsic_hex(&extrinsic);

    // dry-run 是提交前硬预检。任何 RuntimeApi trap、交易无效或 RPC 不可用都必须
    // 停止提交，禁止跳过预检后把 wasm panic 原样推给浏览器。
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
                return Err(chain_signing::dry_run_reject_message(&bytes, raw));
            }
            if bytes.len() > 1 && bytes[1] != 0x00 {
                return Err(format!("交易执行会失败: DispatchError (hex: {s})"));
            }
        }
        Err(e) => {
            return Err(chain_signing::preflight_reject_message(&e));
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
/// 即已被打包(InBestBlock 可靠代理)。观察窗口结束只表示本次请求尚未确认，
/// 不得解释为 PoW 已超过最晚出块时间或交易必然失效。
pub(crate) async fn wait_nonce_consumed(
    public_key: &str,
    submitted_nonce: u32,
) -> Result<(), String> {
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_secs(WAIT_CONFIRMATION_OBSERVATION_SECS);
    loop {
        let current = fetch_nonce(public_key).await?;
        if current > submitted_nonce {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!(
                "交易确认观察窗口已结束({WAIT_CONFIRMATION_OBSERVATION_SECS} 秒):nonce {submitted_nonce} 尚未消费，交易仍可能继续等待进块"
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

#[cfg(test)]
mod tests {
    use citizenchain as runtime;
    use codec::Encode;

    /// 材料构建离线自洽:同输入同产物,call 解码回等值,>256B 审阅载荷仍完整保留。
    #[test]
    fn signing_material_roundtrip_and_hash_rule() {
        let call = runtime::RuntimeCall::System(frame_system::Call::remark {
            remark: vec![7u8; 8],
        });
        let call_data = call.encode();
        let genesis = [9u8; 32];
        let m =
            chain_signing::build_signing_material(&call_data, &genesis, 5, 1, 1).expect("material");
        assert_eq!(m.call.encode(), call_data);
        // 小载荷:签名输入 == 审阅 payload 本体。
        assert!(m.payload.len() <= 256);
        assert_eq!(m.signing_bytes, m.payload);

        let big_call = runtime::RuntimeCall::System(frame_system::Call::remark {
            remark: vec![7u8; 400],
        });
        let big = chain_signing::build_signing_material(&big_call.encode(), &genesis, 5, 1, 1)
            .expect("material");
        // 大载荷:QR 仍必须拿到完整审阅 payload；只有实际签名输入是 32 字节 blake2_256。
        assert!(big.payload.len() > 256);
        assert_eq!(big.signing_bytes.len(), 32);
        assert_ne!(big.payload, big.signing_bytes);
    }

    #[test]
    fn tail_data_in_call_is_rejected() {
        let call = runtime::RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
        let mut data = call.encode();
        data.push(0xff);
        assert!(chain_signing::decode_runtime_call(&data).is_err());
    }
}
