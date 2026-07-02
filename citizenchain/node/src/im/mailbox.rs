use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    binding::{ImDeviceBinding, RegisterImDeviceRequest},
    envelope::{ImEnvelope, ImEnvelopeAck, ImEnvelopeState, SubmitImEnvelopeRequest},
};

const MAX_PENDING_ENVELOPES_PER_OWNER: usize = 10_000;
const MAX_ENCRYPTED_PAYLOAD_HEX_LEN: usize = 2 * 1024 * 1024;
const ACK_TOMBSTONE_TTL_MILLIS: u64 = 7 * 24 * 60 * 60 * 1000;

#[derive(serde::Deserialize, serde::Serialize)]
struct PersistentMailboxSnapshot {
    bindings_by_account: HashMap<String, HashMap<String, ImDeviceBinding>>,
    pending_by_owner: HashMap<String, Vec<ImEnvelope>>,
    authorized_devices_by_owner: HashMap<String, HashSet<String>>,
    acked_by_owner: HashMap<String, HashMap<String, u64>>,
}

/// 私人通信全节点的多钱包账号 mailbox。
///
/// 正常节点启动后通过 `attach_storage` 绑定到 `base-path/im/mailbox.json`，
/// 每次设备登记、投递或 ack 后都会写回快照。单元测试仍可使用纯内存模式。
#[derive(Debug)]
pub(crate) struct ImMailbox {
    storage_path: Option<PathBuf>,
    bindings_by_account: HashMap<String, HashMap<String, ImDeviceBinding>>,
    pending_by_owner: HashMap<String, Vec<ImEnvelope>>,
    authorized_devices_by_owner: HashMap<String, HashSet<String>>,
    acked_by_owner: HashMap<String, HashMap<String, u64>>,
}

impl Default for ImMailbox {
    fn default() -> Self {
        Self {
            storage_path: None,
            bindings_by_account: HashMap::new(),
            pending_by_owner: HashMap::new(),
            authorized_devices_by_owner: HashMap::new(),
            acked_by_owner: HashMap::new(),
        }
    }
}

impl ImMailbox {
    /// 绑定持久化快照文件，并从磁盘恢复 mailbox。
    pub(crate) fn attach_storage(&mut self, file_path: PathBuf) -> Result<(), String> {
        if file_path.exists() {
            let bytes =
                fs::read(&file_path).map_err(|e| format!("读取 IM mailbox 持久化文件失败: {e}"))?;
            let snapshot: PersistentMailboxSnapshot = serde_json::from_slice(&bytes)
                .map_err(|e| format!("解析 IM mailbox 持久化文件失败: {e}"))?;
            self.bindings_by_account = snapshot.bindings_by_account;
            self.pending_by_owner = snapshot.pending_by_owner;
            self.authorized_devices_by_owner = snapshot.authorized_devices_by_owner;
            self.acked_by_owner = snapshot.acked_by_owner;
        }

        self.storage_path = Some(file_path);
        self.prune_expired(now_millis());
        self.persist()
    }

    /// 登记本机钱包聊天账号的 IM 设备绑定。
    pub(crate) fn register_owner_device(
        &mut self,
        request: RegisterImDeviceRequest,
    ) -> Result<ImDeviceBinding, String> {
        request.validate()?;
        let binding = ImDeviceBinding::from(request);
        self.authorized_devices_by_owner
            .entry(binding.wallet_account.clone())
            .or_default()
            .insert(binding.im_device_id.clone());
        self.bindings_by_account
            .entry(binding.wallet_account.clone())
            .or_default()
            .insert(binding.im_device_id.clone(), binding.clone());
        self.persist()?;
        Ok(binding)
    }

