//! QR_V1 统一二维码协议 envelope。
//!
//! 唯一事实源: `memory/01-architecture/qr/qr-protocol-spec.md`。
//! 本模块只保留 OnChina 后端需要的紧凑签名请求/响应结构。

mod sign_request;

pub(crate) use sign_request::{build_sign_request, build_sign_request_bytes};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub const QR_V1: &str = "QR_V1";

/// QR_V1 顶层 k 字段。登录也复用签名请求/响应场景。
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

fn registry_action_code(action_key: &str) -> u16 {
    qr_protocol::action_by_key(action_key)
        .unwrap_or_else(|error| panic!("QR action registry 缺少 {action_key}: {error}"))
        .action_code
}

pub(crate) fn action_label_zh(action_key: &str) -> String {
    qr_protocol::action_by_key(action_key)
        .unwrap_or_else(|error| panic!("QR action registry 缺少 {action_key}: {error}"))
        .action_label_zh
}

static ACTION_LOGIN_CODE: LazyLock<u16> = LazyLock::new(|| registry_action_code("login"));
static ACTION_CITIZEN_IDENTITY_CODE: LazyLock<u16> =
    LazyLock::new(|| registry_action_code("citizen_identity"));
static ACTION_ONCHINA_ADMIN_CODE: LazyLock<u16> =
    LazyLock::new(|| registry_action_code("onchina_admin_action"));

pub(crate) fn action_login() -> u16 {
    *ACTION_LOGIN_CODE
}

/// 公民链上身份 payload 确认(非链交易,b.d=VotingIdentityPayload SCALE bytes),
/// 公民钱包对 `signing_message(OP_SIGN_CITIZEN_IDENTITY, b.d)` 签名。
pub(crate) fn action_citizen_identity() -> u16 {
    *ACTION_CITIZEN_IDENTITY_CODE
}

/// 注册局管理员治理文本确认(非链动作,b.d=onchina_admin_governance canonical JSON),
/// 对应 qr-action-registry.md 非链动作码 a=3。
pub(crate) fn action_onchina_admin() -> u16 {
    *ACTION_ONCHINA_ADMIN_CODE
}
// 链交易动作码(机构治理/管理员集合)不在此处发明扁平常量:
// 统一用 `core::institution_call::chain_action_code(pallet,call)` 派生(b.a 与 b.d 同源),
// 旧机构直接创建 call 5 已关闭。机构管理员变更由 entity 治理结果驱动，
// 不存在 public/private admins 的直接集合变更动作。
// 详见 qr-action-registry.md「链交易动作码」。

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignRequestBody {
    /// a:动作码。登录=1,公民绑定=2,链上中国平台管理员动作=3。
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

#[derive(Debug, Clone)]
pub struct SignResponseBody {
    /// 0x + 32B hex 公钥。parse 时由 b.u 解码得到。
    pub pubkey: String,
    /// 0x + 64B hex 签名。parse 时由 b.s 解码得到。
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrEnvelope<B> {
    /// p:协议版本,固定 QR_V1。
    #[serde(rename = "p")]
    pub proto: String,
    /// k:二维码场景数字码。
    #[serde(rename = "k")]
    pub kind: u8,
    /// i:临时二维码一次性 id。
    #[serde(rename = "i", skip_serializing_if = "Option::is_none", default)]
    pub id: Option<String>,
    /// e:过期秒级时间戳。
    #[serde(rename = "e", skip_serializing_if = "Option::is_none", default)]
    pub expires_at: Option<i64>,
    /// b:场景 body。
    #[serde(rename = "b")]
    pub body: B,
}

pub type SignRequestEnvelope = QrEnvelope<SignRequestBody>;
pub type SignResponseEnvelope = QrEnvelope<SignResponseBody>;

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
        action: action_login(),
        sig_alg: 1,
        pubkey: pubkey_hex_to_b64(sys_pubkey).unwrap_or_default(),
        payload: URL_SAFE_NO_PAD.encode(format!("{}|{}", system, sys_sig).as_bytes()),
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

#[derive(Debug)]
pub enum QrParseError {
    BadJson(String),
    BadProto(String),
    BadKind(String),
    BadField(String),
}

impl std::fmt::Display for QrParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadJson(m) => write!(f, "QR JSON 非法: {}", m),
            Self::BadProto(m) => write!(f, "p 必须为 QR_V1,实际: {}", m),
            Self::BadKind(m) => write!(f, "未知 k: {}", m),
            Self::BadField(m) => write!(f, "字段错误: {}", m),
        }
    }
}

impl std::error::Error for QrParseError {}

#[derive(Deserialize)]
struct CompactResponseBody {
    u: String,
    s: String,
}

/// 解析 QR_V1/k=2 签名响应。后端收到签名方响应后使用。
pub fn parse_sign_response(raw: &str) -> Result<SignResponseEnvelope, QrParseError> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| QrParseError::BadJson(e.to_string()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| QrParseError::BadJson("不是对象".into()))?;

    match obj.get("p").and_then(|v| v.as_str()) {
        Some(QR_V1) => {}
        other => return Err(QrParseError::BadProto(format!("{:?}", other))),
    }
    match obj.get("k").and_then(|v| v.as_u64()) {
        Some(2) => {}
        other => return Err(QrParseError::BadKind(format!("{:?}", other))),
    }

    let id = obj
        .get("i")
        .and_then(|v| v.as_str())
        .ok_or_else(|| QrParseError::BadField("i 必填".into()))?
        .to_string();
    let expires_at = obj
        .get("e")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| QrParseError::BadField("e 必填整数".into()))?;
    let body_val = obj
        .get("b")
        .ok_or_else(|| QrParseError::BadField("b 必填".into()))?;
    let body: CompactResponseBody = serde_json::from_value(body_val.clone())
        .map_err(|e| QrParseError::BadField(format!("b: {}", e)))?;
    let pubkey = b64_to_prefixed_hex(&body.u, 32, "b.u")?;
    let signature = b64_to_prefixed_hex(&body.s, 64, "b.s")?;

    Ok(SignResponseEnvelope {
        proto: QR_V1.to_string(),
        kind: QrKind::SignResponse.code(),
        id: Some(id),
        expires_at: Some(expires_at),
        body: SignResponseBody { pubkey, signature },
    })
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

fn b64_to_prefixed_hex(value: &str, len: usize, field: &str) -> Result<String, QrParseError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| QrParseError::BadField(format!("{} 必须为 base64url", field)))?;
    if bytes.len() != len {
        return Err(QrParseError::BadField(format!(
            "{} 长度必须为 {} 字节",
            field, len
        )));
    }
    Ok(format!("0x{}", hex::encode(bytes)))
}

fn normalize_hex_no_prefix(value: &str) -> String {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value)
        .to_lowercase()
}
