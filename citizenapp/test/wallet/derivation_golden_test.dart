// HD 派生金标（model B,冷热共享单源,与 citizenwallet 逐字节一致）。
//
// 契约：一套助记词 → 一个 mini-secret 种子 → 全部 `//index` 硬派生（含账户0 = `//0`,
//   无 bare 根）→ 每账户一对公私钥、一个 ss58(2027)、一把自己的 child mini-secret(32B)。
//
// 不变量（本测试钉死）：
//   1) junction 硬派生标准正确 —— //Alice 对齐 substrate 权威 Alice AccountId;
//   2) model B 核心 —— fromSeed(childMiniSecret) 逐字节 == <助记词>//index;
//   3) //0 //1 //2 的 accountId / ss58 / childMiniSecret 逐字节钉死（== citizenwallet 冷端金标）。
//
// citizenapp 无根热端与 citizenwallet 冷端**派生完全一致**,仅存储不同(此端只存 child、不存种子)。
import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:sr25519/sr25519.dart' as sr;
import 'package:substrate_bip39/substrate_bip39.dart';

/// 固定测试助记词(substrate dev 助记词,全网公开,仅测试用)。
const String kDevPhrase =
    'bottom drive obey lake curtain smoke basket hold race lonely fit walk';

/// 本链 SS58 前缀。
const int kSs58 = 2027;

/// substrate 权威向量:dev 助记词 //Alice 的 sr25519 AccountId。
const String kAlicePub =
    'd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';

/// 金标:dev 助记词下 //0 //1 //2 的 (accountId, ss58, childMiniSecret)。
const List<(String, String, String)> kGoldenAccounts = [
  (
    '0x2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972',
    'w5CZACAABUbK4jspzPB5be9trhtSgRCRZFafGe7kvFPvxq8M2',
    '0x914dded06277afbe5b0e8a30bce539ec8a9552a784d08e530dc7c2915c478393',
  ),
  (
    '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48',
    'w5FhUDLW4BxsE1QXK4sNjPZ8rqSnK2QeVpUfXzqczpWdxChxV',
    '0x4433c3ada0cf37c3050d5435321872f4f84ef53d8b5f1f1560689d500b882245',
  ),
  (
    '0x46f136b564e1fad55031404dd84e5cd3fa76bfe7cc7599b39d38fd06663bbc0a',
    'w5DBpRvbgkersZohanGQiXa4qQLS1n7VQaSFwBaq4irJmgDn5',
    '0x5418179cea7224f2d9d2ab437773c2fdb266e52ef7fa52c0d9c15c6ca6068748',
  ),
];

String _hex(List<int> b) =>
    b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

Future<Uint8List> _miniSecret(String mnemonic) async {
  final entropy =
      bip39m.Mnemonic.fromSentence(mnemonic, bip39m.Language.english).entropy;
  return Uint8List.fromList(await CryptoScheme.miniSecretFromEntropy(entropy));
}

/// 复现 WalletManager 的 child mini-secret 提取(金标据此校验)。
List<int> _childMiniSecret(List<int> seed, int index) {
  final junctions = SecretUri.fromStr('//$index').junctions;
  var rootSk = sr.MiniSecretKey.fromRawKey(seed).expandEd25519();
  late List<int> child;
  for (final j in junctions) {
    final cc = j.junctionId.sublist(0, 32);
    final derived = rootSk.hardDeriveMiniSecretKey(const <int>[], cc);
    child = derived.$1.encode();
    rootSk = derived.$1.expandEd25519();
  }
  return child;
}

void main() {
  test('junction 硬派生对齐 substrate 权威 Alice', () async {
    final alice = await Keyring.sr25519.fromUri('$kDevPhrase//Alice');
    alice.ss58Format = kSs58;
    expect(_hex(alice.bytes().toList(growable: false)), kAlicePub);
  });

  test('model B 核心:fromSeed(childMiniSecret) == <助记词>//index', () async {
    final ms = await _miniSecret(kDevPhrase);
    for (var index = 0; index < 3; index++) {
      final child = _childMiniSecret(ms, index);
      final recon = Keyring.sr25519.fromSeed(Uint8List.fromList(child))
        ..ss58Format = kSs58;
      final ref = await Keyring.sr25519.fromUri('$kDevPhrase//$index');
      ref.ss58Format = kSs58;
      expect(
        _hex(recon.bytes().toList(growable: false)),
        _hex(ref.bytes().toList(growable: false)),
        reason: '//$index 公钥不一致',
      );
      expect(recon.address, ref.address, reason: '//$index ss58 不一致');
    }
  });

  test('//0 //1 //2 金标逐字节钉死(accountId / ss58 / childMiniSecret)', () async {
    final ms = await _miniSecret(kDevPhrase);
    for (var index = 0; index < kGoldenAccounts.length; index++) {
      final child = _childMiniSecret(ms, index);
      final kp = Keyring.sr25519.fromSeed(Uint8List.fromList(child))
        ..ss58Format = kSs58;
      final (expectedId, expectedSs58, expectedChild) = kGoldenAccounts[index];
      expect(
        '0x${_hex(kp.bytes().toList(growable: false))}',
        expectedId,
        reason: '//$index accountId 漂移',
      );
      expect(kp.address, expectedSs58, reason: '//$index ss58 漂移');
      expect('0x${_hex(child)}', expectedChild,
          reason: '//$index childMiniSecret 漂移');
    }
  });
}
