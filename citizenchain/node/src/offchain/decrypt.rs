// 清算行管理员密钥解密(unlock)流程。
//
// 与 governance/activation.rs 中的"激活"语义一致——wumin 冷钱包扫码签 challenge,
// 节点本地验签——但本流程**仅在清算行 tab 用**,术语为"解密"以区别于 NRC/PRC/PRB
// 的"激活"。区别:
// - 激活(activation):写入 activated-admins.json 长期持久化
// - 解密(decrypt):仅写入内存 HashMap,节点重启自动清空,无 TTL
//
// 实际私钥的 AES-GCM 加密文件由 CLI 启动路径(`--clearing-bank-password`)生成,
// `offchain::keystore::OffchainKeystore` 加载到 `KeystoreBatchSigner` 的
// `Arc<RwLock<Option<SigningKey>>>` 槽位。本模块的"解密"含义是:
//   1. wumin 签 challenge → 节点 sr25519 验签 → 证明操作员持有该公钥的冷钱包
//   2. 把 (pubkey, sfid_id) 标记为内存内"授权可用",packer 攒批前 cross-check
//      该入口存在才会启动签名(防误用启动密码加载的 SigningKey)
//
// Step 3(wumin/wuminapp 完工后)再做完整的"per-admin 加密 seed 文件 +
// challenge-derived AES key"模型;Step 2 先把 UI 与协议跑通即可。

use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::governance::signing::{
    pubkey_to_ss58, sha256_hash_public, QrSignRequest, QrSignResponse, SignRequestBody,
};

use super::types::{DecryptAdminRequestResult, DecryptedAdminInfo};

const PROTOCOL_VERSION: &str = "WUMIN_QR_V1";
const DECRYPT_PREFIX: &[u8; 14] = b"GMB_DECRYPT_V1";
/// challenge payload 长度:14 + 48 + 32 + 8 + 16 = 118 字节
const CHALLENGE_TOTAL_LEN: usize = 14 + 48 + 32 + 8 + 16;
const DEFAULT_TTL_SECS: u64 = 90;

/// 当前正在内存中"已解密"的管理员表(节点重启清空)。
///
/// key = lowercase pubkey hex(不含 0x),value = (sfid_id, decrypted_at_ms)。
static DECRYPTED_ADMINS: OnceLock<Mutex<HashMap<String, MemoryEntry>>> = OnceLock::new();

/// 等待"解密"响应的进行中 challenge 上下文。前端拿到 request_id,扫描回执时
/// 由 verify 阶段从此表查回 payload 做本地验签。
static PENDING_CHALLENGES: OnceLock<Mutex<HashMap<String, ChallengeContext>>> = OnceLock::new();

#[derive(Clone)]
struct MemoryEntry {
    sfid_id: String,
    decrypted_at_ms: u64,
}

#[derive(Clone)]
struct ChallengeContext {
    pubkey_hex: String,
    sfid_id: String,
    payload: Vec<u8>,
    issued_at_ms: u64,
}

fn decrypted_map() -> &'static Mutex<HashMap<String, MemoryEntry>> {
    DECRYPTED_ADMINS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn pending_map() -> &'static Mutex<HashMap<String, ChallengeContext>> {
    PENDING_CHALLENGES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 拼装 challenge payload:`PREFIX(14) || sfid_id(48 padded) || pubkey(32) || ts_le(8) || nonce(16)`。
fn build_challenge_payload(pubkey_bytes: &[u8; 32], sfid_id: &str, timestamp: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(CHALLENGE_TOTAL_LEN);
    out.extend_from_slice(DECRYPT_PREFIX);

    let id_bytes = sfid_id.as_bytes();
    let mut id_buf = [0u8; 48];
    let copy_len = id_bytes.len().min(48);
    id_buf[..copy_len].copy_from_slice(&id_bytes[..copy_len]);
    out.extend_from_slice(&id_buf);

    out.extend_from_slice(pubkey_bytes);
    out.extend_from_slice(&timestamp.to_le_bytes());

    let nonce: [u8; 16] = rand::thread_rng().gen();
    out.extend_from_slice(&nonce);
    out
}

fn generate_request_id() -> String {
    let bytes: [u8; 16] = rand::thread_rng().gen();
    format!("decrypt-{}", hex::encode(bytes))
}

