import 'dart:convert';
import 'dart:io';

import '../crypto/im_binding_payload.dart';
import '../crypto/im_mls_boundary.dart';
import '../im_session_models.dart';
import '../proto/im_envelope.pb.dart';
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

  /// node 端要求的端点类型。
  String get kind {
    if (multiaddr.startsWith('/ip4/')) return 'ip4';
    if (multiaddr.startsWith('/ip6/')) return 'ip6';
    if (multiaddr.startsWith('/dns4/')) return 'dns4';
    if (multiaddr.startsWith('/dnsaddr/')) return 'dnsaddr';
    return 'unknown';
  }

  /// 转为 node 端 `ImNodeEndpoint` JSON。
  Map<String, Object?> toJson() {
    return {
      'peer_id': peerId,
      'multiaddr': multiaddr,
      'kind': kind,
    };
  }

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

/// 私人节点传输的密文信封。
///
/// 中文注释：节点端 Spike RPC 仍使用 JSON 包一层路由字段，但
/// [encryptedPayload] 已经可以是完整 GMB_IM_V1 Protobuf envelope bytes。
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

  /// 已加密载荷；当前正式传输完整 Protobuf envelope bytes。
  final List<int> encryptedPayload;

  /// 创建时间，毫秒时间戳。
  final int createdAtMillis;

  /// TTL，毫秒。
  final int ttlMillis;

  /// 提交给 node 的 opaque bytes hex 字符串。
  String get encryptedPayloadHex => encryptedPayload
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();

  /// 从正式 Protobuf envelope 构造节点投递载荷。
  factory ImPrivateNodeEnvelopeDraft.fromEnvelope(ImEnvelope envelope) {
    return ImPrivateNodeEnvelopeDraft(
      envelopeId: envelope.envelopeId,
      conversationId: envelope.conversationId,
      senderChatAccount: envelope.senderChatAccount,
      recipientChatAccount: envelope.recipientChatAccount,
      senderDeviceId: envelope.senderDeviceId,
      encryptedPayload: envelope.writeToBuffer(),
      createdAtMillis: envelope.createdAtMillis.toInt(),
      ttlMillis: envelope.ttlMillis.toInt(),
    );
  }

  /// 转为 node 端 `ImEnvelope` JSON。
  Map<String, Object?> toEnvelopeJson() {
    return {
      'protocol_version': 1,
      'envelope_id': envelopeId,
      'conversation_id': conversationId,
      'sender_chat_account': senderChatAccount,
      'recipient_chat_account': recipientChatAccount,
      'sender_device_id': senderDeviceId,
      'encrypted_payload_hex': encryptedPayloadHex,
      'created_at_millis': createdAtMillis,
      'ttl_millis': ttlMillis,
    };
  }

  /// 从 node 端 `ImEnvelope` JSON 还原。
  factory ImPrivateNodeEnvelopeDraft.fromJson(Map<String, dynamic> json) {
    final payloadHex = (json['encrypted_payload_hex'] ?? '').toString();
    return ImPrivateNodeEnvelopeDraft(
      envelopeId: (json['envelope_id'] ?? '').toString(),
      conversationId: (json['conversation_id'] ?? '').toString(),
      senderChatAccount: (json['sender_chat_account'] ?? '').toString(),
      recipientChatAccount: (json['recipient_chat_account'] ?? '').toString(),
      senderDeviceId: (json['sender_device_id'] ?? '').toString(),
      encryptedPayload: _hexToBytes(payloadHex),
      createdAtMillis: (json['created_at_millis'] as num?)?.toInt() ?? 0,
      ttlMillis: (json['ttl_millis'] as num?)?.toInt() ?? 0,
    );
  }

  /// 尝试把 opaque bytes 解析为正式 Protobuf envelope。
  ImEnvelope parseEnvelope() => ImEnvelope.fromBuffer(encryptedPayload);
}

/// 私人通信全节点传输骨架。
///
/// 手机只连接自己的私人通信全节点；跨用户投递由自己的节点直连对方私人节点。
class ImPrivateNodeTransport implements ImTransport {
  const ImPrivateNodeTransport({
    required this.ownerChatAccount,
    required this.ownerDeviceId,
    required this.ownerNodeEndpoint,
    this.ownerRpcUrl = 'http://127.0.0.1:9944/',
    this.defaultTtlMillis = 30 * 24 * 60 * 60 * 1000,
  });

  /// 本机正在使用的通信钱包账户。
  final String ownerChatAccount;

  /// 本机 wuminapp 的 IM 设备 ID。
  final String ownerDeviceId;

  /// 用户自己的通信节点端点。
  final ImPrivateNodeEndpoint ownerNodeEndpoint;

