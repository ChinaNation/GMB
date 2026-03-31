//! CPMS 端 RSABSSA 盲签名客户端模块
//!
//! 标准 RSABSSA-SHA384-PSS-Randomized（RFC 9474）。
//! 签名原文 `sfid-anon-cert-v1|{province_code}|{anon_pubkey}` 整体盲化。

use blind_rsa_signatures::{
    BlindSignature, BlindingResult, MessageRandomizer, Signature,
    PSS, Randomized, Sha384, DefaultRng,
};

type BssaPublicKey = blind_rsa_signatures::PublicKey<Sha384, PSS, Randomized>;

/// 盲化结果。
pub(crate) struct BlindingOutput {
    pub(crate) blind_msg_hex: String,
    pub(crate) blinding_secret: Vec<u8>,
}

/// 解盲结果。
pub(crate) struct FinalizedSignature {
    pub(crate) signature_hex: String,
    pub(crate) msg_randomizer_hex: Option<String>,
}

fn parse_rsa_pubkey(pem: &str) -> Result<BssaPublicKey, String> {
    BssaPublicKey::from_pem(pem).map_err(|e| format!("parse SFID RSA public key failed: {e}"))
}

/// 盲化消息。
///
/// 签名原文 = `sfid-anon-cert-v1|{province_code}|{anon_pubkey}`
pub(crate) fn blind_message(
    rsa_public_key_pem: &str,
    anon_pubkey_hex: &str,
    province_code: &str,
) -> Result<BlindingOutput, String> {
    let pk = parse_rsa_pubkey(rsa_public_key_pem)?;
    let message = format!("sfid-anon-cert-v1|{}|{}", province_code, anon_pubkey_hex);

    let blinding_result = pk
        .blind(&mut DefaultRng, message.as_bytes())
        .map_err(|e| format!("blind failed: {e}"))?;

    let blind_msg_hex = hex::encode(&blinding_result.blind_message.0);
    let blinding_secret = serialize_blinding_result(&blinding_result);

    Ok(BlindingOutput {
        blind_msg_hex,
        blinding_secret,
    })
}

/// 解盲签名。
pub(crate) fn finalize_signature(
    rsa_public_key_pem: &str,
    blind_sig_hex: &str,
    blinding_secret: &[u8],
    anon_pubkey_hex: &str,
    province_code: &str,
) -> Result<FinalizedSignature, String> {
    let pk = parse_rsa_pubkey(rsa_public_key_pem)?;
    let message = format!("sfid-anon-cert-v1|{}|{}", province_code, anon_pubkey_hex);

    let blind_sig_bytes =
        hex::decode(blind_sig_hex.trim().trim_start_matches("0x"))
            .map_err(|_| "blind_sig hex decode failed".to_string())?;
    let blind_sig = BlindSignature::from(blind_sig_bytes);
    let blinding_result = deserialize_blinding_result(blinding_secret)?;

    let sig = pk
        .finalize(&blind_sig, &blinding_result, message.as_bytes())
        .map_err(|e| format!("finalize failed: {e}"))?;

    let msg_randomizer_hex = blinding_result
        .msg_randomizer
        .as_ref()
        .map(|r| hex::encode(r.0));

    Ok(FinalizedSignature {
        signature_hex: format!("0x{}", hex::encode(sig.0)),
        msg_randomizer_hex,
    })
}

fn serialize_blinding_result(result: &BlindingResult) -> Vec<u8> {
    let mut out = Vec::new();
    let secret_bytes = &result.secret.0;
    out.extend_from_slice(&(secret_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(secret_bytes);
    out.extend_from_slice(&(result.blind_message.0.len() as u32).to_le_bytes());
    out.extend_from_slice(&result.blind_message.0);
    match &result.msg_randomizer {
        Some(r) => {
            out.push(1);
            out.extend_from_slice(&r.0);
        }
        None => out.push(0),
    }
    out
}

fn deserialize_blinding_result(data: &[u8]) -> Result<BlindingResult, String> {
    let mut pos = 0;
    if data.len() < 4 {
        return Err("blinding_secret too short".to_string());
    }
    let secret_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if data.len() < pos + secret_len + 4 {
        return Err("blinding_secret truncated at secret".to_string());
    }
    let secret = blind_rsa_signatures::Secret::from(data[pos..pos + secret_len].to_vec());
    pos += secret_len;
    let blind_msg_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    if data.len() < pos + blind_msg_len + 1 {
        return Err("blinding_secret truncated at blind_msg".to_string());
    }
    let blind_message =
        blind_rsa_signatures::BlindMessage::from(data[pos..pos + blind_msg_len].to_vec());
    pos += blind_msg_len;
    let msg_randomizer = if data[pos] == 1 {
        pos += 1;
        if data.len() < pos + 32 {
            return Err("blinding_secret truncated at randomizer".to_string());
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&data[pos..pos + 32]);
        Some(MessageRandomizer::from(arr))
    } else {
        None
    };

    Ok(BlindingResult {
        blind_message,
        secret,
        msg_randomizer,
    })
}
