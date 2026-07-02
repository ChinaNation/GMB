use codec::Encode;
use primitives::sign::{signing_message, OP_SIGN_IM_WALLET_BINDING};
use serde::{Deserialize, Serialize};
use sp_core::Pair;

use super::endpoint::ImNodeEndpoint;

/// 钱包聊天账户与 IM 设备、私人通信全节点的绑定请求。
///
/// 钱包签名只证明“此 IM 设备归属此钱包聊天账户”，不参与消息加密。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RegisterImDeviceRequest {
    /// 用户可见聊天账户，也是聊天窗口发公民币时的付款账户。
    pub(crate) wallet_account: String,
    /// 手机本地生成的 IM 设备 ID。
    pub(crate) im_device_id: String,
    /// IM 设备公钥；Spike 阶段只做结构登记，不解析密码学格式。
    pub(crate) im_device_pubkey: String,
    /// 私人通信全节点 PeerId。
    pub(crate) node_peer_id: String,
    /// 私人通信全节点可达端点。
    pub(crate) node_endpoints: Vec<ImNodeEndpoint>,
    /// 绑定凭证过期时间，毫秒时间戳。
    pub(crate) expires_at_millis: u64,
    /// 防重放 nonce。
    pub(crate) nonce: String,
    /// 钱包账户对 `signing_message(OP_SIGN_IM_WALLET_BINDING, payload)` 的签名。
    pub(crate) wallet_signature: String,
}

impl RegisterImDeviceRequest {
    /// 构造稳定 SCALE 签名载荷。
    pub(crate) fn signing_payload(&self) -> Vec<u8> {
        let endpoints = self
            .node_endpoints
            .iter()
            .map(|endpoint| endpoint.multiaddr.as_str())
            .collect::<Vec<_>>();
        (
            self.wallet_account.as_str(),
            self.im_device_id.as_str(),
            self.im_device_pubkey.as_str(),
            self.node_peer_id.as_str(),
            endpoints,
            self.expires_at_millis,
            self.nonce.as_str(),
        )
            .encode()
    }

    /// 构造统一哈希域签名消息。
    pub(crate) fn signing_message(&self) -> [u8; 32] {
        signing_message(OP_SIGN_IM_WALLET_BINDING, &self.signing_payload())
    }

    /// 校验绑定请求的私人节点边界。
    pub(crate) fn validate(&self) -> Result<(), String> {
        require_non_empty("wallet_account", &self.wallet_account)?;
        require_non_empty("im_device_id", &self.im_device_id)?;
        require_non_empty("im_device_pubkey", &self.im_device_pubkey)?;
        require_non_empty("node_peer_id", &self.node_peer_id)?;
        require_non_empty("nonce", &self.nonce)?;
        require_non_empty("wallet_signature", &self.wallet_signature)?;
        if self.node_endpoints.is_empty() {
            return Err("IM 绑定必须至少包含一个私人节点端点".to_string());
        }
        for endpoint in &self.node_endpoints {
            endpoint.validate()?;
            if endpoint.peer_id != self.node_peer_id {
                return Err("IM 绑定端点 PeerId 必须等于 node_peer_id".to_string());
            }
        }
        self.verify_wallet_signature()?;
        Ok(())
    }

    fn verify_wallet_signature(&self) -> Result<(), String> {
        let wallet_pubkey =
            crate::governance::signing::decode_ss58_to_pubkey(&self.wallet_account)?;
        let signature_hex = self
            .wallet_signature
            .strip_prefix("0x")
            .or_else(|| self.wallet_signature.strip_prefix("0X"))
            .unwrap_or(&self.wallet_signature);
        let signature_bytes =
            hex::decode(signature_hex).map_err(|e| format!("IM 绑定签名 hex 解码失败: {e}"))?;
        let signature = sp_core::sr25519::Signature::from_raw(
            <[u8; 64]>::try_from(signature_bytes.as_slice())
                .map_err(|_| "IM 绑定签名长度必须为 64 字节".to_string())?,
        );
        let public = sp_core::sr25519::Public::from_raw(wallet_pubkey);
        if !sp_core::sr25519::Pair::verify(&signature, &self.signing_message(), &public) {
            return Err("IM 绑定钱包签名验证失败".to_string());
        }
        Ok(())
    }
}

/// 已登记的 IM 设备绑定。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ImDeviceBinding {
    /// 用户可见聊天账户。
    pub(crate) wallet_account: String,
    /// IM 设备 ID。
    pub(crate) im_device_id: String,
    /// IM 设备公钥。
    pub(crate) im_device_pubkey: String,
    /// 私人通信全节点 PeerId。
    pub(crate) node_peer_id: String,
    /// 私人通信全节点端点。
    pub(crate) node_endpoints: Vec<ImNodeEndpoint>,
    /// 绑定凭证过期时间，毫秒时间戳。
    pub(crate) expires_at_millis: u64,
    /// 防重放 nonce。
    pub(crate) nonce: String,
    /// 钱包签名。
    pub(crate) wallet_signature: String,
    /// 节点返回的 SCALE 签名载荷 hex，方便公民端调试签名一致性。
    pub(crate) signing_payload_hex: String,
}

impl From<RegisterImDeviceRequest> for ImDeviceBinding {
    fn from(request: RegisterImDeviceRequest) -> Self {
        let signing_payload_hex = format!("0x{}", hex::encode(request.signing_payload()));
        Self {
            wallet_account: request.wallet_account,
            im_device_id: request.im_device_id,
            im_device_pubkey: request.im_device_pubkey,
            node_peer_id: request.node_peer_id,
            node_endpoints: request.node_endpoints,
            expires_at_millis: request.expires_at_millis,
            nonce: request.nonce,
            wallet_signature: request.wallet_signature,
            signing_payload_hex,
        }
    }
}

fn require_non_empty(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("IM 绑定字段 {field_name} 不能为空"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::RegisterImDeviceRequest;
    use crate::im::endpoint::ImNodeEndpoint;
    use sp_core::{sr25519, Pair};

    fn signed_request() -> RegisterImDeviceRequest {
        let pair = sr25519::Pair::from_seed(&[0x42; 32]);
        let wallet_account = crate::governance::signing::pubkey_to_ss58(pair.public().as_ref())
            .expect("test public key should encode");
        let mut request = RegisterImDeviceRequest {
            wallet_account,
            im_device_id: "alice-phone".to_string(),
            im_device_pubkey: "0xabc".to_string(),
            node_peer_id: "12D3KooWTest".to_string(),
            node_endpoints: vec![ImNodeEndpoint::checked(
                "12D3KooWTest",
                "/ip4/127.0.0.1/tcp/30333/wss/p2p/12D3KooWTest",
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

    #[test]
    fn binding_payload_is_scale_and_signature_validates() {
        let request = signed_request();

        assert!(request.signing_payload().len() > request.wallet_account.len());
        assert!(request.validate().is_ok());
    }

    #[test]
    fn rejects_forged_wallet_signature() {
        let mut request = signed_request();
        request.wallet_signature = format!("0x{}", "00".repeat(64));

        let err = request
            .validate()
            .expect_err("forged signature must be rejected");

        assert!(err.contains("签名验证失败"));
    }
}
