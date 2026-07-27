// HD 派生金标（冷热共享单源，Step 2 citizenapp 必须逐字节复用）。
//
// 契约（ADR-022 修订后地基）：一套助记词 → 一个 mini-secret 种子 →
//   · 账户 0 = 根派生(bare) = Keyring.sr25519.fromSeed(miniSecret)，逐字节等于历史直出，
//     护住已上链的 bare 地址（如创世 9c3e…1068）；
//   · 账户 N≥1 = <助记词>//N 硬 junction 派生，等价于 seed-only 的 0x<miniSecret>//N；
//   · 每账户一对公私钥、一个 ss58(2027) 地址。
//
// 三条不变量由本测试钉死：
//   1) junction 硬派生标准正确 —— //Alice 对齐 substrate 权威 Alice AccountId；
//   2) base 一致 —— fromSeed(miniSecretFromEntropy) == fromUri(bare)，
//      即 seedFromEntropy == miniSecretFromEntropy，//N base 不漂；
//   3) seed-only 等价 —— 0x<miniSecret>//N == <助记词>//N（冷签不必解密助记词）。
//
// 向量基于固定 dev 助记词生成（2026-07-26 Phase 0 落定）。
import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:substrate_bip39/crypto_scheme.dart';

/// 固定测试助记词（substrate dev 助记词，全网公开，仅测试用）。
const String kDevPhrase =
    'bottom drive obey lake curtain smoke basket hold race lonely fit walk';

/// 本链 SS58 前缀。
const int kSs58 = 2027;

/// substrate 权威向量：dev 助记词 //Alice 的 sr25519 AccountId。
const String kAlicePub =
    'd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';

/// 金标：dev 助记词下 账户0(根)/账户1(//1)/账户2(//2) 的 (accountId, ss58)。
const List<(String, String)> kGoldenAccounts = [
  (
    '0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a',
    'w5DBnqoUytARopdnyWhmBq7ZPr74cJJewugoafJJynKLrirdE',
  ),
  (
    '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48',
    'w5FhUDLW4BxsE1QXK4sNjPZ8rqSnK2QeVpUfXzqczpWdxChxV',
  ),
  (
    '0x46f136b564e1fad55031404dd84e5cd3fa76bfe7cc7599b39d38fd06663bbc0a',
    'w5DBpRvbgkersZohanGQiXa4qQLS1n7VQaSFwBaq4irJmgDn5',
  ),
];

String _hex(List<int> b) =>
    b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

Future<Uint8List> _miniSecret(String mnemonic) async {
  final entropy =
      bip39m.Mnemonic.fromSentence(mnemonic, bip39m.Language.english).entropy;
  return Uint8List.fromList(await CryptoScheme.miniSecretFromEntropy(entropy));
}

Future<KeyPair> _account(int index, Uint8List miniSecret) async {
  final KeyPair kp = index == 0
      ? Keyring.sr25519.fromSeed(miniSecret)
      : await Keyring.sr25519.fromUri('$kDevPhrase//$index');
  kp.ss58Format = kSs58;
  return kp;
}

void main() {
  test('junction 硬派生对齐 substrate 权威 Alice', () async {
    final alice = await Keyring.sr25519.fromUri('$kDevPhrase//Alice');
    alice.ss58Format = kSs58;
    expect(_hex(alice.bytes().toList(growable: false)), kAlicePub);
  });

  test('账户0 base 一致：fromSeed(miniSecret) == fromUri(bare)', () async {
    final ms = await _miniSecret(kDevPhrase);
    final fromSeed = Keyring.sr25519.fromSeed(ms)..ss58Format = kSs58;
    final fromUriBare = await Keyring.sr25519.fromUri(kDevPhrase);
    fromUriBare.ss58Format = kSs58;
    expect(
      _hex(fromSeed.bytes().toList(growable: false)),
      _hex(fromUriBare.bytes().toList(growable: false)),
    );
  });

  test('seed-only 等价：0x<miniSecret>//N == <助记词>//N', () async {
    final ms = await _miniSecret(kDevPhrase);
    final msHex = _hex(ms);
    for (final n in [1, 2]) {
      final viaMnemonic = await Keyring.sr25519.fromUri('$kDevPhrase//$n');
      final viaSeed = await Keyring.sr25519.fromUri('0x$msHex//$n');
      expect(
        _hex(viaSeed.bytes().toList(growable: false)),
        _hex(viaMnemonic.bytes().toList(growable: false)),
        reason: '//$n seed-only 派生不一致',
      );
    }
  });

  test('账户 0/1/2 金标向量逐字节钉死', () async {
    final ms = await _miniSecret(kDevPhrase);
    for (var index = 0; index < kGoldenAccounts.length; index++) {
      final kp = await _account(index, ms);
      final (expectedAccountId, expectedSs58) = kGoldenAccounts[index];
      expect(
        '0x${_hex(kp.bytes().toList(growable: false))}',
        expectedAccountId,
        reason: '账户$index accountId 漂移',
      );
      expect(kp.address, expectedSs58, reason: '账户$index ss58 漂移');
    }
  });
}
