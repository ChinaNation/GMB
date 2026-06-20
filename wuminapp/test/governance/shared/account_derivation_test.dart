// account_derivation 统一派生原语单测(ADR-018 §九,公权机构卡0)。
//
// golden 向量取自 governance_institution_registry.generated.dart 的国储会/中枢省
// 制度账户 hex —— 这些地址即链上派生结果,可交叉验证本端派生与
// citizenchain primitives::core_const::derive_duoqian_account 字节对齐:
//   preimage = b"DUOQIAN" || op_tag || ss58.to_le_bytes() || payload
//   OP_MAIN(0x00)/OP_FEE(0x01)/OP_AN(0x03)/OP_HE(0x04): payload = sfid_number
//   OP_INSTITUTION(0x06): payload = sfid_number || account_name

import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:wuminapp_mobile/governance/shared/account_derivation.dart';
import 'package:wuminapp_mobile/governance/shared/reserved_account_names.dart';

void main() {
  // 国储会 LN001-GCB05-944805165-2026
  const nrcSfid = 'LN001-GCB05-944805165-2026';
  const nrcMain =
      '39936ebd8564c61f315662ff859d8fb5470ac3f1b4bfbf86746aff391d14db3d';
  const nrcFee =
      '66d1de031e332facb67bd20ae428e187ae4bbf3caa0a1421bd0023c49fb228d3';
  const nrcSafety =
      'c878e700bde52b5c9c2a94bcf5296c4f6a75ca61b8e920a4e53a01c6da433e52';
  const nrcHe =
      'ce19b7f0df3e9ba6c88b02364aa97cd1994df25aaa86c36e790ee85eea009f76';
  // 中枢省 ZS001-GCB0R-016974075-2026
  const prcSfid = 'ZS001-GCB0R-016974075-2026';
  const prcMain =
      '1a2853434d5b7bb336670dab136b2479a029fdbbb447f49482f09be80660024a';
  const prcFee =
      '5bc1f22ef6e4147e61ac745f50e77f17656c0d6d789d600a1ffe014e5d44ab58';

  group('机构账户派生 golden 向量(链上注册表交叉验证)', () {
    test('国储会主账户 OP_MAIN', () {
      expect(
          hexFromAccountId(deriveInstitutionMainAccountId(nrcSfid)), nrcMain);
    });

    test('国储会费用账户 OP_FEE', () {
      expect(hexFromAccountId(deriveInstitutionFeeAccountId(nrcSfid)), nrcFee);
    });

    test('中枢省主账户 / 费用账户', () {
      expect(
          hexFromAccountId(deriveInstitutionMainAccountId(prcSfid)), prcMain);
      expect(hexFromAccountId(deriveInstitutionFeeAccountId(prcSfid)), prcFee);
    });

    test('名字路由:主账户/费用账户与显式派生一致', () {
      expect(
        hexFromAccountId(
          deriveInstitutionAccountIdByName(nrcSfid, kReservedNameMain),
        ),
        nrcMain,
      );
      expect(
        hexFromAccountId(
          deriveInstitutionAccountIdByName(nrcSfid, kReservedNameFee),
        ),
        nrcFee,
      );
    });

    test('名字路由:安全基金 OP_AN / 两和基金 OP_HE', () {
      expect(
        hexFromAccountId(
          deriveInstitutionAccountIdByName(nrcSfid, kReservedNameAnquan),
        ),
        nrcSafety,
      );
      expect(
        hexFromAccountId(
          deriveInstitutionAccountIdByName(nrcSfid, kReservedNameHe),
        ),
        nrcHe,
      );
    });
  });

  group('自定义账户 OP_INSTITUTION(0x06)', () {
    test('payload 追加 account_name,与手工构造一致', () {
      const name = '业务专户A';
      final expected = Hasher.blake2b256.hash(
        Uint8List.fromList(<int>[
          ...utf8.encode('DUOQIAN'),
          0x06,
          2027 & 0xFF,
          (2027 >> 8) & 0xFF,
          ...utf8.encode(nrcSfid),
          ...utf8.encode(name),
        ]),
      );
      expect(
        hexFromAccountId(deriveInstitutionCustomAccountId(nrcSfid, name)),
        hexFromAccountId(expected),
      );
    });

    test('自定义账户地址不同于主账户', () {
      expect(
        hexFromAccountId(deriveInstitutionCustomAccountId(nrcSfid, '专户')),
        isNot(hexFromAccountId(deriveInstitutionMainAccountId(nrcSfid))),
      );
    });

    test('空账户名抛错(对齐链端 EmptyAccountName)', () {
      expect(
        () => deriveInstitutionCustomAccountId(nrcSfid, ''),
        throwsArgumentError,
      );
    });

    test('自定义名命中受限保留名抛错', () {
      expect(
        () => deriveInstitutionCustomAccountId(nrcSfid, kReservedNameMain),
        throwsArgumentError,
      );
    });
  });

  group('个人多签派生(0x05)归位后行为不变', () {
    test('SS58 输出与 core 派生一致', () {
      final creator = Uint8List.fromList(List<int>.generate(32, (i) => i));
      final viaCore = ss58FromAccountId(
        deriveDuoqianAccountId(
          opTag: kOpPersonal,
          payload: <int>[...creator, ...utf8.encode('个人钱包')],
        ),
      );
      expect(
        deriveDuoqianPersonalAddress(
          creatorPubkey: creator,
          accountName: '个人钱包',
        ),
        viaCore,
      );
    });

    test('creator 非 32 字节抛错', () {
      expect(
        () => deriveDuoqianPersonalAddress(
          creatorPubkey: Uint8List(31),
          accountName: 'x',
        ),
        throwsArgumentError,
      );
    });
  });
}
