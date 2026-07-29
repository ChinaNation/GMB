import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:citizenapp/security/local_cipher.dart';

import 'mls_session.dart';

/// 公民 Chat 的 MLS 本地状态目录。
///
/// OpenMLS provider storage 由 Rust native 写入该目录；Dart 只管理目录位置、
/// 下传状态信封密钥，以及 application 早于 Welcome 到达时的 pending 队列。
///
/// 该目录下**一律不得出现明文**：`openmls_storage.bin` / `device.bin` 由 Rust
/// 用 [stateKey] 做 AES-256-GCM 信封；`pending_inbound.bin` 由本类同钥加密。
class MlsStateStore {
  const MlsStateStore(this.directory, {required this.stateKey});

  final Directory directory;

  /// MLS 本地状态密钥（32 字节，来自 `LocalKeyPurpose.mls` 子钥）。
  final Uint8List stateKey;

  String get path => directory.path;

  /// 下传给 Rust native 的小写 hex 形式。
  String get stateKeyHex =>
      stateKey.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  static const String _pendingAad = 'citizenapp.local/mls|pending_inbound';

  Future<void> ensureReady() async {
    if (!directory.existsSync()) {
      await directory.create(recursive: true);
    }
    _purgeLegacyPlaintext();
  }

  File get _pendingFile => File('${directory.path}/pending_inbound.bin');

  /// 清除历史遗留的**明文** pending 队列。
  ///
  /// 旧 `pending_inbound.json` 明文保存待处理 envelope，留在磁盘上等于加密白做。
  /// 开发期零用户，直接删除、不做迁移与兼容读取。
  void _purgeLegacyPlaintext() {
    final legacy = File('${directory.path}/pending_inbound.json');
    if (legacy.existsSync()) {
      legacy.deleteSync();
    }
  }

  Future<void> queuePendingInbound(MlsWireMessage message) async {
    await ensureReady();
    final existing = await readPendingInbound();
    existing.add(message);
    final encoded = existing.map(_wireMessageToJson).toList();
    await _writePending(encoded);
  }

  Future<List<MlsWireMessage>> readPendingInbound() async {
    if (!_pendingFile.existsSync()) {
      return [];
    }
    final blob = await _pendingFile.readAsString();
    if (blob.trim().isEmpty) {
      return [];
    }
    // 解密失败必须抛出：静默返回空会让早到的 application message 被悄悄丢弃。
    final raw = await LocalCipher.decryptString(
      key: stateKey,
      blob: blob,
      aad: _pendingAad,
    );
    if (raw.trim().isEmpty) {
      return [];
    }
    final items = (jsonDecode(raw) as List).cast<Map<String, dynamic>>();
    return items.map(_wireMessageFromJson).toList();
  }

  Future<void> clearPendingInbound() async {
    if (_pendingFile.existsSync()) {
      await _writePending(const <Map<String, Object?>>[]);
    }
  }

  Future<void> _writePending(List<Map<String, Object?>> items) async {
    final blob = await LocalCipher.encryptString(
      key: stateKey,
      plaintext: jsonEncode(items),
      aad: _pendingAad,
    );
    await _pendingFile.writeAsString(blob, flush: true);
  }
}

Map<String, Object?> _wireMessageToJson(MlsWireMessage message) {
  return {
    'conversation_id': message.conversationId,
    'message_kind': message.messageKind.wireName,
    'cipher_suite': message.cipherSuite,
    'wire_hex': _bytesToHex(message.wireBytes),
    'ratchet_tree_hex': message.ratchetTreeBytes == null
        ? null
        : _bytesToHex(message.ratchetTreeBytes!),
  };
}

MlsWireMessage _wireMessageFromJson(Map<String, dynamic> json) {
  return MlsWireMessage(
    conversationId: (json['conversation_id'] ?? '').toString(),
    messageKind: MlsMessageKind.fromWireName(
      (json['message_kind'] ?? '').toString(),
    ),
    cipherSuite: (json['cipher_suite'] ?? '').toString(),
    wireBytes: _hexToBytes((json['wire_hex'] ?? '').toString()),
    ratchetTreeBytes: json['ratchet_tree_hex'] == null
        ? null
        : _hexToBytes(json['ratchet_tree_hex'].toString()),
  );
}

String _bytesToHex(List<int> bytes) {
  return bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}

List<int> _hexToBytes(String value) {
  final normalized = value.startsWith('0x') ? value.substring(2) : value;
  if (normalized.length.isOdd) {
    throw const FormatException('Chat MLS pending hex 长度必须为偶数');
  }
  final bytes = <int>[];
  for (var i = 0; i < normalized.length; i += 2) {
    bytes.add(int.parse(normalized.substring(i, i + 2), radix: 16));
  }
  return bytes;
}
