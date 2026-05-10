import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wumin/signer/payload_decoder.dart';

void main() {
  String hexOf(List<int> payload) =>
      '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';

  List<int> compactVec(String text) {
    final bytes = utf8.encode(text);
    return [bytes.length << 2, ...bytes];
  }

  List<int> subjectIdFromText(int kind, String text) {
    final out = List<int>.filled(48, 0);
    out[0] = kind;
    final bytes = utf8.encode(text);
    for (var i = 0; i < bytes.length && i < 47; i++) {
      out[i + 1] = bytes[i];
    }
    return out;
  }

  List<int> subjectIdFromAccount(int kind, List<int> account) {
    final out = List<int>.filled(48, 0);
    out[0] = kind;
    for (var i = 0; i < account.length && i < 32; i++) {
      out[i + 1] = account[i];
    }
    return out;
  }

  List<int> u128LeForTest(BigInt value) {
    final out = List<int>.filled(16, 0);
    var tmp = value;
    for (var i = 0; i < 16; i++) {
      out[i] = (tmp & BigInt.from(0xFF)).toInt();
      tmp = tmp >> 8;
    }
    return out;
  }

  List<int> u16Le(int value) => [value & 0xff, (value >> 8) & 0xff];

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

      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'transfer');
      expect(decoded.fields['amount_yuan'], '234.00 GMB');
      expect(decoded.fields['to'], dest.address);
    });

    // Phase 3(2026-04-22)「投票引擎统一入口整改」:
    // 所有业务 pallet 的 vote_X 已物理删除,所有管理员投票统一走
    // InternalVote::cast(22.0)。

    test('decodes internal_vote (pallet=22 call=0) approve=true', () {
      // [0x16, 0x00, u64_le proposal_id=42, bool approve=true]
      final payload = Uint8List.fromList([
        0x16, 0x00,
        42, 0, 0, 0, 0, 0, 0, 0,
        1, // approve = true
      ]);
      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'internal_vote');
      expect(decoded.fields['proposal_id'], '42');
      expect(decoded.fields['approve'], 'true');
      expect(decoded.summary, contains('赞成'));
    });

    test('decodes internal_vote (pallet=22 call=0) approve=false', () {
      final payload = Uint8List.fromList([
        0x16,
        0x00,
        7,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
      ]);
      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex);
      expect(decoded!.action, 'internal_vote');
      expect(decoded.fields['approve'], 'false');
      expect(decoded.summary, contains('反对'));
    });

    test('decodes joint_vote (pallet=23 call=0)', () {
      // JointVote.cast_admin = pallet 23 / call 0。
      final payload = Uint8List.fromList([
        0x17,
        0x00,
        7,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        ...List.filled(48, 0),
        0,
      ]);
      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'joint_vote');
      expect(decoded.fields['proposal_id'], '7');
      expect(decoded.fields['approve'], 'false');
      expect(decoded.summary, contains('反对'));
    });

    test('decodes cast_referendum (pallet=23 call=1) ADR-008 step3 双层凭证', () {
      // JointVote.cast_referendum = pallet 23 / call 1(联合公投联合公投)。
      // ADR-008 step3：SCALE 末尾在 approve 前加 (province, signer_admin_pubkey)。
      final province = utf8.encode('安徽省');
      final adminPubkey = List<int>.generate(32, (i) => 0xA0 + (i & 0x0F));
      final payload = Uint8List.fromList([
        0x17, 0x01,
        99, 0, 0, 0, 0, 0, 0, 0, // proposal_id = 99 u64_le
        ...List.filled(32, 0), // binding_id = 0x00 × 32
        0, // Vec nonce len = 0
        0, // Vec sig len = 0
        // ★ ADR-008 step3 新字段
        province.length << 2, ...province, // Compact(len) + utf8 bytes
        ...adminPubkey, // [u8;32]
        1, // approve = true
      ]);
      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'cast_referendum');
      expect(decoded.fields['proposal_id'], '99');
      expect(decoded.fields['approve'], 'true');
      expect(decoded.fields['province'], '安徽省');
      expect(
        decoded.fields['signer_admin_pubkey'],
        '0x${adminPubkey.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
      );
    });

    test('cast_referendum 旧 SCALE 字节流(无 province/admin)拒绝解码', () {
      // ADR-008 step3 后 SCALE 必须含新字段。旧字节流长度不足 → null。
      // feedback_no_compatibility:不留兼容垫片,老凭证不识别即拒绝。
      final payload = Uint8List.fromList([
        0x17, 0x01,
        99, 0, 0, 0, 0, 0, 0, 0,
        ...List.filled(32, 0),
        0,
        0,
        1, // 旧版只到 approve, 长度 = 45
      ]);
      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex);
      expect(decoded, isNull, reason: '旧凭证长度 45 < 78 必须被拒绝, 防止白盲签');
    });

    test('decodes finalize_proposal (pallet=9 call=3)', () {
      final payload = Uint8List.fromList([
        0x09,
        0x03,
        15,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
      ]);
      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex);
      expect(decoded!.action, 'finalize_proposal');
      expect(decoded.fields['proposal_id'], '15');
    });

    test('returns null for unknown pallet', () {
      expect(PayloadDecoder.decode('0xff01'), isNull);
    });

    test('returns null for too-short input', () {
      expect(PayloadDecoder.decode('0x02'), isNull);
    });

    // spec_version 门控测试已删:strict 两色模式独家把关,decoder 不再因 spec_version
    // 拒绝解码;chain pallet 索引若变(如新 spec 重排),decoder 解析到错 action ↔
    // display.action 不一致 → mismatched → 禁止签名,无需 spec_version 锁布局。

    test('decodes clearing bank register node call', () {
      const sfidNumber = 'SFR-AH001-ZG1Y-883241719-2026';
      const peerId = '12D3KooWABCDEFG1234567890abcdefghijk';
      const domain = 'l2.example.com';
      final payload = Uint8List.fromList([
        21,
        50,
        ...compactVec(sfidNumber),
        ...compactVec(peerId),
        ...compactVec(domain),
        ...u16Le(9944),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(payload));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'register_clearing_bank');
      expect(decoded.fields['sfid_number'], sfidNumber);
      expect(decoded.fields['peer_id'], peerId);
      expect(decoded.fields['rpc_domain'], domain);
      expect(decoded.fields['rpc_port'], '9944');
    });

    test('decodes clearing bank endpoint update call', () {
      const sfidNumber = 'SFR-AH001-ZG1Y-883241719-2026';
      const domain = 'new-l2.example.com';
      final payload = Uint8List.fromList([
        21,
        51,
        ...compactVec(sfidNumber),
        ...compactVec(domain),
        ...u16Le(443),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(payload));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'update_clearing_bank_endpoint');
      expect(decoded.fields['sfid_number'], sfidNumber);
      expect(decoded.fields['new_domain'], domain);
      expect(decoded.fields['new_port'], '443');
    });

    test('decodes clearing bank unregister call', () {
      const sfidNumber = 'SFR-AH001-ZG1Y-883241719-2026';
      final payload = Uint8List.fromList([
        21,
        52,
        ...compactVec(sfidNumber),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(payload));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'unregister_clearing_bank');
      expect(decoded.fields['sfid_number'], sfidNumber);
    });

    test('decodes clearing bank decrypt challenge', () {
      const sfidNumber = 'SFR-AH001-ZG1Y-883241719-2026';
      final idBytes = List<int>.filled(48, 0);
      final rawId = ascii.encode(sfidNumber);
      for (var i = 0; i < rawId.length; i++) {
        idBytes[i] = rawId[i];
      }
      final payload = Uint8List.fromList([
        ...ascii.encode('GMB_DECRYPT_V1'),
        ...idBytes,
        ...List<int>.filled(32, 0xAA),
        1,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        ...List<int>.filled(16, 0xBB),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(payload));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'decrypt_admin');
      expect(decoded.fields['sfid_number'], sfidNumber);
      expect(decoded.summary, contains('解密清算行管理员'));
    });

    test('decodes propose_sweep_to_main 国储会 (pallet=19 call=2)', () {
      // Phase 2 重排：propose_sweep_to_main 由原 call=5 迁到 call=2。
      const sfidNumber = 'GFR-LN001-CB0X-944805165-2026';
      final idBytes = subjectIdFromText(0x01, sfidNumber);
      const amount = 10000;
      final amountBytes = List<int>.filled(16, 0);
      amountBytes[0] = amount & 0xff;
      amountBytes[1] = (amount >> 8) & 0xff;

      final payload = Uint8List.fromList([
        0x13,
        0x02,
        ...idBytes,
        ...amountBytes,
      ]);

      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_sweep_to_main');
      expect(decoded.fields['institution'], '国家储备委员会');
      expect(decoded.fields['amount_yuan'], '100.00 GMB');
    });

    test('decodes propose_sweep_to_main 省储会 (pallet=19 call=2)', () {
      const sfidNumber = 'GFR-ZS001-CB0Y-016974075-2026';
      final idBytes = subjectIdFromText(0x01, sfidNumber);
      final amountBytes = List<int>.filled(16, 0);
      amountBytes[0] = 0x10;

      final payload = Uint8List.fromList([
        0x13,
        0x02,
        ...idBytes,
        ...amountBytes,
      ]);
      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex);

      expect(decoded, isNotNull);
      expect(decoded!.fields['institution'], '中枢省储备委员会');
    });

    test('rejects legacy naked subject for propose_sweep_to_main', () {
      const sfidNumber = 'GFR-LN001-CB0X-944805165-2026';
      final idBytes = List<int>.filled(48, 0);
      final idChars = sfidNumber.codeUnits;
      for (var i = 0; i < idChars.length; i++) {
        idBytes[i] = idChars[i];
      }

      final payload = Uint8List.fromList([
        0x13,
        0x02,
        ...idBytes,
        ...List<int>.filled(16, 0),
      ]);

      expect(PayloadDecoder.decode(hexOf(payload)), isNull,
          reason: 'D/ADR-015 后内置机构必须带 0x01 主体类型,不兼容裸 sfid');
    });

    test('decodes propose_transfer for institution account subject', () {
      final institutionAccount = List<int>.filled(32, 0x66);
      final beneficiary = List<int>.filled(32, 0x44);
      final payload = Uint8List.fromList([
        0x13,
        0x00,
        0x02,
        ...subjectIdFromAccount(0x05, institutionAccount),
        ...beneficiary,
        ...u128LeForTest(BigInt.from(12345)),
        0x10,
        ...utf8.encode('test'),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(payload));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_transfer');
      expect(decoded.fields['institution'], startsWith('机构账户 66666666'));
      expect(decoded.fields['amount_yuan'], '123.45 GMB');
      expect(decoded.fields['remark'], 'test');
    });

    test('rejects SfidInstitution subject for propose_transfer', () {
      final payload = Uint8List.fromList([
        0x13,
        0x00,
        0x02,
        ...subjectIdFromText(0x02, 'GFR-LN001-CB0X-944805165-2026'),
        ...List<int>.filled(32, 0x44),
        ...u128LeForTest(BigInt.one),
        0x00,
      ]);

      expect(PayloadDecoder.decode(hexOf(payload)), isNull,
          reason: '0x02 只保留机构归属/检索语义,不能作为多签转账支出账户');
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

      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex);

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

    test('decodes retry_passed_proposal (pallet=9 call=4)', () {
      // Phase 4(2026-05-02): 业务 pallet 的 execute_xxx wrapper 全部物理删除,
      // 手动重试统一收口至 VotingEngine::retry_passed_proposal(9.4)。
      final payload = buildProposalIdPayload(0x09, 0x04, 100);
      final decoded = PayloadDecoder.decode(encodeHex(payload));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'retry_passed_proposal');
      expect(decoded.fields['proposal_id'], '100');
      expect(decoded.summary, contains('#100'));
    });

    test('decodes cancel_passed_proposal with empty reason (pallet=9 call=5)',
        () {
      // SCALE: [0x09][0x05][proposal_id u64_le][Compact<u32> 0]
      final builder = BytesBuilder()
        ..add([0x09, 0x05])
        ..add(Uint8List.fromList(
            buildProposalIdPayload(0x09, 0x05, 401).sublist(2, 10)))
        ..add([0x00]); // Compact<u32> 0 (空 reason)
      final payload = builder.toBytes();
      final decoded = PayloadDecoder.decode(encodeHex(payload));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cancel_passed_proposal');
      expect(decoded.fields['proposal_id'], '401');
      expect(decoded.fields['reason'], '');
    });

    test('decodes cancel_passed_proposal with utf8 reason (pallet=9 call=5)',
        () {
      final reason = utf8.encode('密钥不可执行');
      // Compact<u32> for reason length (small => single byte mode, len << 2)
      final compactLen = reason.length << 2;
      final builder = BytesBuilder()
        ..add([0x09, 0x05])
        ..add(Uint8List.fromList(
            buildProposalIdPayload(0x09, 0x05, 402).sublist(2, 10)))
        ..add([compactLen])
        ..add(reason);
      final payload = builder.toBytes();
      final decoded = PayloadDecoder.decode(encodeHex(payload));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cancel_passed_proposal');
      expect(decoded.fields['proposal_id'], '402');
      expect(decoded.fields['reason'], '密钥不可执行');
    });

    test('rejects deleted business wrappers (pallet=19/14/12/16)', () {
      // Phase 4 物理删除的旧 call_index 不应再被解码识别。
      final cases = <List<int>>[
        [0x13, 0x03], // execute_transfer
        [0x13, 0x04], // execute_safety_fund_transfer
        [0x13, 0x05], // execute_sweep_to_main
        [0x0e, 0x01], // execute_destroy
        [0x0c, 0x01], // AdminsChange call_index=1 留洞不复用
        [0x10, 0x01], // execute_replace_grandpa_key
        [0x10, 0x02], // cancel_failed_replace_grandpa_key
      ];
      for (final c in cases) {
        final payload = buildProposalIdPayload(c[0], c[1], 999);
        final decoded = PayloadDecoder.decode(encodeHex(payload));
        expect(decoded, isNull,
            reason: 'pallet=${c[0]} call=${c[1]} 应已废弃,decoder 拒绝');
      }
    });

    test('decodes propose_admin_set_change (pallet=12 call=0)', () {
      final subject = subjectIdFromAccount(
        0x03,
        List<int>.generate(32, (i) => 0x80 + i),
      );
      final admin1 = List<int>.filled(32, 0x11);
      final admin2 = List<int>.filled(32, 0x22);
      final payload = Uint8List.fromList([
        0x0c, 0x00,
        0x03, // org = 个人多签
        ...subject,
        0x08, // Compact(2)
        ...admin1,
        ...admin2,
      ]);

      final decoded = PayloadDecoder.decode(encodeHex(payload));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_admin_set_change');
      expect(decoded.fields['org'], '个人多签');
      expect(decoded.fields['subject'],
          '0x${subject.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}');
      expect(
        decoded.fields['new_admins'],
        [
          '0x${admin1.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
          '0x${admin2.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
        ].join(','),
      );
      expect(decoded.summary, contains('管理员集合变更'));
    });

    test('decodes subject-level admin activation payload', () {
      final subject = subjectIdFromAccount(
        0x05,
        List<int>.generate(32, (i) => 0x20 + i),
      );
      final pubkey = List<int>.filled(32, 0xaa);
      final payload = Uint8List.fromList([
        ...utf8.encode('GMB_ACTIVATE_SUBJECT_V1'),
        ...subject,
        0x05, // org = 其他机构账户
        0x03, // kind = InstitutionAccount
        ...pubkey,
        1, 0, 0, 0, 0, 0, 0, 0, // timestamp u64 LE
        ...List<int>.filled(16, 0),
      ]);

      final decoded = PayloadDecoder.decode(encodeHex(payload));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'activate_admin_subject');
      expect(decoded.fields['org'], '其他机构账户');
      expect(decoded.fields['subject'],
          '0x${subject.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}');
      expect(decoded.fields['pubkey'], '0x${'aa' * 32}');
    });

    test('rejects legacy admin activation payload', () {
      final payload = Uint8List.fromList([
        ...utf8.encode('GMB_ACTIVATE'),
        ...utf8.encode('GFR-LN001-CB0X-944805165-2026'),
        ...List<int>.filled(48 - 'GFR-LN001-CB0X-944805165-2026'.length, 0),
        1,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        ...List<int>.filled(16, 0),
      ]);

      expect(PayloadDecoder.decode(encodeHex(payload)), isNull);
    });

    test('decodes institution-account admin set change org labels', () {
      final subject = subjectIdFromAccount(
        0x05,
        List<int>.generate(32, (i) => 0x30 + i),
      );
      final admin1 = List<int>.filled(32, 0x44);
      final admin2 = List<int>.filled(32, 0x55);

      for (final entry in {
        0x04: '公权机构账户',
        0x05: '其他机构账户',
      }.entries) {
        final payload = Uint8List.fromList([
          0x0c,
          0x00,
          entry.key,
          ...subject,
          0x08,
          ...admin1,
          ...admin2,
        ]);

        final decoded = PayloadDecoder.decode(encodeHex(payload));

        expect(decoded, isNotNull);
        expect(decoded!.fields['org'], entry.value);
        expect(decoded.fields['subject'],
            '0x${subject.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}');
      }
    });

    test('rejects admin set change subject kind and org mismatch', () {
      final institutionSubject = subjectIdFromAccount(
        0x05,
        List<int>.filled(32, 0x66),
      );
      final personalSubject = subjectIdFromAccount(
        0x03,
        List<int>.filled(32, 0x77),
      );
      final sfidSubject = subjectIdFromText(
        0x02,
        'GFR-LN001-CB0X-944805165-2026',
      );
      final admin1 = List<int>.filled(32, 0x11);
      final admin2 = List<int>.filled(32, 0x22);

      Uint8List payload(int org, List<int> subject) => Uint8List.fromList([
            0x0c,
            0x00,
            org,
            ...subject,
            0x08,
            ...admin1,
            ...admin2,
          ]);

      expect(
          PayloadDecoder.decode(encodeHex(payload(0x03, institutionSubject))),
          isNull);
      expect(PayloadDecoder.decode(encodeHex(payload(0x04, personalSubject))),
          isNull);
      expect(
          PayloadDecoder.decode(encodeHex(payload(0x04, sfidSubject))), isNull);
    });

    test('decodes cleanup_rejected_proposal (pallet=17 call=4)', () {
      final payload = buildProposalIdPayload(0x11, 0x04, 500);
      final decoded = PayloadDecoder.decode(encodeHex(payload));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cleanup_rejected_proposal');
      expect(decoded.fields['proposal_id'], '500');
    });

    test('decodes organization close action as propose_close_institution', () {
      final payload = Uint8List.fromList([
        0x11, 0x01, // OrganizationManage.propose_close
        ...List<int>.filled(32, 0x11),
        ...List<int>.filled(32, 0x22),
      ]);
      final decoded = PayloadDecoder.decode(encodeHex(payload));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_close_institution');
      expect(decoded.fields.keys.toList(), ['duoqian_address', 'beneficiary']);
    });

    test('decodes personal close action as propose_close_personal', () {
      final payload = Uint8List.fromList([
        0x07, 0x01, // PersonalManage.propose_close
        ...List<int>.filled(32, 0x33),
        ...List<int>.filled(32, 0x44),
      ]);
      final decoded = PayloadDecoder.decode(encodeHex(payload));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_close_personal');
      expect(decoded.fields.keys.toList(), ['duoqian_address', 'beneficiary']);
    });

    // -----------------------------------------------------------------------
    // 协议升级 propose_runtime_upgrade / developer_direct_upgrade 的 SCALE decoder 已删
    // (call_data 含 600KB+ WASM,塞不进 QR;server 在 QR 里只放 32 字节 blake2
    // 哈希,decoder 路径不可达)。改走 OfflineSignService 的"哈希直签例外"。
    // 相关回归测试见 wumin/test/signer/offline_sign_service_*_test.dart。
    // -----------------------------------------------------------------------
    // ADR-008 step2d 新加 decoder:
    // - propose_create_institution(17.5):机构多签账户创建提案
    //   (走 SFID 后端 ShengSigningPubkey 双层签发凭证)
    // - propose_resolution_issuance(8.0):决议发行联合提案
    //   (走 PopulationSnapshotVerifier 双层签发)
    // -----------------------------------------------------------------------

    List<int> buildProposeCreateInstitutionPayload({bool legacyTail = false}) {
      List<int> u128Le(BigInt value) {
        final out = List<int>.filled(16, 0);
        var tmp = value;
        for (var i = 0; i < 16; i++) {
          out[i] = (tmp & BigInt.from(0xFF)).toInt();
          tmp = tmp >> 8;
        }
        return out;
      }

      final sfid = utf8.encode('SFR-AH001-1234567890-20260501');
      final instName = utf8.encode('安徽省储行');
      final mainAccount = utf8.encode('主账户');
      final feeAccount = utf8.encode('费用账户');
      final mainAmount = u128Le(BigInt.from(1000000)); // 10,000.00 GMB
      final feeAmount = u128Le(BigInt.from(222)); // 2.22 GMB
      final adminPubkeys = [
        List<int>.filled(32, 0x11),
        List<int>.filled(32, 0x22),
      ];
      final registerNonce = utf8.encode('reg-nonce-001');
      final signature = List<int>.filled(64, 0xDD);
      final province = utf8.encode('安徽省');
      final signerAdminPubkey =
          List<int>.generate(32, (i) => 0xC0 + (i & 0x0F));
      final payload = <int>[
        0x11, 0x05, // pallet=17 call=5
        // sfid_number: Vec<u8>
        (sfid.length << 2) & 0xff,
        ...sfid,
        // institution_name: Vec<u8>
        (instName.length << 2) & 0xff,
        ...instName,
        // accounts: Vec<{name, amount}> count=2
        (2 << 2) & 0xff,
        (mainAccount.length << 2) & 0xff,
        ...mainAccount,
        ...mainAmount,
        (feeAccount.length << 2) & 0xff,
        ...feeAccount,
        ...feeAmount,
        // admin_org: ORG_OTH
        5,
        // admin_count: u32 LE
        2, 0, 0, 0,
        // duoqian_admins: BoundedVec<AccountId32> count=2
        (2 << 2) & 0xff,
        ...adminPubkeys[0],
        ...adminPubkeys[1],
        // threshold: u32 LE = 2
        2, 0, 0, 0,
        // register_nonce: Vec<u8>
        (registerNonce.length << 2) & 0xff,
        ...registerNonce,
        // signature: Vec<u8> 64B (Compact mode 1)
        0x01, 0x01,
        ...signature,
        // ★ province: Vec<u8>
        (province.length << 2) & 0xff,
        ...province,
        // ★ signer_admin_pubkey: [u8;32]
        ...signerAdminPubkey,
      ];
      if (legacyTail) {
        final a3 = utf8.encode('SFR');
        final subType = utf8.encode('SHENG_BANK');
        payload.addAll([
          (a3.length << 2) & 0xff,
          ...a3,
          0x01,
          (subType.length << 2) & 0xff,
          ...subType,
          0x00,
        ]);
      }
      return payload;
    }

    test(
        'decodes propose_create_institution (pallet=17 call=5) 含 province + signer_admin_pubkey',
        () {
      final signerAdminPubkey =
          List<int>.generate(32, (i) => 0xC0 + (i & 0x0F));

      final payload =
          Uint8List.fromList(buildProposeCreateInstitutionPayload());
      final decoded = PayloadDecoder.decode(encodeHex(payload));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_create_institution');
      expect(decoded.fields['sfid_number'], 'SFR-AH001-1234567890-20260501');
      expect(decoded.fields['institution_name'], '安徽省储行');
      expect(decoded.fields['org'], '其他机构账户');
      expect(decoded.fields['admin_count'], '2');
      expect(decoded.fields['threshold'], '2/2');
      expect(decoded.fields['total_amount_yuan'], '10,002.22 GMB');
      expect(decoded.fields['amount_主账户'], '10,000.00 GMB');
      expect(decoded.fields['amount_费用账户'], '2.22 GMB');
      expect(decoded.fields.containsKey('a3'), isFalse);
      expect(decoded.fields['province'], '安徽省');
      expect(
        decoded.fields['signer_admin_pubkey'],
        '0x${signerAdminPubkey.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
      );
    });

    test('propose_create_institution 带旧尾字段时拒绝解码', () {
      final payload = Uint8List.fromList(
          buildProposeCreateInstitutionPayload(legacyTail: true));
      final decoded = PayloadDecoder.decode(encodeHex(payload));
      expect(decoded, isNull,
          reason: 'P-TX-001 禁止 a3/sub_type/parent_sfid_number 旧尾字段');
    });

    test('decodes current propose_create_personal without threshold field', () {
      final name = utf8.encode('家庭基金');
      final admins = [
        List<int>.filled(32, 0x11),
        List<int>.filled(32, 0x22),
        List<int>.filled(32, 0x33),
      ];
      final payload = Uint8List.fromList([
        0x07,
        0x00,
        (name.length << 2) & 0xff,
        ...name,
        (admins.length << 2) & 0xff,
        ...admins.expand((admin) => admin),
        ...u128LeForTest(BigInt.from(12345)),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(payload));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_create_personal');
      expect(decoded.fields['account_name'], '家庭基金');
      expect(decoded.fields['admin_count'], '3');
      expect(decoded.fields['regular_threshold'], '2/3');
      expect(decoded.fields['create_threshold'], '3/3');
      expect(decoded.fields['amount_yuan'], '123.45 GMB');
      expect(decoded.fields.containsKey('threshold'), isFalse);
    });

    test(
        'rejects legacy propose_create_personal with admin_count and threshold',
        () {
      final name = utf8.encode('家庭基金');
      final admins = [
        List<int>.filled(32, 0x11),
        List<int>.filled(32, 0x22),
      ];
      final payload = Uint8List.fromList([
        0x07,
        0x00,
        (name.length << 2) & 0xff,
        ...name,
        2,
        0,
        0,
        0,
        (admins.length << 2) & 0xff,
        ...admins.expand((admin) => admin),
        2,
        0,
        0,
        0,
        ...u128LeForTest(BigInt.from(111)),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(payload));

      expect(decoded, isNull);
    });

    // -----------------------------------------------------------------------
    // ADR-008 step2d 双端字节一致性 fixture:
    // 三组凭证的 SCALE 字节流由 Python 生成器(链端 codec.encode 等价产出)固化,
    // 统一真源在 ../memory/06-quality/fixtures/，wumin / wuminapp / 链端 runtime
    // 三处必须产出同一序列。
    // 任何一端编码漂移 → 这里直接断言失败。
    // -----------------------------------------------------------------------

    Map<String, dynamic> readFixture() {
      final candidates = [
        File('../memory/06-quality/fixtures/step2d_credential_payload.json'),
        File('memory/06-quality/fixtures/step2d_credential_payload.json'),
      ];
      final file = candidates.firstWhere(
        (candidate) => candidate.existsSync(),
        orElse: () => candidates.first,
      );
      final raw = file.readAsStringSync();
      return jsonDecode(raw) as Map<String, dynamic>;
    }

    test(
        'fixture step2d cast_referendum: decoder 解出 province + signer_admin_pubkey',
        () {
      final fixture = readFixture();
      final caseEntry = (fixture['cases'] as List)
          .firstWhere((e) => e['name'] == 'cast_referendum');
      final hex = caseEntry['expected_call_data_hex'] as String;
      expect(caseEntry['pallet_index'], 23);
      expect(caseEntry['call_index'], 1);
      expect(hex.toLowerCase().startsWith('0x1701'), isTrue);
      final decoded = PayloadDecoder.decode(hex);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cast_referendum');
      expect(decoded.fields['proposal_id'], '99');
      expect(decoded.fields['approve'], 'true');
      expect(decoded.fields['province'],
          (caseEntry['fields'] as Map)['province_utf8']);
      expect(decoded.fields['signer_admin_pubkey'],
          (caseEntry['fields'] as Map)['signer_admin_pubkey_hex']);
    });

    // 协议升级 fixture step2d propose_runtime_upgrade decoder 用例已删:同上,SCALE decoder
    // 整体下线,fixture 走 OfflineSignService.verifyPayload 的哈希直签例外。

    test('fixture step2d propose_resolution_issuance: decoder 解出新字段', () {
      final fixture = readFixture();
      final caseEntry = (fixture['cases'] as List)
          .firstWhere((e) => e['name'] == 'propose_resolution_issuance');
      final hex = caseEntry['expected_call_data_hex'] as String;
      final decoded = PayloadDecoder.decode(hex);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_resolution_issuance');
      expect(decoded.fields['province'],
          (caseEntry['fields'] as Map)['province_utf8']);
      expect(decoded.fields['signer_admin_pubkey'],
          (caseEntry['fields'] as Map)['signer_admin_pubkey_hex']);
      expect(decoded.fields['eligible_total'],
          (caseEntry['fields'] as Map)['eligible_total'].toString());
      expect(decoded.fields['allocation_count'], '2');
    });

    test(
        'decodes propose_resolution_issuance (pallet=8 call=0) 含 province + signer_admin_pubkey',
        () {
      final reason = utf8.encode('紧急救灾');
      final totalFen = BigInt.from(50000000); // 500_000.00 GMB
      final totalLe = List<int>.filled(16, 0);
      var tmp = totalFen;
      for (var i = 0; i < 16; i++) {
        totalLe[i] = (tmp & BigInt.from(0xFF)).toInt();
        tmp = tmp >> 8;
      }
      // allocations: 2 项, 每项 32B + 16B = 48B
      final alloc1 = [
        ...List<int>.filled(32, 0xA1),
        ...List<int>.filled(16, 0x00),
      ];
      final alloc2 = [
        ...List<int>.filled(32, 0xA2),
        ...List<int>.filled(16, 0x00),
      ];
      const eligible = 7654321;
      final nonceBytes = utf8.encode('snap-001');
      final sigBytes = List<int>.filled(64, 0xEE);
      final province = utf8.encode('安徽省');
      final signerAdmin = List<int>.generate(32, (i) => 0xD0 + (i & 0x0F));

      final payload = Uint8List.fromList([
        0x08, 0x00, // pallet=8 call=0
        (reason.length << 2) & 0xff,
        ...reason,
        ...totalLe,
        // allocations Vec count=2
        (2 << 2) & 0xff,
        ...alloc1,
        ...alloc2,
        // eligible_total u64 LE
        eligible & 0xff,
        (eligible >> 8) & 0xff,
        (eligible >> 16) & 0xff,
        (eligible >> 24) & 0xff,
        0, 0, 0, 0,
        // snapshot_nonce
        (nonceBytes.length << 2) & 0xff,
        ...nonceBytes,
        // signature 64B (Compact mode 1)
        0x01, 0x01,
        ...sigBytes,
        // ★ province
        (province.length << 2) & 0xff,
        ...province,
        // ★ signer_admin_pubkey
        ...signerAdmin,
      ]);
      final decoded = PayloadDecoder.decode(encodeHex(payload));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_resolution_issuance');
      expect(decoded.fields['reason'], '紧急救灾');
      expect(decoded.fields['allocation_count'], '2');
      expect(decoded.fields['eligible_total'], '7654321');
      expect(decoded.fields['province'], '安徽省');
      expect(
        decoded.fields['signer_admin_pubkey'],
        '0x${signerAdmin.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
      );
    });
  });
}
