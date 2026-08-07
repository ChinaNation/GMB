import 'dart:ui' show Locale;

import 'package:flutter_test/flutter_test.dart';
import 'package:local_auth_android/local_auth_android.dart';
import 'package:local_auth_darwin/local_auth_darwin.dart';

import 'package:citizenapp/ui/biometric_auth_text.dart';

/// 生物识别对话框文案口径:默认中文,只有英文手机显示英文,其余语言一律回落中文。
/// 与 iOS `zh-Hans.lproj`/`en.lproj`、Android `values/` + `values-en/` 同一条规则。
void main() {
  tearDown(() => BiometricAuthText.debugLocale = null);

  void setLocale(String languageCode, {String? countryCode}) {
    BiometricAuthText.debugLocale = () => Locale(languageCode, countryCode);
  }

  test('中文手机 → 中文', () {
    setLocale('zh', countryCode: 'CN');
    expect(BiometricAuthText.isEnglish, isFalse);
    expect(BiometricAuthText.pick(zh: '中', en: 'en'), '中');
  });

  test('英文手机 → 英文(含各英语地区)', () {
    for (final country in <String?>[null, 'US', 'GB', 'AU']) {
      setLocale('en', countryCode: country);
      expect(BiometricAuthText.isEnglish, isTrue, reason: 'en_$country');
      expect(BiometricAuthText.pick(zh: '中', en: 'en'), 'en');
    }
  });

  test('其它语言(日/法/德/韩/西)一律回落中文,不得漏成英文', () {
    for (final code in <String>['ja', 'fr', 'de', 'ko', 'es']) {
      setLocale(code);
      expect(BiometricAuthText.isEnglish, isFalse, reason: '$code 应回落中文');
      expect(BiometricAuthText.pick(zh: '中', en: 'en'), '中', reason: code);
    }
  });

  test('messages 同时覆盖 Android 与 iOS 两侧,且随语言切换', () {
    setLocale('zh');
    final zh = BiometricAuthText.messages();
    expect(zh.whereType<AndroidAuthMessages>().single.cancelButton, '取消');
    expect(zh.whereType<AndroidAuthMessages>().single.signInTitle, '身份验证');
    expect(zh.whereType<IOSAuthMessages>().single.cancelButton, '取消');

    setLocale('en');
    final en = BiometricAuthText.messages();
    expect(en.whereType<AndroidAuthMessages>().single.cancelButton, 'Cancel');
    expect(en.whereType<IOSAuthMessages>().single.cancelButton, 'Cancel');
  });

  test('Android 字段无遗漏:本 App 允许设备密码回退,回退路径文案必须给全', () {
    setLocale('zh');
    final android =
        BiometricAuthText.messages().whereType<AndroidAuthMessages>().single;
    for (final value in <String?>[
      android.signInTitle,
      android.biometricHint,
      android.biometricNotRecognized,
      android.biometricRequiredTitle,
      android.biometricSuccess,
      android.cancelButton,
      android.deviceCredentialsRequiredTitle,
      android.deviceCredentialsSetupDescription,
      android.goToSettingsButton,
      android.goToSettingsDescription,
    ]) {
      expect(value, isNotNull);
      expect(value, isNotEmpty);
    }
  });
}
