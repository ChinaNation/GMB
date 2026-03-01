import 'package:shared_preferences/shared_preferences.dart';

class AppSettingsService {
  static const String _kFaceAuthEnabled = 'settings.face_auth_enabled';

  Future<bool> isFaceAuthEnabled() async {
    final prefs = await SharedPreferences.getInstance();
    return prefs.getBool(_kFaceAuthEnabled) ?? true;
  }

  Future<void> setFaceAuthEnabled(bool enabled) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(_kFaceAuthEnabled, enabled);
  }
}
