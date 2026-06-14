/// 公民 IM 钱包账户绑定 payload。
///
/// 钱包账户是用户可见聊天账户和聊天窗口发公民币的付款账户；IM 消息加密仍由
/// 独立设备密钥承担。此 payload 只用于让钱包签名确认“此设备属于此账户”。
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

  /// 用户可见聊天账户，也是公民币转账付款账户。
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
  String canonicalPayload() {
    return [
      'GMB_IM_WALLET_BINDING_V1',
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
