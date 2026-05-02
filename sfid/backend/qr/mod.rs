//! WUMIN_QR_V1 统一二维码协议 envelope。
//!
//! 唯一事实源: `memory/05-architecture/qr-protocol-spec.md`
//! Golden fixtures: `memory/05-architecture/qr-protocol-fixtures/*.json`
//!
//! 与 wuminapp/wumin 的 Dart envelope、citizenchain/sfid/cpms 前端的 TS
//! envelope 字段逐字节一致。本模块仅定义 SFID 后端需要的 kind
//! (login_challenge / login_receipt),其余 kind 后端不参与。

use serde::{Deserialize, Serialize};

pub const WUMIN_QR_V1: &str = "WUMIN_QR_V1";

/// 统一 kind 枚举(snake_case 序列化)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QrKind {
    LoginChallenge,
    LoginReceipt,
    SignRequest,
    SignResponse,
    UserContact,
    UserTransfer,
    UserDuoqian,
}

impl QrKind {
    pub fn wire(&self) -> &'static str {
        match self {
            Self::LoginChallenge => "login_challenge",
            Self::LoginReceipt => "login_receipt",
            Self::SignRequest => "sign_request",
            Self::SignResponse => "sign_response",
            Self::UserContact => "user_contact",
            Self::UserTransfer => "user_transfer",
            Self::UserDuoqian => "user_duoqian",
        }
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::UserContact | Self::UserDuoqian)
    }
}

// ---------- body ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginChallengeBody {
    pub system: String,
    pub sys_pubkey: String,
    pub sys_sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginReceiptBody {
    pub system: String,
    pub pubkey: String,
    pub sig_alg: String,
    pub signature: String,
    pub payload_hash: String,
    pub signed_at: i64,
}

// ---------- envelope ----------

/// 通用 envelope, 按 kind 决定 body 类型。后端一般使用下面两个便利别名。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrEnvelope<B> {
    pub proto: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub issued_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expires_at: Option<i64>,
    pub body: B,
}

pub type LoginChallengeEnvelope = QrEnvelope<LoginChallengeBody>;
pub type LoginReceiptEnvelope = QrEnvelope<LoginReceiptBody>;

impl LoginChallengeEnvelope {
    pub fn new(id: String, issued_at: i64, expires_at: i64, body: LoginChallengeBody) -> Self {
        Self {
            proto: WUMIN_QR_V1.to_string(),
            kind: QrKind::LoginChallenge.wire().to_string(),
            id: Some(id),
            issued_at: Some(issued_at),
            expires_at: Some(expires_at),
            body,
        }
    }
}

impl LoginReceiptEnvelope {
    pub fn new(id: String, issued_at: i64, expires_at: i64, body: LoginReceiptBody) -> Self {
        Self {
            proto: WUMIN_QR_V1.to_string(),
            kind: QrKind::LoginReceipt.wire().to_string(),
            id: Some(id),
            issued_at: Some(issued_at),
            expires_at: Some(expires_at),
            body,
        }
    }
}

/// 唯一的签名原文拼接函数。
///
/// 格式(与 Dart/TS 逐字节一致):
/// ```text
/// WUMIN_QR_V1|<kind>|<id>|<system 或空>|<expires_at 或 0>|<principal>
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
    let pp = principal
        .strip_prefix("0x")
        .or_else(|| principal.strip_prefix("0X"))
        .unwrap_or(principal)
        .to_lowercase();
    format!(
        "{}|{}|{}|{}|{}|{}",
        WUMIN_QR_V1,
        kind.wire(),
        id,
        sys,
        exp,
        pp
    )
}

// ---------- parse helpers ----------

#[derive(Debug)]
pub enum QrParseError {
    BadJson(String),
    BadProto(String),
    BadKind(String),
    BadField(String),
    FixedCodeHasTemporal(String),
}

impl std::fmt::Display for QrParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadJson(m) => write!(f, "QR JSON 非法: {}", m),
            Self::BadProto(m) => write!(f, "proto 必须为 WUMIN_QR_V1, 实际: {}", m),
            Self::BadKind(m) => write!(f, "未知 kind: {}", m),
            Self::BadField(m) => write!(f, "字段错误: {}", m),
            Self::FixedCodeHasTemporal(m) => {
                write!(f, "固定码 {} 不应包含 id/issued_at/expires_at", m)
            }
        }
    }
}

impl std::error::Error for QrParseError {}

/// 解析 login_receipt envelope。后端收到 wumin 冷钱包的回执后使用。
pub fn parse_login_receipt(raw: &str) -> Result<LoginReceiptEnvelope, QrParseError> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| QrParseError::BadJson(e.to_string()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| QrParseError::BadJson("不是对象".into()))?;

    match obj.get("proto").and_then(|v| v.as_str()) {
        Some(WUMIN_QR_V1) => {}
        other => return Err(QrParseError::BadProto(format!("{:?}", other))),
    }
    match obj.get("kind").and_then(|v| v.as_str()) {
        Some("login_receipt") => {}
        other => return Err(QrParseError::BadKind(format!("{:?}", other))),
    }

    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| QrParseError::BadField("id 必填".into()))?
        .to_string();
    let issued_at = obj
        .get("issued_at")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| QrParseError::BadField("issued_at 必填整数".into()))?;
    let expires_at = obj
        .get("expires_at")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| QrParseError::BadField("expires_at 必填整数".into()))?;

    let body_val = obj
        .get("body")
        .ok_or_else(|| QrParseError::BadField("body 必填".into()))?;
    let body: LoginReceiptBody = serde_json::from_value(body_val.clone())
        .map_err(|e| QrParseError::BadField(format!("body: {}", e)))?;
    if body.sig_alg != "sr25519" {
        return Err(QrParseError::BadField(
            "login_receipt.sig_alg 必须为 sr25519".into(),
        ));
    }
    if body.system != "sfid" && body.system != "cpms" {
        return Err(QrParseError::BadField(format!(
            "login_receipt.system 非法: {}",
            body.system
        )));
    }

    Ok(LoginReceiptEnvelope {
        proto: WUMIN_QR_V1.to_string(),
        kind: "login_receipt".to_string(),
        id: Some(id),
        issued_at: Some(issued_at),
        expires_at: Some(expires_at),
        body,
    })
}
