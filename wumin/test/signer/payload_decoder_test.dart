import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wumin/signer/pallet_registry.dart';
import 'package:wumin/signer/payload_decoder.dart';

void main() {
  group('PayloadDecoder', () {
    test('decodes transfer_keep_alive (pallet=2 call=3)', () {
      // 构造：[0x02, 0x03, 0x00 (MultiAddress::Id), 32 bytes addr, Compact amount]
      final dest = Keyring.sr25519.fromSeed(Uint8List(32));
      dest.ss58Format = 2027;
      final destBytes = dest.bytes().toList();

      // Compact(234_00) = 234 元 = 23400 分
      // 23400 = 0x5B68 => Compact two-byte mode: (23400 << 2) | 1 = 93601 = 0x16DA1
      // little-endian: [0xA1, 0x6D, 0x01] — wait, two-byte mode only works up to 2^14-1 = 16383
      // 23400 > 16383 so use four-byte mode: (23400 << 2) | 2 = 93602 = 0x16DA2
      // little-endian 4 bytes: [0xA2, 0x6D, 0x01, 0x00]
      final payload = Uint8List.fromList([
        0x02, 0x03, // pallet + call
        0x00, // MultiAddress::Id
        ...destBytes,
        0xA2, 0x6D, 0x01, 0x00, // Compact(23400)
      ]);

      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: PalletRegistry.supportedSpecVersions.first);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'transfer');
      expect(decoded.fields['amount_yuan'], '234.00 GMB');
      expect(decoded.fields['to'], dest.address);
    });

    // Step 2 · vote_transfer 已替换为 finalize_transfer(离线 QR 聚合签名,
    // 冷钱包不盲签 sr25519 签名聚合),`_decodeVoteTransfer` 已从
    // payload_decoder 移除,原测试同步删除。

    test('decodes joint_vote (pallet=9 call=3)', () {
      // [0x09, 0x03, u64_le proposal_id=7, 48 bytes institution, bool approve=false]
      final payload = Uint8List.fromList([
        0x09, 0x03,
        7, 0, 0, 0, 0, 0, 0, 0, // proposal_id = 7
        ...List.filled(48, 0), // institution_id
        0, // approve = false
      ]);

      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: PalletRegistry.supportedSpecVersions.first);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'joint_vote');
      expect(decoded.fields['proposal_id'], '7');
      expect(decoded.fields['approve'], 'false');
      expect(decoded.summary, contains('反对'));
    });

    test('decodes citizen_vote (pallet=9 call=4)', () {
      // [0x09, 0x04, u64_le proposal_id=99, 32 bytes binding_id,
      //  Vec nonce (compact len 0), Vec sig (compact len 0), bool approve=true]
      final payload = Uint8List.fromList([
        0x09, 0x04,
        99, 0, 0, 0, 0, 0, 0, 0, // proposal_id = 99
        ...List.filled(32, 0), // binding_id
        0, // Vec nonce len=0
        0, // Vec sig len=0
        1, // approve = true
      ]);

      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: PalletRegistry.supportedSpecVersions.first);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'citizen_vote');
      expect(decoded.fields['proposal_id'], '99');
      expect(decoded.fields['approve'], 'true');
    });

    test('returns null for unknown pallet', () {
      const hex = '0xff01';
      expect(PayloadDecoder.decode(hex, specVersion: PalletRegistry.supportedSpecVersions.first), isNull);
    });

    test('returns null for too-short input', () {
      expect(PayloadDecoder.decode('0x02', specVersion: PalletRegistry.supportedSpecVersions.first), isNull);
    });

    test('returns null for unsupported specVersion', () {
      expect(PayloadDecoder.decode('0x0203', specVersion: 999), isNull);
    });

    test('returns null for null specVersion', () {
      expect(PayloadDecoder.decode('0x0203'), isNull);
    });

    test('decodes propose_sweep_to_main 国储会 (pallet=19 call=5)', () {
      // 回归:国储会 shenfen_id 在 institutions.dart 中存在,应还原为"国家储备委员会"
      // 而不是 fallback 成原始 shenfen_id 字符串。
      // [0x13, 0x05, 48 bytes shenfen_id (零填充), 16 bytes u128_le amount_fen]
      const shenfenId = 'GFR-LN001-CB0C-617776487-20260222';
      final idBytes = List<int>.filled(48, 0);
      final idChars = shenfenId.codeUnits;
      for (var i = 0; i < idChars.length; i++) {
        idBytes[i] = idChars[i];
      }
      // amount = 100.00 GMB = 10000 分
      const amount = 10000;
      final amountBytes = List<int>.filled(16, 0);
      amountBytes[0] = amount & 0xff;
      amountBytes[1] = (amount >> 8) & 0xff;

      final payload = Uint8List.fromList([
        0x13, 0x05,
        ...idBytes,
        ...amountBytes,
      ]);

      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: PalletRegistry.supportedSpecVersions.first);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_sweep_to_main');
      expect(decoded.fields['institution'], '国家储备委员会');
      expect(decoded.fields['amount_yuan'], '100.00 GMB');
    });

    test('decodes propose_sweep_to_main 省储会 (pallet=19 call=5)', () {
      // 回归:省储会 shenfen_id 应还原为中文名。
      const shenfenId = 'GFR-ZS001-CB0X-464088047-20260222';
      final idBytes = List<int>.filled(48, 0);
      final idChars = shenfenId.codeUnits;
      for (var i = 0; i < idChars.length; i++) {
        idBytes[i] = idChars[i];
      }
      final amountBytes = List<int>.filled(16, 0);
      amountBytes[0] = 0x10; // 16 分 = 0.16 元

      final payload = Uint8List.fromList([
        0x13, 0x05,
        ...idBytes,
        ...amountBytes,
      ]);
      final hex = '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: PalletRegistry.supportedSpecVersions.first);

      expect(decoded, isNotNull);
      expect(decoded!.fields['institution'], '中枢省储备委员会');
    });

    test('Compact encoding mode 1 (two-byte)', () {
      // Compact(234): value=234, mode 1 => (234 << 2) | 1 = 937 = 0x03A9
      // little-endian: [0xA9, 0x03]
      // Use in transfer_keep_alive amount
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
      final decoded = PayloadDecoder.decode(hex, specVersion: PalletRegistry.supportedSpecVersions.first);

      expect(decoded, isNotNull);
      expect(decoded!.fields['amount_yuan'], '2.34 GMB');
    });
  });
}
