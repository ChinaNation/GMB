import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/wallet/core/user_identification_settings.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('UserIdentificationSettings', () {
    test('default face auth should be enabled', () async {
      SharedPreferences.setMockInitialValues(<String, Object>{});
      final settings = UserIdentificationSettings();
      expect(await settings.isFaceAuthEnabled(), isTrue);
    });

    test('face auth switch should persist', () async {
      SharedPreferences.setMockInitialValues(<String, Object>{});
      final settings = UserIdentificationSettings();

      await settings.setFaceAuthEnabled(false);
      expect(await settings.isFaceAuthEnabled(), isFalse);

      await settings.setFaceAuthEnabled(true);
      expect(await settings.isFaceAuthEnabled(), isTrue);
    });
  });
}
