//! QR_V1 载荷字段编解码唯一真源。
//!
//! QR body 中的公钥、签名等定长字节字段统一用 base64url(no padding)承载;
//! 进入 Rust 侧后一律转成 ADR-040 规范文本(小写 `0x` + 十六进制)。
//!
//! 本模块是四端 host 侧的唯一实现:`node`(桌面端验签)与 `onchina`(控制台扫码)
//! 必须复用这里的函数,禁止各自再写一份解码。两份实现一旦在长度校验或大小写
//! 上产生分毫差异,同一个二维码就会在一端通过、另一端拒绝。

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

/// base64url 定长字段的解码失败原因。
///
/// 只描述"哪个字段、错在哪",不绑定任何调用方的错误类型;
/// `node` 映射成 `String`,`onchina` 映射成 `QrParseError::BadField`。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CodecError {
    /// 字段不是合法的 base64url(no padding)。
    #[error("{field} 必须为 base64url(no padding)")]
    NotBase64Url { field: String },
    /// 字段解码后字节长度与协议约定不符。
    #[error("{field} 长度无效:期望 {expected} 字节,实际 {actual} 字节")]
    BadLength {
        field: String,
        expected: usize,
        actual: usize,
    },
}

/// base64url(no padding)定长字段 → ADR-040 规范文本(小写 `0x` + 十六进制)。
///
/// `expected_len` 是解码后的**字节**数(公钥 32、签名 64);`field` 用于错误信息定位,
/// 传 QR body 中的字段名(如 `b.u`、`b.s`)。
pub fn b64_to_prefixed_hex(
    value: &str,
    expected_len: usize,
    field: &str,
) -> Result<String, CodecError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| CodecError::NotBase64Url {
            field: field.to_string(),
        })?;
    if bytes.len() != expected_len {
        return Err(CodecError::BadLength {
            field: field.to_string(),
            expected: expected_len,
            actual: bytes.len(),
        });
    }
    Ok(format!("0x{}", hex::encode(bytes)))
}

/// 字节 → base64url(no padding)。QR body 写入侧的唯一编码入口。
pub fn bytes_to_b64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// 32 字节公钥 → base64url(no padding);长度不符即拒绝,不做截断或补齐。
pub fn public_key_b64(public_key_bytes: &[u8], field: &str) -> Result<String, CodecError> {
    if public_key_bytes.len() != PUBLIC_KEY_BYTES {
        return Err(CodecError::BadLength {
            field: field.to_string(),
            expected: PUBLIC_KEY_BYTES,
            actual: public_key_bytes.len(),
        });
    }
    Ok(bytes_to_b64(public_key_bytes))
}

/// 公钥字节长度(sr25519 / ed25519 均为 32)。
pub const PUBLIC_KEY_BYTES: usize = 32;
/// 签名字节长度(sr25519 / ed25519 均为 64)。
pub const SIGNATURE_BYTES: usize = 64;

#[cfg(test)]
// 编解码夹具异常必须立即中止测试,断言式解包仅限本测试模块。
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn decodes_public_key_to_lowercase_prefixed_hex() {
        let raw = [0xABu8; PUBLIC_KEY_BYTES];
        let encoded = bytes_to_b64(&raw);
        let hex_text = b64_to_prefixed_hex(&encoded, PUBLIC_KEY_BYTES, "b.u")
            .expect("32 字节公钥必须解码成功");
        assert_eq!(hex_text, format!("0x{}", "ab".repeat(PUBLIC_KEY_BYTES)));
    }

    #[test]
    fn round_trips_signature_length() {
        let raw = [0x01u8; SIGNATURE_BYTES];
        let encoded = bytes_to_b64(&raw);
        assert!(b64_to_prefixed_hex(&encoded, SIGNATURE_BYTES, "b.s").is_ok());
    }

    #[test]
    fn rejects_wrong_length_without_truncating() {
        let encoded = bytes_to_b64(&[0u8; 31]);
        assert_eq!(
            b64_to_prefixed_hex(&encoded, PUBLIC_KEY_BYTES, "b.u"),
            Err(CodecError::BadLength {
                field: "b.u".to_string(),
                expected: 32,
                actual: 31,
            })
        );
    }

    #[test]
    fn rejects_non_base64url_input() {
        // 标准 base64 的 `+` `/` 与 padding `=` 都不属于 base64url(no padding)。
        assert_eq!(
            b64_to_prefixed_hex("++//", 32, "b.u"),
            Err(CodecError::NotBase64Url {
                field: "b.u".to_string()
            })
        );
    }

    #[test]
    fn public_key_b64_rejects_wrong_length() {
        assert!(public_key_b64(&[0u8; 33], "b.u").is_err());
        assert!(public_key_b64(&[0u8; PUBLIC_KEY_BYTES], "b.u").is_ok());
    }
}
