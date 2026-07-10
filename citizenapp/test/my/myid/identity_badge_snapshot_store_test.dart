import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  test('身份徽章快照按钱包账户隔离', () async {
    final preferences = await SharedPreferences.getInstance();
    final store = IdentityBadgeSnapshotStore(
      preferences: preferences,
      nowProvider: () => DateTime.fromMillisecondsSinceEpoch(1234),
    );

    await store.write(
      walletAccount: 'wallet_a',
      identityLevel: 'voting',
    );
    await store.write(
      walletAccount: 'wallet_b',
      identityLevel: 'candidate',
    );

    final walletA = await store.read('wallet_a');
    final walletB = await store.read('wallet_b');
    expect(walletA?.identityLevel, 'voting');
    expect(walletA?.updatedAtMillis, 1234);
    expect(walletB?.identityLevel, 'candidate');
  });

  test('损坏或账户不匹配的快照会被清除', () async {
    final preferences = await SharedPreferences.getInstance();
    final store = IdentityBadgeSnapshotStore(preferences: preferences);
    const key = 'identity_badge_snapshot_v1:wallet_a';

    await preferences.setString(key, '{broken');
    expect(await store.read('wallet_a'), isNull);
    expect(preferences.containsKey(key), isFalse);

    await preferences.setString(
      key,
      '{"schema_version":1,"wallet_account":"wallet_b",'
      '"identity_level":"voting","updated_at_millis":1}',
    );
    expect(await store.read('wallet_a'), isNull);
    expect(preferences.containsKey(key), isFalse);
  });

  test('不接受非正式身份档', () async {
    final preferences = await SharedPreferences.getInstance();
    final store = IdentityBadgeSnapshotStore(preferences: preferences);

    await expectLater(
      store.write(walletAccount: 'wallet_a', identityLevel: 'admin'),
      throwsArgumentError,
    );
    expect(await store.read('wallet_a'), isNull);
  });
}
