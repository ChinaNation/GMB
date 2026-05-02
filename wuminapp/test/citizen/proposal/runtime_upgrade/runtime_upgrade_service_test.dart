import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/citizen/proposal/runtime_upgrade/runtime_upgrade_service.dart';

/// ADR-008 step2d:wuminapp 端 propose_runtime_upgrade SCALE 字节一致性。
///
/// fixture(`test/fixtures/step2d_credential_payload.json`)由 Python 生成,
/// 与链端 commit 81cde87 SCALE 编码逐字节对齐。本测试断言:
///   wuminapp `RuntimeUpgradeService.buildProposeRuntimeUpgradeCallForTest`
///   产出的 call data hex == fixture.expected_call_data_hex
///
/// feedback_qr_signing_two_color.md:任何一端编码漂移即让冷钱包两色识别从绿降黄,
/// 用户拒绝盲签 → fixture 失败优先暴露这种回归。
void main() {
  Map<String, dynamic> readFixture() {
    final file = File('test/fixtures/step2d_credential_payload.json');
    final raw = file.readAsStringSync();
    return jsonDecode(raw) as Map<String, dynamic>;
  }

  Uint8List hexToBytes(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    final out = Uint8List(clean.length ~/ 2);
    for (var i = 0; i < out.length; i++) {
      out[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return out;
  }

  String hexEncode(Uint8List bytes) =>
      '0x${bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';

  group('RuntimeUpgradeService SCALE 字节一致性 (step2d fixture)', () {
    test('propose_runtime_upgrade call data 与 fixture 逐字节一致', () {
      final fixture = readFixture();
      final caseEntry = (fixture['cases'] as List)
          .firstWhere((e) => e['name'] == 'propose_runtime_upgrade');
      final fields = caseEntry['fields'] as Map<String, dynamic>;
      final expectedHex =
          (caseEntry['expected_call_data_hex'] as String).toLowerCase();

      final reason = fields['reason_utf8'] as String;
      final wasm = hexToBytes(fields['wasm_hex'] as String);
      final eligibleTotal = fields['eligible_total'] as int;
      final snapshotNonce =
          Uint8List.fromList(utf8.encode(fields['snapshot_nonce_utf8'] as String));
      final signature = hexToBytes(fields['signature_hex'] as String);
      final province =
          Uint8List.fromList(utf8.encode(fields['province_utf8'] as String));
      final signerAdminPubkey =
          hexToBytes(fields['signer_admin_pubkey_hex'] as String);

      final actual = RuntimeUpgradeService.buildProposeRuntimeUpgradeCallForTest(
        reason: reason,
        wasmCode: wasm,
        eligibleTotal: eligibleTotal,
        snapshotNonce: snapshotNonce,
        signature: signature,
        province: province,
        signerAdminPubkey: signerAdminPubkey,
      );
      final actualHex = hexEncode(actual).toLowerCase();
      expect(actualHex, expectedHex,
          reason: '链端=wumin=wuminapp 三端 SCALE 字节必须一致 (ADR-008 step2d)');
      expect(actual.length, caseEntry['expected_byte_length'] as int);
    });

    test('signer_admin_pubkey 长度非 32 → 拒绝构造 call data', () {
      expect(
        () => RuntimeUpgradeService.buildProposeRuntimeUpgradeCallForTest(
          reason: 'x',
          wasmCode: Uint8List.fromList([0x01]),
          eligibleTotal: 1,
          snapshotNonce: Uint8List(0),
          signature: Uint8List(0),
          province: Uint8List.fromList(utf8.encode('安徽省')),
          signerAdminPubkey: Uint8List(31), // 短 1 字节
        ),
        throwsArgumentError,
      );
    });
  });
}
