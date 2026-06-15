use serde::{Deserialize, Serialize};

/// IM 密文信封协议版本。
pub(crate) const GMB_IM_PROTOCOL_VERSION: u16 = 1;

/// IM 密文信封。
///
/// Spike 阶段用 hex 字符串承载模拟密文字节；后续 Protobuf 固化后，
/// `encrypted_payload_hex` 会替换为标准 `mls_wire_message` bytes 字段。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ImEnvelope {
    /// 协议版本，当前固定为 1。
    pub(crate) protocol_version: u16,
    /// 全局去重 ID。
    pub(crate) envelope_id: String,
    /// 会话 ID。
    pub(crate) conversation_id: String,
    /// 发送方可见聊天账户，即钱包账户。
    pub(crate) sender_chat_account: String,
    /// 接收方可见聊天账户，即钱包账户。
    pub(crate) recipient_chat_account: String,
    /// 发送设备 ID。
    pub(crate) sender_device_id: String,
    /// 加密后的 OpenMLS wire bytes；Spike 阶段临时使用 hex。
    pub(crate) encrypted_payload_hex: String,
    /// 创建时间，毫秒时间戳。
    pub(crate) created_at_millis: u64,
    /// 过期时间窗口，毫秒。
    pub(crate) ttl_millis: u64,
}

impl ImEnvelope {
    /// 执行本地结构校验，不尝试解密密文。
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.protocol_version != GMB_IM_PROTOCOL_VERSION {
            return Err("IM 信封协议版本不支持".to_string());
        }
        require_non_empty("envelope_id", &self.envelope_id)?;
        require_non_empty("conversation_id", &self.conversation_id)?;
        require_non_empty("sender_chat_account", &self.sender_chat_account)?;
        require_non_empty("recipient_chat_account", &self.recipient_chat_account)?;
        require_non_empty("sender_device_id", &self.sender_device_id)?;
        require_non_empty("encrypted_payload_hex", &self.encrypted_payload_hex)?;
        validate_hex_payload(&self.encrypted_payload_hex)?;
        if self.ttl_millis == 0 {
            return Err("IM 信封 ttl_millis 必须大于 0".to_string());
        }
        Ok(())
    }
}

/// 提交密文信封到私人通信全节点的请求。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SubmitImEnvelopeRequest {
    /// 目标钱包聊天账户。私人 mailbox 只接受投递给该账户的消息。
    pub(crate) mailbox_owner_chat_account: String,
    /// 密文信封。
    pub(crate) envelope: ImEnvelope,
}

impl SubmitImEnvelopeRequest {
    /// 校验请求不试图让本节点保存第三方消息。
    pub(crate) fn validate(&self) -> Result<(), String> {
        require_non_empty(
            "mailbox_owner_chat_account",
            &self.mailbox_owner_chat_account,
        )?;
        self.envelope.validate()?;
        if self.mailbox_owner_chat_account != self.envelope.recipient_chat_account {
            return Err("私人通信全节点只能接收投递给目标钱包账户的密文信封".to_string());
        }
        Ok(())
    }
}

/// IM 信封状态。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum ImEnvelopeState {
    /// 已进入私人 mailbox 等待已授权手机拉取。
    StoredForOwner,
    /// 已授权手机已确认。
    AcknowledgedByOwner,
}

/// IM 信封确认结果。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ImEnvelopeAck {
    /// 信封 ID。
    pub(crate) envelope_id: String,
    /// 当前状态。
    pub(crate) state: ImEnvelopeState,
}

fn require_non_empty(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("IM 字段 {field_name} 不能为空"));
    }
    Ok(())
}

fn validate_hex_payload(value: &str) -> Result<(), String> {
    if value.len() % 2 != 0 {
        return Err("IM 密文 hex 长度必须为偶数".to_string());
    }
    hex::decode(value)
        .map(|_| ())
        .map_err(|_| "IM 密文必须是合法小写或大写 hex".to_string())
}

#[cfg(test)]
mod tests {
    use super::{ImEnvelope, SubmitImEnvelopeRequest, GMB_IM_PROTOCOL_VERSION};

    fn sample_envelope() -> ImEnvelope {
        ImEnvelope {
            protocol_version: GMB_IM_PROTOCOL_VERSION,
            envelope_id: "env-1".to_string(),
            conversation_id: "conv-1".to_string(),
            sender_chat_account: "alice".to_string(),
            recipient_chat_account: "bob".to_string(),
            sender_device_id: "alice-phone".to_string(),
            encrypted_payload_hex: "aabbcc".to_string(),
            created_at_millis: 1,
            ttl_millis: 60_000,
        }
    }

    #[test]
    fn accepts_valid_envelope_request() {
        let request = SubmitImEnvelopeRequest {
            mailbox_owner_chat_account: "bob".to_string(),
            envelope: sample_envelope(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn rejects_third_party_mailbox_request() {
        let request = SubmitImEnvelopeRequest {
            mailbox_owner_chat_account: "carol".to_string(),
            envelope: sample_envelope(),
        };

        let err = request
            .validate()
            .expect_err("third-party mailbox must be rejected");
        assert!(err.contains("目标钱包账户"));
    }
}
