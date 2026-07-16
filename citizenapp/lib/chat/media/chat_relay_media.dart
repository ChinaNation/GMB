// 大媒体(>100MB)中转编排:把 MediaRelayCrypto(客户端流式加密)与 Cloudflare R2
// 瞬时中转 transport 串起来。发送=加密→申请槽→流式 PUT 密文;接收=换 URL→流式 GET
// 密文→流式解密落盘→ack(触发服务端删)。全程流式,5GB 不进内存;Cloudflare 只经手
// 密文、拿不到内容密钥。详见 memory/05-modules/citizenapp/chat/CHAT_GROUP_TECHNICAL.md §11。

import 'dart:convert';
import 'dart:io';
import 'dart:math';

import '../chat_flow.dart' show ChatRelayDescriptor;
import '../transport/chat_cloud_transport.dart';
import 'media_relay_crypto.dart';

class ChatRelayMedia {
  ChatRelayMedia._();

  /// 加密源文件 → 申请上传槽 → 流式 PUT 密文到 R2。返回 E2E 控制消息所需描述子。
  /// 临时密文文件用后即删。
  static Future<ChatRelayDescriptor> upload({
    required ChatCloudTransport transport,
    required String sourcePath,
    required int byteSize,
    required Directory tempDirectory,
    int recipientCount = 1,
  }) async {
    await tempDirectory.create(recursive: true);
    final key = MediaRelayCrypto.newContentKey();
    final encPath = '${tempDirectory.path}/relay-up-${_nonce()}.enc';
    final encSize = await MediaRelayCrypto.encryptFile(
      sourcePath: sourcePath,
      destPath: encPath,
      key: key,
    );
    try {
      // 服务端按**明文** byteSize 门控(>100MB,≤5GB);recipientCount 供群删时机。
      final init = await transport.initRelayUpload(
        byteSize: byteSize,
        recipientCount: recipientCount,
      );
      final objectKey = (init['object_key'] ?? '').toString();
      if (objectKey.isEmpty) {
        throw StateError('中转上传槽申请失败');
      }
      await _streamPut(
        transport.relayBlobUri(objectKey),
        transport.sessionBearer,
        encPath,
        encSize,
      );
      return ChatRelayDescriptor(
        relayObjectKey: objectKey,
        contentKeyB64: base64Encode(key),
        chunkSize: MediaRelayCrypto.defaultChunkSize,
        encSize: encSize,
      );
    } finally {
      await _deleteQuietly(encPath);
    }
  }

  /// 换取下载 URL → 流式 GET 密文 → 流式解密落 [destPath] → ack(触发服务端删)。
  static Future<void> download({
    required ChatCloudTransport transport,
    required String relayObjectKey,
    required String contentKeyB64,
    required String destPath,
    required Directory tempDirectory,
  }) async {
    await tempDirectory.create(recursive: true);
    final key = base64Decode(contentKeyB64);
    final encPath = '${tempDirectory.path}/relay-dl-${_nonce()}.enc';
    try {
      await _streamGet(
        transport.relayBlobUri(relayObjectKey),
        transport.sessionBearer,
        encPath,
      );
      await MediaRelayCrypto.decryptFile(
        sourcePath: encPath,
        destPath: destPath,
        key: key,
      );
      // 拉取成功即 ack;失败不 ack(留待 TTL 或下次)。
      await transport.relayAck(relayObjectKey);
    } finally {
      await _deleteQuietly(encPath);
    }
  }

  static Future<void> _streamPut(
    Uri uri,
    String? bearer,
    String filePath,
    int contentLength,
  ) async {
    final client = HttpClient();
    try {
      final request = await client.putUrl(uri);
      request.headers
          .set(HttpHeaders.contentTypeHeader, 'application/octet-stream');
      if ((bearer ?? '').isNotEmpty) {
        request.headers.set(HttpHeaders.authorizationHeader, 'Bearer $bearer');
      }
      request.contentLength = contentLength;
      await request.addStream(File(filePath).openRead());
      final response = await request.close();
      await response.drain<void>();
      if (response.statusCode >= 300) {
        throw StateError('中转上传失败: HTTP ${response.statusCode}');
      }
    } finally {
      client.close(force: true);
    }
  }

  static Future<void> _streamGet(
    Uri uri,
    String? bearer,
    String destPath,
  ) async {
    final client = HttpClient();
    try {
      final request = await client.getUrl(uri);
      if ((bearer ?? '').isNotEmpty) {
        request.headers.set(HttpHeaders.authorizationHeader, 'Bearer $bearer');
      }
      final response = await request.close();
      if (response.statusCode >= 300) {
        throw StateError('中转下载失败: HTTP ${response.statusCode}');
      }
      final sink = File(destPath).openWrite();
      await response.pipe(sink);
    } finally {
      client.close(force: true);
    }
  }

  static Future<void> _deleteQuietly(String path) async {
    try {
      final file = File(path);
      if (await file.exists()) {
        await file.delete();
      }
    } catch (_) {
      // 临时密文清理失败不影响主流程。
    }
  }

  static String _nonce() {
    final random = Random.secure();
    final bytes = List<int>.generate(8, (_) => random.nextInt(256));
    return bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
  }
}
