import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:citizenwallet/signer/institution_code.dart';
import 'package:citizenwallet/signer/payload_decoder.dart';

void main() {
  String hexOf(List<int> payload) =>
      '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';

  String hexLower(List<int> payload) =>
      payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  List<int> compactVec(String text) {
    final bytes = utf8.encode(text);
    return [bytes.length << 2, ...bytes];
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

  List<int> u32Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
      ];

  List<int> u64Le(int value) {
    final out = List<int>.filled(8, 0);
    var tmp = value;
    for (var i = 0; i < 8; i++) {
      out[i] = tmp & 0xff;
      tmp >>= 8;
    }
    return out;
  }

  List<int> compactU32(int value) {
    if (value < 64) return [value << 2];
    if (value < 16384) {
      final v = (value << 2) | 1;
      return [v & 0xff, (v >> 8) & 0xff];
    }
    final v = (value << 2) | 2;
    return [v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff, (v >> 24) & 0xff];
  }

  // SigningPayload 扩展尾,布局与节点端 build_signing_payload / citizenapp
  // polkadart 编码一致:era(0x00 immortal) + Compact<nonce> + Compact<tip>
  // + mode(0x00) + spec(4) + tx(4) + genesis(32) + birth=genesis(32) + None。
  // 真实 QR payload_hex = call_data + 本尾部;链上分支夹具必须带尾构造,
  // 裸 call_data 会被 decoder 的尾部校验拒绝(decodeFailed → 红色)。
  final tailGenesis = List<int>.generate(32, (i) => 0x49 ^ i);
  List<int> signingTail({int nonce = 1, int tip = 0}) => [
        0x00,
        ...compactU32(nonce),
        ...compactU32(tip),
        0x00,
        1, 0, 0, 0, // spec_version u32 LE
        1, 0, 0, 0, // tx_version u32 LE
        ...tailGenesis,
        ...tailGenesis,
        0x00,
      ];

  Uint8List withSigningTail(List<int> callData, {int nonce = 1}) =>
      Uint8List.fromList([...callData, ...signingTail(nonce: nonce)]);

  List<int> bytesFromHex(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return List<int>.generate(
      clean.length ~/ 2,
      (i) => int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    );
  }

  String ss58FromBytes(List<int> bytes) => Keyring().encodeAddress(bytes, 2027);

  String ss58FromHex(String value) {
    final clean = value.startsWith('0x') ? value.substring(2) : value;
    final bytes = List<int>.generate(
      clean.length ~/ 2,
      (i) => int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    );
    return ss58FromBytes(bytes);
  }

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

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

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
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

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
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded!.action, 'internal_vote');
      expect(decoded.fields['approve'], 'false');
      expect(decoded.summary, contains('反对'));
    });

    test('decodes joint_vote (pallet=23 call=0)', () {
      // JointVote.cast_admin = pallet 23 / call 0，机构参数为 AccountId32。
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
        ...List.filled(32, 0),
        0,
      ]);
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'joint_vote');
      expect(decoded.fields['proposal_id'], '7');
      expect(decoded.fields['approve'], 'false');
      expect(decoded.summary, contains('反对'));
    });

    test('decodes cast_referendum (pallet=23 call=1) admins 签发凭证', () {
      // JointVote.cast_referendum = pallet 23 / call 1(联合公投联合公投)。
      final issuerCid = utf8.encode('CN000-GZF0A-000000001-2026');
      final issuerMain = List<int>.generate(32, (i) => 0x90 + (i & 0x0F));
      final signerPubkey = List<int>.generate(32, (i) => 0xA0 + (i & 0x0F));
      final scopeProvince = utf8.encode('安徽省');
      final scopeCity = utf8.encode('合肥市');
      final payload = Uint8List.fromList([
        0x17, 0x01,
        99, 0, 0, 0, 0, 0, 0, 0, // proposal_id = 99 u64_le
        ...List.filled(32, 0), // binding_id = 0x00 × 32
        0, // Vec nonce len = 0
        0, // Vec sig len = 0
        issuerCid.length << 2, ...issuerCid,
        ...issuerMain,
        ...signerPubkey,
        scopeProvince.length << 2, ...scopeProvince,
        scopeCity.length << 2, ...scopeCity,
        1, // approve = true
      ]);
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'cast_referendum');
      expect(decoded.fields['proposal_id'], '99');
      expect(decoded.fields['approve'], 'true');
      expect(decoded.fields['issuer_cid_number'], 'CN000-GZF0A-000000001-2026');
      expect(decoded.fields['issuer_main_account'], ss58FromBytes(issuerMain));
      expect(decoded.fields['scope_province_name'], '安徽省');
      expect(decoded.fields['scope_city_name'], '合肥市');
      expect(
        decoded.fields['signer_pubkey'],
        ss58FromBytes(signerPubkey),
      );
    });

    test('cast_referendum 缺少 issuer/admins 字段时拒绝解码', () {
      // 当前 SCALE 必须含签发机构、签发管理员和作用域。缺字段字节流长度不足 → null。
      final payload = Uint8List.fromList([
        0x17, 0x01,
        99, 0, 0, 0, 0, 0, 0, 0,
        ...List.filled(32, 0),
        0,
        0,
        1, // 只到 approve,长度 = 45。
      ]);
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded, isNull, reason: '缺 issuer/admins/scope 的旧凭证必须被拒绝');
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
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded!.action, 'finalize_proposal');
      expect(decoded.fields['proposal_id'], '15');
    });

    test('returns null for unknown pallet', () {
      expect(PayloadDecoder.decode('0xff01'), isNull);
    });

    test('returns null for too-short input', () {
      expect(PayloadDecoder.decode('0x02'), isNull);
    });

    test('decodes cid_admin_action with SS58 review fields', () {
      final actor = '0x${List.filled(32, '11').join()}';
      final target = '0x${List.filled(32, '22').join()}';
      final payload = jsonEncode({
        'domain': 'cid_admin_governance',
        'qr_proto': 'QR_V1',
        'action_id': 'admin-action-test',
        'action_type': 'PASSKEY_REGISTER',
        'actor_pubkey': actor,
        'actor_province_name': '广东省',
        'target': target,
        'request_hash': '0x${List.filled(32, '33').join()}',
        'before_hash': 'none',
        'after_hash': '0x${List.filled(32, '44').join()}',
        'expires_at': 1779984120,
      });

      final decoded = PayloadDecoder.decode(hexOf(utf8.encode(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'cid_admin_action');
      expect(decoded.fields['action_type'], '更新 Passkey');
      expect(decoded.reviewFields['actor_province_name'], '广东省');
      expect(decoded.reviewFields['actor_pubkey'], ss58FromHex(actor));
      expect(decoded.reviewFields['target'], ss58FromHex(target));
      expect(decoded.reviewFields.containsKey('payload_hash'), isFalse);
    });

    test('decodes cid admin action labels', () {
      final actor = '0x${List.filled(32, '11').join()}';
      final target = '0x${List.filled(32, '22').join()}';
      final cases = {
        'CREATE_ADMIN': '新增管理员',
        'UPDATE_ADMIN': '编辑管理员',
        'DELETE_ADMIN': '删除管理员',
      };

      for (final entry in cases.entries) {
        final payload = jsonEncode({
          'domain': 'cid_admin_governance',
          'qr_proto': 'QR_V1',
          'action_id': 'admin-action-${entry.key}',
          'action_type': entry.key,
          'actor_pubkey': actor,
          'actor_province_name': '广东省',
          'target': target,
          'request_hash': '0x${List.filled(32, '33').join()}',
          'before_hash': 'none',
          'after_hash': '0x${List.filled(32, '44').join()}',
          'expires_at': 1779984120,
        });

        final decoded = PayloadDecoder.decode(hexOf(utf8.encode(payload)));

        expect(decoded, isNotNull);
        expect(decoded!.fields['action_type'], entry.value);
      }
    });

    test('decodes archive_delete with SS58 review fields', () {
      final admin = '0x${List.filled(32, '22').join()}';
      final payload =
          'CPMS_ARCHIVE_DELETE_V1|adc_test|archive_internal|ARCHIVE123|$admin|1779984120';

      final decoded = PayloadDecoder.decode(hexOf(utf8.encode(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'archive_delete');
      expect(decoded.fields['admin_pubkey'], ss58FromHex(admin));
      expect(decoded.reviewFields['admin_pubkey'], ss58FromHex(admin));
      expect(decoded.reviewFields.containsKey('archive_id'), isFalse);
    });

    test('decodes clearing bank register node call', () {
      const cidNumber = 'AH001-SZG1Z-883241719-2026';
      const peerId = '12D3KooWABCDEFG1234567890abcdefghijk';
      const domain = 'l2.example.com';
      final payload = Uint8List.fromList([
        21,
        50,
        ...compactVec(cidNumber),
        ...compactVec(peerId),
        ...compactVec(domain),
        ...u16Le(9944),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'register_clearing_bank');
      expect(decoded.fields['cid_number'], cidNumber);
      expect(decoded.fields['peer_id'], peerId);
      expect(decoded.fields['rpc_domain'], domain);
      expect(decoded.fields['rpc_port'], '9944');
    });

    test('decodes clearing bank endpoint update call', () {
      const cidNumber = 'AH001-SZG1Z-883241719-2026';
      const domain = 'new-l2.example.com';
      final payload = Uint8List.fromList([
        21,
        51,
        ...compactVec(cidNumber),
        ...compactVec(domain),
        ...u16Le(443),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'update_clearing_bank_endpoint');
      expect(decoded.fields['cid_number'], cidNumber);
      expect(decoded.fields['new_domain'], domain);
      expect(decoded.fields['new_port'], '443');
    });

    test('decodes clearing bank unregister call', () {
      const cidNumber = 'AH001-SZG1Z-883241719-2026';
      final payload = Uint8List.fromList([
        21,
        52,
        ...compactVec(cidNumber),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'unregister_clearing_bank');
      expect(decoded.fields['cid_number'], cidNumber);
    });

    test('decodes clearing bank decrypt challenge', () {
      const cidNumber = 'AH001-SZG1Z-883241719-2026';
      final idBytes = List<int>.filled(48, 0);
      final rawId = ascii.encode(cidNumber);
      for (var i = 0; i < rawId.length; i++) {
        idBytes[i] = rawId[i];
      }
      final payload = Uint8List.fromList([
        // ADR-026 Phase 2 二进制前缀 GMB || 0x19。
        0x47, 0x4D, 0x42, 0x19,
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
      expect(decoded.fields['cid_number'], cidNumber);
      expect(decoded.summary, contains('解密清算行管理员'));
    });

    test('decodes propose_sweep_to_main AccountId32 (pallet=19 call=2)', () {
      final institutionAccount = List<int>.filled(32, 0x66);
      const amount = 10000;
      final amountBytes = List<int>.filled(16, 0);
      amountBytes[0] = amount & 0xff;
      amountBytes[1] = (amount >> 8) & 0xff;

      final payload = Uint8List.fromList([
        0x13,
        0x02,
        ...institutionAccount,
        ...amountBytes,
      ]);

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_sweep_to_main');
      expect(
        decoded.fields['institution'],
        '机构账户 ${ss58FromBytes(institutionAccount)}',
      );
      expect(decoded.fields['amount_yuan'], '100.00 GMB');
    });

    test('rejects legacy 48-byte sweep account payload', () {
      final legacySubject = List<int>.filled(48, 0);
      final amountBytes = List<int>.filled(16, 0);
      amountBytes[0] = 0x10;

      final payload = Uint8List.fromList([
        0x13,
        0x02,
        ...legacySubject,
        ...amountBytes,
      ]);

      expect(PayloadDecoder.decode(hexOf(withSigningTail(payload))), isNull,
          reason: '目标态只接受机构多签 AccountId32,不兼容旧 48B 主体');
    });

    test('decodes propose_transfer for institution AccountId32', () {
      final institutionAccount = List<int>.filled(32, 0x66);
      final beneficiary = List<int>.filled(32, 0x44);
      final payload = Uint8List.fromList([
        0x13,
        0x00,
        ...InstitutionCode.codeBytes('CGOV'), // 机构账户码(取代旧 org=5)
        ...institutionAccount,
        ...beneficiary,
        ...u128LeForTest(BigInt.from(12345)),
        0x10,
        ...utf8.encode('test'),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_transfer');
      expect(
        decoded.fields['institution'],
        '机构账户 ${ss58FromBytes(institutionAccount)}',
      );
      expect(decoded.fields['amount_yuan'], '123.45 GMB');
      expect(decoded.fields['remark'], 'test');
    });

    test('rejects legacy 48-byte transfer account payload', () {
      // 旧布局 [org:u8][subject:48B]。新布局是 [institution_code:4B][account:32B],
      // 48B 主体读出的"机构码"是垃圾且非法人机构码 → null。
      final payload = Uint8List.fromList([
        0x13,
        0x00,
        0x02,
        ...List<int>.filled(48, 0x22),
        ...List<int>.filled(32, 0x44),
        ...u128LeForTest(BigInt.one),
        0x00,
      ]);

      expect(PayloadDecoder.decode(hexOf(withSigningTail(payload))), isNull,
          reason: '目标态只接受机构多签 AccountId32,不兼容旧 48B 主体');
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

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

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
    // 保证节点 Tauri UI / citizenapp 发出的手动兜底 QR 在冷钱包走 🟢 绿色。
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
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
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
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
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
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cancel_passed_proposal');
      expect(decoded.fields['proposal_id'], '402');
      expect(decoded.fields['reason'], '密钥不可执行');
    });

    test('rejects deleted business wrappers (pallet=19/14/12/16)', () {
      // Phase 4 物理删除的 call_index 不应再被解码识别。
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
        final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
        expect(decoded, isNull,
            reason: 'pallet=${c[0]} call=${c[1]} 应已废弃,decoder 拒绝');
      }
    });

    test('decodes propose_admin_set_change (pallet=12 call=0)', () {
      final account = List<int>.generate(32, (i) => 0x80 + i);
      final admin1 = List<int>.filled(32, 0x11);
      final admin2 = List<int>.filled(32, 0x22);
      final payload = Uint8List.fromList([
        0x0c, 0x00,
        ...InstitutionCode.codeBytes('PMUL'), // institution_code = 个人多签
        ...account,
        0x08, // Compact(2)
        ...admin1,
        ...admin2,
        ...u32Le(2),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_admin_set_change');
      expect(decoded.fields['institution_code'], '个人多签');
      expect(decoded.fields['account'], '0x${hexLower(account)}');
      expect(
        decoded.fields['admins'],
        [
          '0x${admin1.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
          '0x${admin2.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
        ].join(','),
      );
      expect(decoded.reviewFields['new_threshold'], '2/2');
      expect(decoded.summary, contains('管理员集合变更'));
    });

    test('rejects propose_admin_set_change without new_threshold', () {
      final account = List<int>.generate(32, (i) => 0x80 + i);
      final payload = Uint8List.fromList([
        0x0c,
        0x00,
        ...InstitutionCode.codeBytes('PMUL'),
        ...account,
        0x08,
        ...List<int>.filled(32, 0x11),
        ...List<int>.filled(32, 0x22),
      ]);

      expect(PayloadDecoder.decode(hexOf(withSigningTail(payload))), isNull);
    });

    test('rejects propose_admin_set_change with trailing bytes', () {
      final account = List<int>.generate(32, (i) => 0x80 + i);
      final payload = Uint8List.fromList([
        0x0c,
        0x00,
        ...InstitutionCode.codeBytes('PMUL'),
        ...account,
        0x08,
        ...List<int>.filled(32, 0x11),
        ...List<int>.filled(32, 0x22),
        ...u32Le(2),
        0xff,
      ]);

      expect(PayloadDecoder.decode(hexOf(withSigningTail(payload))), isNull);
    });

    test('rejects propose_admin_set_change below majority threshold', () {
      final account = List<int>.generate(32, (i) => 0x80 + i);
      final payload = Uint8List.fromList([
        0x0c, 0x00,
        ...InstitutionCode.codeBytes('PMUL'),
        ...account,
        0x0c, // Compact(3)
        ...List<int>.filled(32, 0x11),
        ...List<int>.filled(32, 0x22),
        ...List<int>.filled(32, 0x33),
        ...u32Le(1),
      ]);

      expect(PayloadDecoder.decode(hexOf(withSigningTail(payload))), isNull);
    });

    test('rejects builtin governance admin change with wrong fixed threshold',
        () {
      final account = List<int>.generate(32, (i) => 0x40 + i);
      final payload = Uint8List.fromList([
        0x0c, 0x00,
        ...InstitutionCode.codeBytes('NRC'), // 国储会固定治理档 19/13
        ...account,
        0x4c, // Compact(19)
        for (var i = 0; i < 19; i++) ...List<int>.filled(32, i + 1),
        ...u32Le(12),
      ]);

      expect(PayloadDecoder.decode(hexOf(withSigningTail(payload))), isNull);
    });

    test('decodes account-level admin activation payload', () {
      final account = List<int>.generate(32, (i) => 0x20 + i);
      final pubkey = List<int>.filled(32, 0xaa);
      final payload = Uint8List.fromList([
        // ADR-026 Phase 2 二进制前缀 GMB || 0x18。
        0x47, 0x4D, 0x42, 0x18,
        ...account,
        ...InstitutionCode.codeBytes('CGOV'), // 机构账户码(取代旧 org=5)
        0x02, // kind = InstitutionAccount
        ...pubkey,
        1, 0, 0, 0, 0, 0, 0, 0, // timestamp u64 LE
        ...List<int>.filled(16, 0),
      ]);

      final decoded = PayloadDecoder.decode(encodeHex(payload));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'activate_admin_account');
      expect(decoded.fields['institution_code'], 'CGOV');
      expect(decoded.fields['account'], '0x${hexLower(account)}');
      expect(decoded.fields['pubkey'], ss58FromBytes(pubkey));
      expect(decoded.reviewFields['account'], ss58FromBytes(account));
      expect(decoded.reviewFields['pubkey'], ss58FromBytes(pubkey));
    });

    test('decodes institution-account admin set change institution_code labels',
        () {
      final account = List<int>.generate(32, (i) => 0x30 + i);
      final admin1 = List<int>.filled(32, 0x44);
      final admin2 = List<int>.filled(32, 0x55);

      // 公权机构账户码(CGOV)与私权机构账户码(SFLP)都属注册多签机构账户,
      // codeLabel 返回码字符串本身(非固定治理档/个人多签特化)。
      for (final code in const ['CGOV', 'SFLP']) {
        final payload = Uint8List.fromList([
          0x0c,
          0x00,
          ...InstitutionCode.codeBytes(code),
          ...account,
          0x08,
          ...admin1,
          ...admin2,
          ...u32Le(2),
        ]);

        final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

        expect(decoded, isNotNull);
        expect(decoded!.fields['institution_code'], code);
        expect(decoded.fields['account'], '0x${hexLower(account)}');
      }
    });

    test('rejects legacy 48-byte admin account payload', () {
      // 旧布局 [org:u8][subject:48B]。新布局 [institution_code:4B][account:32B],
      // 48B 主体下偏移全部错位 → 签名尾校验失败 → null。
      final legacySubject = List<int>.filled(48, 0x66);
      final admin1 = List<int>.filled(32, 0x11);
      final admin2 = List<int>.filled(32, 0x22);

      final payload = Uint8List.fromList([
        0x0c,
        0x00,
        0x04,
        ...legacySubject,
        0x08,
        ...admin1,
        ...admin2,
        ...u32Le(2),
      ]);

      expect(PayloadDecoder.decode(hexOf(withSigningTail(payload))), isNull);
    });

    test('decodes cleanup_rejected_proposal (pallet=17 call=4)', () {
      final payload = buildProposalIdPayload(0x11, 0x04, 500);
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cleanup_rejected_proposal');
      expect(decoded.fields['proposal_id'], '500');
    });

    test('decodes organization close action as propose_close_institution', () {
      // 机构注销 propose_close(17.1) 携带注册局签发的注销凭证:
      // account + beneficiary + register_nonce(Vec) + signature(Vec)
      // + issuer_cid_number(Vec) + issuer_main_account(32) + signer_pubkey(32)。
      // 链端签名见 organization-manage/src/lib.rs::propose_close(ADR-023 §6.3)。
      final registerNonce = utf8.encode('reg-nonce-001');
      final signature = List<int>.filled(64, 0xDD);
      final issuerCid = utf8.encode('CN000-GZF0A-000000001-2026');
      final issuerMain = List<int>.generate(32, (i) => 0xB0 + (i & 0x0F));
      final signerPubkey = List<int>.generate(32, (i) => 0xC0 + (i & 0x0F));
      final payload = <int>[
        0x11, 0x01, // OrganizationManage.propose_close
        ...List<int>.filled(32, 0x11), // account
        ...List<int>.filled(32, 0x22), // beneficiary
        // register_nonce: Vec<u8>
        (registerNonce.length << 2) & 0xff,
        ...registerNonce,
        // signature: Vec<u8> 64B (Compact mode 1)
        0x01, 0x01,
        ...signature,
        // issuer_cid_number: Vec<u8>
        (issuerCid.length << 2) & 0xff,
        ...issuerCid,
        // issuer_main_account: AccountId32
        ...issuerMain,
        // signer_pubkey: [u8;32]
        ...signerPubkey,
      ];
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_close_institution');
      expect(decoded.fields.keys.toList(), ['account', 'beneficiary']);
    });

    test('rejects legacy organization close payload without credential', () {
      final payload = Uint8List.fromList([
        0x11, 0x01, // OrganizationManage.propose_close
        ...List<int>.filled(32, 0x11),
        ...List<int>.filled(32, 0x22),
      ]);
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded, isNull);
    });

    test('decodes personal close action as propose_close_personal', () {
      final payload = Uint8List.fromList([
        0x07, 0x01, // PersonalManage.propose_close
        ...List<int>.filled(32, 0x33),
        ...List<int>.filled(32, 0x44),
      ]);
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_close_personal');
      expect(decoded.fields.keys.toList(), ['account', 'beneficiary']);
    });

    // -----------------------------------------------------------------------
    // 协议升级 propose_runtime_upgrade / developer_direct_upgrade 的 SCALE decoder 已删
    // (call_data 含 600KB+ WASM,塞不进 QR;server 在 QR 里只放 32 字节 blake2
    // 哈希,decoder 路径不可达)。改走 OfflineSignService 的"哈希直签例外"。
    // 相关回归测试见 citizenwallet/test/signer/offline_sign_service_*_test.dart。
    // -----------------------------------------------------------------------
    // 机构/决议创建 decoder:
    // - propose_create_institution(17.5):机构多签账户创建提案
    //   (走 CID 后端签发机构 admins 凭证)
    // - propose_resolution_issuance(8.0):决议发行联合提案
    //   (人口快照由 JointVote 单独准备)
    // -----------------------------------------------------------------------

    List<int> buildProposeCreateInstitutionPayload({
      bool extraTail = false,
      String secondAccountName = '费用账户',
    }) {
      List<int> u128Le(BigInt value) {
        final out = List<int>.filled(16, 0);
        var tmp = value;
        for (var i = 0; i < 16; i++) {
          out[i] = (tmp & BigInt.from(0xFF)).toInt();
          tmp = tmp >> 8;
        }
        return out;
      }

      final cid = utf8.encode('AH001-SCB0N-202605010-2026');
      final instName = utf8.encode('安徽省储行');
      final mainAccount = utf8.encode('主账户');
      final feeAccount = utf8.encode(secondAccountName);
      final mainAmount = u128Le(BigInt.from(1000000)); // 10,000.00 GMB
      final feeAmount = u128Le(BigInt.from(222)); // 2.22 GMB
      final admins = [
        List<int>.filled(32, 0x11),
        List<int>.filled(32, 0x22),
      ];
      final registerNonce = utf8.encode('reg-nonce-001');
      final signature = List<int>.filled(64, 0xDD);
      final issuerCid = utf8.encode('CN000-GZF0A-000000001-2026');
      final issuerMain = List<int>.generate(32, (i) => 0xB0 + (i & 0x0F));
      final signerPubkey = List<int>.generate(32, (i) => 0xC0 + (i & 0x0F));
      final scopeProvince = utf8.encode('安徽省');
      final scopeCity = utf8.encode('合肥市');
      final payload = <int>[
        0x11, 0x05, // pallet=17 call=5
        // cid_number: Vec<u8>
        (cid.length << 2) & 0xff,
        ...cid,
        // cid_full_name: Vec<u8>
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
        // institution_code: [u8;4] 机构账户码(取代旧 org=ORG_OTH=5)
        ...InstitutionCode.codeBytes('CGOV'),
        // admins_len: u32 LE
        2, 0, 0, 0,
        // admins: BoundedVec<AccountId32> count=2
        (2 << 2) & 0xff,
        ...admins[0],
        ...admins[1],
        // threshold: u32 LE = 2
        2, 0, 0, 0,
        // register_nonce: Vec<u8>
        (registerNonce.length << 2) & 0xff,
        ...registerNonce,
        // signature: Vec<u8> 64B (Compact mode 1)
        0x01, 0x01,
        ...signature,
        // issuer_cid_number: Vec<u8>
        (issuerCid.length << 2) & 0xff,
        ...issuerCid,
        // issuer_main_account: AccountId32
        ...issuerMain,
        // signer_pubkey: [u8;32]
        ...signerPubkey,
        // scope_province_name / scope_city_name
        (scopeProvince.length << 2) & 0xff,
        ...scopeProvince,
        (scopeCity.length << 2) & 0xff,
        ...scopeCity,
      ];
      if (extraTail) {
        final subjectProperty = utf8.encode('S');
        final subType = utf8.encode('SHENG_BANK');
        payload.addAll([
          (subjectProperty.length << 2) & 0xff,
          ...subjectProperty,
          0x01,
          (subType.length << 2) & 0xff,
          ...subType,
          0x00,
        ]);
      }
      return payload;
    }

    test('decodes propose_create_institution (pallet=17 call=5) 含 issuer/scope',
        () {
      final issuerMain = List<int>.generate(32, (i) => 0xB0 + (i & 0x0F));
      final signerPubkey = List<int>.generate(32, (i) => 0xC0 + (i & 0x0F));

      final payload =
          Uint8List.fromList(buildProposeCreateInstitutionPayload());
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_create_institution');
      expect(decoded.fields['cid_number'], 'AH001-SCB0N-202605010-2026');
      expect(decoded.fields['cid_full_name'], '安徽省储行');
      expect(decoded.fields['institution_code'], 'CGOV');
      expect(decoded.fields['admins_len'], '2');
      expect(decoded.fields['threshold'], '2/2');
      expect(decoded.fields['total_amount_yuan'], '10,002.22 GMB');
      expect(decoded.fields['amount_主账户'], '10,000.00 GMB');
      expect(decoded.fields['amount_费用账户'], '2.22 GMB');
      expect(decoded.fields.containsKey('subject_property'), isFalse);
      expect(decoded.fields['issuer_cid_number'], 'CN000-GZF0A-000000001-2026');
      expect(decoded.fields['issuer_main_account'], ss58FromBytes(issuerMain));
      expect(decoded.fields['scope_province_name'], '安徽省');
      expect(decoded.fields['scope_city_name'], '合肥市');
      expect(
        decoded.fields['signer_pubkey'],
        ss58FromBytes(signerPubkey),
      );
    });

    test('propose_create_institution 带多余尾字段时拒绝解码', () {
      final payload = Uint8List.fromList(
          buildProposeCreateInstitutionPayload(extraTail: true));
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded, isNull,
          reason:
              'P-TX-001 禁止 subject_property/sub_type/parent_cid_number 多余尾字段');
    });

    // CANON 决策2：制度专属保留名（永久质押/安全基金/两和基金）禁止作为机构
    // 自定义账户名，命中即 decodeFailed（红色拒签）。取值逐字对齐链端 primitives。
    for (final forbidden in const ['永久质押', '安全基金', '两和基金']) {
      test('propose_create_institution 账户名命中保留名「$forbidden」时拒绝解码', () {
        final payload = Uint8List.fromList(
          buildProposeCreateInstitutionPayload(secondAccountName: forbidden),
        );
        final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
        expect(decoded, isNull, reason: '制度专属保留名不可作为机构自定义账户注册，必须红色拒签');
      });
    }

    // 主账户/费用账户是强制默认账户，正常出现在创建凭证里，维持识别。
    test('propose_create_institution 主账户/费用账户强制默认账户维持识别', () {
      final payload = Uint8List.fromList(
        buildProposeCreateInstitutionPayload(secondAccountName: '费用账户'),
      );
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_create_institution');
      expect(decoded.fields['amount_主账户'], '10,000.00 GMB');
      expect(decoded.fields['amount_费用账户'], '2.22 GMB');
    });

    test('decodes current propose_create_personal with regular_threshold field',
        () {
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
        ...u32Le(3),
        ...u128LeForTest(BigInt.from(12345)),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_create_personal');
      expect(decoded.fields['account_name'], '家庭基金');
      expect(decoded.fields['admins_len'], '3');
      expect(decoded.fields['regular_threshold'], '3/3');
      expect(decoded.fields['create_threshold'], '3/3');
      expect(decoded.fields['amount_yuan'], '123.45 GMB');
      expect(decoded.fields.containsKey('threshold'), isFalse);
    });

    test('rejects propose_create_personal without regular_threshold', () {
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

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNull);
    });

    test('rejects propose_create_personal regular_threshold below majority',
        () {
      final name = utf8.encode('家庭基金');
      final admins = [
        List<int>.filled(32, 0x11),
        List<int>.filled(32, 0x22),
        List<int>.filled(32, 0x33),
        List<int>.filled(32, 0x44),
      ];
      final payload = Uint8List.fromList([
        0x07,
        0x00,
        (name.length << 2) & 0xff,
        ...name,
        (admins.length << 2) & 0xff,
        ...admins.expand((admin) => admin),
        ...u32Le(2),
        ...u128LeForTest(BigInt.from(12345)),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNull);
    });

    test('rejects propose_create_personal with admins_len and threshold', () {
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

      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));

      expect(decoded, isNull);
    });

    // -----------------------------------------------------------------------
    // ADR-008 step2d 双端字节一致性 fixture:
    // 三组凭证的 SCALE 字节流由 Python 生成器(链端 codec.encode 等价产出)固化,
    // 统一真源在 ../memory/06-quality/fixtures/，citizenwallet / citizenapp / 链端 runtime
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

    test('fixture step2d cast_referendum: decoder 解出 issuer/scope 字段', () {
      final fixture = readFixture();
      final caseEntry = (fixture['cases'] as List)
          .firstWhere((e) => e['name'] == 'cast_referendum');
      final hex = caseEntry['expected_call_data_hex'] as String;
      expect(caseEntry['pallet_index'], 23);
      expect(caseEntry['call_index'], 1);
      expect(hex.toLowerCase().startsWith('0x1701'), isTrue);
      // fixture 固化的是纯 call_data,真实 QR 还带签名扩展尾。
      final decoded =
          PayloadDecoder.decode(hexOf(withSigningTail(bytesFromHex(hex))));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cast_referendum');
      expect(decoded.fields['proposal_id'], '99');
      expect(decoded.fields['approve'], 'true');
      expect(decoded.fields['issuer_cid_number'],
          (caseEntry['fields'] as Map)['issuer_cid_number']);
      expect(
          decoded.fields['issuer_main_account'],
          ss58FromHex((caseEntry['fields'] as Map)['issuer_main_account_hex']
              as String));
      expect(decoded.fields['scope_province_name'],
          (caseEntry['fields'] as Map)['scope_province_name']);
      expect(decoded.fields['scope_city_name'],
          (caseEntry['fields'] as Map)['scope_city_name']);
      expect(
          decoded.fields['signer_pubkey'],
          ss58FromHex(
              (caseEntry['fields'] as Map)['signer_pubkey_hex'] as String));
    });

    // 协议升级 fixture step2d propose_runtime_upgrade decoder 用例已删:同上,SCALE decoder
    // 整体下线,fixture 走 OfflineSignService.verifyPayload 的哈希直签例外。

    test('fixture step2d propose_resolution_issuance: decoder 解出新字段', () {
      final fixture = readFixture();
      final caseEntry = (fixture['cases'] as List)
          .firstWhere((e) => e['name'] == 'propose_resolution_issuance');
      final hex = caseEntry['expected_call_data_hex'] as String;
      // fixture 固化的是纯 call_data,真实 QR 还带签名扩展尾。
      final decoded =
          PayloadDecoder.decode(hexOf(withSigningTail(bytesFromHex(hex))));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_resolution_issuance');
      expect(decoded.fields['allocation_count'], '2');
      expect(decoded.fields.containsKey('eligible_total'), isFalse);
    });

    test('decodes propose_resolution_issuance (pallet=8 call=0) 当前字段', () {
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
      final payload = Uint8List.fromList([
        0x08, 0x00, // pallet=8 call=0
        (reason.length << 2) & 0xff,
        ...reason,
        ...totalLe,
        // allocations Vec count=2
        (2 << 2) & 0xff,
        ...alloc1,
        ...alloc2,
      ]);
      final decoded = PayloadDecoder.decode(hexOf(withSigningTail(payload)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_resolution_issuance');
      expect(decoded.fields['reason'], '紧急救灾');
      expect(decoded.fields['allocation_count'], '2');
      expect(decoded.fields.containsKey('eligible_total'), isFalse);
    });

    // -----------------------------------------------------------------------
    // 签名扩展尾校验(2026-06-10):真实 QR payload_hex = call_data + 扩展尾。
    // 历史 bug:84080b6a 把多个分支改成"严格到尾"却没算扩展尾,
    // 国储会转账提案等 9 类提案扫码必红。本组用例锁死两端约定:
    // 带合法尾 → 解码成功;裸 call_data / 篡改尾 → null(红色拒签)。
    // -----------------------------------------------------------------------

    List<int> buildNrcTransferCallData() => [
          0x13, 0x00,
          ...InstitutionCode.codeBytes('NRC'), // institution_code = 国储会
          ...List<int>.filled(32, 0x66), // institution AccountId32
          ...List<int>.filled(32, 0x44), // beneficiary
          ...u128LeForTest(BigInt.from(12345)),
          0x00, // remark 空 Vec
        ];

    test('decodes 国储会 propose_transfer 带真实签名扩展尾', () {
      final decoded = PayloadDecoder.decode(
          hexOf(withSigningTail(buildNrcTransferCallData())));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_transfer');
      expect(decoded.fields['institution'], '国储会');
      expect(decoded.fields['amount_yuan'], '123.45 GMB');
      expect(decoded.fields['remark'], '');
    });

    test('propose_transfer 大 nonce(两字节 Compact)尾部同样接受', () {
      final decoded = PayloadDecoder.decode(
          hexOf(withSigningTail(buildNrcTransferCallData(), nonce: 1000)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_transfer');
    });

    test('rejects 裸 call_data(无签名扩展尾)', () {
      expect(PayloadDecoder.decode(hexOf(buildNrcTransferCallData())), isNull,
          reason: '真实 SigningPayload 必带扩展尾,裸 call_data 拒签');
    });

    test('rejects 篡改的签名扩展尾', () {
      final callData = buildNrcTransferCallData();
      final good = withSigningTail(callData);

      // era ≠ immortal(0x00)
      final badEra = Uint8List.fromList(good);
      badEra[callData.length] = 0x15;
      expect(PayloadDecoder.decode(hexOf(badEra)), isNull);

      // immortal 下 birth hash 必等于 genesis hash
      final badBirth = Uint8List.fromList(good);
      badBirth[badBirth.length - 2] ^= 0xff;
      expect(PayloadDecoder.decode(hexOf(badBirth)), isNull);

      // call_data 与尾部之间夹带多余字节
      expect(
          PayloadDecoder.decode(hexOf([...callData, 0xee, ...signingTail()])),
          isNull);

      // 尾部末尾多挂字节
      expect(PayloadDecoder.decode(hexOf([...good, 0x00])), isNull);
    });
  });

  // 立法院 LegislationYuan(27) + 立法投票 LegislationVote(28),布局逐字段对齐
  // citizenchain runtime + citizenapp legislation_codec。夹具必须带签名扩展尾。
  group('立法 pallet 解码(LegislationYuan 27 / LegislationVote 28)', () {
    // 机构码 [u8;4] 右补 0。
    List<int> code4(String code) {
      final raw = utf8.encode(code);
      final out = List<int>.filled(4, 0);
      for (var i = 0; i < 4 && i < raw.length; i++) {
        out[i] = raw[i];
      }
      return out;
    }

    final acct = List<int>.generate(32, (i) => (i + 1) & 0xff);

    // (InstitutionCode, AccountId32) 平铺 36 字节。
    List<int> body(String code) => [...code4(code), ...acct];

    // 一章一节一条无款的最小章节树。
    List<int> minimalChapters() => [
          ...compactU32(1), // 1 章
          ...u32Le(1), // Chapter.number
          ...compactVec('总则'), // Chapter.title
          0x00, // Chapter.title_en None
          ...compactU32(1), // 1 节
          ...u32Le(1), // Section.number
          ...compactVec('第一节'), // Section.title
          0x00, // Section.title_en None
          ...compactU32(1), // 1 条
          ...u32Le(1), // Article.number
          ...compactVec('第一条'), // Article.title
          0x00, // Article.title_en None
          ...compactVec('正文内容'), // Article.body
          0x00, // Article.body_en None
          ...compactU32(0), // 0 款
        ];

    test('decodes propose_enact_law (27.0)', () {
      final callData = [
        27, 0,
        1, // tier = National(1)
        ...u32Le(110000), // scope_code
        ...compactU32(2), // houses 2 项
        ...body('NLG'),
        ...body('NLG'),
        ...body('PRS'), // proposer_body
        ...body('PRS'), // executive
        0x00, // legislature None
        2, // vote_type = Major(2)
        ...compactVec('教育法'), // title
        0x00, // title_en None
        ...minimalChapters(),
        ...u32Le(5000), // effective_at
      ];
      final decoded =
          PayloadDecoder.decode(hexOf(withSigningTail(callData)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_enact_law');
      expect(decoded.fields['title'], '教育法');
      expect(decoded.fields['tier'], '国家级');
      expect(decoded.fields['vote_type'], '重要案');
      expect(decoded.fields['chapter_count'], '1');
      expect(decoded.fields['article_count'], '1');
      expect(decoded.fields['effective_at'], '5000');
    });

    test('rejects propose_enact_law with tier=Constitution(0)', () {
      final callData = [
        27, 0,
        0, // tier = Constitution(0) → 立法入口禁止新立宪法
        ...u32Le(0),
        ...compactU32(1),
        ...body('NLG'),
        ...body('PRS'),
        ...body('PRS'),
        0x00,
        0,
        ...compactVec('宪法'),
        0x00,
        ...minimalChapters(),
        ...u32Le(1),
      ];
      expect(PayloadDecoder.decode(hexOf(withSigningTail(callData))), isNull);
    });

    test('rejects propose_enact_law with out-of-range vote_type', () {
      final callData = [
        27, 0,
        1,
        ...u32Le(0),
        ...compactU32(1),
        ...body('NLG'),
        ...body('PRS'),
        ...body('PRS'),
        0x00,
        9, // 非法 vote_type
        ...compactVec('法'),
        0x00,
        ...minimalChapters(),
        ...u32Le(1),
      ];
      expect(PayloadDecoder.decode(hexOf(withSigningTail(callData))), isNull);
    });

    test('decodes propose_amend_law (27.1)', () {
      final callData = [
        27, 1,
        ...u64Le(42), // law_id
        ...body('PLG'),
        ...body('PGV'),
        0x01, ...body('PLG'), // legislature Some
        4, // vote_type = Special(4)
        ...compactVec('修订版'),
        0x00,
        ...minimalChapters(),
        ...u32Le(7777),
      ];
      final decoded =
          PayloadDecoder.decode(hexOf(withSigningTail(callData)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_amend_law');
      expect(decoded.fields['law_id'], '42');
      expect(decoded.fields['title'], '修订版');
      expect(decoded.fields['vote_type'], '特别案（强制公投）');
      expect(decoded.fields['effective_at'], '7777');
    });

    test('decodes propose_repeal_law (27.2)', () {
      final callData = [
        27, 2,
        ...u64Le(7), // law_id
        ...body('CLEG'),
        ...body('CGOV'),
        0x00, // legislature None
        0, // vote_type = Regular(0)
      ];
      final decoded =
          PayloadDecoder.decode(hexOf(withSigningTail(callData)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_repeal_law');
      expect(decoded.fields['law_id'], '7');
      expect(decoded.fields['vote_type'], '常规案');
    });

    test('decodes prepare_population_snapshot (28.0)', () {
      final issuerAccount = List<int>.generate(32, (i) => (i + 5) & 0xff);
      final signerPubkey = List<int>.generate(32, (i) => (i + 9) & 0xff);
      final callData = [
        28, 0,
        ...u64Le(123456), // eligible_total
        ...compactVec('nonce-x'), // snapshot_nonce
        ...compactVec('sig-y'), // signature
        ...compactVec('NLG0000001'), // issuer_cid_number
        ...issuerAccount,
        ...signerPubkey,
        ...compactVec('某省'), // scope_province_name
        ...compactVec('某市'), // scope_city_name
      ];
      final decoded =
          PayloadDecoder.decode(hexOf(withSigningTail(callData)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'prepare_legislation_snapshot');
      expect(decoded.fields['eligible_total'], '123456');
      expect(decoded.fields['issuer_cid_number'], 'NLG0000001');
      expect(decoded.fields['issuer_main_account'], ss58FromBytes(issuerAccount));
      expect(decoded.fields['signer_pubkey'], ss58FromBytes(signerPubkey));
      expect(decoded.fields['scope_province_name'], '某省');
      expect(decoded.fields['scope_city_name'], '某市');
    });

    test('decodes cast_house_vote (28.1)', () {
      final callData = [28, 1, ...u64Le(99), 0x01];
      final decoded =
          PayloadDecoder.decode(hexOf(withSigningTail(callData)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cast_house_vote');
      expect(decoded.fields['proposal_id'], '99');
      expect(decoded.fields['approve'], 'true');
    });

    test('decodes cast_referendum_vote (28.2)', () {
      final bindingId = List<int>.generate(32, (i) => (i + 2) & 0xff);
      final issuerAccount = List<int>.generate(32, (i) => (i + 5) & 0xff);
      final signerPubkey = List<int>.generate(32, (i) => (i + 9) & 0xff);
      final callData = [
        28, 2,
        ...u64Le(55), // proposal_id
        ...bindingId, // binding_id [u8;32]
        ...compactVec('nonce-z'), // nonce
        ...compactVec('sig-w'), // signature
        ...compactVec('CID12345'), // issuer_cid_number
        ...issuerAccount,
        ...signerPubkey,
        ...compactVec('粤'), // scope_province_name
        ...compactVec('深圳'), // scope_city_name
        0x00, // approve = false
      ];
      final decoded =
          PayloadDecoder.decode(hexOf(withSigningTail(callData)));
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cast_referendum_vote');
      expect(decoded.fields['proposal_id'], '55');
      expect(decoded.fields['approve'], 'false');
      expect(decoded.fields['issuer_cid_number'], 'CID12345');
      expect(decoded.fields['scope_city_name'], '深圳');
    });

    test('decodes executive_sign (28.3) / override_sign (28.4) / guard_vote (28.5)',
        () {
      final exec = PayloadDecoder.decode(
          hexOf(withSigningTail([28, 3, ...u64Le(1), 0x01])));
      expect(exec?.action, 'executive_sign');
      final override = PayloadDecoder.decode(
          hexOf(withSigningTail([28, 4, ...u64Le(2), 0x00])));
      expect(override?.action, 'override_sign');
      final guard = PayloadDecoder.decode(
          hexOf(withSigningTail([28, 5, ...u64Le(3), 0x01])));
      expect(guard?.action, 'guard_vote');
    });

    test('rejects 裸 call_data 无签名尾(立法投票)', () {
      expect(PayloadDecoder.decode(hexOf([28, 1, ...u64Le(1), 0x01])), isNull);
    });
  });

  // ADR-026 Phase 2 二进制前缀域金标:冷钱包 decode() 必须能解析 node/citizenapp
  // 用相同 4B 前缀(GMB||0x18 / GMB||0x19)构造的 payload。fixture 是 Rust 切片
  // 导出的副本(canonical 真源 primitives/tests/fixtures),四方逐字节锁步。
  group('二进制前缀域金标(node/citizenapp 构造 ↔ 冷钱包 decode 锁步)', () {
    final file = File('test/signer/fixtures/binary_prefix_domain_vectors.json');
    if (!file.existsSync()) {
      test('二进制前缀域金标 fixture 尚未生成', () {
        markTestSkipped('缺少 fixture —— 由 Rust 切片导出');
      }, skip: '缺少 fixture');
      return;
    }
    final root = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
    final vectors = (root['vectors'] as List).cast<Map<String, dynamic>>();
    final byName = {for (final v in vectors) v['name'] as String: v};

    test('ACTIVATE_ADMIN fixture payload → decode 解出 activate_admin_account',
        () {
      final v = byName['ACTIVATE_ADMIN']!;
      final decoded = PayloadDecoder.decode('0x${v['payload_hex']}');
      expect(decoded, isNotNull);
      expect(decoded!.action, 'activate_admin_account');
      // institution_code = "NRC"(fixture sample),kind=0 与 NRC 固定治理码匹配。
      expect(decoded.fields['institution_code'], isNotEmpty);
    });

    test('DECRYPT fixture payload → decode 解出 decrypt_admin + cid_number', () {
      final v = byName['DECRYPT']!;
      final inputs = v['sample_inputs'] as Map<String, dynamic>;
      final decoded = PayloadDecoder.decode('0x${v['payload_hex']}');
      expect(decoded, isNotNull);
      expect(decoded!.action, 'decrypt_admin');
      expect(decoded.fields['cid_number'], inputs['cid_number']);
    });
  });
}
