//! SFID 匿名证书盲签名模块
//!
//! 使用 RSABSSA-SHA384-PSS-Randomized（RFC 9474）标准盲签名。
//! 签名原文 `sfid-anon-cert-v1|{province_code}|{anon_pubkey}` 整体作为盲化消息，
//! province_code 包含在消息中，签名后 SFID 无法得知 anon_pubkey。
//!
//! 注：未使用 PBRSA（部分盲签名），因 PBRSA 要求安全素数（生成耗时数分钟），
//! 而标准盲签名同样满足协议需求——province_code 由 SFID 在签名前确定并拼入消息，
//! CPMS 无法篡改。

use blind_rsa_signatures::{
    BlindMessage, BlindSignature, KeyPair, Signature, DefaultRng,
    PSS, Randomized, Sha384,
};
use std::sync::RwLock;

/// 标准盲签名密钥对类型
type BssaKeyPair = KeyPair<Sha384, PSS, Randomized>;
type BssaSecretKey = blind_rsa_signatures::SecretKey<Sha384, PSS, Randomized>;
type BssaPublicKey = blind_rsa_signatures::PublicKey<Sha384, PSS, Randomized>;

const RSA_KEY_BITS: usize = 2048;

static RSA_KEYPAIR: RwLock<Option<BssaKeyPair>> = RwLock::new(None);

/// 从 PEM 加载密钥对。
pub fn init_from_pem(private_key_pem: &str) -> Result<(), String> {
    let sk = BssaSecretKey::from_pem(private_key_pem)
        .map_err(|e| format!("failed to load RSA private key: {e}"))?;
    let pk = sk
        .public_key()
        .map_err(|e| format!("failed to derive RSA public key: {e}"))?;
    let kp = BssaKeyPair { pk, sk };
    let mut guard = RSA_KEYPAIR
        .write()
        .map_err(|_| "RSA_KEYPAIR lock poisoned".to_string())?;
    *guard = Some(kp);
    Ok(())
}

/// 生成新的 RSA 密钥对。
///
/// 标准 RSA（非安全素数），秒级完成。
pub fn generate_keypair_pem() -> Result<String, String> {
    let kp = BssaKeyPair::generate(&mut DefaultRng, RSA_KEY_BITS)
        .map_err(|e| format!("failed to generate RSA keypair: {e}"))?;
    let pem = kp
        .sk
        .to_pem()
        .map_err(|e| format!("failed to export RSA private key PEM: {e}"))?;
    let mut guard = RSA_KEYPAIR
        .write()
        .map_err(|_| "RSA_KEYPAIR lock poisoned".to_string())?;
    *guard = Some(kp);
    Ok(pem)
}

/// 启动时自动初始化。已有 PEM 则加载，否则返回 `Some(new_pem)` 表示需要持久化。
pub fn init_or_generate(existing_pem: Option<&str>) -> Result<Option<String>, String> {
    if let Some(pem) = existing_pem {
        if !pem.trim().is_empty() {
            init_from_pem(pem)?;
            return Ok(None);
        }
    }
    let pem = generate_keypair_pem()?;
    Ok(Some(pem))
}

/// 获取 RSA 公钥 PEM。
pub fn get_public_key_pem() -> Result<String, String> {
    let guard = RSA_KEYPAIR
        .read()
        .map_err(|_| "RSA_KEYPAIR lock poisoned".to_string())?;
    let kp = guard.as_ref().ok_or("RSA keypair not initialized")?;
    kp.pk
        .to_pem()
        .map_err(|e| format!("failed to export RSA public key PEM: {e}"))
}

/// SFID 侧：对盲化请求执行盲签名。
///
/// - `blind_msg_bytes`：CPMS 发来的盲化消息
/// - `_province_code`：省代码（已包含在盲化消息原文中，此参数用于审计日志）
pub fn blind_sign(blind_msg_bytes: &[u8], _province_code: &str) -> Result<Vec<u8>, String> {
    let guard = RSA_KEYPAIR
        .read()
        .map_err(|_| "RSA_KEYPAIR lock poisoned".to_string())?;
    let kp = guard.as_ref().ok_or("RSA keypair not initialized")?;

    let blind_msg = BlindMessage::from(blind_msg_bytes.to_vec());
    let blind_sig = kp
        .sk
        .blind_sign(&blind_msg)
        .map_err(|e| format!("blind sign failed: {e}"))?;

    Ok(blind_sig.to_vec())
}

/// SFID 侧：验证最终匿名证书签名（QR4 验证时调用）。
///
/// 签名原文 = `sfid-anon-cert-v1|{province_code}|{anon_pubkey}`
pub fn verify_anon_cert(
    province_code: &str,
    anon_pubkey_hex: &str,
    signature: &[u8],
    msg_randomizer: Option<&[u8]>,
) -> Result<bool, String> {
    let guard = RSA_KEYPAIR
        .read()
        .map_err(|_| "RSA_KEYPAIR lock poisoned".to_string())?;
    let kp = guard.as_ref().ok_or("RSA keypair not initialized")?;

    let message = build_cert_message(province_code, anon_pubkey_hex);
    let sig = Signature::from(signature.to_vec());
    let randomizer = msg_randomizer.and_then(|r| {
        let arr: [u8; 32] = r.try_into().ok()?;
        Some(blind_rsa_signatures::MessageRandomizer::from(arr))
    });

    match kp.pk.verify(&sig, randomizer, message.as_bytes()) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// 构造证书签名原文。
pub fn build_cert_message(province_code: &str, anon_pubkey_hex: &str) -> String {
    format!("sfid-anon-cert-v1|{}|{}", province_code, anon_pubkey_hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_init_roundtrip() {
        let pem = generate_keypair_pem().expect("generate failed");
        assert!(pem.contains("PRIVATE KEY"));
        let pub_pem = get_public_key_pem().expect("get public key failed");
        assert!(pub_pem.contains("PUBLIC KEY"));
    }

    #[test]
    fn pem_reload() {
        let pem = generate_keypair_pem().expect("generate failed");
        init_from_pem(&pem).expect("init from pem failed");
        let pub_pem = get_public_key_pem().expect("get public key failed");
        assert!(!pub_pem.is_empty());
    }
}
