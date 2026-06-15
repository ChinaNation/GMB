import 'package:wuminapp_mobile/qr/envelope.dart';

/// kind = im_node_pairing
///
/// 中文注释：这是公民扫描自己电脑区块链软件通信节点二维码时使用的 body。
/// 它只保存通信节点配对信息，不添加联系人，不改变聊天账户。
class ImNodePairingBody implements QrBody {
  const ImNodePairingBody({
    required this.nodePeerId,
    required this.rpcUrl,
    required this.nodeMultiaddr,
    required this.endpointKind,
    required this.pairingNonce,
    required this.createdAtMillis,
    required this.expiresAtMillis,
  });

  static const proto = 'GMB_IM_NODE_PAIRING_V1';

  final String nodePeerId;
  final String rpcUrl;
  final String nodeMultiaddr;
  final String endpointKind;
  final String pairingNonce;
  final int createdAtMillis;
  final int expiresAtMillis;

  factory ImNodePairingBody.fromJson(Map<String, dynamic> json) {
    final bodyProto = json['proto'];
    if (bodyProto != proto) {
      throw FormatException('通信节点配对协议无效：$bodyProto');
    }
    final body = ImNodePairingBody(
      nodePeerId: _requireString(json, 'node_peer_id'),
      rpcUrl: _requireString(json, 'rpc_url'),
      nodeMultiaddr: _requireString(json, 'node_multiaddr'),
      endpointKind: _requireString(json, 'endpoint_kind'),
      pairingNonce: _requireString(json, 'pairing_nonce'),
      createdAtMillis: _requireInt(json, 'created_at_millis'),
      expiresAtMillis: _requireInt(json, 'expires_at_millis'),
    );
    body.validate();
    return body;
  }

  @override
  Map<String, dynamic> toJson() => {
        'proto': proto,
        'node_peer_id': nodePeerId,
        'rpc_url': rpcUrl,
        'node_multiaddr': nodeMultiaddr,
        'endpoint_kind': endpointKind,
        'pairing_nonce': pairingNonce,
        'created_at_millis': createdAtMillis,
        'expires_at_millis': expiresAtMillis,
      };

  /// 校验二维码字段，防止把非通信节点二维码保存成本机节点。
  void validate() {
    if (nodePeerId.trim().isEmpty) {
      throw const FormatException('通信节点 PeerId 不能为空');
    }
    final rpc = rpcUrl.trim();
    if (!rpc.startsWith('http://') && !rpc.startsWith('https://')) {
      throw const FormatException('通信节点 RPC URL 必须使用 http 或 https');
    }
    final multiaddr = nodeMultiaddr.trim();
    final allowedEndpoint = multiaddr.startsWith('/ip4/') ||
        multiaddr.startsWith('/ip6/') ||
        multiaddr.startsWith('/dns4/') ||
        multiaddr.startsWith('/dnsaddr/');
    if (!allowedEndpoint) {
      throw const FormatException('通信节点端点只允许 ip4、ip6、dns4 或 dnsaddr');
    }
    if (!multiaddr.endsWith('/p2p/$nodePeerId')) {
      throw const FormatException('通信节点二维码无效：PeerId 与 multiaddr 不一致');
    }
    if (pairingNonce.trim().isEmpty) {
      throw const FormatException('通信节点配对 nonce 不能为空');
    }
    if (expiresAtMillis <= createdAtMillis) {
      throw const FormatException('通信节点二维码过期时间无效');
    }
  }

  bool get isExpired =>
      DateTime.now().millisecondsSinceEpoch >= expiresAtMillis;
}

String _requireString(Map<String, dynamic> json, String key) {
  final value = json[key];
  if (value is! String || value.trim().isEmpty) {
    throw FormatException('$key 必填且必须为非空字符串');
  }
  return value.trim();
}

int _requireInt(Map<String, dynamic> json, String key) {
  final value = json[key];
  if (value is! int) {
    throw FormatException('$key 必填且必须为整数');
  }
  return value;
}
