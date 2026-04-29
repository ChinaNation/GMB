import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wumin/signer/pallet_registry.dart';
import 'package:wumin/signer/payload_decoder.dart';

void main() {
  final spec = PalletRegistry.supportedSpecVersions.first;

  group('PayloadDecoder', () {
    test('decodes transfer_keep_alive (pallet=2 call=3)', () {
      final dest = Keyring.sr25519.fromSeed(Uint8List(32));
      dest.ss58Format = 2027;
      final destBytes = dest.bytes().toList();

      // 23400 分 = 234 元,Compact four-byte mode:(23400 << 2) | 2
      final payload = Uint8List.fromList([
        0x02, 0x03,
        0x00, // MultiAddress::Id
        ...destBytes,
        0xA2, 0x6D, 0x01, 0x00, // Compact(23400)
      ]);

      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'transfer');
      expect(decoded.fields['amount_yuan'], '234.00 GMB');
      expect(decoded.fields['to'], dest.address);
    });

    // Phase 3(2026-04-22)「投票引擎统一入口整改」:
    // 所有业务 pallet 的 vote_X 已物理删除,所有管理员投票统一走
    // VotingEngine::internal_vote(9.0)。

    test('decodes internal_vote (pallet=9 call=0) approve=true', () {
      // [0x09, 0x00, u64_le proposal_id=42, bool approve=true]
      final payload = Uint8List.fromList([
        0x09, 0x00,
        42, 0, 0, 0, 0, 0, 0, 0,
        1, // approve = true
      ]);
      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'internal_vote');
      expect(decoded.fields['proposal_id'], '42');
      expect(decoded.fields['approve'], 'true');
      expect(decoded.summary, contains('赞成'));
    });

    test('decodes internal_vote (pallet=9 call=0) approve=false', () {
      final payload = Uint8List.fromList([
        0x09, 0x00,
        7, 0, 0, 0, 0, 0, 0, 0,
        0,
      ]);
      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);
      expect(decoded!.action, 'internal_vote');
      expect(decoded.fields['approve'], 'false');
      expect(decoded.summary, contains('反对'));
    });

    test('decodes joint_vote (pallet=9 call=1)', () {
      // Phase 2 重排：joint_vote 由原 call=3 迁到 call=1。
      final payload = Uint8List.fromList([
        0x09, 0x01,
        7, 0, 0, 0, 0, 0, 0, 0,
        ...List.filled(48, 0),
        0,
      ]);
      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'joint_vote');
      expect(decoded.fields['proposal_id'], '7');
      expect(decoded.fields['approve'], 'false');
      expect(decoded.summary, contains('反对'));
    });

    test('decodes citizen_vote (pallet=9 call=2)', () {
      // Phase 2 重排：citizen_vote 由原 call=4 迁到 call=2。
      final payload = Uint8List.fromList([
        0x09, 0x02,
        99, 0, 0, 0, 0, 0, 0, 0,
        ...List.filled(32, 0),
        0, // Vec nonce len=0
        0, // Vec sig len=0
        1,
      ]);
      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'citizen_vote');
      expect(decoded.fields['proposal_id'], '99');
      expect(decoded.fields['approve'], 'true');
    });

    test('decodes finalize_proposal (pallet=9 call=3)', () {
      final payload = Uint8List.fromList([
        0x09, 0x03,
        15, 0, 0, 0, 0, 0, 0, 0,
      ]);
      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);
      expect(decoded!.action, 'finalize_proposal');
      expect(decoded.fields['proposal_id'], '15');
    });

    test('returns null for unknown pallet', () {
      expect(PayloadDecoder.decode('0xff01', specVersion: spec), isNull);
    });

    test('returns null for too-short input', () {
      expect(PayloadDecoder.decode('0x02', specVersion: spec), isNull);
    });

    test('returns null for unsupported specVersion (spec=1 旧版)', () {
      expect(PayloadDecoder.decode('0x0900', specVersion: 1), isNull);
    });

    test('returns null for unsupported specVersion (未来版)', () {
      expect(PayloadDecoder.decode('0x0900', specVersion: 999), isNull);
    });

    test('returns null for null specVersion', () {
      expect(PayloadDecoder.decode('0x0900'), isNull);
    });

    test('decodes propose_sweep_to_main 国储会 (pallet=19 call=2)', () {
      // Phase 2 重排：propose_sweep_to_main 由原 call=5 迁到 call=2。
      const shenfenId = 'GFR-LN001-CB0C-617776487-20260222';
      final idBytes = List<int>.filled(48, 0);
      final idChars = shenfenId.codeUnits;
      for (var i = 0; i < idChars.length; i++) {
        idBytes[i] = idChars[i];
      }
      const amount = 10000;
      final amountBytes = List<int>.filled(16, 0);
      amountBytes[0] = amount & 0xff;
      amountBytes[1] = (amount >> 8) & 0xff;

      final payload = Uint8List.fromList([
        0x13, 0x02,
        ...idBytes,
        ...amountBytes,
      ]);

      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_sweep_to_main');
      expect(decoded.fields['institution'], '国家储备委员会');
      expect(decoded.fields['amount_yuan'], '100.00 GMB');
    });

    test('decodes propose_sweep_to_main 省储会 (pallet=19 call=2)', () {
      const shenfenId = 'GFR-ZS001-CB0X-464088047-20260222';
      final idBytes = List<int>.filled(48, 0);
      final idChars = shenfenId.codeUnits;
      for (var i = 0; i < idChars.length; i++) {
        idBytes[i] = idChars[i];
      }
      final amountBytes = List<int>.filled(16, 0);
      amountBytes[0] = 0x10;

      final payload = Uint8List.fromList([
        0x13, 0x02,
        ...idBytes,
        ...amountBytes,
      ]);
      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.fields['institution'], '中枢省储备委员会');
    });

    test('Compact encoding mode 1 (two-byte)', () {
      final dest = Keyring.sr25519.fromSeed(Uint8List(32));
      dest.ss58Format = 2027;
      final destBytes = dest.bytes().toList();

      final payload = Uint8List.fromList([
        0x02, 0x03,
        0x00,
        ...destBytes,
        0xA9, 0x03, // Compact(234) two-byte mode
      ]);

      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.fields['amount_yuan'], '2.34 GMB');
    });

    // -----------------------------------------------------------------------
    // Phase 3(2026-04-22)新增:8 个 execute / cleanup / cancel 类 call。
    // 链端签名统一 `fn <name>(origin, proposal_id: u64)`,
    // 冷钱包走通用 _decodeProposalIdOnly 解码器。
    //
    // 所有分支的 fields 按 Registry 统一为
    //   { proposal_id: <decimal string> }
    // 保证节点 Tauri UI / wuminapp 发出的手动兜底 QR 在冷钱包走 🟢 绿色。
    // -----------------------------------------------------------------------

    Uint8List buildProposalIdPayload(int palletIdx, int callIdx, int id) {
      return Uint8List.fromList([
        palletIdx,
        callIdx,
        id & 0xff,
        (id >> 8) & 0xff,
        (id >> 16) & 0xff,
        (id >> 24) & 0xff,
        (id >> 32) & 0xff,
        (id >> 40) & 0xff,
        (id >> 48) & 0xff,
        (id >> 56) & 0xff,
      ]);
    }

    String encodeHex(Uint8List bytes) =>
        '0x${bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';

    test('decodes execute_transfer (pallet=19 call=3)', () {
      final payload = buildProposalIdPayload(0x13, 0x03, 100);
      final decoded = PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_transfer');
      expect(decoded.fields['proposal_id'], '100');
      expect(decoded.summary, contains('#100'));
    });

    test('decodes execute_safety_fund_transfer (pallet=19 call=4)', () {
      final payload = buildProposalIdPayload(0x13, 0x04, 101);
      final decoded = PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_safety_fund_transfer');
      expect(decoded.fields['proposal_id'], '101');
    });

    test('decodes execute_sweep_to_main (pallet=19 call=5)', () {
      final payload = buildProposalIdPayload(0x13, 0x05, 102);
      final decoded = PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_sweep_to_main');
      expect(decoded.fields['proposal_id'], '102');
    });

    test('decodes execute_destroy (pallet=14 call=1)', () {
      final payload = buildProposalIdPayload(0x0e, 0x01, 200);
      final decoded = PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_destroy');
      expect(decoded.fields['proposal_id'], '200');
    });

    test('decodes execute_admin_replacement (pallet=12 call=1)', () {
      final payload = buildProposalIdPayload(0x0c, 0x01, 300);
      final decoded = PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_admin_replacement');
      expect(decoded.fields['proposal_id'], '300');
    });

    test('decodes execute_replace_grandpa_key (pallet=16 call=1)', () {
      final payload = buildProposalIdPayload(0x10, 0x01, 400);
      final decoded = PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_replace_grandpa_key');
      expect(decoded.fields['proposal_id'], '400');
    });

    test('decodes cancel_failed_replace_grandpa_key (pallet=16 call=2)', () {
      final payload = buildProposalIdPayload(0x10, 0x02, 401);
      final decoded = PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cancel_failed_replace_grandpa_key');
      expect(decoded.fields['proposal_id'], '401');
    });

    test('decodes cleanup_rejected_proposal (pallet=17 call=4)', () {
      final payload = buildProposalIdPayload(0x11, 0x04, 500);
      final decoded = PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cleanup_rejected_proposal');
      expect(decoded.fields['proposal_id'], '500');
    });

    // -----------------------------------------------------------------------
    // propose_runtime_upgrade / developer_direct_upgrade 字段对齐(2026-04-22):
    // Registry 要求 fields 含 `wasm_hash`(sha256 of code, 与节点 Tauri UI
    // 用同一算法计算)和 `eligible_total`(propose_runtime_upgrade 独有)。
    // -----------------------------------------------------------------------

    test('decodes developer_direct_upgrade 含 wasm_hash (sha256)', () {
      // WASM 内容:4 字节 "abcd" 便于手算 sha256。
      // sha256("abcd") = 88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589
      final wasmBytes = [0x61, 0x62, 0x63, 0x64];
      final wasmLen = wasmBytes.length;
      final payload = Uint8List.fromList([
        0x0d, 0x02, // pallet=13 call=2
        // Compact<u32>(wasmLen) single-byte mode (wasmLen<64): (wasmLen<<2)|0
        (wasmLen << 2) & 0xff,
        ...wasmBytes,
      ]);
      final decoded = PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'developer_direct_upgrade');
      expect(decoded.fields['wasm_size'], '0 KB'); // 4 字节 < 1 KB
      expect(
        decoded.fields['wasm_hash'],
        '0x88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589',
      );
    });

    test('decodes propose_runtime_upgrade 含 wasm_hash + eligible_total', () {
      // reason="ok" + wasm="abcd" + eligible_total=1234567
      // sha256("abcd") 同上 test。
      final reasonBytes = 'ok'.codeUnits; // 2 字节
      final wasmBytes = [0x61, 0x62, 0x63, 0x64]; // 4 字节
      const eligibleTotal = 1234567;

      final payload = Uint8List.fromList([
        0x0d, 0x00, // pallet=13 call=0
        (reasonBytes.length << 2) & 0xff, // Compact(2)
        ...reasonBytes,
        (wasmBytes.length << 2) & 0xff, // Compact(4)
        ...wasmBytes,
        // u64_le(1234567)
        eligibleTotal & 0xff,
        (eligibleTotal >> 8) & 0xff,
        (eligibleTotal >> 16) & 0xff,
        (eligibleTotal >> 24) & 0xff,
        0, 0, 0, 0,
      ]);
      final decoded = PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_runtime_upgrade');
      expect(decoded.fields['reason'], 'ok');
      expect(decoded.fields['wasm_size'], '0 KB');
      expect(
        decoded.fields['wasm_hash'],
        '0x88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589',
      );
      expect(decoded.fields['eligible_total'], '1234567');
    });
  });
}