    /// 提交密文信封到目标钱包账号 mailbox。
    pub(crate) fn submit_envelope(
        &mut self,
        request: SubmitImEnvelopeRequest,
    ) -> Result<ImEnvelopeAck, String> {
        request.validate()?;
        self.validate_capacity(&request.envelope)?;
        self.prune_expired(now_millis());
        self.ensure_account(&request.mailbox_owner_chat_account)?;
        let envelope_id = request.envelope.envelope_id.clone();
        if self.is_acked(&request.mailbox_owner_chat_account, &envelope_id) {
            return Ok(ImEnvelopeAck {
                envelope_id,
                state: ImEnvelopeState::AcknowledgedByOwner,
            });
        }

        let pending = self
            .pending_by_owner
            .entry(request.mailbox_owner_chat_account.clone())
            .or_default();
        let exists = pending
            .iter()
            .any(|envelope| envelope.envelope_id == envelope_id);
        if !exists {
            if pending.len() >= MAX_PENDING_ENVELOPES_PER_OWNER {
                return Err("IM 钱包账号 mailbox 待收密文已达到容量上限".to_string());
            }
            pending.push(request.envelope);
        }
        self.persist()?;
        Ok(ImEnvelopeAck {
            envelope_id,
            state: ImEnvelopeState::StoredForOwner,
        })
    }

    /// 已授权手机拉取指定钱包账号的待收密文。
    pub(crate) fn fetch_pending(
        &mut self,
        owner_wallet_account: &str,
        device_id: &str,
    ) -> Result<Vec<ImEnvelope>, String> {
        self.prune_expired(now_millis());
        self.persist()?;
        self.ensure_account(owner_wallet_account)?;
        self.ensure_authorized_device(owner_wallet_account, device_id)?;
        Ok(self
            .pending_by_owner
            .get(owner_wallet_account)
            .cloned()
            .unwrap_or_default())
    }

    /// 已授权手机确认指定钱包账号的某个信封。
    pub(crate) fn ack_envelope(
        &mut self,
        owner_wallet_account: &str,
        device_id: &str,
        envelope_id: &str,
    ) -> Result<ImEnvelopeAck, String> {
        let now = now_millis();
        self.prune_expired(now);
        self.ensure_account(owner_wallet_account)?;
        self.ensure_authorized_device(owner_wallet_account, device_id)?;
        let pending = self
            .pending_by_owner
            .entry(owner_wallet_account.to_string())
            .or_default();
        pending.retain(|envelope| envelope.envelope_id != envelope_id);
        self.acked_by_owner
            .entry(owner_wallet_account.to_string())
            .or_default()
            .insert(envelope_id.to_string(), now);
        self.persist()?;
        Ok(ImEnvelopeAck {
            envelope_id: envelope_id.to_string(),
            state: ImEnvelopeState::AcknowledgedByOwner,
        })
    }

    /// 校验请求中的钱包聊天账户已授权到本私人通信全节点。
    ///
    /// KeyPackage 发布由已授权手机通过本机 RPC 发起，因此也必须先通过
    /// mailbox 授权边界，避免未授权账号发布预密钥包。
    pub(crate) fn ensure_owner_account(&self, owner_wallet_account: &str) -> Result<(), String> {
        self.ensure_account(owner_wallet_account)
    }

    fn ensure_account(&self, owner_wallet_account: &str) -> Result<(), String> {
        if self.bindings_by_account.contains_key(owner_wallet_account) {
            Ok(())
        } else {
            Err("IM 私人通信全节点尚未授权该钱包账户".to_string())
        }
    }

    fn ensure_authorized_device(
        &self,
        owner_wallet_account: &str,
        device_id: &str,
    ) -> Result<(), String> {
        if self
            .authorized_devices_by_owner
            .get(owner_wallet_account)
            .map(|devices| devices.contains(device_id))
            .unwrap_or(false)
        {
            Ok(())
        } else {
            Err("IM 设备未获得该钱包账户授权".to_string())
        }
    }

    fn validate_capacity(&self, envelope: &ImEnvelope) -> Result<(), String> {
        if envelope.encrypted_payload_hex.len() > MAX_ENCRYPTED_PAYLOAD_HEX_LEN {
            return Err("IM 密文超过单信封大小上限".to_string());
        }
        Ok(())
    }

    fn is_acked(&self, owner_wallet_account: &str, envelope_id: &str) -> bool {
        self.acked_by_owner
            .get(owner_wallet_account)
            .and_then(|items| items.get(envelope_id))
            .is_some()
    }

