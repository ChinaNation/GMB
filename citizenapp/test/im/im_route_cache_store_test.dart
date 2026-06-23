import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/im/storage/im_isar_store.dart';
import 'package:citizenapp/isar/wallet_isar.dart';

void main() {
  setUp(() async {
    await WalletIsar.instance.resetForTest();
  });

  tearDown(() async {
    await WalletIsar.instance.resetForTest();
  });

  test('IM route cache creates, reads, and replaces route records', () async {
    final store = ImIsarStore();

    await store.upsertRouteRecord(
      const ImRouteRecord(
        walletChatAccount: 'bob-wallet',
        routeDisplayName: 'Bob',
        deviceId: 'bob-phone',
        devicePublicKeyHex: '0a0b',
        safetyNumber: '12 34',
        nodePeerId: 'peer-bob',
        nodeMultiaddr: '/ip6/::1/tcp/30334/p2p/peer-bob',
        note: 'first',
      ),
    );

    final created = await store.getRouteRecord('bob-wallet');
    expect(created, isNotNull);
    expect(created!.routeDisplayName, 'Bob');
    expect(created.nodeMultiaddr, startsWith('/ip6/'));

    await store.upsertRouteRecord(
      ImRouteRecord(
        walletChatAccount: 'bob-wallet',
        routeDisplayName: 'Bob New',
        deviceId: created.deviceId,
        devicePublicKeyHex: created.devicePublicKeyHex,
        safetyNumber: created.safetyNumber,
        nodePeerId: created.nodePeerId,
        nodeMultiaddr: created.nodeMultiaddr,
        createdAtMillis: created.createdAtMillis,
      ),
    );

    final routes = await store.readRouteRecords();
    expect(routes, hasLength(1));
    expect(routes.single.routeDisplayName, 'Bob New');
    expect(routes.single.createdAtMillis, created.createdAtMillis);
    expect(
      routes.single.updatedAtMillis,
      greaterThanOrEqualTo(created.updatedAtMillis ?? 0),
    );
  });
}
