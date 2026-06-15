import 'dart:convert';
import 'dart:io';

import 'im_mls_session.dart';

/// 公民 IM 的 MLS 本地状态目录。
///
/// OpenMLS provider storage 由 Rust native 写入该目录；Dart 只管理目录位置
/// 和 application 早于 Welcome 到达时的 pending 队列。
class ImMlsStateStore {
  const ImMlsStateStore(this.directory);

  final Directory directory;

  String get path => directory.path;

  Future<void> ensureReady() async {
    if (!directory.existsSync()) {
      await directory.create(recursive: true);
    }
  }

  File get _pendingFile => File('${directory.path}/pending_inbound.json');

  Future<void> queuePendingInbound(ImMlsWireMessage message) async {
    await ensureReady();
    final existing = await readPendingInbound();
    existing.add(message);
    final encoded = existing.map(_wireMessageToJson).toList();
    await _pendingFile.writeAsString(jsonEncode(encoded), flush: true);
  }

  Future<List<ImMlsWireMessage>> readPendingInbound() async {
    if (!_pendingFile.existsSync()) {
      return [];
    }
    final raw = await _pendingFile.readAsString();
    if (raw.trim().isEmpty) {
      return [];
    }
    final items = (jsonDecode(raw) as List).cast<Map<String, dynamic>>();
    return items.map(_wireMessageFromJson).toList();
  }

  Future<void> clearPendingInbound() async {
    if (_pendingFile.existsSync()) {
      await _pendingFile.writeAsString('[]', flush: true);
    }
  }
}

Map<String, Object?> _wireMessageToJson(ImMlsWireMessage message) {
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

ImMlsWireMessage _wireMessageFromJson(Map<String, dynamic> json) {
  return ImMlsWireMessage(
    conversationId: (json['conversation_id'] ?? '').toString(),
    messageKind: ImMlsMessageKind.fromWireName(
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
    throw const FormatException('IM MLS pending hex 长度必须为偶数');
  }
  final bytes = <int>[];
  for (var i = 0; i < normalized.length; i += 2) {
    bytes.add(int.parse(normalized.substring(i, i + 2), radix: 16));
  }
  return bytes;
}
