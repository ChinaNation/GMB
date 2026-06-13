// MultisigDiscoveryCoordinator 单测(ADR-018 §九)。
//
// 覆盖**不依赖链上 RPC** 的本地路径(扫描/落库受 smoldot 真链依赖,留给端到端):
// - 空钱包 → 直接返回 empty,不发链
// - 30 分钟节流(SharedPreferences 持久化,统一 key)
// - lastDiscoveryAt 读取

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/governance/shared/multisig_discovery_coordinator.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const secureStorageChannel =
      MethodChannel('plugins.it_nomads.com/flutter_secure_storage');

  setUp(() {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, (call) async => null);
  });

  test('空钱包 → 直接返回 empty,不发链', () async {
    final coordinator = MultisigDiscoveryCoordinator();
    final result = await coordinator.discoverAll(
      myPubkeysHex: const <String>{},
    );
    expect(result.institution.institutionsScanned, 0);
    expect(result.personal.subjectsScanned, 0);
    expect(result.anyChanged, isFalse);
  });

  test('30 分钟内重复调用返回 empty(节流生效)', () async {
    final justNow = DateTime.now().subtract(const Duration(minutes: 1));
    SharedPreferences.setMockInitialValues({
      'multisig_discovery_last_at_ms': justNow.millisecondsSinceEpoch,
    });
    final coordinator = MultisigDiscoveryCoordinator();
    final result = await coordinator.discoverAll(
      myPubkeysHex: const {'aabbcc'},
      // 不传 force,应被节流拦截
    );
    expect(result.institution.institutionsScanned, 0);
    expect(result.personal.subjectsScanned, 0);
  });

  test('lastDiscoveryAt 持久化读取', () async {
    final fixed = DateTime.fromMillisecondsSinceEpoch(1700000000000);
    SharedPreferences.setMockInitialValues({
      'multisig_discovery_last_at_ms': fixed.millisecondsSinceEpoch,
    });
    final coordinator = MultisigDiscoveryCoordinator();
    expect(await coordinator.lastDiscoveryAt(), fixed);
  });

  test('lastDiscoveryAt 无值返回 null', () async {
    final coordinator = MultisigDiscoveryCoordinator();
    expect(await coordinator.lastDiscoveryAt(), isNull);
  });
}