/// 构造解密请求 QR JSON,把 ChallengeContext 暂存以备验签。
pub fn build_decrypt_admin_request(
    pubkey_hex: &str,
    sfid_id: &str,
) -> Result<DecryptAdminRequestResult, String> {
    let clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    if clean.len() != 64 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效,应为 64 位十六进制".to_string());
    }
    if sfid_id.is_empty() || sfid_id.len() > 48 {
        return Err("sfid_id 长度需在 1..=48".to_string());
    }
    let pubkey_bytes = hex::decode(&clean).map_err(|e| format!("公钥解码失败:{e}"))?;
    let pubkey_arr: [u8; 32] = pubkey_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "公钥长度必须为 32 字节".to_string())?;

    let timestamp = now_secs();
    let payload = build_challenge_payload(&pubkey_arr, sfid_id, timestamp);
    let payload_hex = format!("0x{}", hex::encode(&payload));
    let payload_hash = sha256_hash_public(&payload);
    let payload_hash_hex = format!("0x{}", hex::encode(payload_hash));
    let request_id = generate_request_id();
    let account_ss58 = pubkey_to_ss58(&pubkey_bytes)?;

    // display.fields 提供给 wumin decoder 构造确认页(action=decrypt_admin)。
    // Step 3 wumin 端补对应 decoder 分支后,签名页文案为"解密管理员 - {sfid_id}"。
    let display = serde_json::json!({
        "action": "decrypt_admin",
        "summary": format!("解密清算行管理员 - {sfid_id}"),
        "fields": [
            { "key": "sfid_id", "label": "机构身份码", "value": sfid_id }
        ]
    });

    let now = now_secs();
    let request = QrSignRequest {
        proto: PROTOCOL_VERSION.to_string(),
        kind: "sign_request".to_string(),
        id: request_id.clone(),
        issued_at: now,
        expires_at: now + DEFAULT_TTL_SECS,
        body: SignRequestBody {
            address: account_ss58,
            pubkey: format!("0x{clean}"),
            sig_alg: "sr25519".to_string(),
            payload_hex: payload_hex.clone(),
            spec_version: 0,
            display,
        },
    };

    let request_json =
        serde_json::to_string(&request).map_err(|e| format!("序列化签名请求失败:{e}"))?;

    pending_map()
        .lock()
        .map_err(|_| "decrypt 待处理表锁异常".to_string())?
        .insert(
            request_id.clone(),
            ChallengeContext {
                pubkey_hex: clean,
                sfid_id: sfid_id.to_string(),
                payload,
                issued_at_ms: now_ms(),
            },
        );

    Ok(DecryptAdminRequestResult {
        request_json,
        request_id,
        expected_payload_hash: payload_hash_hex,
        payload_hex,
    })
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyDecryptAdminInput {
    pub request_id: String,
    pub pubkey_hex: String,
    pub expected_payload_hash: String,
    pub response_json: String,
}

/// 验证 wumin 签名响应,通过则把 (pubkey, sfid_id) 写入内存解密表。
pub fn verify_and_decrypt_admin(
    input: VerifyDecryptAdminInput,
) -> Result<DecryptedAdminInfo, String> {
    let response: QrSignResponse =
        serde_json::from_str(&input.response_json).map_err(|e| format!("解析签名响应失败:{e}"))?;

    if response.proto != PROTOCOL_VERSION {
        return Err(format!(
            "协议版本不匹配:期望 {PROTOCOL_VERSION},实际 {}",
            response.proto
        ));
    }
    if response.id != input.request_id {
        return Err("请求 ID 不匹配".to_string());
    }

    let pubkey_clean = input
        .pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(&input.pubkey_hex)
        .to_ascii_lowercase();
    let response_pubkey = response
        .body
        .pubkey
        .strip_prefix("0x")
        .unwrap_or(&response.body.pubkey)
        .to_ascii_lowercase();
    if response_pubkey != pubkey_clean {
        return Err("公钥不匹配".to_string());
    }

    let expected_hash = input
        .expected_payload_hash
        .strip_prefix("0x")
        .unwrap_or(&input.expected_payload_hash)
        .to_ascii_lowercase();
    let response_hash = response
        .body
        .payload_hash
        .strip_prefix("0x")
        .unwrap_or(&response.body.payload_hash)
        .to_ascii_lowercase();
    if response_hash != expected_hash {
        return Err("payload hash 不匹配,签名数据可能被篡改".to_string());
    }

    // 拉回原 challenge payload 做本地 sr25519 验签。
    let context = {
        let mut guard = pending_map()
            .lock()
            .map_err(|_| "decrypt 待处理表锁异常".to_string())?;
        guard
            .remove(&input.request_id)
            .ok_or_else(|| "未找到对应的 challenge 上下文(已过期或被消费)".to_string())?
    };
    if context.pubkey_hex != pubkey_clean {
        return Err("challenge 上下文公钥与请求不一致".to_string());
    }

    // expected_payload_hash 必须等于 SHA-256(payload)
    let local_hash = {
        let mut h = Sha256::new();
        h.update(&context.payload);
        let r = h.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&r);
        format!("0x{}", hex::encode(out))
    };
    if local_hash != format!("0x{expected_hash}") {
        return Err("本地重新计算的 payload hash 与请求不一致".to_string());
    }

    // sr25519 验签
    let sig_hex = response
        .body
        .signature
        .strip_prefix("0x")
        .unwrap_or(&response.body.signature);
    if sig_hex.len() != 128 {
        return Err(format!("签名长度无效:期望 128 hex,实际 {}", sig_hex.len()));
    }
    let signature_bytes = hex::decode(sig_hex).map_err(|e| format!("签名解码失败:{e}"))?;
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败:{e}"))?;
    use sp_core::crypto::Pair;
    use sp_core::sr25519::{Public, Signature};
    let public = Public::from_raw(
        <[u8; 32]>::try_from(pubkey_bytes.as_slice()).map_err(|_| "公钥长度必须为 32 字节")?,
    );
    let signature = Signature::from_raw(
        <[u8; 64]>::try_from(signature_bytes.as_slice()).map_err(|_| "签名长度必须为 64 字节")?,
    );
    if !sp_core::sr25519::Pair::verify(&signature, &context.payload, &public) {
        return Err("sr25519 签名验证失败,无法证明对该公钥的控制".to_string());
    }

    let now = now_ms();
    decrypted_map()
        .lock()
        .map_err(|_| "decrypt 内存表锁异常".to_string())?
        .insert(
            pubkey_clean.clone(),
            MemoryEntry {
                sfid_id: context.sfid_id.clone(),
                decrypted_at_ms: now,
            },
        );

    log::info!(
        "[ClearingBank] 管理员 {} 已解密(sfid={},耗时 {} ms)",
        &pubkey_clean[..8],
        context.sfid_id,
        now.saturating_sub(context.issued_at_ms),
    );

    Ok(DecryptedAdminInfo {
        pubkey_hex: format!("0x{pubkey_clean}"),
        sfid_id: context.sfid_id,
        decrypted_at_ms: now,
    })
}

