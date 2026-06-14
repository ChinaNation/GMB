import '../im_session_models.dart';
import 'im_transport.dart';

/// 私人通信全节点端点。
class ImPrivateNodeEndpoint {
  const ImPrivateNodeEndpoint({
    required this.peerId,
    required this.multiaddr,
  });

  /// 节点 PeerId，必须与 multiaddr 的 `/p2p/<peer_id>` 一致。
  final String peerId;

  /// libp2p multiaddr，支持 ip4、ip6、dns4、dnsaddr。
  final String multiaddr;

  /// 是否 IPv6 端点。
  bool get isIpv6 => multiaddr.startsWith('/ip6/');

  /// 是否用户自有域名端点。
  bool get isDns =>
      multiaddr.startsWith('/dns4/') || multiaddr.startsWith('/dnsaddr/');

  /// 本地轻量校验，真实拨号仍由节点端 sc-network 执行。
  String? validate() {
    if (peerId.trim().isEmpty) {
      return 'IM 节点 PeerId 不能为空';
    }
    if (multiaddr.trim().isEmpty) {
      return 'IM 节点 multiaddr 不能为空';
    }
    final allowedPrefix = multiaddr.startsWith('/ip4/') ||
        multiaddr.startsWith('/ip6/') ||
        multiaddr.startsWith('/dns4/') ||
        multiaddr.startsWith('/dnsaddr/');
    if (!allowedPrefix) {
      return 'IM 节点端点只允许 ip4、ip6、dns4 或 dnsaddr';
    }
    if (!multiaddr.endsWith('/p2p/$peerId')) {
      return 'IM 节点 multiaddr 必须以 /p2p/<peer_id> 结束';
    }
    return null;
  }
}

/// 私人节点传输的密文信封草案。
class ImPrivateNodeEnvelopeDraft {
  const ImPrivateNodeEnvelopeDraft({
    required this.envelopeId,
    required this.conversationId,
    required this.senderChatAccount,
    required this.recipientChatAccount,
    required this.senderDeviceId,
    required this.encryptedPayload,
    required this.createdAtMillis,
    required this.ttlMillis,
  });

  /// 全局去重 ID。
  final String envelopeId;

  /// 会话 ID。
  final String conversationId;

  /// 发送方钱包聊天账户。
  final String senderChatAccount;

  /// 接收方钱包聊天账户。
  final String recipientChatAccount;

  /// 发送设备 ID。
  final String senderDeviceId;

  /// 已加密载荷；明文不得进入私人通信全节点。
  final List<int> encryptedPayload;

  /// 创建时间，毫秒时间戳。
  final int createdAtMillis;

  /// TTL，毫秒。
  final int ttlMillis;

  /// Spike 阶段提交给 node 的 hex 字符串，后续由 Protobuf bytes 替换。
  String get encryptedPayloadHex => encryptedPayload
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();
}

/// 私人通信全节点传输骨架。
///
/// 真实实现会通过手机到自家节点的安全通道调用节点 IM 命令，再由节点直连对方
/// 私人通信全节点。当前类只固定页面层依赖的传输边界，不执行网络发送。
class ImPrivateNodeTransport implements ImTransport {
  const ImPrivateNodeTransport({
    required this.ownerChatAccount,
    required this.ownerDeviceId,
    required this.ownerNodeEndpoint,
  });

  /// owner 钱包聊天账户。
  final String ownerChatAccount;

  /// owner 手机 IM 设备 ID。
  final String ownerDeviceId;

  /// owner 自己的私人通信全节点端点。
  final ImPrivateNodeEndpoint ownerNodeEndpoint;

  @override
  ImTransportType get type => ImTransportType.privateNode;

  @override
  Future<ImDeliveryResult> sendEncryptedEnvelope({
    required String envelopeId,
    required List<int> envelopeBytes,
  }) async {
    final endpointError = ownerNodeEndpoint.validate();
    if (endpointError != null) {
      return ImDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: ImMessageDeliveryState.failed,
        errorMessage: endpointError,
      );
    }

    return ImDeliveryResult(
      envelopeId: envelopeId,
      transportType: type,
      state: ImMessageDeliveryState.queued,
      errorMessage: 'IM 私人节点真实传输尚未接入',
    );
  }
}
