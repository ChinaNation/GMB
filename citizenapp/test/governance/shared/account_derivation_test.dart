// account_derivation 统一派生原语单测(ADR-018 §九,公权机构卡0)。
//
// golden 向量取自 governance_institution_registry.generated.dart 的国储会/中枢省
// 制度账户 hex —— 这些地址即链上派生结果,可交叉验证本端派生与
// citizenchain primitives::core_const::derive_account 字节对齐:
//   preimage = b"GMB" || op_tag || ss58.to_le_bytes() || payload
//   OP_MAIN(0x00)/OP_FEE(0x01)/OP_AN(0x03)/OP_HE(0x04): payload = cid_number
//   OP_INSTITUTION(0x06): payload = cid_number || account_name

import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:citizenapp/governance/shared/account_derivation.dart';
import 'package:citizenapp/governance/shared/reserved_account_names.dart';

void main() {
  // 国储会 LN001-NRC0G-944805165-2026（T3/T4 新机构码 + GMB 域，单源 china_cb.rs）
  const nrcCid = 'LN001-NRC0G-944805165-2026';
  const nrcMain =
      'b38e86de933984b3a6b4190fc9d4b020ff44b38471a8a65bbf95b440e05c5153';
  const nrcFee =
      '7c0c099ee4df10c5bd3f618ddf132b6d15390fa27d2c1369f70aeb6b5f3907e5';
  const nrcSafety =
      'd78abac2e0a7772e72ba663313718e97288377d9ca2ca1467c710058f8b5effa';
  const nrcHe =
      '4ac779852c175087c445c35efecfef3ce6e0232702152ea2283f0b5ec3952e53';
  // 中枢省 ZS001-PRC0E-016974075-2026（T3/T4 新机构码 + GMB 域，单源 china_cb.rs）
  const prcCid = 'ZS001-PRC0E-016974075-2026';
  const prcMain =
      '65c057a38041753f31f1d891f4d1ce79326291cb4d340a125dd7dc33710783dd';
  const prcFee =
      '54bad80b12cedbf7a1569fb96d18d90c4793949a356eb16c6304841af81001dd';

  group('机构账户派生 golden 向量(链上注册表交叉验证)', () {
    test('国储会主账户 OP_MAIN', () {
      expect(hexFromAccountId(deriveInstitutionMainAccountId(nrcCid)), nrcMain);
    });

    test('国储会费用账户 OP_FEE', () {
      expect(hexFromAccountId(deriveInstitutionFeeAccountId(nrcCid)), nrcFee);
    });

    test('中枢省主账户 / 费用账户', () {
      expect(hexFromAccountId(deriveInstitutionMainAccountId(prcCid)), prcMain);
      expect(hexFromAccountId(deriveInstitutionFeeAccountId(prcCid)), prcFee);
    });

    test('名字路由:主账户/费用账户与显式派生一致', () {
      expect(
        hexFromAccountId(
          deriveInstitutionAccountIdByName(nrcCid, kReservedNameMain),
        ),
        nrcMain,
      );
      expect(
        hexFromAccountId(
          deriveInstitutionAccountIdByName(nrcCid, kReservedNameFee),
        ),
        nrcFee,
      );
    });

    test('名字路由:安全基金 OP_AN / 两和基金 OP_HE', () {
      expect(
        hexFromAccountId(
          deriveInstitutionAccountIdByName(nrcCid, kReservedNameAnquan),
        ),
        nrcSafety,
      );
      expect(
        hexFromAccountId(
          deriveInstitutionAccountIdByName(nrcCid, kReservedNameHe),
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
          ...utf8.encode('GMB'),
          0x06,
          2027 & 0xFF,
          (2027 >> 8) & 0xFF,
          ...utf8.encode(nrcCid),
          ...utf8.encode(name),
        ]),
      );
      expect(
        hexFromAccountId(deriveInstitutionCustomAccountId(nrcCid, name)),
        hexFromAccountId(expected),
      );
    });

    test('自定义账户不同于主账户', () {
      expect(
        hexFromAccountId(deriveInstitutionCustomAccountId(nrcCid, '专户')),
        isNot(hexFromAccountId(deriveInstitutionMainAccountId(nrcCid))),
      );
    });

    test('空账户名抛错(对齐链端 EmptyAccountName)', () {
      expect(
        () => deriveInstitutionCustomAccountId(nrcCid, ''),
        throwsArgumentError,
      );
    });

    test('自定义名命中受限保留名抛错', () {
      expect(
        () => deriveInstitutionCustomAccountId(nrcCid, kReservedNameMain),
        throwsArgumentError,
      );
    });
  });

  group('个人多签派生(0x05)归位后行为不变', () {
    test('SS58 输出与 core 派生一致', () {
      final creator = Uint8List.fromList(List<int>.generate(32, (i) => i));
      final viaCore = ss58FromAccountId(
        deriveAccountId(
          opTag: kOpPersonal,
          payload: <int>[...creator, ...utf8.encode('个人钱包')],
        ),
      );
      expect(
        derivePersonalAccountSs58(
          creatorPubkey: creator,
          accountName: '个人钱包',
        ),
        viaCore,
      );
    });

    test('creator 非 32 字节抛错', () {
      expect(
        () => derivePersonalAccountSs58(
          creatorPubkey: Uint8List(31),
          accountName: 'x',
        ),
        throwsArgumentError,
      );
    });
  });
}
