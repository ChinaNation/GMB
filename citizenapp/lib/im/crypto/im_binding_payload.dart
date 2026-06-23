import 'package:citizenapp/signer/signing.dart' show kImWalletBindingDomain;

/// 公民 IM 钱包账户绑定 payload。
///
/// 钱包账户是用户可见聊天账户；IM 消息加密仍由独立设备密钥承担。
/// 此 payload 只用于让钱包签名确认“此设备属于此账户”。
class ImBindingPayload {
  const ImBindingPayload({
    required this.walletAccount,
    required this.imDeviceId,
    required this.imDevicePubkey,
    required this.nodePeerId,
    required this.nodeEndpoints,
    required this.expiresAtMillis,
    required this.nonce,
  });

  /// 用户可见聊天账户。
  final String walletAccount;

  /// 手机本地生成的 IM 设备 ID。
  final String imDeviceId;

  /// IM 设备公钥；真实 OpenMLS 接入后由密码模块提供。
  final String imDevicePubkey;

  /// 自己私人通信全节点 PeerId。
  final String nodePeerId;

  /// 自己节点可达端点，支持 IPv4、IPv6、dns4 和 dnsaddr。
  final List<String> nodeEndpoints;

  /// 绑定凭证过期时间，毫秒时间戳。
  final int expiresAtMillis;

  /// 防重放 nonce。
  final String nonce;

  /// 构造与 node 端一致的稳定签名载荷。
  ///
  /// ADR-026 Phase 2:IM 钱包绑定**不是**签名 op_tag(既不经 signingMessage 做
  /// hash,也不作二进制前缀签名)。载荷是 `|` 拼接的 UTF-8 canonical 字符串(钱包
  /// 对整段字符串签名),与 node `im/binding.rs::canonical_payload` 逐字节一致。
  /// 域首段 [kImWalletBindingDomain] 是单一权威源(对齐 primitives::sign::
  /// IM_WALLET_BINDING_DOMAIN),保留原构造不改为二进制形态。
  String canonicalPayload() {
    return [
      kImWalletBindingDomain,
      walletAccount,
      imDeviceId,
      imDevicePubkey,
      nodePeerId,
      nodeEndpoints.join(','),
      expiresAtMillis.toString(),
      nonce,
    ].join('|');
  }

  /// 转为提交给私人通信全节点的 JSON map。
  Map<String, Object?> toUnsignedJson() {
    return {
      'wallet_account': walletAccount,
      'im_device_id': imDeviceId,
      'im_device_pubkey': imDevicePubkey,
      'node_peer_id': nodePeerId,
      'node_endpoints': nodeEndpoints,
      'expires_at_millis': expiresAtMillis,
      'nonce': nonce,
    };
  }
}
