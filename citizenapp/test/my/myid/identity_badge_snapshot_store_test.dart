import 'package:citizenapp/my/myid/identity_badge_snapshot_store.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  const cidA = 'GD-CTZN1-000000001-2026';
  const cidB = 'GD-CTZN1-000000002-2026';

  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  test('身份徽章快照按永久 CID 隔离', () async {
    final preferences = await SharedPreferences.getInstance();
    final store = IdentityBadgeSnapshotStore(
      preferences: preferences,
      nowProvider: () => DateTime.fromMillisecondsSinceEpoch(1234),
    );

    await store.write(
      cidNumber: cidA,
      identityLevel: 'voting',
    );
    await store.write(
      cidNumber: cidB,
      identityLevel: 'candidate',
    );

    final citizenA = await store.read(cidA);
    final citizenB = await store.read(cidB);
    expect(citizenA?.identityLevel, 'voting');
    expect(citizenA?.updatedAtMillis, 1234);
    expect(citizenB?.identityLevel, 'candidate');
  });

  test('损坏或 CID 不匹配的快照会被清除', () async {
    final preferences = await SharedPreferences.getInstance();
    final store = IdentityBadgeSnapshotStore(preferences: preferences);
    const key = 'identity_badge_snapshot_by_cid:$cidA';

    await preferences.setString(key, '{broken');
    expect(await store.read(cidA), isNull);
    expect(preferences.containsKey(key), isFalse);

    await preferences.setString(
      key,
      '{"schema_version":1,"cid_number":"$cidB",'
      '"identity_level":"voting","updated_at_millis":1}',
    );
    expect(await store.read(cidA), isNull);
    expect(preferences.containsKey(key), isFalse);
  });

  test('不接受非正式身份档', () async {
    final preferences = await SharedPreferences.getInstance();
    final store = IdentityBadgeSnapshotStore(preferences: preferences);

    await expectLater(
      store.write(cidNumber: cidA, identityLevel: 'admin'),
      throwsArgumentError,
    );
    expect(await store.read(cidA), isNull);
  });
}
