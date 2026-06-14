use std::collections::{HashMap, HashSet};

use super::{
    binding::{ImDeviceBinding, RegisterImDeviceRequest},
    envelope::{ImEnvelope, ImEnvelopeAck, ImEnvelopeState, SubmitImEnvelopeRequest},
};

/// 私人通信全节点的内存态 mailbox。
///
/// Spike 阶段只用于验证边界和 Tauri 命令行为；后续真实实现必须替换为
/// IM 专属持久化存储，并继续保持“只服务 owner”的约束。
#[derive(Debug, Default)]
pub(crate) struct ImMailbox {
    owner_wallet_account: Option<String>,
    bindings_by_device: HashMap<String, ImDeviceBinding>,
    pending_by_owner: HashMap<String, Vec<ImEnvelope>>,
    authorized_devices: HashSet<String>,
}

impl ImMailbox {
    /// 登记 owner 的 IM 设备绑定。
    pub(crate) fn register_owner_device(
        &mut self,
        request: RegisterImDeviceRequest,
    ) -> Result<ImDeviceBinding, String> {
        request.validate()?;
        match &self.owner_wallet_account {
            Some(owner) if owner != &request.wallet_account => {
                return Err("私人通信全节点只能绑定一个 owner 钱包聊天账户".to_string());
            }
            None => {
                self.owner_wallet_account = Some(request.wallet_account.clone());
            }
            _ => {}
        }

        let binding = ImDeviceBinding::from(request);
        self.authorized_devices.insert(binding.im_device_id.clone());
        self.bindings_by_device
            .insert(binding.im_device_id.clone(), binding.clone());
        Ok(binding)
    }

    /// 提交密文信封到 owner mailbox。
    pub(crate) fn submit_envelope(
        &mut self,
        request: SubmitImEnvelopeRequest,
    ) -> Result<ImEnvelopeAck, String> {
        request.validate()?;
        self.ensure_owner(&request.mailbox_owner_chat_account)?;
        let envelope_id = request.envelope.envelope_id.clone();
        let pending = self
            .pending_by_owner
            .entry(request.mailbox_owner_chat_account)
            .or_default();
        if !pending
            .iter()
            .any(|envelope| envelope.envelope_id == envelope_id)
        {
            pending.push(request.envelope);
        }
        Ok(ImEnvelopeAck {
            envelope_id,
            state: ImEnvelopeState::StoredForOwner,
        })
    }

    /// owner 手机拉取待收密文。
    pub(crate) fn fetch_pending(
        &self,
        owner_wallet_account: &str,
        device_id: &str,
    ) -> Result<Vec<ImEnvelope>, String> {
        self.ensure_owner(owner_wallet_account)?;
        self.ensure_authorized_device(device_id)?;
        Ok(self
            .pending_by_owner
            .get(owner_wallet_account)
            .cloned()
            .unwrap_or_default())
    }

    /// owner 手机确认某个信封。
    pub(crate) fn ack_envelope(
        &mut self,
        owner_wallet_account: &str,
        device_id: &str,
        envelope_id: &str,
    ) -> Result<ImEnvelopeAck, String> {
        self.ensure_owner(owner_wallet_account)?;
        self.ensure_authorized_device(device_id)?;
        let pending = self
            .pending_by_owner
            .entry(owner_wallet_account.to_string())
            .or_default();
        pending.retain(|envelope| envelope.envelope_id != envelope_id);
        Ok(ImEnvelopeAck {
            envelope_id: envelope_id.to_string(),
            state: ImEnvelopeState::AcknowledgedByOwner,
        })
    }

    fn ensure_owner(&self, owner_wallet_account: &str) -> Result<(), String> {
        match &self.owner_wallet_account {
            Some(owner) if owner == owner_wallet_account => Ok(()),
            Some(_) => Err("私人通信全节点拒绝第三方 mailbox 访问".to_string()),
            None => Err("IM 私人通信全节点尚未绑定 owner".to_string()),
        }
    }

    fn ensure_authorized_device(&self, device_id: &str) -> Result<(), String> {
        if self.authorized_devices.contains(device_id) {
            Ok(())
        } else {
            Err("IM 设备未获得 owner 授权".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ImMailbox;
    use crate::im::{
        binding::RegisterImDeviceRequest,
        endpoint::ImNodeEndpoint,
        envelope::{ImEnvelope, SubmitImEnvelopeRequest, GMB_IM_PROTOCOL_VERSION},
    };

    fn sample_binding() -> RegisterImDeviceRequest {
        RegisterImDeviceRequest {
            wallet_account: "bob".to_string(),
            im_device_id: "bob-phone".to_string(),
            im_device_pubkey: "0xabc".to_string(),
            node_peer_id: "12D3KooWTest".to_string(),
            node_endpoints: vec![ImNodeEndpoint::checked(
                "12D3KooWTest",
                "/ip6/2001:db8::1/tcp/443/wss/p2p/12D3KooWTest",
            )
            .expect("test endpoint should be valid")],
            expires_at_millis: 1_800_000,
            nonce: "nonce-1".to_string(),
            wallet_signature: "0xsig".to_string(),
        }
    }

    fn sample_envelope(recipient: &str) -> ImEnvelope {
        ImEnvelope {
            protocol_version: GMB_IM_PROTOCOL_VERSION,
            envelope_id: "env-1".to_string(),
            conversation_id: "conv-1".to_string(),
            sender_chat_account: "alice".to_string(),
            recipient_chat_account: recipient.to_string(),
            sender_device_id: "alice-phone".to_string(),
            encrypted_payload_hex: "aabbcc".to_string(),
            created_at_millis: 1,
            ttl_millis: 60_000,
        }
    }

    #[test]
    fn stores_only_owner_envelopes() {
        let mut mailbox = ImMailbox::default();
        mailbox
            .register_owner_device(sample_binding())
            .expect("owner device should register");

        mailbox
            .submit_envelope(SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: "bob".to_string(),
                envelope: sample_envelope("bob"),
            })
            .expect("owner envelope should be stored");

        let err = mailbox
            .submit_envelope(SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: "carol".to_string(),
                envelope: sample_envelope("carol"),
            })
            .expect_err("third-party mailbox must be rejected");
        assert!(err.contains("第三方"));
    }

    #[test]
    fn fetch_and_ack_require_authorized_owner_device() {
        let mut mailbox = ImMailbox::default();
        mailbox
            .register_owner_device(sample_binding())
            .expect("owner device should register");
        mailbox
            .submit_envelope(SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: "bob".to_string(),
                envelope: sample_envelope("bob"),
            })
            .expect("owner envelope should be stored");

        let pending = mailbox
            .fetch_pending("bob", "bob-phone")
            .expect("authorized device can fetch");
        assert_eq!(pending.len(), 1);

        mailbox
            .ack_envelope("bob", "bob-phone", "env-1")
            .expect("authorized device can ack");
        assert!(mailbox
            .fetch_pending("bob", "bob-phone")
            .expect("authorized device can fetch")
            .is_empty());
    }
}
