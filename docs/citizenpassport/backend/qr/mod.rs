//! QR_V1 统一二维码协议 envelope。
//!
//! 唯一事实源: `memory/01-architecture/qr/qr-protocol-spec.md`。
//! CPMS 后端只生成登录签名请求和档案删除签名请求。

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};

pub const QR_V1: &str = "QR_V1";

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrKind {
    SignRequest = 1,
    SignResponse = 2,
    UserContact = 3,
    UserTransfer = 4,
}

impl QrKind {
    pub fn code(self) -> u8 {
        self as u8
    }
}

pub const ACTION_LOGIN: u16 = 1;
pub const ACTION_CPMS_ARCHIVE_DELETE: u16 = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignRequestBody {
    /// a:动作码。登录=1,CPMS 档案删除=4。
    #[serde(rename = "a")]
    pub action: u16,
    /// g:签名算法。1 固定为 sr25519。
    #[serde(rename = "g")]
    pub sig_alg: u8,
    /// u:签名者/系统公钥,32B base64url(no padding)。
    #[serde(rename = "u")]
    pub pubkey: String,
    /// d:待签 payload bytes 的 base64url(no padding)。
    #[serde(rename = "d")]
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrEnvelope<B> {
    #[serde(rename = "p")]
    pub proto: String,
    #[serde(rename = "k")]
    pub kind: u8,
    #[serde(rename = "i", skip_serializing_if = "Option::is_none", default)]
    pub id: Option<String>,
    #[serde(rename = "e", skip_serializing_if = "Option::is_none", default)]
    pub expires_at: Option<i64>,
    #[serde(rename = "b")]
    pub body: B,
}

pub type SignRequestEnvelope = QrEnvelope<SignRequestBody>;

impl SignRequestEnvelope {
    pub fn new(id: String, _issued_at: i64, expires_at: i64, body: SignRequestBody) -> Self {
        Self {
            proto: QR_V1.to_string(),
            kind: QrKind::SignRequest.code(),
            id: Some(id),
            expires_at: Some(expires_at),
            body,
        }
    }
}

/// 登录签名请求 payload 固定为 `system|sys_sig` 的 UTF-8 字节。
pub fn login_request_body(system: &str, sys_pubkey: &str, sys_sig: &str) -> SignRequestBody {
    SignRequestBody {
        action: ACTION_LOGIN,
        sig_alg: 1,
        pubkey: pubkey_hex_to_b64(sys_pubkey).unwrap_or_default(),
        payload: bytes_to_b64(format!("{}|{}", system, sys_sig).as_bytes()),
    }
}

/// 唯一的签名原文拼接函数。
///
/// 格式(与 Dart/TS 逐字节一致):
/// ```text
/// QR_V1|<k>|<id>|<system 或空>|<expires_at 或 0>|<principal>
/// ```
/// `principal` 去掉 `0x` 前缀,小写。
pub fn build_signature_message(
    kind: QrKind,
    id: &str,
    system: Option<&str>,
    expires_at: Option<i64>,
    principal: &str,
) -> String {
    let sys = system.unwrap_or("");
    let exp = expires_at.unwrap_or(0);
    let pp = normalize_hex_no_prefix(principal);
    format!("{}|{}|{}|{}|{}|{}", QR_V1, kind.code(), id, sys, exp, pp)
}

pub(crate) fn pubkey_hex_to_b64(value: &str) -> Option<String> {
    let cleaned = normalize_hex_no_prefix(value);
    let bytes = hex::decode(cleaned).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    Some(URL_SAFE_NO_PAD.encode(bytes))
}

pub(crate) fn bytes_to_b64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn normalize_hex_no_prefix(value: &str) -> String {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value)
        .to_lowercase()
}
