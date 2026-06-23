import 'package:citizenapp/signer/signing.dart' show kImWalletBindingDomain;

/// 钱包账户与 IM 设备身份的绑定草案。
///
/// 产品层使用钱包账户作为聊天账户；但 IM 端到端加密必须使用独立设备密钥。
/// 钱包签名只证明“这个 IM 设备属于这个钱包账户”，不参与消息加密。
class ImWalletBindingDraft {
  const ImWalletBindingDraft({
    required this.walletAccount,
    required this.imDeviceId,
    required this.imDevicePubkey,
    required this.communicationNodePeerId,
    required this.nodeEndpoints,
    required this.expiresAt,
    required this.nonce,
    this.walletSignature,
  });

  /// 用户可见聊天账户；同一个钱包地址也可在交易模块中作为转账账户。
  final String walletAccount;

  /// 手机本地生成的 IM 设备 ID。
  final String imDeviceId;

  /// IM 设备公钥，后续由 OpenMLS / HPKE 绑定真实密钥材料。
  final String imDevicePubkey;

  /// 自己私人通信全节点的 PeerId。
  final String communicationNodePeerId;

  /// 自己节点的 IPv4 / IPv6 / dnsaddr 可达端点。
  final List<String> nodeEndpoints;

  /// 绑定凭证过期时间。
  final DateTime expiresAt;

  /// 防重放 nonce。
  final String nonce;

  /// 钱包账户对绑定载荷的签名；为空表示尚未完成钱包确认。
  final String? walletSignature;

  /// 构造稳定签名载荷。
  ///
  /// ADR-026 Phase 2:IM 钱包绑定**不是**签名 op_tag,为 `|` 拼接 UTF-8 字符串
  /// (与 node `im/binding.rs` 逐字节一致),非 signing_message 二进制形态。
  /// 域首段 [kImWalletBindingDomain] 单源对齐 primitives::sign::
  /// IM_WALLET_BINDING_DOMAIN,详见 im_binding_payload.dart。
  String canonicalPayload() {
    return [
      kImWalletBindingDomain,
      walletAccount,
      imDeviceId,
      imDevicePubkey,
      communicationNodePeerId,
      nodeEndpoints.join(','),
      expiresAt.toUtc().toIso8601String(),
      nonce,
    ].join('|');
  }

  /// 绑定是否已过期。
  bool get isExpired => DateTime.now().isAfter(expiresAt);
}
