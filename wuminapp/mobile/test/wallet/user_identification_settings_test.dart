import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/wallet/core/user_identification_settings.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_isar.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    await WalletIsar.instance.ensureTestCoreInitialized();
  });

  setUp(() async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    await WalletIsar.instance.resetForTest();
  });

  group('UserIdentificationSettings', () {
    test('default face auth should be enabled', () async {
      final settings = UserIdentificationSettings();
      expect(await settings.isFaceAuthEnabled(), isTrue);
    });

    test('face auth switch should persist', () async {
      final settings = UserIdentificationSettings();

      await settings.setFaceAuthEnabled(false);
      expect(await settings.isFaceAuthEnabled(), isFalse);

      await settings.setFaceAuthEnabled(true);
      expect(await settings.isFaceAuthEnabled(), isTrue);
    });
  });
}
