//! CITIZEN_QR_V1 统一二维码协议 envelope。
//!
//! 唯一事实源: `memory/01-architecture/qr/qr-protocol-spec.md`
//! Golden fixtures: `memory/01-architecture/qr/qr-protocol-fixtures/*.json`
//!
//! 与 citizenwallet 的 Dart envelope、citizenchain/sfid/cpms 前端的 TS envelope 字段逐字节一致。

use serde::{Deserialize, Serialize};

pub const CITIZEN_QR_V1: &str = "CITIZEN_QR_V1";

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
        }
    }
}

// ---------- body ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginChallengeBody {
    pub system: String,
    pub sys_pubkey: String,
    pub sys_sig: String,
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

impl LoginChallengeEnvelope {
    pub fn new(id: String, issued_at: i64, expires_at: i64, body: LoginChallengeBody) -> Self {
        Self {
            proto: CITIZEN_QR_V1.to_string(),
            kind: QrKind::LoginChallenge.wire().to_string(),
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
/// CITIZEN_QR_V1|<kind>|<id>|<system 或空>|<expires_at 或 0>|<principal>
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
        CITIZEN_QR_V1,
        kind.wire(),
        id,
        sys,
        exp,
        pp
    )
}
