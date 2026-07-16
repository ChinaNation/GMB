// 二进制前缀域(ACTIVATE_ADMIN/DECRYPT)金标向量测试(ADR-026 Phase 2)。
//
// 金标 fixture 由 Rust 切片导出(canonical 真源
// citizenchain/runtime/primitives/tests/fixtures/binary_prefix_domain_vectors.json
// 的副本)。这两个域**不经 signingMessage 做 hash**:冷钱包对整段原始可解析
// payload 直接 sr25519 签名,node 按字节偏移解析。本测试逐字节断言 Dart 端按
// 相同字段值(nonce 全 0、timestamp=1700000000)构造的 payload == fixture
// payload_hex == Rust 构造,防四方(node/冷钱包/citizenapp)布局漂移。
//
// 注意:真实运行 nonce 为随机 16B、timestamp 为当前秒,故此处复刻确定性输入
// 验证**布局常量**(前缀 4B、各字段偏移/长度),而非业务随机值。

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/signer/signing.dart';

const _fixturePath = 'test/signer/fixtures/binary_prefix_domain_vectors.json';

void main() {
  final file = File(_fixturePath);
  if (!file.existsSync()) {
    test('二进制前缀域金标 fixture 尚未生成', () {
      markTestSkipped('缺少 $_fixturePath —— 由 Rust 切片导出');
    }, skip: '缺少 $_fixturePath');
    return;
  }

  final root = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
  final vectors = (root['vectors'] as List).cast<Map<String, dynamic>>();
  final byName = {for (final v in vectors) v['name'] as String: v};

  group('二进制前缀域金标(链端 node ↔ 冷钱包 ↔ citizenapp 逐字节对齐)', () {
    test('fixture 域常量为 GMB,与 Dart kGmbSignDomain 一致', () {
      expect(root['domain'], 'GMB');
      expect(kGmbSignDomain, equals(const [0x47, 0x4D, 0x42]));
      expect(kBinaryPrefixLen, 4);
    });

    test('binaryDomainPrefix(0x18) == ACTIVATE_ADMIN fixture prefix', () {
      final v = byName['ACTIVATE_ADMIN']!;
      expect(kOpSignActivateAdmin, 0x18);
      expect(
        _hexLower(binaryDomainPrefix(kOpSignActivateAdmin)),
        (v['prefix_hex'] as String).toLowerCase(),
      );
    });

    test('binaryDomainPrefix(0x19) == DECRYPT fixture prefix', () {
      final v = byName['DECRYPT']!;
      expect(kOpSignDecrypt, 0x19);
      expect(
        _hexLower(binaryDomainPrefix(kOpSignDecrypt)),
        (v['prefix_hex'] as String).toLowerCase(),
      );
    });

    test('ACTIVATE_ADMIN payload 逐字节 == fixture == Rust', () {
      final v = byName['ACTIVATE_ADMIN']!;
      final inputs = v['sample_inputs'] as Map<String, dynamic>;
      final payload = activateAdminPayload(
        cidNumber: inputs['cid_number'] as String,
        institutionCode: _hexToBytes(inputs['institution_code_hex'] as String),
        kind: inputs['kind'] as int,
        adminPubkey: _hexToBytes(inputs['pubkey_hex'] as String),
        timestamp: inputs['timestamp'] as int,
        nonce: _hexToBytes(inputs['nonce_hex'] as String),
      );
      expect(payload.length, v['total_len'] as int);
      expect(_hexLower(payload), (v['payload_hex'] as String).toLowerCase());
    });

    test('DECRYPT challenge 逐字节 == fixture == Rust', () {
      final v = byName['DECRYPT']!;
      final inputs = v['sample_inputs'] as Map<String, dynamic>;
      final payload = decryptAdminPayload(
        cidNumber: inputs['cid_number'] as String,
        adminPubkey: _hexToBytes(inputs['pubkey_hex'] as String),
        timestamp: inputs['timestamp'] as int,
        nonce: _hexToBytes(inputs['nonce_hex'] as String),
      );
      expect(payload.length, v['total_len'] as int);
      expect(_hexLower(payload), (v['payload_hex'] as String).toLowerCase());
    });
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