  /// 用户自己的通信节点 JSON-RPC URL。
  final String ownerRpcUrl;

  /// 接口 fallback 的默认 TTL。
  final int defaultTtlMillis;

  @override
  ImTransportType get type => ImTransportType.privateNode;

  /// 读取用户自己的通信节点 IM 能力。
  Future<Map<String, dynamic>> getCapability() async {
    final result = await _callRpc('im_getCapability', []);
    return (result as Map).cast<String, dynamic>();
  }

  /// 登记本机手机设备绑定。
  Future<void> registerOwnerDevice({
    required ImBindingPayload binding,
    required String walletSignature,
  }) async {
    final endpoints = binding.nodeEndpoints
        .map((multiaddr) => ImPrivateNodeEndpoint(
              peerId: binding.nodePeerId,
              multiaddr: multiaddr,
            ))
        .map((endpoint) {
      final error = endpoint.validate();
      if (error != null) {
        throw Exception(error);
      }
      return endpoint.toJson();
    }).toList();

    await _callRpc('im_registerOwnerDevice', [
      {
        'wallet_account': binding.walletAccount,
        'im_device_id': binding.imDeviceId,
        'im_device_pubkey': binding.imDevicePubkey,
        'node_peer_id': binding.nodePeerId,
        'node_endpoints': endpoints,
        'expires_at_millis': binding.expiresAtMillis,
        'nonce': binding.nonce,
        'wallet_signature': walletSignature,
      }
    ]);
  }

  /// 发布本机 OpenMLS KeyPackage 到自己的私人通信全节点。
  Future<ImMlsKeyPackage> publishKeyPackage(ImMlsKeyPackage keyPackage) async {
    final result = await _callRpc('im_publishKeyPackage', [
      keyPackage.toPublishJson(),
    ]);
    return ImMlsKeyPackage.fromNodeJson(
        (result as Map).cast<String, dynamic>());
  }

  /// 通过自己的私人通信全节点从对方私人节点拉取 KeyPackage。
  Future<List<ImMlsKeyPackage>> fetchDirectKeyPackages({
    required ImPrivateNodeEndpoint remoteEndpoint,
    required String ownerChatAccount,
    required String requesterChatAccount,
    int limit = 1,
  }) async {
    final endpointError = remoteEndpoint.validate();
    if (endpointError != null) {
      throw Exception(endpointError);
    }

    final result = await _callRpc('im_fetchDirectKeyPackages', [
      {
        'remote_endpoint': remoteEndpoint.toJson(),
        'fetch': {
          'owner_wallet_account': ownerChatAccount,
          'requester_chat_account': requesterChatAccount,
          'limit': limit,
        },
      }
    ]);
    final map = (result as Map).cast<String, dynamic>();
    if (map['kind'] == 'Error') {
      throw Exception((map['body'] ?? 'IM KeyPackage 拉取失败').toString());
    }
    if (map['kind'] != 'KeyPackages') {
      throw Exception('IM KeyPackage 响应类型错误:${map['kind']}');
    }
    final rows = (map['body'] as List).cast<dynamic>();
    return rows
        .map((row) => ImMlsKeyPackage.fromNodeJson(
              (row as Map).cast<String, dynamic>(),
            ))
        .toList();
  }

  /// 通过自己的私人通信全节点声明已消费对方一次性 KeyPackage。
  Future<ImMlsKeyPackage> consumeDirectKeyPackage({
    required ImPrivateNodeEndpoint remoteEndpoint,
    required String ownerChatAccount,
    required String keyPackageId,
    required String requesterChatAccount,
  }) async {
    final endpointError = remoteEndpoint.validate();
    if (endpointError != null) {
      throw Exception(endpointError);
    }

    final result = await _callRpc('im_consumeDirectKeyPackage', [
      {
        'remote_endpoint': remoteEndpoint.toJson(),
        'consume': {
          'owner_wallet_account': ownerChatAccount,
          'key_package_id': keyPackageId,
          'requester_chat_account': requesterChatAccount,
        },
      }
    ]);
    final map = (result as Map).cast<String, dynamic>();
    if (map['kind'] == 'Error') {
      throw Exception((map['body'] ?? 'IM KeyPackage 消费失败').toString());
    }
    if (map['kind'] != 'KeyPackageConsumed') {
      throw Exception('IM KeyPackage 响应类型错误:${map['kind']}');
    }
    return ImMlsKeyPackage.fromNodeJson(
      (map['body'] as Map).cast<String, dynamic>(),
    );
  }

