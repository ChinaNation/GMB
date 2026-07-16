import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;
import 'package:citizenapp/8964/services/square_request_signer.dart';

import '../chat_models.dart';
import '../crypto/mls_boundary.dart';
import '../proto/chat_envelope.pb.dart';
import 'chat_transport.dart';

const _chatServiceUnavailable = 'Cloudflare Chat 瞬时转发尚未配置';

/// Cloudflare 互联网 Chat 传输，只转发当前请求中的密文和WebRTC信令。
class ChatCloudTransport implements ChatTransport {
  ChatCloudTransport({
    required this.ownerAccount,
    required this.ownerDeviceId,
    this.serviceBaseUrl,
    this.sessionToken,
    this.requestSigner,
    http.Client? httpClient,
    this.requestTimeout = const Duration(seconds: 12),
  }) : _httpClient = httpClient ?? http.Client();

  final String ownerAccount;
  final String ownerDeviceId;
  final Uri? serviceBaseUrl;
  final String? sessionToken;
  final SquareDeviceSigner? requestSigner;
  final Duration requestTimeout;
  final http.Client _httpClient;

  @override
  ChatTransportType get type => ChatTransportType.cloudflare;

  Future<void> registerDevice({
    required String devicePublicKeyHex,
    required String pushProvider,
    required String pushToken,
    required String bindingSignature,
    required int expiresAtMillis,
    required String nonce,
  }) async {
    await _postJson('/v1/chat/devices/register', {
      'device_id': ownerDeviceId,
      'device_public_key_hex': devicePublicKeyHex,
      'push_provider': pushProvider,
      'push_token': pushToken,
      'binding_signature': bindingSignature,
      'expires_at': expiresAtMillis,
      'nonce': nonce,
    });
  }

  Future<void> publishKeyPackage(MlsKeyPackage keyPackage) async {
    await _postJson('/v1/chat/keypackages', {
      'owner_account': keyPackage.ownerAccount,
      'device_id': keyPackage.deviceId,
      'device_public_key_hex': keyPackage.devicePublicKeyHex,
      'key_package_id': keyPackage.keyPackageId,
      'key_package': _base64UrlEncode(keyPackage.keyPackageBytes),
      'cipher_suite': keyPackage.cipherSuite,
      'created_at': keyPackage.createdAtMillis,
      'expires_at': keyPackage.expiresAtMillis,
    });
  }

  Future<List<MlsKeyPackage>> fetchKeyPackages({
    required String ownerAccount,
    required String requesterAccount,
    int limit = 1,
  }) async {
    final json = await _getJson(
      '/v1/chat/keypackages/${Uri.encodeComponent(ownerAccount)}',
      queryParameters: {'limit': limit.toString()},
    );
    final items = json['key_packages'];
    if (items is! List) {
      throw const FormatException('Cloudflare KeyPackage 响应格式无效');
    }
    return items
        .whereType<Map<String, dynamic>>()
        .map(_keyPackageFromJson)
        .toList(growable: false);
  }

  Future<MlsKeyPackage> consumeKeyPackage({
    required String ownerAccount,
    required String keyPackageId,
    required String requesterAccount,
  }) async {
    final json = await _postJson('/v1/chat/keypackages/consume', {
      'owner_account': ownerAccount,
      'key_package_id': keyPackageId,
      'requester_account': requesterAccount,
    });
    final item = json['key_package'];
    if (item is! Map<String, dynamic>) {
      throw const FormatException('Cloudflare KeyPackage 消费响应格式无效');
    }
    return _keyPackageFromJson(item);
  }

  Future<bool> sendSignal({
    required String recipientAccount,
    String? recipientDeviceId,
    required Map<String, dynamic> signal,
  }) async {
    final json = await _postJson('/v1/chat/signals', {
      'sender_device_id': ownerDeviceId,
      'recipient_account': recipientAccount,
      'recipient_device_id': recipientDeviceId ?? '',
      'signal': signal,
    });
    return json['delivery_state'] == 'sent';
  }

