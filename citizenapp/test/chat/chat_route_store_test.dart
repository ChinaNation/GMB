import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/isar/app_isar.dart';

void main() {
  setUp(() async {
    await WalletIsar.instance.resetForTest();
  });

  tearDown(() async {
    await WalletIsar.instance.resetForTest();
  });

  test('Chat route cache creates, reads, and replaces route records', () async {
    final store = ChatStore();

    await store.upsertRouteRecord(
      const ChatRoute(
        peerAccount: 'bob-wallet',
        routeDisplayName: 'Bob',
        deviceId: 'bob-phone',
        devicePublicKeyHex: '0a0b',
        safetyNumber: '12 34',
        cloudflareMailboxId: 'bob-wallet',
        nearbyPeerHint: 'bob-nearby',
        note: 'first',
      ),
    );

    final created = await store.getRouteRecord('bob-wallet');
    expect(created, isNotNull);
    expect(created!.routeDisplayName, 'Bob');
    expect(created.cloudflareMailboxId, 'bob-wallet');
    expect(created.nearbyPeerHint, 'bob-nearby');

    await store.upsertRouteRecord(
      ChatRoute(
        peerAccount: 'bob-wallet',
        routeDisplayName: 'Bob New',
        deviceId: created.deviceId,
        devicePublicKeyHex: created.devicePublicKeyHex,
        safetyNumber: created.safetyNumber,
        cloudflareMailboxId: created.cloudflareMailboxId,
        nearbyPeerHint: created.nearbyPeerHint,
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
