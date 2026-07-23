import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import '../support/isar_test_env.dart';

/// 验证 WalletManager.reorderWallets 是否正确写入 sortOrder,
/// 并检查 sortBySortOrder().thenByWalletIndex() 排序顺序。
void main() {
  useIsolatedIsar();

  TestWidgetsFlutterBinding.ensureInitialized();

  const secureStorageChannel =
      MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  const localAuthChannel = MethodChannel('plugins.flutter.io/local_auth');
  final secureStorage = <String, String>{};

  setUp(() async {
    secureStorage.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, (call) async {
      final args = (call.arguments as Map?)?.cast<String, dynamic>() ??
          <String, dynamic>{};
      final key = args['key']?.toString();
      switch (call.method) {
        case 'read':
          return key == null ? null : secureStorage[key];
        case 'write':
          if (key != null) {
            secureStorage[key] = args['value']?.toString() ?? '';
          }
          return null;
        case 'delete':
          if (key != null) {
            secureStorage.remove(key);
          }
          return null;
        case 'deleteAll':
          secureStorage.clear();
          return null;
        case 'containsKey':
          return key != null && secureStorage.containsKey(key);
        case 'readAll':
          return Map<String, String>.from(secureStorage);
        default:
          return null;
      }
    });
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(localAuthChannel, (call) async {
      switch (call.method) {
        case 'isDeviceSupported':
          return true;
        case 'deviceSupportsBiometrics':
          return false;
        case 'getAvailableBiometrics':
          return const <String>[];
        case 'authenticate':
          return true;
        default:
          return null;
      }
    });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(localAuthChannel, null);
  });

  /// 在 Isar 中直接构造 3 个钱包 entity,避开 createWallet 的生物认证 + 助记词派生,
  /// 让 reorder 测试聚焦在 sortOrder 字段本身。
  Future<void> seedThreeWallets() async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      for (var i = 1; i <= 3; i++) {
        final entity = WalletProfileEntity()
          ..walletIndex = i
          ..walletName = 'wallet_$i'
          ..walletIcon = 'wallet'
          ..balance = 0
          ..ss58Address = 'addr_$i'
          ..accountId = '0x${i.toRadixString(16).padLeft(64, '0')}'
          ..alg = 'sr25519'
          ..ss58 = 2027
          ..createdAtMillis = i
          ..source = 'test'
          ..signMode = 'local'
          ..sortOrder = 0;
        await isar.walletProfileEntitys.put(entity);
      }
    });
  }

  group('WalletManager.reorderWallets', () {
    test('reorderWallets 写入指定顺序后 getWallets 按新顺序返回', () async {
      await seedThreeWallets();
      final manager = WalletManager();

      // 把顺序改成 [3, 1, 2]
      await manager.reorderWallets([3, 1, 2]);
      final wallets = await manager.getWallets();
      expect(wallets.map((w) => w.walletIndex).toList(), [3, 1, 2]);
      expect(wallets[0].sortOrder, 0);
      expect(wallets[1].sortOrder, 1);
      expect(wallets[2].sortOrder, 2);
    });

    test('reorderWallets 自增 walletsRevision(切默认用户的全端广播信号)', () async {
      await seedThreeWallets();
      final manager = WalletManager();

      final before = WalletManager.walletsRevision.value;
      var notified = 0;
      void listener() => notified++;
      WalletManager.walletsRevision.addListener(listener);
      addTearDown(() => WalletManager.walletsRevision.removeListener(listener));

      await manager.reorderWallets([3, 1, 2]);

      expect(WalletManager.walletsRevision.value, before + 1);
      expect(notified, 1);
    });

    test('renameWallet(昵称=钱包名)自增 walletsRevision', () async {
      await seedThreeWallets();
      final manager = WalletManager();

      final before = WalletManager.walletsRevision.value;
      await manager.renameWallet(1, '新昵称');

      expect(WalletManager.walletsRevision.value, before + 1);
      final wallets = await manager.getWallets();
      expect(wallets.firstWhere((w) => w.walletIndex == 1).walletName, '新昵称');
    });

    test('sortOrder 相同(全 0)时按 walletIndex 升序兜底', () async {
      // 直接写 3 个 sortOrder 全为 0 的 entity，验证稳定兜底顺序。
      final isar = await WalletIsar.instance.db();
      await isar.writeTxn(() async {
        for (final i in [3, 1, 2]) {
          final entity = WalletProfileEntity()
            ..walletIndex = i
            ..walletName = 'wallet_$i'
            ..walletIcon = 'wallet'
            ..balance = 0
            ..ss58Address = 'addr_$i'
            ..accountId = '0x${i.toRadixString(16).padLeft(64, '0')}'
            ..alg = 'sr25519'
            ..ss58 = 2027
            ..createdAtMillis = i
            ..source = 'test'
            ..signMode = 'local'
            ..sortOrder = 0;
          await isar.walletProfileEntitys.put(entity);
        }
      });
      final manager = WalletManager();
      final wallets = await manager.getWallets();
      // sortOrder 全为 0,按 walletIndex 升序 → [1, 2, 3]
      expect(wallets.map((w) => w.walletIndex).toList(), [1, 2, 3]);
    });

    test('读取不会改写持久化 sortOrder', () async {
      final isar = await WalletIsar.instance.db();
      await isar.writeTxn(() async {
        for (final i in [2, 3, 1]) {
          final entity = WalletProfileEntity()
            ..walletIndex = i
            ..walletName = 'wallet_$i'
            ..walletIcon = 'wallet'
            ..balance = 0
            ..ss58Address = 'addr_$i'
            ..accountId = '0x${i.toRadixString(16).padLeft(64, '0')}'
            ..alg = 'sr25519'
            ..ss58 = 2027
            ..createdAtMillis = i
            ..source = 'test'
            ..signMode = 'local'
            ..sortOrder = 0;
          await isar.walletProfileEntitys.put(entity);
        }
      });
      final manager = WalletManager();
      final wallets = await manager.getWallets();
      expect(wallets.map((w) => w.walletIndex).toList(), [1, 2, 3]);
      expect(wallets[0].sortOrder, 0);
      expect(wallets[1].sortOrder, 0);
      expect(wallets[2].sortOrder, 0);
    });
  });
}
