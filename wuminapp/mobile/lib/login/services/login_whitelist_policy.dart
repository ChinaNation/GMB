import 'package:wuminapp_mobile/login/models/login_models.dart';
import 'package:wuminapp_mobile/login/models/login_exception.dart';
import 'package:wuminapp_mobile/login/services/login_whitelist_store.dart';

class LoginWhitelistPolicy {
  LoginWhitelistPolicy({LoginWhitelistStore? whitelistStore})
      : _whitelistStore = whitelistStore ?? LoginWhitelistStore();

  final LoginWhitelistStore _whitelistStore;

  Future<void> assertAllowed(WuminLoginChallenge challenge) async {
    final config = await _whitelistStore.load();
    final audAllowed = config.audWhitelist[challenge.system] ?? const <String>{};
    if (!audAllowed.contains(challenge.aud)) {
      throw LoginException(
        LoginErrorCode.unauthorizedAud,
        '登录来源未授权（aud=${challenge.aud}）。',
      );
    }

    final originAllowed =
        config.originWhitelist[challenge.system] ?? const <String>{};
    if (!originAllowed.contains(challenge.origin)) {
      throw LoginException(
        LoginErrorCode.unauthorizedOrigin,
        '登录设备未授权（origin=${challenge.origin}）。',
      );
    }
  }
}