    fn prune_expired(&mut self, now: u64) {
        for envelopes in self.pending_by_owner.values_mut() {
            envelopes.retain(|envelope| {
                envelope
                    .created_at_millis
                    .checked_add(envelope.ttl_millis)
                    .map(|expires_at| expires_at > now)
                    .unwrap_or(false)
            });
        }
        for acked in self.acked_by_owner.values_mut() {
            acked.retain(|_, acked_at| {
                acked_at
                    .checked_add(ACK_TOMBSTONE_TTL_MILLIS)
                    .map(|expires_at| expires_at > now)
                    .unwrap_or(false)
            });
        }
    }

    fn persist(&self) -> Result<(), String> {
        let Some(path) = &self.storage_path else {
            return Ok(());
        };
        let snapshot = PersistentMailboxSnapshot {
            bindings_by_account: self.bindings_by_account.clone(),
            pending_by_owner: self.pending_by_owner.clone(),
            authorized_devices_by_owner: self.authorized_devices_by_owner.clone(),
            acked_by_owner: self.acked_by_owner.clone(),
        };
        persist_snapshot(path, &snapshot)
    }
}

fn persist_snapshot(path: &Path, snapshot: &PersistentMailboxSnapshot) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "IM mailbox 持久化路径缺少父目录".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("创建 IM mailbox 目录失败: {e}"))?;
    let tmp_path = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(snapshot)
        .map_err(|e| format!("序列化 IM mailbox 快照失败: {e}"))?;
    fs::write(&tmp_path, bytes).map_err(|e| format!("写入 IM mailbox 临时文件失败: {e}"))?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("替换 IM mailbox 旧快照失败: {e}"))?;
    }
    fs::rename(&tmp_path, path).map_err(|e| format!("提交 IM mailbox 快照失败: {e}"))?;
    Ok(())
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{now_millis, ImMailbox};
    use crate::im::{
        binding::RegisterImDeviceRequest,
        endpoint::ImNodeEndpoint,
        envelope::{ImEnvelope, SubmitImEnvelopeRequest, GMB_IM_PROTOCOL_VERSION},
    };
    use sp_core::{sr25519, Pair};
    use std::fs;

    fn sample_binding() -> RegisterImDeviceRequest {
        sample_binding_for(0x24, "bob-phone")
    }

    fn sample_binding_for(seed_byte: u8, device_id: &str) -> RegisterImDeviceRequest {
        let pair = sr25519::Pair::from_seed(&[seed_byte; 32]);
        let wallet_account = crate::governance::signing::pubkey_to_ss58(pair.public().as_ref())
            .expect("test public key should encode");
        let mut request = RegisterImDeviceRequest {
            wallet_account,
            im_device_id: device_id.to_string(),
            im_device_pubkey: "0xabc".to_string(),
            node_peer_id: "12D3KooWTest".to_string(),
            node_endpoints: vec![ImNodeEndpoint::checked(
                "12D3KooWTest",
                "/ip6/2001:db8::1/tcp/443/wss/p2p/12D3KooWTest",
            )
            .expect("test endpoint should be valid")],
            expires_at_millis: 1_800_000,
            nonce: "nonce-1".to_string(),
            wallet_signature: String::new(),
        };
        let signature = pair.sign(&request.signing_message());
        request.wallet_signature = format!("0x{}", hex::encode(signature.0));
        request
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
            created_at_millis: now_millis(),
            ttl_millis: 60_000,
        }
    }

    #[test]
    fn rejects_unregistered_wallet_mailbox() {
        let mut mailbox = ImMailbox::default();
        let binding = sample_binding();
        let account = binding.wallet_account.clone();
        mailbox
            .register_owner_device(binding)
            .expect("wallet device should register");

        mailbox
            .submit_envelope(SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: account.clone(),
                envelope: sample_envelope(&account),
            })
            .expect("owner envelope should be stored");

        let err = mailbox
            .submit_envelope(SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: "carol".to_string(),
                envelope: sample_envelope("carol"),
            })
            .expect_err("unregistered wallet mailbox must be rejected");
        assert!(err.contains("尚未授权"));
    }

    #[test]
    fn supports_multiple_wallet_accounts_on_one_node() {
        let mut mailbox = ImMailbox::default();
        let bob_binding = sample_binding();
        let bob_account = bob_binding.wallet_account.clone();
        mailbox
            .register_owner_device(bob_binding)
            .expect("bob device should register");
        let alice_binding = sample_binding_for(0x25, "alice-phone");
        let alice_account = alice_binding.wallet_account.clone();
        mailbox
            .register_owner_device(alice_binding)
            .expect("alice device should register");

        mailbox
            .submit_envelope(SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: bob_account.clone(),
                envelope: sample_envelope(&bob_account),
            })
            .expect("bob envelope should be stored");
        mailbox
            .submit_envelope(SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: alice_account.clone(),
                envelope: ImEnvelope {
                    envelope_id: "env-2".to_string(),
                    ..sample_envelope(&alice_account)
                },
            })
            .expect("alice envelope should be stored");

        assert_eq!(
            mailbox
                .fetch_pending(&bob_account, "bob-phone")
                .expect("bob device can fetch")
                .len(),
            1
        );
        assert_eq!(
            mailbox
                .fetch_pending(&alice_account, "alice-phone")
                .expect("alice device can fetch")
                .len(),
            1
        );
    }

    #[test]
    fn fetch_and_ack_require_authorized_owner_device() {
        let mut mailbox = ImMailbox::default();
        let binding = sample_binding();
        let account = binding.wallet_account.clone();
        mailbox
            .register_owner_device(binding)
            .expect("wallet device should register");
        mailbox
            .submit_envelope(SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: account.clone(),
                envelope: sample_envelope(&account),
            })
            .expect("owner envelope should be stored");

        let pending = mailbox
            .fetch_pending(&account, "bob-phone")
            .expect("authorized device can fetch");
        assert_eq!(pending.len(), 1);

        mailbox
            .ack_envelope(&account, "bob-phone", "env-1")
            .expect("authorized device can ack");
        assert!(mailbox
            .fetch_pending(&account, "bob-phone")
            .expect("authorized device can fetch")
            .is_empty());
    }

    #[test]
    fn persists_pending_and_ack_state() {
        let file_path =
            std::env::temp_dir().join(format!("gmb-im-mailbox-test-{}.json", now_millis()));
        let _ = fs::remove_file(&file_path);

        let mut mailbox = ImMailbox::default();
        mailbox
            .attach_storage(file_path.clone())
            .expect("storage should attach");
        let binding = sample_binding();
        let account = binding.wallet_account.clone();
        mailbox
            .register_owner_device(binding)
            .expect("wallet device should register");
        mailbox
            .submit_envelope(SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: account.clone(),
                envelope: sample_envelope(&account),
            })
            .expect("owner envelope should be stored");

        let mut reloaded = ImMailbox::default();
        reloaded
            .attach_storage(file_path.clone())
            .expect("storage should reload");
        let pending = reloaded
            .fetch_pending(&account, "bob-phone")
            .expect("pending should survive reload");
        assert_eq!(pending.len(), 1);

        reloaded
            .ack_envelope(&account, "bob-phone", "env-1")
            .expect("ack should persist");

        let mut after_ack_reload = ImMailbox::default();
        after_ack_reload
            .attach_storage(file_path.clone())
            .expect("storage should reload after ack");
        assert!(after_ack_reload
            .fetch_pending(&account, "bob-phone")
            .expect("pending should stay empty after ack reload")
            .is_empty());
        let ack = after_ack_reload
            .submit_envelope(SubmitImEnvelopeRequest {
                mailbox_owner_chat_account: account.clone(),
                envelope: sample_envelope(&account),
            })
            .expect("duplicate acked envelope should not requeue");
        assert_eq!(
            ack.state,
            crate::im::envelope::ImEnvelopeState::AcknowledgedByOwner
        );
        assert!(after_ack_reload
            .fetch_pending(&account, "bob-phone")
            .expect("duplicate acked envelope should stay hidden")
            .is_empty());

        let _ = fs::remove_file(file_path);
    }
}
