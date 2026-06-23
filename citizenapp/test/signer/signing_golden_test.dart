// 全仓签名消息金标向量测试(ADR-026 Tier 2)。
//
// 金标 fixture 由 Rust 切片导出(canonical 真源
// citizenchain/runtime/primitives/tests/fixtures/signing_domain_vectors.json
// 的副本),逐条断言 Dart 镜像 signingMessage(op_tag, scale_payload) == message_hex,
// 防止 Dart 与链端 primitives::sign 漂移。
//
// 治理 5 个 op_tag(0x10-0x14)message_hex 任何时候不得变化;7 个新折字符串域
// (0x15-0x1B)落创世前冻结。
//
// fixture 缺席时(Rust 切片尚未落地)skip,不阻塞纯 Dart 测试。

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/signer/signing.dart';

const _fixturePath = 'test/signer/fixtures/signing_domain_vectors.json';

void main() {
  final file = File(_fixturePath);
  if (!file.existsSync()) {
    test('签名金标 fixture 尚未生成(Rust 切片落地后启用)', () {
      markTestSkipped('缺少 $_fixturePath —— 由 Rust signing golden 切片导出');
    }, skip: '缺少 $_fixturePath');
    return;
  }

  final root = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
  final vectors = (root['vectors'] as List).cast<Map<String, dynamic>>();

  group('签名消息金标向量(链端 primitives::sign ↔ Dart 逐字节对齐)', () {
    test('fixture 域常量为 GMB,与 Dart kGmbSignDomain 一致', () {
      expect(root['domain'], 'GMB');
      expect(kGmbSignDomain, equals(const [0x47, 0x4D, 0x42]));
    });

    for (final v in vectors) {
      final name = v['name'] as String;
      final opTag = int.parse((v['op_tag'] as String).substring(2), radix: 16);
      final scalePayload = _hexToBytes(v['scale_payload_hex'] as String);
      final expectedHex = (v['message_hex'] as String).toLowerCase();

      test('$name (op_tag=0x${opTag.toRadixString(16)}) → $expectedHex', () {
        final actual = signingMessage(opTag: opTag, scalePayload: scalePayload);
        expect(_hexLower(actual), expectedHex);
      });
    }
  });
}

Uint8List _hexToBytes(String hex) {
  final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
  final out = Uint8List(clean.length ~/ 2);
  for (var i = 0; i < out.length; i++) {
    out[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
  }
  return out;
}

String _hexLower(Uint8List bytes) =>
    bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
