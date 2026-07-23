import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:citizenapp/my/myid/voting_identity_payload.dart';

/// 按 onchina `build_voting_identity_payload` 的字节布局构造夹具:
/// compact(len)+cid || publicKey(32) || age(1) || valid_from(u32le) ||
/// valid_until(u32le) || status(1) || compact+province || compact+city ||
/// compact+town。长度均 < 64,compact 恒为单字节 len<<2。
Uint8List buildPayload({
  String cidNumber = 'BJ110198512345678',
  int? publicKeyByte,
  int age = 22,
  int validFrom = 20260101,
  int validUntil = 20360101,
  int status = 0,
  String province = '11',
  String city = '01',
  String town = '001',
}) {
  final out = <int>[];
  void pushVec(String text) {
    final bytes = utf8.encode(text);
    out.add(bytes.length << 2);
    out.addAll(bytes);
  }

  void pushU32Le(int value) {
    out.addAll([
      value & 0xff,
      (value >> 8) & 0xff,
      (value >> 16) & 0xff,
      (value >> 24) & 0xff,
    ]);
  }

  pushVec(cidNumber);
  out.addAll(List.filled(32, publicKeyByte ?? 0xaa));
  out.add(age);
  pushU32Le(validFrom);
  pushU32Le(validUntil);
  out.add(status);
  pushVec(province);
  pushVec(city);
  pushVec(town);
  return Uint8List.fromList(out);
}

Uint8List buildCandidatePayload() {
  final out = <int>[...buildPayload()];
  void pushVec(String text) {
    final bytes = utf8.encode(text);
    out.add(bytes.length << 2);
    out.addAll(bytes);
  }

  pushVec('11');
  pushVec('01');
  pushVec('002');
  pushVec('测');
  pushVec('试公民');
  out.add(1);
  // birth_date: u32 YYYYMMDD(LE),20000131。
  const birthDate = 20000131;
  out.addAll([
    birthDate & 0xff,
    (birthDate >> 8) & 0xff,
    (birthDate >> 16) & 0xff,
    (birthDate >> 24) & 0xff,
  ]);
  return Uint8List.fromList(out);
}

void main() {
  group('VotingIdentityConsentPayload.decode', () {
    test('解码完整载荷并生成中文确认条目', () {
      final decoded = VotingIdentityConsentPayload.decode(buildPayload());

      expect(decoded, isNotNull);
      expect(decoded!.identityLevel, CitizenIdentityConsentLevel.voting);
      expect(decoded.cidNumber, 'BJ110198512345678');
      expect(decoded.accountId, '0x${'aa' * 32}');
      expect(
        decoded.ss58Address,
        Keyring().encodeAddress(List.filled(32, 0xaa), 2027),
      );
      expect(decoded.ageYears, 22);
      expect(decoded.statusNormal, isTrue);
      expect(decoded.provinceCode, '11');
      expect(decoded.cityCode, '01');
      expect(decoded.townCode, '001');

      final entries = Map.fromEntries(
        decoded.reviewEntries.map((e) => MapEntry(e.$1, e.$2)),
      );
      expect(entries['周岁年龄'], '22周岁');
      expect(entries['护照有效期'], '2026-01-01 至 2036-01-01');
      expect(entries['身份状态'], '正常');
      expect(entries['居住地'], '11 / 01 / 001');
      // 中文标签必须全量存在,禁止英文 key 直出。
      expect(entries.keys, containsAll(['CID编号', '公民钱包账户']));
    });

    test('解码参选身份载荷并展示额外公开档案字段', () {
      final decoded = VotingIdentityConsentPayload.decode(
        buildCandidatePayload(),
      );

      expect(decoded, isNotNull);
      expect(decoded!.identityLevel, CitizenIdentityConsentLevel.candidate);
      expect(decoded.isCandidate, isTrue);
      expect(decoded.birthProvinceCode, '11');
      expect(decoded.birthCityCode, '01');
      expect(decoded.birthTownCode, '002');
      expect(decoded.familyName, '测');
      expect(decoded.givenName, '试公民');
      expect(decoded.citizenSexLabel, '女');
      expect(decoded.birthDate, 20000131);

      final entries = Map.fromEntries(
        decoded.reviewEntries.map((e) => MapEntry(e.$1, e.$2)),
      );
      expect(entries['身份类型'], '参选身份');
      expect(entries['出生地'], '11 / 01 / 002');
      expect(entries['出生日期'], '2000-01-31');
      expect(entries['公民姓名'], '测试公民');
      expect(entries['公民性别'], '女');
    });

    test('注销状态展示为注销', () {
      final decoded = VotingIdentityConsentPayload.decode(
        buildPayload(status: 1),
      );
      expect(decoded, isNotNull);
      expect(decoded!.statusNormal, isFalse);
    });

    test('拒绝未满16周岁', () {
      expect(
        VotingIdentityConsentPayload.decode(buildPayload(age: 15)),
        isNull,
      );
    });

    test('拒绝未知状态值', () {
      expect(
        VotingIdentityConsentPayload.decode(buildPayload(status: 2)),
        isNull,
      );
    });

    test('拒绝有效期倒挂', () {
      expect(
        VotingIdentityConsentPayload.decode(
          buildPayload(validFrom: 20360101, validUntil: 20260101),
        ),
        isNull,
      );
    });

    test('拒绝非法日期', () {
      expect(
        VotingIdentityConsentPayload.decode(buildPayload(validFrom: 20261301)),
        isNull,
      );
    });

    test('拒绝截断载荷', () {
      final full = buildPayload();
      final truncated = Uint8List.sublistView(full, 0, full.length - 1);
      expect(VotingIdentityConsentPayload.decode(truncated), isNull);
    });

    test('拒绝尾部残留字节', () {
      final withTrailing = Uint8List.fromList([...buildPayload(), 0x00]);
      expect(VotingIdentityConsentPayload.decode(withTrailing), isNull);
    });

    test('拒绝空 CID', () {
      final bytes = buildPayload(cidNumber: '');
      expect(VotingIdentityConsentPayload.decode(bytes), isNull);
    });
  });
}
