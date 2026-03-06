import 'package:flutter_test/flutter_test.dart';
import 'package:local_auth/local_auth.dart';
import 'package:wuminapp_mobile/login/models/login_exception.dart';
import 'package:wuminapp_mobile/wallet/core/user_identification.dart';
import 'package:wuminapp_mobile/wallet/core/user_identification_settings.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('UserIdentificationService', () {
    test('should skip local auth when face auth switch is off', () async {
      final localAuth = _FakeLocalAuthentication(
        supported: false,
        biometrics: const <BiometricType>[],
        authenticateResult: false,
      );
      final settings = _FakeUserIdentificationSettings(enabled: false);
      final service = UserIdentificationService(
        localAuth: localAuth,
        settingsService: settings,
      );

      await service.confirmBeforeSign();

      expect(localAuth.isDeviceSupportedCalls, 0);
      expect(localAuth.getBiometricsCalls, 0);
      expect(localAuth.authenticateCalls, 0);
    });

    test('should throw biometricUnavailable when biometrics missing', () async {
      final localAuth = _FakeLocalAuthentication(
        supported: true,
        biometrics: const <BiometricType>[],
        authenticateResult: true,
      );
      final settings = _FakeUserIdentificationSettings(enabled: true);
      final service = UserIdentificationService(
        localAuth: localAuth,
        settingsService: settings,
      );

      await expectLater(
        service.confirmBeforeSign(),
        throwsA(
          isA<LoginException>().having(
            (e) => e.code,
            'code',
            LoginErrorCode.biometricUnavailable,
          ),
        ),
      );
      expect(localAuth.isDeviceSupportedCalls, 1);
      expect(localAuth.getBiometricsCalls, 1);
      expect(localAuth.authenticateCalls, 0);
    });
  });
}

class _FakeUserIdentificationSettings extends UserIdentificationSettings {
  _FakeUserIdentificationSettings({required this.enabled});

  bool enabled;

  @override
  Future<bool> isFaceAuthEnabled() async {
    return enabled;
  }

  @override
  Future<void> setFaceAuthEnabled(bool enabled) async {
    this.enabled = enabled;
  }
}

class _FakeLocalAuthentication extends LocalAuthentication {
  _FakeLocalAuthentication({
    required this.supported,
    required this.biometrics,
    required this.authenticateResult,
  });

  final bool supported;
  final List<BiometricType> biometrics;
  final bool authenticateResult;
  int isDeviceSupportedCalls = 0;
  int getBiometricsCalls = 0;
  int authenticateCalls = 0;

  @override
  Future<bool> isDeviceSupported() async {
    isDeviceSupportedCalls += 1;
    return supported;
  }

  @override
  Future<List<BiometricType>> getAvailableBiometrics() async {
    getBiometricsCalls += 1;
    return biometrics;
  }

  @override
  Future<bool> authenticate({
    required String localizedReason,
    Iterable authMessages = const <dynamic>[],
    AuthenticationOptions options = const AuthenticationOptions(),
  }) async {
    authenticateCalls += 1;
    return authenticateResult;
  }
}