  Future<Future<void> Function()?> connectRealtime({
    required Future<void> Function(Map<String, dynamic> message) onMessage,
    Future<void> Function()? onDisconnected,
  }) async {
    final uri = _wsUri('/v1/chat/ws');
    WebSocket socket;
    try {
      socket = await WebSocket.connect(uri.toString(),
              headers: await _wsHeaders(uri))
          .timeout(requestTimeout);
    } catch (_) {
      return null;
    }
    var closedByClient = false;
    late final StreamSubscription<dynamic> subscription;
    subscription = socket.listen(
      (event) {
        final text = event is List<int> ? utf8.decode(event) : event.toString();
        final decoded = jsonDecode(text);
        if (decoded is Map<String, dynamic>) unawaited(onMessage(decoded));
      },
      onDone: () {
        if (!closedByClient) {
          unawaited(onDisconnected?.call() ?? Future<void>.value());
        }
      },
      onError: (_) {
        if (!closedByClient) {
          unawaited(onDisconnected?.call() ?? Future<void>.value());
        }
      },
      cancelOnError: true,
    );
    return () async {
      closedByClient = true;
      await subscription.cancel();
      await socket.close(WebSocketStatus.normalClosure, 'client_close');
    };
  }

  @override
  Future<ChatDeliveryResult> sendEncryptedEnvelope({
    required String envelopeId,
    required List<int> envelopeBytes,
  }) async {
    ChatEnvelope envelope;
    try {
      envelope = ChatEnvelope.fromBuffer(envelopeBytes);
    } catch (error) {
      return ChatDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: ChatMessageDeliveryState.failed,
        errorMessage: '密文 Envelope 格式无效: $error',
      );
    }
    if (serviceBaseUrl == null || (sessionToken ?? '').trim().isEmpty) {
      return ChatDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: ChatMessageDeliveryState.queued,
        errorMessage: _chatServiceUnavailable,
      );
    }
    try {
      final json = await _postJson('/v1/chat/envelopes', {
        'envelope_id': envelope.envelopeId,
        'sender_device_id': envelope.senderDeviceId,
        'recipient_account': envelope.recipientAccount,
        'recipient_device_id': '',
        'envelope': _base64UrlEncode(envelopeBytes),
      });
      return ChatDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: json['delivery_state'] == 'sent'
            ? ChatMessageDeliveryState.sent
            : ChatMessageDeliveryState.queued,
      );
    } catch (error) {
      return ChatDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: ChatMessageDeliveryState.queued,
        errorMessage: error.toString(),
      );
    }
  }

  // ==== 大媒体(>100MB)Cloudflare R2 瞬时中转 ====
  // 服务端按会员(仅薪火)+ 尺寸(>100MB,≤5GB)门控;拒 ≤100MB / 非薪火。
  // 只有 >100MB 走此路,其余一切不经 R2。

  /// 申请上传槽:声明**明文**字节数(供 >100MB 门),返回 {object_key, ttl_millis}。
  /// 服务端此处按会员(仅薪火)+ 尺寸门控。
  Future<Map<String, dynamic>> initRelayUpload({
    required int byteSize,
    int recipientCount = 1,
  }) {
    return _postJson('/v1/chat/relay/init', {
      'byte_size': byteSize,
      'recipient_count': recipientCount,
    });
  }

  /// 收件人已拉取确认:1:1 一人 ack 即删;否则等 TTL 兜底删。
  Future<void> relayAck(String objectKey) async {
    await _postJson(
      '/v1/chat/relay/${Uri.encodeComponent(objectKey)}/ack',
      const {},
    );
  }

  /// blob 代理端点绝对地址(Worker 转发到 R2,会话 bearer 鉴权)。
  Uri relayBlobUri(String objectKey) =>
      _uri('/v1/chat/relay/${Uri.encodeComponent(objectKey)}/blob');

  /// 供大媒体流式 PUT/GET 使用的会话 bearer(blob 只用 bearer 鉴权,内容为 E2E 密文)。
  String? get sessionBearer => sessionToken;

  Future<Map<String, dynamic>> _getJson(String path,
      {Map<String, String>? queryParameters}) async {
    final uri = _uri(path, queryParameters: queryParameters);
    final response = await _httpClient
        .get(uri, headers: await _headers('GET', uri, ''))
        .timeout(requestTimeout);
    return _decodeResponse(response, uri);
  }

  Future<Map<String, dynamic>> _postJson(
      String path, Map<String, Object?> body) async {
    final uri = _uri(path);
    final encoded = jsonEncode(body);
    final response = await _httpClient
        .post(uri, headers: await _headers('POST', uri, encoded), body: encoded)
        .timeout(requestTimeout);
    return _decodeResponse(response, uri);
  }

  Uri _uri(String path, {Map<String, String>? queryParameters}) {
    final base = serviceBaseUrl;
    if (base == null || (sessionToken ?? '').trim().isEmpty) {
      throw StateError(_chatServiceUnavailable);
    }
    // 正式 API 使用同域 `/api` 前缀，不能用 Uri.resolve 丢掉该前缀。
    final root = base.toString().replaceFirst(RegExp(r'/+$'), '');
    final uri = Uri.parse('$root$path');
    return queryParameters == null
        ? uri
        : uri.replace(queryParameters: queryParameters);
  }

  Uri _wsUri(String path) {
    final uri = _uri(path);
    return uri.replace(scheme: uri.scheme == 'https' ? 'wss' : 'ws');
  }

  Future<Map<String, String>> _headers(
      String method, Uri uri, String body) async {
    final token = sessionToken?.trim() ?? '';
    final headers = <String, String>{
      'authorization': 'Bearer $token',
      'content-type': 'application/json; charset=utf-8',
      'accept': 'application/json',
    };
    final signer = requestSigner;
    if (signer != null) {
      headers.addAll(await squareRequestHeaders(
        method: method,
        uri: uri,
        body: body,
        sessionToken: token,
        sign: signer,
      ));
    }
    return headers;
  }

  Future<Map<String, String>> _wsHeaders(Uri uri) async {
    final headers = await _headers('GET', uri, '');
    headers['x-chat-device'] = ownerDeviceId;
    return headers;
  }
}

