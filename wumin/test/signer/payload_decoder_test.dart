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
    // VotingEngineSystem::internal_vote(9.0)。

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
  });
}
