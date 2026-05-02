import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/duoqian/shared/duoqian_manage_service.dart';

/// ADR-008 step2b:wuminapp 端 `propose_create_institution`
/// (DuoqianManage pallet=17 / call_index=5,14 参)SCALE 字节一致性测试。
///
/// 不依赖外部 fixture(目前 step2d 的 Python 生成器尚未覆盖该 extrinsic);
/// 测试就地按照"链端 SCALE 入参顺序 → 字节序列"手工拼装 expected,
/// 与 wuminapp `DuoqianManageService.buildProposeCreateInstitutionCallForTest`
/// 输出逐字节对照。任何一端编码漂移即让冷钱包 decoder 17.5 落黄牌
/// (feedback_qr_signing_two_color.md)→ 该测试直接断言失败。
void main() {
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

  /// 单字节模式 Compact<u32>(适用于 0..=63)。
  int compactSingleByte(int v) => (v << 2) & 0xff;

  Uint8List u32Le(int v) {
    final bd = ByteData(4);
    bd.setUint32(0, v, Endian.little);
    return bd.buffer.asUint8List();
  }

  Uint8List u128Le(BigInt value) {
    final out = Uint8List(16);
    var v = value;
    for (var i = 0; i < 16; i++) {
      out[i] = (v & BigInt.from(0xff)).toInt();
      v = v >> 8;
    }
    return out;
  }

  group('DuoqianManageService.buildProposeCreateInstitutionCallForTest', () {
    test('14 参 SCALE 字节序列与链端 codec 一致(call_index=5)', () {
      final sfidId = utf8.encode('SFR-AH001-1234567890-20260501');
      final institutionName = utf8.encode('安徽省储行');
      final accountName0 = utf8.encode('主');
      final accountName1 = utf8.encode('费用');
      final amount0 = BigInt.from(100000); // 1000.00 元
      final amount1 = BigInt.from(50000); // 500.00 元
      final adminA = List<int>.filled(32, 0x11);
      final adminB = List<int>.filled(32, 0x22);
      final registerNonce = utf8.encode('reg-nonce-001');
      final signature = List<int>.filled(64, 0xDD);
      final province = utf8.encode('安徽省');
      final signerAdminPubkey =
          List<int>.generate(32, (i) => 0xC0 + (i & 0x0F));
      final a3 = utf8.encode('SFR');
      final subType = utf8.encode('SHENG_BANK');

      // ── 手工拼装 expected ──
      final expected = <int>[
        0x11, 0x05, // pallet=17 call=5
        // sfid_id: BoundedVec<u8>
        compactSingleByte(sfidId.length), ...sfidId,
        // institution_name
        compactSingleByte(institutionName.length), ...institutionName,
        // accounts: count=2
        compactSingleByte(2),
        // accounts[0]
        compactSingleByte(accountName0.length), ...accountName0,
        ...u128Le(amount0),
        // accounts[1]
        compactSingleByte(accountName1.length), ...accountName1,
        ...u128Le(amount1),
        // admin_count: u32 LE
        ...u32Le(2),
        // duoqian_admins: count=2
        compactSingleByte(2), ...adminA, ...adminB,
        // threshold: u32 LE
        ...u32Le(2),
        // register_nonce
        compactSingleByte(registerNonce.length), ...registerNonce,
        // signature: 64B → Compact mode 1(0x01 0x01)
        0x01, 0x01, ...signature,
        // ★ province
        compactSingleByte(province.length), ...province,
        // ★ signer_admin_pubkey: [u8;32]
        ...signerAdminPubkey,
        // a3
        compactSingleByte(a3.length), ...a3,
        // sub_type: Some(...)
        0x01, compactSingleByte(subType.length), ...subType,
        // parent_sfid_id: None
        0x00,
      ];

      final actual = DuoqianManageService.buildProposeCreateInstitutionCallForTest(
        sfidId: Uint8List.fromList(sfidId),
        institutionName: Uint8List.fromList(institutionName),
        accounts: [
          InstitutionInitialAccountInput(
            accountName: Uint8List.fromList(accountName0),
            amountFen: amount0,
          ),
          InstitutionInitialAccountInput(
            accountName: Uint8List.fromList(accountName1),
            amountFen: amount1,
          ),
        ],
        adminCount: 2,
        adminPubkeys: [
          Uint8List.fromList(adminA),
          Uint8List.fromList(adminB),
        ],
        threshold: 2,
        registerNonce: Uint8List.fromList(registerNonce),
        signature: Uint8List.fromList(signature),
        province: Uint8List.fromList(province),
        signerAdminPubkey: Uint8List.fromList(signerAdminPubkey),
        a3: Uint8List.fromList(a3),
        subType: Uint8List.fromList(subType),
        parentSfidId: null,
      );

      expect(
        hexEncode(actual).toLowerCase(),
        hexEncode(Uint8List.fromList(expected)).toLowerCase(),
        reason: '链端 = wumin decoder 17.5 = wuminapp 三端 SCALE 字节必须一致 '
            '(ADR-008 step2b)',
      );
      // 头 2 字节必须是 [0x11, 0x05](pallet=17 / call=5)
      expect(actual[0], 0x11);
      expect(actual[1], 0x05);
    });

    test('signer_admin_pubkey 长度非 32 → 拒绝构造 call data', () {
      expect(
        () => DuoqianManageService.buildProposeCreateInstitutionCallForTest(
          sfidId: Uint8List.fromList(utf8.encode('SFR-AH001-1-20260501')),
          institutionName: Uint8List.fromList(utf8.encode('安徽省储行')),
          accounts: const [],
          adminCount: 0,
          adminPubkeys: const [],
          threshold: 0,
          registerNonce: Uint8List(0),
          signature: Uint8List(0),
          province: Uint8List.fromList(utf8.encode('安徽省')),
          signerAdminPubkey: Uint8List(31), // 短 1 字节
          a3: Uint8List.fromList(utf8.encode('SFR')),
          subType: null,
          parentSfidId: null,
        ),
        throwsArgumentError,
      );
    });

    test('parent_sfid_id Some 分支:Option<BoundedVec<u8>> 编码为 0x01 + Compact len + bytes',
        () {
      final parent = utf8.encode('SFR-AH000-PARENT-20260101');
      final actual = DuoqianManageService.buildProposeCreateInstitutionCallForTest(
        sfidId: Uint8List.fromList(utf8.encode('FFR-AH001-CHILD-20260501')),
        institutionName: Uint8List.fromList(utf8.encode('支行')),
        accounts: const [],
        adminCount: 0,
        adminPubkeys: const [],
        threshold: 0,
        registerNonce: Uint8List(0),
        signature: Uint8List(0),
        province: Uint8List.fromList(utf8.encode('安徽省')),
        signerAdminPubkey: hexToBytes(
            '0xc0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf'),
        a3: Uint8List.fromList(utf8.encode('FFR')),
        subType: null,
        parentSfidId: Uint8List.fromList(parent),
      );

      // 末尾必须以 [0x01, Compact(parent.length), ...parent] 结尾
      final tail = actual.sublist(actual.length - (1 + 1 + parent.length));
      expect(tail[0], 0x01); // Some
      expect(tail[1], (parent.length << 2) & 0xff); // Compact 单字节模式
      expect(tail.sublist(2), parent);
    });
  });
}
