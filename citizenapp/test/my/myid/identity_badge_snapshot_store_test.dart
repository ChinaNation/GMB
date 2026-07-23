import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  const accountA =
      '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  const accountB =
      '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';

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
      accountId: accountA,
      identityLevel: 'voting',
    );
    await store.write(
      accountId: accountB,
      identityLevel: 'candidate',
    );

    final walletA = await store.read(accountA);
    final walletB = await store.read(accountB);
    expect(walletA?.identityLevel, 'voting');
    expect(walletA?.updatedAtMillis, 1234);
    expect(walletB?.identityLevel, 'candidate');
  });

  test('损坏或账户不匹配的快照会被清除', () async {
    final preferences = await SharedPreferences.getInstance();
    final store = IdentityBadgeSnapshotStore(preferences: preferences);
    const key = 'identity_badge_snapshot_v1:$accountA';

    await preferences.setString(key, '{broken');
    expect(await store.read(accountA), isNull);
    expect(preferences.containsKey(key), isFalse);

    await preferences.setString(
      key,
      '{"schema_version":1,"account_id":"$accountB",'
      '"identity_level":"voting","updated_at_millis":1}',
    );
    expect(await store.read(accountA), isNull);
    expect(preferences.containsKey(key), isFalse);
  });

  test('不接受非正式身份档', () async {
    final preferences = await SharedPreferences.getInstance();
    final store = IdentityBadgeSnapshotStore(preferences: preferences);

    await expectLater(
      store.write(accountId: accountA, identityLevel: 'admin'),
      throwsArgumentError,
    );
    expect(await store.read(accountA), isNull);
  });
}
