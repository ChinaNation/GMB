import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';
import 'package:citizenapp/wallet/core/hardware_bound_seed_vault.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import '../support/fake_secure_seed_store.dart';
import '../support/isar_test_env.dart';

const _mnemonicA =
    'legal winner thank year wave sausage worth useful legal winner thank yellow';
// 另一条合法但派生不同公钥的助记词，用于自愈"助记词不一致"分支。
const _mnemonicB =
    'abandon abandon abandon abandon abandon abandon abandon abandon '
    'abandon abandon abandon about';

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

  // 动钱动权验证已上移到 WalletManager 的 local_auth；单测里把该 channel
  // 打桩为「验证通过」，让 signWithWallet/verifyWalletAccess 走到 seed 读取与
  // 自愈分支（否则纯 Dart 环境无插件实现，authenticate 抛 MissingPluginException）。
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

  group('WalletManager — 热钱包创建/导入/删除', () {
    test('create/import/delete 保持 profile 与安全存储同步', () async {
      final manager = WalletManager();

      final created = await manager.createWallet();
      expect(created.profile.walletIndex, 1);
      expect(created.profile.alg, 'sr25519');
      expect(created.profile.ss58, 2027);
      expect(created.profile.signMode, 'local');
      expect(created.mnemonic.trim().split(RegExp(r'\s+')).length, 12);
      // seed（64 hex）与助记词分别落入严档 / 宽档金库。
      expect(fakeStore.seeds[1], isNotNull);
      expect(fakeStore.seeds[1]!.length, 64);
      expect(fakeStore.mnemonics[1], created.mnemonic);

      final imported = await manager.importWallet(_mnemonicA);
      expect(imported.walletIndex, 2);
      expect(imported.signMode, 'local');
      expect(fakeStore.seeds[2], isNotNull);
      expect(fakeStore.mnemonics[2], _mnemonicA);

      await manager.deleteWallet(2);
      expect(fakeStore.seeds.containsKey(2), isFalse);
      expect(fakeStore.mnemonics.containsKey(2), isFalse);

      await manager.deleteWallet(1);
      expect(await manager.getWallet(), isNull);
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.seeds, isEmpty);
      expect(fakeStore.mnemonics, isEmpty);
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
        address:
            '0x2222222222222222222222222222222222222222222222222222222222222222',
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
      expect(fakeStore.seeds, isEmpty);
    });
  });

  group('门禁0 fail-closed：设备子钥注册强绑定', () {
    tearDown(() => WalletManager.subkeyRegistrar = null);

    Future<void> failingRegistrar({
      required int walletIndex,
      required String ownerAccount,
      required Future<String> Function(Uint8List bindingMessage) signBinding,
    }) async {
      throw Exception('设备子钥注册失败：网络不可用');
    }

    test('createWallet 注册失败 → 整笔回滚，无残留', () async {
      WalletManager.subkeyRegistrar = failingRegistrar;
      final manager = WalletManager();
      await expectLater(manager.createWallet(), throwsA(isA<Exception>()));
      // 钱包未落库、seed/助记词无残留 → WalletGate 维持 needsWallet。
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.seeds, isEmpty);
      expect(fakeStore.mnemonics, isEmpty);
    });

    test('importWallet 注册失败 → 整笔回滚，无残留', () async {
      WalletManager.subkeyRegistrar = failingRegistrar;
      final manager = WalletManager();
      await expectLater(
        manager.importWallet(_mnemonicA),
        throwsA(isA<Exception>()),
      );
      expect(await manager.getWallets(), isEmpty);
      expect(fakeStore.seeds, isEmpty);
      expect(fakeStore.mnemonics, isEmpty);
    });

    test('createWallet 注册成功 → 落库并用主钥对绑定证明签名', () async {
      String? seenOwner;
      WalletManager.subkeyRegistrar = ({
        required int walletIndex,
        required String ownerAccount,
        required Future<String> Function(Uint8List bindingMessage) signBinding,
      }) async {
        seenOwner = ownerAccount;
        final signature = await signBinding(Uint8List(32));
        expect(signature.startsWith('0x'), isTrue);
      };
      final manager = WalletManager();
      final created = await manager.createWallet();
      expect(seenOwner, created.profile.address);
      expect((await manager.getWallets()).length, 1);
      expect(fakeStore.seeds[created.profile.walletIndex], isNotNull);
    });
  });

  group('WalletManager — 统一签名', () {
    final payload = Uint8List.fromList(List<int>.generate(32, (i) => i));

    test('统一签名：每次都读一次 seed（无会话缓存）', () async {
      final manager = WalletManager();
      await manager.importWallet(_mnemonicA);
      fakeStore.readSeedCount = 0;

      final sig = await manager.signWithWallet(1, payload);
      await manager.signWithWallet(1, payload);

      expect(sig.length, 64);
      // 两次签名 = 两次读 seed（两次验证），不复用、无会话密钥。
      expect(fakeStore.readSeedCount, 2);
    });

    test('verifyWalletAccess 读一次 seed 触发一次验证（切换身份用）', () async {
      final manager = WalletManager();
      await manager.importWallet(_mnemonicA);
      fakeStore.readSeedCount = 0;

      await manager.verifyWalletAccess(1);

      expect(fakeStore.readSeedCount, 1);
    });

    test('AuthCancelled 上抛，绝不自愈重写 seed', () async {
      final manager = WalletManager();
      await manager.importWallet(_mnemonicA);
      fakeStore.putSeedCount = 0;
      fakeStore.cancelSeedReads.add(1);

      await expectLater(
        manager.signWithWallet(1, payload),
        throwsA(isA<AuthCancelled>()),
      );
      expect(fakeStore.putSeedCount, 0);
    });
  });

  group('WalletManager — seed 严档失效自愈', () {
    final payload = Uint8List.fromList(List<int>.generate(32, (_) => 7));

    test('KEK 失效 → 从助记词自愈重封装并签名成功', () async {
      final manager = WalletManager();
      await manager.importWallet(_mnemonicA);
      expect(fakeStore.putSeedCount, 1);
      final originalSeed = fakeStore.seeds[1];

      fakeStore.invalidatedSeeds.add(1);
      final sig = await manager.signWithWallet(1, payload);

      expect(sig.length, 64);
      expect(fakeStore.putSeedCount, 2); // 自愈重封装一次
      expect(fakeStore.seeds[1], originalSeed); // 重派生得到相同 seed
      expect(fakeStore.invalidatedSeeds.contains(1), isFalse);
    });

    test('助记词也缺失 → 抛需重新导入', () async {
      final manager = WalletManager();
      await manager.importWallet(_mnemonicA);
      fakeStore.invalidatedSeeds.add(1);
      fakeStore.mnemonics.remove(1);

      await expectLater(
        manager.signWithWallet(1, payload),
        throwsA(
          isA<WalletAuthException>()
              .having((e) => e.message, 'message', contains('重新导入')),
        ),
      );
    });

    test('助记词与钱包不一致 → 抛无法恢复，不写错误 seed', () async {
      final manager = WalletManager();
      await manager.importWallet(_mnemonicA);
      fakeStore.putSeedCount = 0;
      fakeStore.invalidatedSeeds.add(1);
      fakeStore.mnemonics[1] = _mnemonicB; // 派生不同公钥

      await expectLater(
        manager.signWithWallet(1, payload),
        throwsA(
          isA<WalletAuthException>()
              .having((e) => e.message, 'message', contains('不一致')),
        ),
      );
      // pubkey 校验在 deleteSeed/putSeed 之前，错误 seed 不落库。
      expect(fakeStore.putSeedCount, 0);
    });
  });

  group('WalletManager — 冷钱包', () {
    const coldPubkeyHex =
        '0x1111111111111111111111111111111111111111111111111111111111111111';

    test('importColdWallet 只存公钥，seed 金库无条目', () async {
      final manager = WalletManager();
      final cold = await manager.importColdWallet(address: coldPubkeyHex);
      expect(cold.signMode, 'external');
      expect(fakeStore.seeds.containsKey(cold.walletIndex), isFalse);
      expect(fakeStore.mnemonics.containsKey(cold.walletIndex), isFalse);
    });

    test('deleteWallet 冷钱包不影响 seed 金库', () async {
      final manager = WalletManager();
      final cold = await manager.importColdWallet(address: coldPubkeyHex);
      await manager.deleteWallet(cold.walletIndex);
      final wallets = await manager.getWallets();
      expect(wallets.where((w) => w.walletIndex == cold.walletIndex), isEmpty);
    });
  });
}
