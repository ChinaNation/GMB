import 'dart:async';
import 'dart:convert';
import 'dart:io';

import '../crypto/im_mls_boundary.dart';
import '../im_session_models.dart';
import '../proto/im_envelope.pb.dart';
import 'im_transport.dart';
import 'package:http/http.dart' as http;

const _cloudflareMailboxPendingMessage = 'Cloudflare 密文 mailbox 尚未配置';

/// Cloudflare 密文 mailbox 传输。
///
/// 本类是互联网聊天的唯一正式远程 transport。Cloudflare 只能接收完整
/// GMB_IM_V1 Protobuf envelope bytes、KeyPackage 和 ack 元数据，不能接触
/// 私聊或群聊明文。
class ImCloudflareTransport implements ImTransport {
  ImCloudflareTransport({
    required this.ownerChatAccount,
    required this.ownerDeviceId,
    this.mailboxBaseUrl,
    this.sessionToken,
    http.Client? httpClient,
    this.requestTimeout = const Duration(seconds: 12),
  }) : _httpClient = httpClient ?? http.Client();

  /// 当前手机正在使用的钱包聊天账户。
  final String ownerChatAccount;

  /// 当前手机本地 IM 设备 ID。
  final String ownerDeviceId;

  /// 后续接入的 Worker API 地址；为空表示还未配置正式远程 mailbox。
  final Uri? mailboxBaseUrl;

  /// Worker 钱包登录态 token。该 token 只授权访问自己的密文 mailbox。
  final String? sessionToken;

  /// HTTP 请求超时时间。
  final Duration requestTimeout;

  final http.Client _httpClient;

  @override
  ImTransportType get type => ImTransportType.cloudflare;

  /// 登记当前 IM 设备。签名由钱包对 IM binding payload 授权生成。
  Future<void> registerDevice({
    required String devicePublicKeyHex,
    required String bindingSignature,
    required int expiresAtMillis,
    required String nonce,
  }) async {
    await _postJson(
      '/v1/chat/devices/register',
      {
        'owner_account': ownerChatAccount,
        'device_id': ownerDeviceId,
        'device_public_key_hex': devicePublicKeyHex,
        'binding_signature': bindingSignature,
        'expires_at': expiresAtMillis,
        'nonce': nonce,
      },
    );
  }

  /// 发布本设备 OpenMLS KeyPackage。
  Future<void> publishKeyPackage(ImMlsKeyPackage keyPackage) async {
    await _postJson(
      '/v1/chat/keypackages',
      {
        'owner_account': keyPackage.ownerChatAccount,
        'device_id': keyPackage.deviceId,
        'device_public_key_hex': keyPackage.devicePublicKeyHex,
        'key_package_id': keyPackage.keyPackageId,
        'key_package': _base64UrlEncode(keyPackage.keyPackageBytes),
        'cipher_suite': keyPackage.cipherSuite,
        'created_at': keyPackage.createdAtMillis,
        'expires_at': keyPackage.expiresAtMillis,
      },
    );
  }

