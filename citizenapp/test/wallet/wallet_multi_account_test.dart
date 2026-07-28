import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:sr25519/sr25519.dart' as sr;
import 'package:substrate_bip39/substrate_bip39.dart';
import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import '../support/fake_secure_seed_store.dart';
import '../support/isar_test_env.dart';

const _mnemonicA =
    'legal winner thank year wave sausage worth useful legal winner thank yellow';
// 另一条合法但派生不同账户0公钥的助记词(归属校验用)。
const _mnemonicB =
    'abandon abandon abandon abandon abandon abandon abandon abandon '
    'abandon abandon abandon about';

String _hex(List<int> b) =>
    b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

Future<Uint8List> _masterSeed(String mnemonic) async {
  final entropy = Mnemonic.fromSentence(mnemonic, Language.english).entropy;
  return Uint8List.fromList(await CryptoScheme.miniSecretFromEntropy(entropy));
}

/// 复现 WalletManager 的账户 child mini-secret 派生(金标同源)。
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

String _accountIdOf(List<int> child) {
  final pair = Keyring.sr25519.fromSeed(Uint8List.fromList(child))
    ..ss58Format = 2027;
  return '0x${_hex(pair.bytes().toList(growable: false))}';
}

class _MemoryBlobStore implements VaultBlobStore {
  final Map<String, String> values = <String, String>{};
  @override
  Future<String?> read(String key) async => values[key];
  @override
  Future<void> write(String key, String value) async => values[key] = value;
  @override
  Future<void> delete(String key) async => values.remove(key);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  useIsolatedIsar();

  late FakeSecureSeedStore fakeStore;
  const localAuthChannel = MethodChannel('plugins.flutter.io/local_auth');

  setUp(() async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    fakeStore = FakeSecureSeedStore();
    WalletManager.debugSeedStore = fakeStore;
    WalletManager.debugContactKeyStore = _MemoryBlobStore();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(localAuthChannel, (call) async {
      switch (call.method) {
        case 'authenticate':
          return true;
        case 'getAvailableBiometrics':
          return <String>['fingerprint'];
        case 'isDeviceSupported':
        case 'deviceSupportsBiometrics':
        case 'canCheckBiometrics':
          return true;
        default:
          return null;
      }
    });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(localAuthChannel, null);
  });

  // 建一只热钱包(账户0),返回 masterId(= 账户0.accountId)。
  Future<String> seedWallet() async =>
      (await WalletManager().importWallet(_mnemonicA)).accountId;

  group('WalletManager 多账户(ROOTLESS 批量指定序号)', () {
    test('addAccounts 连续 [1,2,3]:建账户 + 各存自己的 //index child', () async {
      final masterId = await seedWallet();
      final manager = WalletManager();
      final master = await _masterSeed(_mnemonicA);

      final added = await manager.addAccounts(masterId, _mnemonicA, [1, 2, 3]);
      expect(added.map((a) => a.accountIndex).toList(), [1, 2, 3]);
      expect(
        (await manager.getAccounts(masterId)).map((a) => a.accountIndex).toList(),
        [0, 1, 2, 3],
      );
      for (final index in [1, 2, 3]) {
        final child = _childMiniSecret(master, index);
        expect(fakeStore.accountKeys[_accountIdOf(child)], _hex(child),
            reason: '账户//$index 存的必须是其 child');
      }
      // 账户0 + 3 个 = 4 把 child。
      expect(fakeStore.accountKeys.length, 4);
    });

    test('addAccounts 断续 [1,5,9] 同样成立', () async {
      final masterId = await seedWallet();
      final manager = WalletManager();
      final added = await manager.addAccounts(masterId, _mnemonicA, [1, 5, 9]);
      expect(added.map((a) => a.accountIndex).toList(), [1, 5, 9]);
      expect(
        (await manager.getAccounts(masterId)).map((a) => a.accountIndex).toList(),
        [0, 1, 5, 9],
      );
    });

    test('输入去重 [1,1,2] 拒', () async {
      final masterId = await seedWallet();
      await expectLater(
        WalletManager().addAccounts(masterId, _mnemonicA, [1, 1, 2]),
        throwsA(isA<Exception>()),
      );
    });

    test('越界 [0] / [1990] 拒', () async {
      final masterId = await seedWallet();
      await expectLater(WalletManager().addAccounts(masterId, _mnemonicA, [0]),
          throwsA(isA<Exception>()));
      await expectLater(
          WalletManager().addAccounts(masterId, _mnemonicA, [1990]),
          throwsA(isA<Exception>()));
    });

    test('已存在序号拒', () async {
      final masterId = await seedWallet();
      final manager = WalletManager();
      await manager.addAccounts(masterId, _mnemonicA, [1]);
      await expectLater(manager.addAccounts(masterId, _mnemonicA, [1]),
          throwsA(isA<Exception>()));
    });

    test('助记词归属不符拒且不留残账户', () async {
      final masterId = await seedWallet();
      final before = fakeStore.accountKeys.length;
      await expectLater(WalletManager().addAccounts(masterId, _mnemonicB, [1]),
          throwsA(isA<WalletAuthException>()));
      expect(fakeStore.accountKeys.length, before, reason: '归属校验先于写入,无残留');
    });

    test('addNextAccount 取下一序号(既有最大+1)', () async {
      final masterId = await seedWallet();
      final manager = WalletManager();
      await manager.addAccounts(masterId, _mnemonicA, [1, 2]);
      expect((await manager.addNextAccount(masterId, _mnemonicA)).accountIndex, 3);
    });

    test('getAccountPrivateKey 返回该账户 child', () async {
      final masterId = await seedWallet();
      final manager = WalletManager();
      final added = await manager.addAccounts(masterId, _mnemonicA, [1]);
      final expected = _childMiniSecret(await _masterSeed(_mnemonicA), 1);
      expect(await manager.getAccountPrivateKey(added.single.accountId),
          '0x${_hex(expected)}');
    });

    test('signForAccountId //index 签名可被该账户公钥验证', () async {
      final masterId = await seedWallet();
      final manager = WalletManager();
      final added = await manager.addAccounts(masterId, _mnemonicA, [1]);
      final child = _childMiniSecret(await _masterSeed(_mnemonicA), 1);
      final pair = Keyring.sr25519.fromSeed(Uint8List.fromList(child))
        ..ss58Format = 2027;
      final msg = Uint8List.fromList([1, 2, 3, 4]);
      final sig = await manager.signForAccountId(added.single.accountId, msg);
      expect(pair.verify(msg, sig), isTrue);
    });

    test('deleteAccount 锚点守卫:账户0 有兄弟时拒', () async {
      final masterId = await seedWallet();
      final manager = WalletManager();
      await manager.addAccounts(masterId, _mnemonicA, [1]);
      // masterId == 账户0.accountId
      await expectLater(
          manager.deleteAccount(masterId), throwsA(isA<Exception>()));
    });

    test('deleteAccount 叶子可删;删至空级联删整钱包', () async {
      final masterId = await seedWallet();
      final manager = WalletManager();
      final added = await manager.addAccounts(masterId, _mnemonicA, [1]);
      await manager.deleteAccount(added.single.accountId);
      expect(
        (await manager.getAccounts(masterId)).map((a) => a.accountIndex).toList(),
        [0],
      );
      // 账户0 此时无兄弟 → 删它级联删整钱包。
      await manager.deleteAccount(masterId);
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);
    });
  });
}
