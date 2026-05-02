import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wumin/signer/pallet_registry.dart';
import 'package:wumin/signer/payload_decoder.dart';

void main() {
  final spec = PalletRegistry.supportedSpecVersions.first;

  String hexOf(List<int> payload) =>
      '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';

  List<int> compactVec(String text) {
    final bytes = utf8.encode(text);
    return [bytes.length << 2, ...bytes];
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
      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'internal_vote');
      expect(decoded.fields['proposal_id'], '42');
      expect(decoded.fields['approve'], 'true');
      expect(decoded.summary, contains('赞成'));
    });

    test('decodes internal_vote (pallet=9 call=0) approve=false', () {
      final payload = Uint8List.fromList([
        0x09,
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
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);
      expect(decoded!.action, 'internal_vote');
      expect(decoded.fields['approve'], 'false');
      expect(decoded.summary, contains('反对'));
    });

    test('decodes joint_vote (pallet=9 call=1)', () {
      // Phase 2 重排：joint_vote 由原 call=3 迁到 call=1。
      final payload = Uint8List.fromList([
        0x09,
        0x01,
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
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'joint_vote');
      expect(decoded.fields['proposal_id'], '7');
      expect(decoded.fields['approve'], 'false');
      expect(decoded.summary, contains('反对'));
    });

    test('decodes citizen_vote (pallet=9 call=2) ADR-008 step3 双层凭证', () {
      // Phase 2 重排：citizen_vote 由原 call=4 迁到 call=2。
      // ADR-008 step3：SCALE 末尾在 approve 前加 (province, signer_admin_pubkey)。
      final province = utf8.encode('安徽省');
      final adminPubkey = List<int>.generate(32, (i) => 0xA0 + (i & 0x0F));
      final payload = Uint8List.fromList([
        0x09, 0x02,
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
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'citizen_vote');
      expect(decoded.fields['proposal_id'], '99');
      expect(decoded.fields['approve'], 'true');
      expect(decoded.fields['province'], '安徽省');
      expect(
        decoded.fields['signer_admin_pubkey'],
        '0x${adminPubkey.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
      );
    });

    test('citizen_vote 旧 SCALE 字节流(无 province/admin)拒绝解码', () {
      // ADR-008 step3 后 SCALE 必须含新字段。旧字节流长度不足 → null。
      // feedback_no_compatibility:不留兼容垫片,老凭证不识别即拒绝。
      final payload = Uint8List.fromList([
        0x09, 0x02,
        99, 0, 0, 0, 0, 0, 0, 0,
        ...List.filled(32, 0),
        0,
        0,
        1, // 旧版只到 approve, 长度 = 45
      ]);
      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);
      expect(decoded, isNull,
          reason: '旧凭证长度 45 < 78 必须被拒绝, 防止白盲签');
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

    test('decodes clearing bank register node call', () {
      const sfidId = 'SFR-AH001-ZG1Y-883241719-20260428';
      const peerId = '12D3KooWABCDEFG1234567890abcdefghijk';
      const domain = 'l2.example.com';
      final payload = Uint8List.fromList([
        21,
        50,
        ...compactVec(sfidId),
        ...compactVec(peerId),
        ...compactVec(domain),
        ...u16Le(9944),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(payload), specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'register_clearing_bank');
      expect(decoded.fields['sfid_id'], sfidId);
      expect(decoded.fields['peer_id'], peerId);
      expect(decoded.fields['rpc_domain'], domain);
      expect(decoded.fields['rpc_port'], '9944');
    });

    test('decodes clearing bank endpoint update call', () {
      const sfidId = 'SFR-AH001-ZG1Y-883241719-20260428';
      const domain = 'new-l2.example.com';
      final payload = Uint8List.fromList([
        21,
        51,
        ...compactVec(sfidId),
        ...compactVec(domain),
        ...u16Le(443),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(payload), specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'update_clearing_bank_endpoint');
      expect(decoded.fields['sfid_id'], sfidId);
      expect(decoded.fields['new_domain'], domain);
      expect(decoded.fields['new_port'], '443');
    });

    test('decodes clearing bank unregister call', () {
      const sfidId = 'SFR-AH001-ZG1Y-883241719-20260428';
      final payload = Uint8List.fromList([
        21,
        52,
        ...compactVec(sfidId),
      ]);

      final decoded = PayloadDecoder.decode(hexOf(payload), specVersion: spec);

      expect(decoded, isNotNull);
      expect(decoded!.action, 'unregister_clearing_bank');
      expect(decoded.fields['sfid_id'], sfidId);
    });

    test('decodes clearing bank decrypt challenge without specVersion', () {
      const sfidId = 'SFR-AH001-ZG1Y-883241719-20260428';
      final idBytes = List<int>.filled(48, 0);
      final rawId = ascii.encode(sfidId);
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
      expect(decoded.fields['sfid_id'], sfidId);
      expect(decoded.summary, contains('解密清算行管理员'));
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
        0x13,
        0x02,
        ...idBytes,
        ...amountBytes,
      ]);

      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
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
        0x13,
        0x02,
        ...idBytes,
        ...amountBytes,
      ]);
      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
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

      final hex =
          '0x${payload.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}';
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
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_transfer');
      expect(decoded.fields['proposal_id'], '100');
      expect(decoded.summary, contains('#100'));
    });

    test('decodes execute_safety_fund_transfer (pallet=19 call=4)', () {
      final payload = buildProposalIdPayload(0x13, 0x04, 101);
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_safety_fund_transfer');
      expect(decoded.fields['proposal_id'], '101');
    });

    test('decodes execute_sweep_to_main (pallet=19 call=5)', () {
      final payload = buildProposalIdPayload(0x13, 0x05, 102);
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_sweep_to_main');
      expect(decoded.fields['proposal_id'], '102');
    });

    test('decodes execute_destroy (pallet=14 call=1)', () {
      final payload = buildProposalIdPayload(0x0e, 0x01, 200);
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_destroy');
      expect(decoded.fields['proposal_id'], '200');
    });

    test('decodes execute_admin_replacement (pallet=12 call=1)', () {
      final payload = buildProposalIdPayload(0x0c, 0x01, 300);
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_admin_replacement');
      expect(decoded.fields['proposal_id'], '300');
    });

    test('decodes execute_replace_grandpa_key (pallet=16 call=1)', () {
      final payload = buildProposalIdPayload(0x10, 0x01, 400);
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'execute_replace_grandpa_key');
      expect(decoded.fields['proposal_id'], '400');
    });

    test('decodes cancel_failed_replace_grandpa_key (pallet=16 call=2)', () {
      final payload = buildProposalIdPayload(0x10, 0x02, 401);
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'cancel_failed_replace_grandpa_key');
      expect(decoded.fields['proposal_id'], '401');
    });

    test('decodes cleanup_rejected_proposal (pallet=17 call=4)', () {
      final payload = buildProposalIdPayload(0x11, 0x04, 500);
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
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
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'developer_direct_upgrade');
      expect(decoded.fields['wasm_size'], '0 KB'); // 4 字节 < 1 KB
      expect(
        decoded.fields['wasm_hash'],
        '0x88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589',
      );
    });

    test(
        'decodes propose_runtime_upgrade 含 wasm_hash + eligible_total + province + signer_admin_pubkey',
        () {
      // reason="ok" + wasm="abcd" + eligible_total=1234567
      // sha256("abcd") 同上 test。
      // ADR-008 step3:SCALE 末尾在 signature 后加 (province, signer_admin_pubkey)。
      final reasonBytes = 'ok'.codeUnits; // 2 字节
      final wasmBytes = [0x61, 0x62, 0x63, 0x64]; // 4 字节
      const eligibleTotal = 1234567;
      // 模拟 SFID 后端返回的 nonce + 64 字节 sr25519 签名
      final nonceBytes = utf8.encode('snap-2026-05-01-AH');
      final sigBytes = List<int>.filled(64, 0xCC);
      final province = utf8.encode('安徽省');
      final adminPubkey = List<int>.generate(32, (i) => 0xB0 + (i & 0x0F));

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
        // snapshot_nonce: Vec<u8>
        (nonceBytes.length << 2) & 0xff,
        ...nonceBytes,
        // signature: Vec<u8> (64 字节, len=64=0x40 → Compact mode 1: (0x40<<2)|1 = 0x101 → 两字节)
        // 0x101 = 0x01 0x01 LE
        0x01, 0x01,
        ...sigBytes,
        // ★ province: Vec<u8>
        (province.length << 2) & 0xff,
        ...province,
        // ★ signer_admin_pubkey: [u8;32]
        ...adminPubkey,
      ]);
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_runtime_upgrade');
      expect(decoded.fields['reason'], 'ok');
      expect(decoded.fields['wasm_size'], '0 KB');
      expect(
        decoded.fields['wasm_hash'],
        '0x88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589',
      );
      expect(decoded.fields['eligible_total'], '1234567');
      expect(decoded.fields['province'], '安徽省');
      expect(
        decoded.fields['signer_admin_pubkey'],
        '0x${adminPubkey.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
      );
    });

    // -----------------------------------------------------------------------
    // ADR-008 step2d 新加 decoder:
    // - propose_create_institution(17.5):机构多签账户创建提案
    //   (走 SFID 后端 ShengSigningPubkey 双层签发凭证)
    // - propose_resolution_issuance(8.0):决议发行联合提案
    //   (走 PopulationSnapshotVerifier 双层签发)
    // -----------------------------------------------------------------------

    test('decodes propose_create_institution (pallet=17 call=5) 含 province + signer_admin_pubkey',
        () {
      final sfid = utf8.encode('SFR-AH001-1234567890-20260501');
      final instName = utf8.encode('安徽省储行');
      // accounts: 1 个账户 = (name="主", amount=10_000_00 fen = 10000 元)
      final accountName = utf8.encode('主');
      final amountFen = BigInt.from(1000000); // 10000.00 GMB
      final amountLeBytes = List<int>.filled(16, 0);
      var tmp = amountFen;
      for (var i = 0; i < 16; i++) {
        amountLeBytes[i] = (tmp & BigInt.from(0xFF)).toInt();
        tmp = tmp >> 8;
      }
      final adminPubkeys = [
        List<int>.filled(32, 0x11),
        List<int>.filled(32, 0x22),
      ];
      final registerNonce = utf8.encode('reg-nonce-001');
      final signature = List<int>.filled(64, 0xDD);
      final province = utf8.encode('安徽省');
      final signerAdminPubkey = List<int>.generate(32, (i) => 0xC0 + (i & 0x0F));
      final a3 = utf8.encode('SFR');
      final subType = utf8.encode('SHENG_BANK');

      final payload = Uint8List.fromList([
        0x11, 0x05, // pallet=17 call=5
        // sfid_id: Vec<u8>
        (sfid.length << 2) & 0xff,
        ...sfid,
        // institution_name: Vec<u8>
        (instName.length << 2) & 0xff,
        ...instName,
        // accounts: Vec<{name, amount}> count=1
        (1 << 2) & 0xff,
        // account[0].name
        (accountName.length << 2) & 0xff,
        ...accountName,
        // account[0].amount: u128 LE
        ...amountLeBytes,
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
        // a3: Vec<u8>
        (a3.length << 2) & 0xff,
        ...a3,
        // sub_type: Option<Vec<u8>> = Some(...)
        0x01,
        (subType.length << 2) & 0xff,
        ...subType,
        // parent_sfid_id: Option<Vec<u8>> = None
        0x00,
      ]);
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_create_institution');
      expect(decoded.fields['sfid_id'], 'SFR-AH001-1234567890-20260501');
      expect(decoded.fields['institution_name'], '安徽省储行');
      expect(decoded.fields['admin_count'], '2');
      expect(decoded.fields['threshold'], '2/2');
      expect(decoded.fields['amount_yuan'], '10,000.00 GMB');
      expect(decoded.fields['a3'], 'SFR');
      expect(decoded.fields['province'], '安徽省');
      expect(
        decoded.fields['signer_admin_pubkey'],
        '0x${signerAdminPubkey.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
      );
    });

    // -----------------------------------------------------------------------
    // ADR-008 step2d 双端字节一致性 fixture(test/fixtures/step2d_credential_payload.json):
    // 三组凭证的 SCALE 字节流由 Python 生成器(链端 codec.encode 等价产出)固化,
    // wumin / wuminapp / 链端 runtime 三处必须产出同一序列。
    // 任何一端编码漂移 → 这里直接断言失败。
    // -----------------------------------------------------------------------

    Map<String, dynamic> readFixture() {
      final file = File('test/fixtures/step2d_credential_payload.json');
      final raw = file.readAsStringSync();
      return jsonDecode(raw) as Map<String, dynamic>;
    }

    test('fixture step2d citizen_vote: decoder 解出 province + signer_admin_pubkey',
        () {
      final fixture = readFixture();
      final caseEntry = (fixture['cases'] as List)
          .firstWhere((e) => e['name'] == 'citizen_vote');
      final hex = caseEntry['expected_call_data_hex'] as String;
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'citizen_vote');
      expect(decoded.fields['proposal_id'], '99');
      expect(decoded.fields['approve'], 'true');
      expect(decoded.fields['province'],
          (caseEntry['fields'] as Map)['province_utf8']);
      expect(decoded.fields['signer_admin_pubkey'],
          (caseEntry['fields'] as Map)['signer_admin_pubkey_hex']);
    });

    test('fixture step2d propose_runtime_upgrade: decoder 解出新字段',
        () {
      final fixture = readFixture();
      final caseEntry = (fixture['cases'] as List)
          .firstWhere((e) => e['name'] == 'propose_runtime_upgrade');
      final hex = caseEntry['expected_call_data_hex'] as String;
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);
      expect(decoded, isNotNull);
      expect(decoded!.action, 'propose_runtime_upgrade');
      expect(decoded.fields['province'],
          (caseEntry['fields'] as Map)['province_utf8']);
      expect(decoded.fields['signer_admin_pubkey'],
          (caseEntry['fields'] as Map)['signer_admin_pubkey_hex']);
      expect(decoded.fields['eligible_total'],
          (caseEntry['fields'] as Map)['eligible_total'].toString());
    });

    test('fixture step2d propose_resolution_issuance: decoder 解出新字段',
        () {
      final fixture = readFixture();
      final caseEntry = (fixture['cases'] as List)
          .firstWhere((e) => e['name'] == 'propose_resolution_issuance');
      final hex = caseEntry['expected_call_data_hex'] as String;
      final decoded = PayloadDecoder.decode(hex, specVersion: spec);
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

    test('decodes propose_resolution_issuance (pallet=8 call=0) 含 province + signer_admin_pubkey',
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
      final decoded =
          PayloadDecoder.decode(encodeHex(payload), specVersion: spec);
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