/// 列出某机构当前在内存中已解密的管理员。
pub fn list_decrypted_admins(sfid_id: &str) -> Vec<DecryptedAdminInfo> {
    let guard = match decrypted_map().lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    guard
        .iter()
        .filter(|(_, v)| v.sfid_id == sfid_id)
        .map(|(k, v)| DecryptedAdminInfo {
            pubkey_hex: format!("0x{k}"),
            sfid_id: v.sfid_id.clone(),
            decrypted_at_ms: v.decrypted_at_ms,
        })
        .collect()
}

/// 将某管理员从内存解密表移除。前端"重新加锁"用。
pub fn lock_decrypted_admin(pubkey_hex: &str) -> Result<(), String> {
    let clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    let mut guard = decrypted_map()
        .lock()
        .map_err(|_| "decrypt 内存表锁异常".to_string())?;
    if guard.remove(&clean).is_none() {
        return Err("该公钥未在解密状态".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_payload_layout_is_118_bytes() {
        let p = build_challenge_payload(&[0xAA; 32], "SFR-TEST", 1234567890);
        assert_eq!(p.len(), CHALLENGE_TOTAL_LEN);
        assert_eq!(&p[..14], DECRYPT_PREFIX);
    }

    #[test]
    fn challenge_payload_pubkey_position() {
        let p = build_challenge_payload(&[0xCC; 32], "FFR-X", 0);
        assert_eq!(&p[14 + 48..14 + 48 + 32], &[0xCC; 32]);
    }

    #[test]
    fn build_decrypt_admin_request_rejects_short_pubkey() {
        let err = build_decrypt_admin_request("0xAA", "SFR-X").unwrap_err();
        assert!(err.contains("公钥格式"));
    }

    #[test]
    fn list_decrypted_admins_filters_by_sfid() {
        decrypted_map().lock().unwrap().insert(
            "aa".repeat(32),
            MemoryEntry {
                sfid_id: "SFR-A".to_string(),
                decrypted_at_ms: 1,
            },
        );
        decrypted_map().lock().unwrap().insert(
            "bb".repeat(32),
            MemoryEntry {
                sfid_id: "SFR-B".to_string(),
                decrypted_at_ms: 2,
            },
        );
        let r = list_decrypted_admins("SFR-A");
        assert_eq!(r.len(), 1);
        assert!(r[0].pubkey_hex.contains("aa"));

        // 清理(避免污染其他 case)
        decrypted_map().lock().unwrap().clear();
    }

    #[test]
    fn lock_decrypted_admin_removes_entry() {
        decrypted_map().lock().unwrap().insert(
            "cc".repeat(32),
            MemoryEntry {
                sfid_id: "SFR-C".to_string(),
                decrypted_at_ms: 10,
            },
        );
        assert!(lock_decrypted_admin(&format!("0x{}", "cc".repeat(32))).is_ok());
        assert!(lock_decrypted_admin(&format!("0x{}", "cc".repeat(32))).is_err());
    }
}
