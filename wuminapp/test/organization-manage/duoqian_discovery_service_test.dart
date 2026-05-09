// 机构多签反向索引发现服务测试。
//
// 重点覆盖**不依赖链上 RPC** 的本地路径:
// - 30 分钟节流(SharedPreferences 持久化)
// - 空钱包(myPubkeysHex 空集)→ 直接返回 empty stats
//
// 链上扫描路径(state_getKeysPaged + fetchStorageBatch + AddressRegisteredSfid)
// 受 smoldot 真链依赖,留给端到端校核覆盖,本测试不模拟。

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/organization-manage/shared/duoqian_discovery_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const secureStorageChannel =
      MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  final secureStorage = <String, String>{};

  setUpAll(() async {
    await WalletIsar.instance.ensureTestCoreInitialized();
  });

  setUp(() async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    secureStorage.clear();
    await WalletIsar.instance.resetForTest();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, (call) async => null);
  });

  test('空钱包(无任何账户)→ 直接返回 empty stats,不发 RPC', () async {
    final service = DuoqianDiscoveryService();
    final stats = await service.discoverByMyWallets(
      myPubkeysHex: const <String>{},
    );
    expect(stats.institutionsScanned, 0);
    expect(stats.matchedPersonals, 0);
    expect(stats.matchedSfidAccounts, 0);
    expect(stats.newlyAdded, 0);
    expect(stats.orphansRemoved, 0);
  });

  test('30 分钟内重复调用返回 empty(节流生效)', () async {
    // 预设上次扫描时间 = 1 分钟前
    final justNow = DateTime.now().subtract(const Duration(minutes: 1));
    SharedPreferences.setMockInitialValues({
      'duoqian_discovery_last_at_ms': justNow.millisecondsSinceEpoch,
    });

    final service = DuoqianDiscoveryService();
    final stats = await service.discoverByMyWallets(
      myPubkeysHex: const {'aabbcc'},
      // 不传 force,应该被节流拦截
    );
    // 被节流时返回 empty(不调链 → institutionsScanned=0 即可证明被拦截)
    expect(stats.institutionsScanned, 0);
    expect(stats.matchedPersonals, 0);
  });

  test('lastDiscoveryAt 持久化读取', () async {
    final fixed = DateTime.fromMillisecondsSinceEpoch(1700000000000);
    SharedPreferences.setMockInitialValues({
      'duoqian_discovery_last_at_ms': fixed.millisecondsSinceEpoch,
    });
    final service = DuoqianDiscoveryService();
    final last = await service.lastDiscoveryAt();
    expect(last, fixed);
  });

  test('lastDiscoveryAt 无值返回 null', () async {
    final service = DuoqianDiscoveryService();
    final last = await service.lastDiscoveryAt();
    expect(last, isNull);
  });
}
