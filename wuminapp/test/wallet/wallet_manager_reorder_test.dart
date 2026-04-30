import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 中文注释:验证 WalletManager.reorderWallets 是否正确写入 sortOrder,
/// 并检查 sortBySortOrder().thenByWalletIndex() 排序顺序。
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const secureStorageChannel =
      MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  const localAuthChannel = MethodChannel('plugins.flutter.io/local_auth');
  final secureStorage = <String, String>{};

  setUpAll(() async {
    await WalletIsar.instance.ensureTestCoreInitialized();
  });

  setUp(() async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    secureStorage.clear();
    await WalletIsar.instance.resetForTest();
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

  /// 中文注释:每个 test 文件结束后必须 close 并删除磁盘 db,
  /// 否则同物理目录下其他 test 文件的 setUp 在新 isolate 中
  /// 看到的是「未初始化的 _isar 但磁盘有残留」,resetForTest 不会清盘,
  /// _openAndMigrate 会复用残留 → walletIndex 索引被占,后续 test 失败。
  tearDownAll(() async {
    await WalletIsar.instance.resetForTest();
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
          ..address = 'addr_$i'
          ..pubkeyHex = 'pub_$i'
          ..alg = 'sr25519'
          ..ss58 = 2027
          ..createdAtMillis = i
          ..source = 'test'
          ..signMode = 'local'
          ..sortOrder = 0;
        await isar.walletProfileEntitys.put(entity);
      }
    });
    // 触发一次性迁移 flag,让后续 getWallets 不会再覆写 sortOrder。
    final manager = WalletManager();
    await manager.getWallets();
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

    test('sortOrder 相同(全 0)时按 walletIndex 升序兜底', () async {
      // 直接写 3 个 sortOrder 全为 0 的 entity,跳过迁移 flag,模拟边界场景。
      SharedPreferences.setMockInitialValues(<String, Object>{
        'wallet_sort_order_initialized': true,
      });
      final isar = await WalletIsar.instance.db();
      await isar.writeTxn(() async {
        for (final i in [3, 1, 2]) {
          final entity = WalletProfileEntity()
            ..walletIndex = i
            ..walletName = 'wallet_$i'
            ..walletIcon = 'wallet'
            ..balance = 0
            ..address = 'addr_$i'
            ..pubkeyHex = 'pub_$i'
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

    test('首次进入会按 walletIndex 升序填 sortOrder(无感迁移)', () async {
      // 模拟旧版本数据:flag 未设置,3 个钱包 sortOrder 全为 0(默认)。
      SharedPreferences.setMockInitialValues(<String, Object>{});
      final isar = await WalletIsar.instance.db();
      await isar.writeTxn(() async {
        for (final i in [2, 3, 1]) {
          final entity = WalletProfileEntity()
            ..walletIndex = i
            ..walletName = 'wallet_$i'
            ..walletIcon = 'wallet'
            ..balance = 0
            ..address = 'addr_$i'
            ..pubkeyHex = 'pub_$i'
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
      // 迁移按原 walletIndex 升序填 sortOrder = 0/1/2,顺序仍是 1/2/3。
      expect(wallets.map((w) => w.walletIndex).toList(), [1, 2, 3]);
      expect(wallets[0].sortOrder, 0);
      expect(wallets[1].sortOrder, 1);
      expect(wallets[2].sortOrder, 2);
    });
  });
}