  /// 通过自己的私人通信全节点向对方私人通信全节点投递密文。
  Future<ImDeliveryResult> submitDirectEnvelope({
    required ImPrivateNodeEndpoint remoteEndpoint,
    required ImPrivateNodeEnvelopeDraft draft,
  }) async {
    final endpointError = remoteEndpoint.validate();
    if (endpointError != null) {
      return ImDeliveryResult(
        envelopeId: draft.envelopeId,
        transportType: type,
        state: ImMessageDeliveryState.failed,
        errorMessage: endpointError,
      );
    }

    try {
      final result = await _callRpc('im_submitDirectEnvelope', [
        {
          'remote_endpoint': remoteEndpoint.toJson(),
          'submit': {
            'mailbox_owner_chat_account': draft.recipientChatAccount,
            'envelope': draft.toEnvelopeJson(),
          },
        }
      ]);
      final map = (result as Map).cast<String, dynamic>();
      if (map['kind'] == 'EnvelopeAck') {
        return ImDeliveryResult(
          envelopeId: draft.envelopeId,
          transportType: type,
          state: ImMessageDeliveryState.sent,
        );
      }
      return ImDeliveryResult(
        envelopeId: draft.envelopeId,
        transportType: type,
        state: ImMessageDeliveryState.failed,
        errorMessage: (map['body'] ?? 'IM 投递失败').toString(),
      );
    } catch (e) {
      return ImDeliveryResult(
        envelopeId: draft.envelopeId,
        transportType: type,
        state: ImMessageDeliveryState.failed,
        errorMessage: e.toString(),
      );
    }
  }

  /// 拉取当前通信钱包账号 mailbox 中待收密文。
  Future<List<ImPrivateNodeEnvelopeDraft>> fetchPending() async {
    final result = await _callRpc('im_fetchPending', [
      ownerChatAccount,
      ownerDeviceId,
    ]);
    final rows = (result as List).cast<dynamic>();
    return rows
        .map((row) => ImPrivateNodeEnvelopeDraft.fromJson(
              (row as Map).cast<String, dynamic>(),
            ))
        .toList();
  }

  /// 确认本机手机已经处理某个密文信封。
  Future<void> ackEnvelope(String envelopeId) async {
    await _callRpc('im_ackEnvelope', [
      ownerChatAccount,
      ownerDeviceId,
      envelopeId,
    ]);
  }

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

    final now = DateTime.now().millisecondsSinceEpoch;
    final draft = ImPrivateNodeEnvelopeDraft(
      envelopeId: envelopeId,
      conversationId: envelopeId,
      senderChatAccount: ownerChatAccount,
      recipientChatAccount: ownerChatAccount,
      senderDeviceId: ownerDeviceId,
      encryptedPayload: envelopeBytes,
      createdAtMillis: now,
      ttlMillis: defaultTtlMillis,
    );

    try {
      await _callRpc('im_submitEnvelope', [
        {
          'mailbox_owner_chat_account': ownerChatAccount,
          'envelope': draft.toEnvelopeJson(),
        }
      ]);
      return ImDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: ImMessageDeliveryState.sent,
      );
    } catch (e) {
      return ImDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: ImMessageDeliveryState.failed,
        errorMessage: e.toString(),
      );
    }
  }

  Future<dynamic> _callRpc(String method, List<dynamic> params) async {
    final uri = Uri.parse(ownerRpcUrl);
    final requestBody = jsonEncode({
      'jsonrpc': '2.0',
      'id': 1,
      'method': method,
      'params': params,
    });
    final client = HttpClient();
    try {
      final request = await client.postUrl(uri).timeout(
            const Duration(seconds: 10),
          );
      request.headers.contentType = ContentType.json;
      request.write(requestBody);
      final response = await request.close().timeout(
            const Duration(seconds: 15),
          );
      final responseText = await response.transform(utf8.decoder).join();
      if (response.statusCode != HttpStatus.ok) {
        throw Exception('IM 节点 RPC HTTP ${response.statusCode}: $responseText');
      }
      final json = jsonDecode(responseText) as Map<String, dynamic>;
      if (json.containsKey('error')) {
        final error = (json['error'] as Map).cast<String, dynamic>();
        throw Exception('IM 节点 RPC 调用失败:${error['message'] ?? '未知错误'}');
      }
      return json['result'];
    } finally {
      client.close(force: true);
    }
  }
}

List<int> _hexToBytes(String value) {
  final normalized = value.startsWith('0x') ? value.substring(2) : value;
  if (normalized.length.isOdd) {
    throw const FormatException('IM 密文 hex 长度必须为偶数');
  }
  final bytes = <int>[];
  for (var i = 0; i < normalized.length; i += 2) {
    bytes.add(int.parse(normalized.substring(i, i + 2), radix: 16));
  }
  return bytes;
}
