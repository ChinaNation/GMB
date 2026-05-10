// 个人多签反向索引发现服务测试。
//
// 重点覆盖不依赖链上 RPC 的本地路径：
// - 30 分钟节流(SharedPreferences 持久化)
// - 空钱包(myPubkeysHex 空集)直接返回 empty stats

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/governance/personal-manage/personal_manage_discovery_service.dart';

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

  test('空钱包直接返回 empty stats,不发 RPC', () async {
    final service = PersonalManageDiscoveryService();
    final stats = await service.discoverByMyWallets(
      myPubkeysHex: const <String>{},
    );
    expect(stats.subjectsScanned, 0);
    expect(stats.matchedPersonals, 0);
    expect(stats.newlyAdded, 0);
    expect(stats.orphansRemoved, 0);
  });

  test('30 分钟内重复调用返回 empty(节流生效)', () async {
    final justNow = DateTime.now().subtract(const Duration(minutes: 1));
    SharedPreferences.setMockInitialValues({
      'personal_manage_discovery_last_at_ms': justNow.millisecondsSinceEpoch,
    });

    final service = PersonalManageDiscoveryService();
    final stats = await service.discoverByMyWallets(
      myPubkeysHex: const {'aabbcc'},
    );
    expect(stats.subjectsScanned, 0);
    expect(stats.matchedPersonals, 0);
  });

  test('lastDiscoveryAt 持久化读取', () async {
    final fixed = DateTime.fromMillisecondsSinceEpoch(1700000000000);
    SharedPreferences.setMockInitialValues({
      'personal_manage_discovery_last_at_ms': fixed.millisecondsSinceEpoch,
    });
    final service = PersonalManageDiscoveryService();
    final last = await service.lastDiscoveryAt();
    expect(last, fixed);
  });

  test('lastDiscoveryAt 无值返回 null', () async {
    final service = PersonalManageDiscoveryService();
    final last = await service.lastDiscoveryAt();
    expect(last, isNull);
  });
}