  /// 从 Cloudflare mailbox 拉取对方账号可用的 OpenMLS KeyPackage。
  Future<List<ImMlsKeyPackage>> fetchKeyPackages({
    required String ownerChatAccount,
    required String requesterChatAccount,
    int limit = 1,
  }) async {
    final json = await _getJson(
      '/v1/chat/keypackages/${Uri.encodeComponent(ownerChatAccount)}',
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

  /// 声明消费对方的一次性 KeyPackage。
  Future<ImMlsKeyPackage> consumeKeyPackage({
    required String ownerChatAccount,
    required String keyPackageId,
    required String requesterChatAccount,
  }) async {
    final json = await _postJson(
      '/v1/chat/keypackages/consume',
      {
        'owner_account': ownerChatAccount,
        'key_package_id': keyPackageId,
        'requester_account': requesterChatAccount,
      },
    );
    final item = json['key_package'];
    if (item is! Map<String, dynamic>) {
      throw const FormatException('Cloudflare KeyPackage 消费响应格式无效');
    }
    return _keyPackageFromJson(item);
  }

  /// 拉取当前设备待收密文 envelope。
  Future<List<ImPendingEncryptedEnvelope>> fetchPending() async {
    final json = await _getJson(
      '/v1/chat/envelopes/pending',
      queryParameters: {
        'owner_account': ownerChatAccount,
        'device_id': ownerDeviceId,
        'limit': '100',
      },
    );
    final items = json['envelopes'];
    if (items is! List) {
      throw const FormatException('Cloudflare pending envelope 响应格式无效');
    }
    return items.whereType<Map<String, dynamic>>().map((item) {
      final envelopeId = (item['envelope_id'] ?? '').toString();
      final envelope = (item['envelope'] ?? '').toString();
      return ImPendingEncryptedEnvelope(
        envelopeId: envelopeId,
        envelopeBytes: _base64UrlDecode(envelope),
      );
    }).toList(growable: false);
  }

  /// 确认当前设备已经处理某个密文 envelope。
  Future<void> ackEnvelope(String envelopeId) async {
    await _postJson(
      '/v1/chat/envelopes/ack',
      {
        'owner_account': ownerChatAccount,
        'device_id': ownerDeviceId,
        'envelope_id': envelopeId,
      },
    );
  }

  /// 为 IM 加密附件申请 manifest 和分片上传地址。
  Future<ImAttachmentUploadPlan> prepareAttachmentUpload({
    required String conversationId,
    required String attachmentId,
    required int manifestByteSize,
    required List<ImAttachmentChunkDraft> chunks,
  }) async {
    final json = await _postJson(
      '/v1/chat/attachments/prepare',
      {
        'owner_account': ownerChatAccount,
        'device_id': ownerDeviceId,
        'conversation_id': conversationId,
        'attachment_id': attachmentId,
        'manifest_byte_size': manifestByteSize,
        'chunks': chunks
            .map(
              (chunk) => {
                'chunk_id': chunk.chunkId,
                'byte_size': chunk.byteSize,
              },
            )
            .toList(growable: false),
      },
    );
    final rawChunks = json['chunks'];
    if (rawChunks is! List) {
      throw const FormatException('Cloudflare 附件上传计划响应格式无效');
    }
    return ImAttachmentUploadPlan(
      attachmentId: (json['attachment_id'] ?? '').toString(),
      manifestObjectKey: (json['manifest_object_key'] ?? '').toString(),
      manifestUploadUrl:
          Uri.parse((json['manifest_upload_url'] ?? '').toString()),
      chunks: rawChunks
          .whereType<Map<String, dynamic>>()
          .map(
            (item) => ImAttachmentUploadTarget(
              chunkId: (item['chunk_id'] ?? '').toString(),
              objectKey: (item['object_key'] ?? '').toString(),
              uploadUrl: Uri.parse((item['upload_url'] ?? '').toString()),
            ),
          )
          .toList(growable: false),
    );
  }

  /// 上传单个加密附件对象。这里的 [bytes] 必须已经在手机本地加密。
  Future<void> uploadAttachmentObject({
    required Uri uploadUrl,
    required List<int> bytes,
    required String contentType,
  }) async {
    final headers = <String, String>{
      'content-type': contentType,
    };
    if (uploadUrl.path.endsWith('/v1/chat/attachments/dev-put')) {
      headers['authorization'] = 'Bearer ${sessionToken ?? ''}';
    }
    final response = await _httpClient
        .put(
          uploadUrl,
          headers: headers,
          body: bytes,
        )
        .timeout(requestTimeout);
    if (response.statusCode < 200 || response.statusCode >= 300) {
      _decodeResponse(response, uploadUrl);
    }
  }

  /// 完成 IM 加密附件上传。Worker 只校验 R2 密文对象存在。
  Future<void> completeAttachmentUpload(
      ImAttachmentCompleteRequest input) async {
    await _postJson(
      '/v1/chat/attachments/complete',
      {
        'owner_account': ownerChatAccount,
        'device_id': ownerDeviceId,
        'conversation_id': input.conversationId,
        'attachment_id': input.attachmentId,
        'manifest_object_key': input.manifestObjectKey,
        'manifest_hash': input.manifestHash,
        'chunk_refs': input.chunkObjectKeys,
      },
    );
  }

  /// 为 IM 加密附件申请 manifest 和分片下载地址。
  Future<ImAttachmentDownloadPlan> prepareAttachmentDownload(
      ImAttachmentDownloadRequest input) async {
    final json = await _postJson(
      '/v1/chat/attachments/download',
      {
        'owner_account': ownerChatAccount,
        'device_id': ownerDeviceId,
        'conversation_id': input.conversationId,
        'attachment_id': input.attachmentId,
        'manifest_object_key': input.manifestObjectKey,
        'manifest_hash': input.manifestHash,
        'chunk_refs': input.chunkObjectKeys,
      },
    );
    final rawChunks = json['chunks'];
    if (rawChunks is! List) {
      throw const FormatException('Cloudflare 附件下载计划响应格式无效');
    }
    return ImAttachmentDownloadPlan(
      attachmentId: (json['attachment_id'] ?? '').toString(),
      manifestObjectKey: (json['manifest_object_key'] ?? '').toString(),
      manifestDownloadUrl:
          Uri.parse((json['manifest_download_url'] ?? '').toString()),
      chunks: rawChunks
          .whereType<Map<String, dynamic>>()
          .map(
            (item) => ImAttachmentDownloadTarget(
              objectKey: (item['object_key'] ?? '').toString(),
              downloadUrl: Uri.parse((item['download_url'] ?? '').toString()),
            ),
          )
          .toList(growable: false),
    );
  }

  /// 下载单个加密附件对象。返回值仍是密文字节，由上层本地解密。
  Future<List<int>> downloadAttachmentObject(Uri downloadUrl) async {
    final headers = <String, String>{};
    if (downloadUrl.path.endsWith('/v1/chat/attachments/dev-get')) {
      headers['authorization'] = 'Bearer ${sessionToken ?? ''}';
    }
    final response = await _httpClient
        .get(downloadUrl, headers: headers)
        .timeout(requestTimeout);
    if (response.statusCode < 200 || response.statusCode >= 300) {
      _decodeResponse(response, downloadUrl);
    }
    return response.bodyBytes;
  }

  /// 连接 Cloudflare mailbox 的实时通知通道。
  ///
  /// WebSocket 只通知“有新密文”，不会推送明文或密文正文；页面收到通知后
  /// 仍调用 pending/ack 旧流程同步，断线时由页面恢复轮询兜底。
  Future<Future<void> Function()?> connectRealtime({
    required Future<void> Function(Map<String, dynamic> notification)
        onNotification,
    Future<void> Function()? onDisconnected,
  }) async {
    final uri = _wsUri(
      '/v1/chat/ws',
      queryParameters: {
        'owner_account': ownerChatAccount,
        'device_id': ownerDeviceId,
      },
    );
    WebSocket socket;
    try {
      socket = await WebSocket.connect(
        uri.toString(),
        headers: _wsHeaders(),
      ).timeout(requestTimeout);
    } catch (_) {
      return null;
    }

    var closedByClient = false;
    late final StreamSubscription<dynamic> subscription;
    subscription = socket.listen(
      (event) {
        final text = event is List<int> ? utf8.decode(event) : event.toString();
        final decoded = jsonDecode(text);
        if (decoded is Map<String, dynamic>) {
          unawaited(onNotification(decoded));
        }
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
  Future<ImDeliveryResult> sendEncryptedEnvelope({
    required String envelopeId,
    required List<int> envelopeBytes,
  }) async {
    try {
      ImEnvelope.fromBuffer(envelopeBytes);
    } catch (error) {
      return ImDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: ImMessageDeliveryState.failed,
        errorMessage: '密文 envelope 格式无效: $error',
      );
    }

    if (mailboxBaseUrl == null || (sessionToken ?? '').trim().isEmpty) {
      return ImDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: ImMessageDeliveryState.failed,
        errorMessage: _cloudflareMailboxPendingMessage,
      );
    }

    final envelope = ImEnvelope.fromBuffer(envelopeBytes);
    try {
      await _postJson(
        '/v1/chat/envelopes',
        {
          'envelope_id': envelope.envelopeId,
          'conversation_id': envelope.conversationId,
          'sender_account': envelope.senderChatAccount,
          'sender_device_id': envelope.senderDeviceId,
          'recipient_account': envelope.recipientChatAccount,
          'recipient_device_id': '',
          'mls_message_kind': _mlsKindName(envelope.mlsMessageKind),
          'envelope': _base64UrlEncode(envelopeBytes),
          'attachment_manifest_key': envelope.attachmentManifestHash.isEmpty ||
                  envelope.chunkRefs.isEmpty
              ? ''
              : envelope.chunkRefs.first,
          'created_at': envelope.createdAtMillis.toInt(),
          'expires_at':
              envelope.createdAtMillis.toInt() + envelope.ttlMillis.toInt(),
        },
      );
      return ImDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: ImMessageDeliveryState.sent,
      );
    } catch (error) {
      return ImDeliveryResult(
        envelopeId: envelopeId,
        transportType: type,
        state: ImMessageDeliveryState.failed,
        errorMessage: error.toString(),
      );
    }
  }

  Future<Map<String, dynamic>> _getJson(
    String path, {
    Map<String, String>? queryParameters,
  }) async {
    final uri = _uri(path, queryParameters: queryParameters);
    final response =
        await _httpClient.get(uri, headers: _headers()).timeout(requestTimeout);
    return _decodeResponse(response, uri);
  }

  Future<Map<String, dynamic>> _postJson(
    String path,
    Map<String, Object?> body,
  ) async {
    final uri = _uri(path);
    final response = await _httpClient
        .post(uri, headers: _headers(), body: jsonEncode(body))
        .timeout(requestTimeout);
    return _decodeResponse(response, uri);
  }

  Uri _uri(String path, {Map<String, String>? queryParameters}) {
    final base = mailboxBaseUrl;
    final token = sessionToken?.trim() ?? '';
    if (base == null || token.isEmpty) {
      throw StateError(_cloudflareMailboxPendingMessage);
    }
    final uri = base.resolve(path);
    return queryParameters == null
        ? uri
        : uri.replace(queryParameters: queryParameters);
  }

  Uri _wsUri(String path, {Map<String, String>? queryParameters}) {
    final uri = _uri(path, queryParameters: queryParameters);
    final scheme = uri.scheme == 'https' ? 'wss' : 'ws';
    return uri.replace(scheme: scheme);
  }

  Map<String, String> _headers() {
    final token = sessionToken?.trim() ?? '';
    return {
      'authorization': 'Bearer $token',
      'content-type': 'application/json; charset=utf-8',
      'accept': 'application/json',
    };
  }

  Map<String, String> _wsHeaders() {
    final token = sessionToken?.trim() ?? '';
    return {
      'authorization': 'Bearer $token',
    };
  }
}

Map<String, dynamic> _decodeResponse(http.Response response, Uri uri) {
  final decoded = jsonDecode(response.body);
  if (decoded is! Map<String, dynamic>) {
    throw FormatException('Cloudflare mailbox 响应不是 JSON 对象', response.body);
  }
  if (response.statusCode < 200 || response.statusCode >= 300) {
    final message =
        (decoded['message'] ?? decoded['error_code'] ?? '').toString();
    throw StateError(
        'Cloudflare mailbox 请求失败 ${response.statusCode}: $message (${uri.path})');
  }
  if (decoded['ok'] != true) {
    throw StateError(
        'Cloudflare mailbox 返回失败: ${decoded['error_code'] ?? 'unknown'}');
  }
  return decoded;
}

ImMlsKeyPackage _keyPackageFromJson(Map<String, dynamic> json) {
  return ImMlsKeyPackage(
    ownerChatAccount: (json['owner_account'] ?? '').toString(),
    deviceId: (json['device_id'] ?? '').toString(),
    devicePublicKeyHex: (json['device_public_key_hex'] ?? '').toString(),
    keyPackageId: (json['key_package_id'] ?? '').toString(),
    keyPackageBytes: _base64UrlDecode((json['key_package'] ?? '').toString()),
    cipherSuite: (json['cipher_suite'] ?? '').toString(),
    createdAtMillis: (json['created_at'] as num?)?.toInt() ?? 0,
    expiresAtMillis: (json['expires_at'] as num?)?.toInt() ?? 0,
    consumedAtMillis: (json['consumed_at'] as num?)?.toInt(),
  );
}

String _mlsKindName(ImMlsWireMessageKind kind) {
  return switch (kind) {
    ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_WELCOME => 'welcome',
    ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_APPLICATION => 'application',
    _ => 'unspecified',
  };
}

String _base64UrlEncode(List<int> bytes) {
  return base64Url.encode(bytes).replaceAll('=', '');
}

List<int> _base64UrlDecode(String value) {
  final normalized = value.padRight((value.length + 3) ~/ 4 * 4, '=');
  return base64Url.decode(normalized);
}
