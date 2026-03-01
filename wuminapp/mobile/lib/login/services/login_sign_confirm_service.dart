import 'package:local_auth/local_auth.dart';
import 'package:flutter/services.dart';
import 'package:wuminapp_mobile/login/models/login_exception.dart';
import 'package:wuminapp_mobile/services/app_settings_service.dart';

class LoginSignConfirmService {
  LoginSignConfirmService({
    LocalAuthentication? localAuth,
    AppSettingsService? settingsService,
  })  : _localAuth = localAuth ?? LocalAuthentication(),
        _settingsService = settingsService ?? AppSettingsService();

  final LocalAuthentication _localAuth;
  final AppSettingsService _settingsService;

  Future<void> confirmBeforeSign({
    String localizedReason = '请验证身份后执行登录签名',
  }) async {
    final enabled = await _settingsService.isFaceAuthEnabled();
    if (!enabled) {
      return;
    }

    try {
      final isSupported = await _localAuth.isDeviceSupported();
      final available = await _localAuth.getAvailableBiometrics();
      if (!isSupported || available.isEmpty) {
        throw const LoginException(
          LoginErrorCode.biometricUnavailable,
          '当前设备未启用生物识别，请在系统设置中录入指纹/人脸后重试',
        );
      }

      final ok = await _localAuth.authenticate(
        localizedReason: localizedReason,
        options: const AuthenticationOptions(
          biometricOnly: true,
          stickyAuth: false,
          useErrorDialogs: true,
        ),
      );
      if (!ok) {
        throw const LoginException(
          LoginErrorCode.biometricRejected,
          '未通过生物识别验证，已取消签名',
        );
      }
    } on PlatformException catch (e) {
      throw LoginException(
        LoginErrorCode.biometricUnavailable,
        '生物识别不可用：${e.message ?? e.code}',
      );
    }
  }
}
