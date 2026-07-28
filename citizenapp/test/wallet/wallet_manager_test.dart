import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:sr25519/sr25519.dart' as sr;
import 'package:substrate_bip39/substrate_bip39.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';
import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import '../support/fake_secure_seed_store.dart';
import '../support/isar_test_env.dart';

const _mnemonicA =
    'legal winner thank year wave sausage worth useful legal winner thank yellow';
// 另一条合法但派生不同公钥的助记词（回归：曾用于自愈"助记词不一致"分支）。
const _mnemonicB =
    'abandon abandon abandon abandon abandon abandon abandon abandon '
    'abandon abandon abandon about';

String _coldSs58(int byte) =>
    Keyring().encodeAddress(List<int>.filled(32, byte), 2027);

String _hex(List<int> b) =>
    b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

/// 助记词 → 母种子（master mini-secret，32B）。
Future<Uint8List> _masterSeed(String mnemonic) async {
  final entropy = Mnemonic.fromSentence(mnemonic, Language.english).entropy;
  return Uint8List.fromList(await CryptoScheme.miniSecretFromEntropy(entropy));
}

/// 复现 WalletManager 的账户 child mini-secret 派生（金标同源）。
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

/// 安全不变量断言：存的必须是账户0(`//0`) 的 child，绝不是母种子。
Future<void> _expectChildStoredNotSeed(
  String mnemonic,
  String storedHex,
) async {
  final master = await _masterSeed(mnemonic);
  final child = _childMiniSecret(master, 0);
  expect(storedHex, _hex(child), reason: '严档存的必须是账户0 //0 的 child');
  expect(storedHex, isNot(_hex(master)), reason: '绝不持久化母种子');
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

  // 动钱动权验证已上移到 WalletManager 的硬件金库读 child；单测里把 local_auth
  // channel 打桩为「验证通过」，让纯 Dart 环境不因缺插件而抛。
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
          return <String>['fingerprint', 'face'];
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

  group('WalletManager — 热钱包创建/导入/删除（ROOTLESS）', () {
    test('create/import/delete 只存账户0 child，不存种子/助记词', () async {
      final manager = WalletManager();

      final created = await manager.createWallet();
      expect(created.profile.walletIndex, 1);
      expect(created.profile.alg, 'sr25519');
      expect(created.profile.ss58, 2027);
      expect(created.profile.signMode, 'local');
      expect(created.mnemonic.trim().split(RegExp(r'\s+')).length, 12);
      // 严档只落账户0 child（64 hex）；无母种子、无助记词档。
      final createdKey = fakeStore.accountKeys[created.profile.accountId];
      expect(createdKey, isNotNull);
      expect(createdKey!.length, 64);
      await _expectChildStoredNotSeed(created.mnemonic, createdKey);
      // 助记词绝不出现在任何持久化条目里（只一次性返回供备份）。
      expect(fakeStore.accountKeys.values, isNot(contains(created.mnemonic)));
      expect(fakeStore.accountKeys.length, 1);
      // 账户0 作为锚点账户同步落库,出现在 getAccounts。
      final createdAccounts =
          await manager.getAccounts(created.profile.accountId);
      expect(createdAccounts.map((a) => a.accountIndex).toList(), [0]);

      // 一台设备仅一个热钱包:删掉当前热钱包后才能导入下一只。
      await manager.deleteWallet(created.profile.walletIndex);
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);

      final imported = await manager.importWallet(_mnemonicA);
      expect(imported.walletIndex, 1);
      expect(imported.signMode, 'local');
      final importedKey = fakeStore.accountKeys[imported.accountId];
      expect(importedKey, isNotNull);
      await _expectChildStoredNotSeed(_mnemonicA, importedKey!);

      await manager.deleteWallet(imported.walletIndex);
      expect(fakeStore.accountKeys.containsKey(imported.accountId), isFalse);
      expect(await manager.getWallet(), isNull);
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);
    });

    test('importWallet 拒绝非法助记词', () async {
      final manager = WalletManager();
      expect(
        () => manager.importWallet('hello world'),
        throwsA(
          isA<Exception>().having(
            (e) => e.toString(),
            'message',
            contains('助记词无效'),
          ),
        ),
      );
    });

    test('getDefaultWallet 忽略冷钱包（WalletGate 门禁判定依据）', () async {
      final manager = WalletManager();

      expect(await manager.getDefaultWallet(), isNull);

      await manager.importColdWallet(
        ss58Address: _coldSs58(0x22),
      );
      expect(await manager.getDefaultWallet(), isNull);

      final imported = await manager.importWallet(_mnemonicA);
      final def = await manager.getDefaultWallet();
      expect(def, isNotNull);
      expect(def!.walletIndex, imported.walletIndex);
      expect(def.isHotWallet, isTrue);
    });

    test('D3：无锁屏设备拒绝创建热钱包（fail-closed）', () async {
      fakeStore.noDeviceLock = true;
      final manager = WalletManager();
      await expectLater(
        manager.createWallet(),
        throwsA(isA<WalletAuthException>()),
      );
      // 未落库、未写密钥。
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);
    });
  });

  group('门禁0 fail-closed：设备子钥注册强绑定', () {
    tearDown(() => WalletManager.subkeyRegistrar = null);

    Future<void> failingRegistrar({
      required int walletIndex,
      required String accountId,
      required Future<String> Function(Uint8List bindingMessage) signBinding,
    }) async {
      throw Exception('设备子钥注册失败：网络不可用');
    }

    test('createWallet 注册失败 → 整笔回滚，无残留', () async {
      WalletManager.subkeyRegistrar = failingRegistrar;
      final manager = WalletManager();
      await expectLater(manager.createWallet(), throwsA(isA<Exception>()));
      // 钱包未落库、child 无残留 → WalletGate 维持 needsWallet。
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);
    });

    test('importWallet 注册失败 → 整笔回滚，无残留', () async {
      WalletManager.subkeyRegistrar = failingRegistrar;
      final manager = WalletManager();
      await expectLater(
        manager.importWallet(_mnemonicA),
        throwsA(isA<Exception>()),
      );
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.accountKeys, isEmpty);
    });

    test('createWallet 注册成功 → 落库并用账户0 对绑定证明签名', () async {
      String? seenAccountId;
      WalletManager.subkeyRegistrar = ({
        required int walletIndex,
        required String accountId,
        required Future<String> Function(Uint8List bindingMessage) signBinding,
      }) async {
        seenAccountId = accountId;
        final signature = await signBinding(Uint8List(32));
        expect(signature.startsWith('0x'), isTrue);
      };
      final manager = WalletManager();
      final created = await manager.createWallet();
      expect(seenAccountId, created.profile.accountId);
      expect((await manager.getWallets()).length, 1);
      expect(fakeStore.accountKeys[created.profile.accountId], isNotNull);
    });
  });

  group('WalletManager — 统一签名', () {
    final payload = Uint8List.fromList(List<int>.generate(32, (i) => i));

    test('统一签名：每次都读一次 child（无会话缓存）', () async {
      final manager = WalletManager();
      await manager.importWallet(_mnemonicA);
      fakeStore.readCount = 0;

      final sig = await manager.signWithWallet(1, payload);
      await manager.signWithWallet(1, payload);

      expect(sig.length, 64);
      // 两次签名 = 两次读 child（两次验证），不复用、无会话密钥。
      expect(fakeStore.readCount, 2);
    });

    test('verifyWalletAccess 读一次 child 触发一次验证（切换身份用）', () async {
      final manager = WalletManager();
      await manager.importWallet(_mnemonicA);
      fakeStore.readCount = 0;

      await manager.verifyWalletAccess(1);

      expect(fakeStore.readCount, 1);
    });

    test('AuthCancelled 上抛，绝不吞没', () async {
      final manager = WalletManager();
      final imported = await manager.importWallet(_mnemonicA);
      fakeStore.putCount = 0;
      fakeStore.cancelReads.add(imported.accountId);

      await expectLater(
        manager.signWithWallet(1, payload),
        throwsA(isA<AuthCancelled>()),
      );
      // 无根 = 无自愈重写。
      expect(fakeStore.putCount, 0);
    });
  });

  group('WalletManager — 密钥失效 fail-closed（无根 = 无自愈）', () {
    final payload = Uint8List.fromList(List<int>.generate(32, (_) => 7));

    test('KEK 失效 → fail-closed 抛需重新导入，绝不重写 child', () async {
      final manager = WalletManager();
      final imported = await manager.importWallet(_mnemonicA);
      fakeStore.putCount = 0;
      fakeStore.invalidatedAccountIds.add(imported.accountId);

      await expectLater(
        manager.signWithWallet(1, payload),
        throwsA(
          isA<WalletAuthException>()
              .having((e) => e.message, 'message', contains('重新导入')),
        ),
      );
      // 无母种子 / 助记词可自愈，绝不重派生重写。
      expect(fakeStore.putCount, 0);
    });

    test('child 条目缺失 → fail-closed 抛需重新导入', () async {
      final manager = WalletManager();
      final imported = await manager.importWallet(_mnemonicA);
      fakeStore.accountKeys.remove(imported.accountId);

      await expectLater(
        manager.signWithWallet(1, payload),
        throwsA(
          isA<WalletAuthException>()
              .having((e) => e.message, 'message', contains('重新导入')),
        ),
      );
    });

    test('回归：曾派生不同公钥的助记词，如今无自愈路径读取它', () async {
      // _mnemonicB 仅作历史回归标记：无自愈后，签名只认严档 child，助记词不再入库。
      final manager = WalletManager();
      final imported = await manager.importWallet(_mnemonicB);
      final key = fakeStore.accountKeys[imported.accountId];
      expect(key, isNotNull);
      await _expectChildStoredNotSeed(_mnemonicB, key!);
    });
  });

  group('WalletManager — 冷钱包', () {
    test('importColdWallet 只存公开账户资料，child 金库无条目', () async {
      final manager = WalletManager();
      final cold = await manager.importColdWallet(ss58Address: _coldSs58(0x11));
      expect(cold.signMode, 'external');
      expect(fakeStore.accountKeys.containsKey(cold.accountId), isFalse);
    });

    test('deleteWallet 冷钱包不影响 child 金库', () async {
      final manager = WalletManager();
      final cold = await manager.importColdWallet(ss58Address: _coldSs58(0x11));
      await manager.deleteWallet(cold.walletIndex);
      final wallets = await manager.getWallets();
      expect(wallets.where((w) => w.walletIndex == cold.walletIndex), isEmpty);
    });
  });
}