Map<String, dynamic> _decodeResponse(http.Response response, Uri uri) {
  final decoded = jsonDecode(response.body);
  if (decoded is! Map<String, dynamic>) {
    throw FormatException('Cloudflare Chat 响应不是JSON对象', response.body);
  }
  if (response.statusCode < 200 ||
      response.statusCode >= 300 ||
      decoded['ok'] != true) {
    final message =
        (decoded['message'] ?? decoded['error_code'] ?? '').toString();
    throw StateError(
        'Cloudflare Chat 请求失败 ${response.statusCode}: $message (${uri.path})');
  }
  return decoded;
}

MlsKeyPackage _keyPackageFromJson(Map<String, dynamic> json) => MlsKeyPackage(
      ownerAccount: (json['owner_account'] ?? '').toString(),
      deviceId: (json['device_id'] ?? '').toString(),
      devicePublicKeyHex: (json['device_public_key_hex'] ?? '').toString(),
      keyPackageId: (json['key_package_id'] ?? '').toString(),
      keyPackageBytes: _base64UrlDecode((json['key_package'] ?? '').toString()),
      cipherSuite: (json['cipher_suite'] ?? '').toString(),
      createdAtMillis: (json['created_at'] as num?)?.toInt() ?? 0,
      expiresAtMillis: (json['expires_at'] as num?)?.toInt() ?? 0,
    );

String _base64UrlEncode(List<int> bytes) =>
    base64Url.encode(bytes).replaceAll('=', '');

List<int> _base64UrlDecode(String value) {
  final normalized = value.padRight((value.length + 3) ~/ 4 * 4, '=');
  return base64Url.decode(normalized);
}
