use serde::Serialize;

/// 私人通信全节点的固定边界。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImPrivateNodePolicy {
    /// 是否只服务当前用户自己的手机和收件箱。
    pub serves_only_owner: bool,
    /// 是否允许作为第三方 Relay。
    pub allows_third_party_relay: bool,
    /// 是否允许作为公共 rendezvous。
    pub allows_public_rendezvous: bool,
    /// 是否允许替第三方保存消息。
    pub allows_third_party_mailbox: bool,
    /// 是否允许读取或解密消息明文。
    pub can_decrypt_messages: bool,
    /// 是否使用钱包账户作为用户可见聊天账户。
    pub wallet_account_is_chat_account: bool,
    /// 是否允许复用钱包私钥作为 IM 加密密钥。
    pub wallet_key_used_for_message_crypto: bool,
    /// IM 协议名。
    pub protocol_name: &'static str,
    /// 支持的端点类型。
    pub endpoint_kinds: &'static [&'static str],
}

impl ImPrivateNodePolicy {
    /// 返回当前 IM 私人通信全节点的硬边界。
    pub(crate) const fn current() -> Self {
        Self {
            serves_only_owner: true,
            allows_third_party_relay: false,
            allows_public_rendezvous: false,
            allows_third_party_mailbox: false,
            can_decrypt_messages: false,
            wallet_account_is_chat_account: true,
            wallet_key_used_for_message_crypto: false,
            protocol_name: "/gmb/im/1",
            endpoint_kinds: &["ip4", "ip6", "dns4", "dnsaddr"],
        }
    }
}
