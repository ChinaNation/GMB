import 'dart:ui' show Locale, PlatformDispatcher;

import 'package:flutter/foundation.dart' show visibleForTesting;
// AuthMessages 基类由 local_auth_android 一并导出,无需再单独引 platform_interface。
import 'package:local_auth_android/local_auth_android.dart';
import 'package:local_auth_darwin/local_auth_darwin.dart';

/// 生物识别系统对话框的文案单源。
///
/// **口径(全端统一)**:默认中文;只有手机系统语言是英文时才显示英文,其余语言
/// (日、法、德…)一律回落中文。与 iOS `zh-Hans.lproj`/`en.lproj`、Android
/// `values/`(中文默认)+ `values-en/` 的回落规则完全一致。
///
/// **为什么需要它**:`localizedReason` 只是对话框中间那句解释,而标题、提示、
/// 取消按钮等**框架文字由 local_auth 插件提供,默认是英文硬编码串** —— 不传
/// [messages] 就会出现"中文解释 + 英文标题/按钮"的混排。
///
/// 语言判定用 [PlatformDispatcher.instance.locale](设备当前语言),不依赖
/// `BuildContext`:`WalletManager` 等非 Widget 层也要弹这个框。
class BiometricAuthText {
  const BiometricAuthText._();

  /// 语言判定的唯一规则:**只有英文为真**,其余语言(日、法、德…)一律按中文处理。
  /// 抽成纯函数,便于逐语言钉死回落行为。
  static bool isEnglishLocale(Locale locale) => locale.languageCode == 'en';

  /// 设备语言来源。生产恒为 [PlatformDispatcher.instance] 的当前语言;
  /// 测试注入用(与仓库 `debugXxx` 惯例一致),`tearDown` 里置 null 复位。
  ///
  /// 之所以需要这个接缝:生产直读 `PlatformDispatcher.instance` 时,
  /// `TestPlatformDispatcher.localeTestValue` 覆写的是测试包装器而非真单例,
  /// 覆盖不到本类 —— 不留接缝这段逻辑就完全不可测。
  @visibleForTesting
  static Locale Function()? debugLocale;

  static Locale get _locale =>
      debugLocale?.call() ?? PlatformDispatcher.instance.locale;

  /// 仅英文手机为真;其余语言一律按中文处理(默认语言)。
  static bool get isEnglish => isEnglishLocale(_locale);

  /// 按当前语言二选一。调用方用它给 `localizedReason` 取值。
  static String pick({required String zh, required String en}) =>
      isEnglish ? en : zh;

  /// 传给 `LocalAuthentication.authenticate(authMessages: ...)`。
  ///
  /// 本 App 允许设备密码回退(`biometricOnly: false`),因此设备凭据相关文案
  /// 也要给全,否则用户走到回退路径时会撞见英文串。
  static List<AuthMessages> messages() => <AuthMessages>[
        AndroidAuthMessages(
          signInTitle: pick(zh: '身份验证', en: 'Authentication required'),
          biometricHint: pick(zh: '验证您的身份', en: 'Verify your identity'),
          biometricNotRecognized:
              pick(zh: '未能识别,请重试', en: 'Not recognized. Try again.'),
          biometricRequiredTitle:
              pick(zh: '需要生物识别', en: 'Biometrics required'),
          biometricSuccess: pick(zh: '验证成功', en: 'Verified'),
          cancelButton: pick(zh: '取消', en: 'Cancel'),
          deviceCredentialsRequiredTitle:
              pick(zh: '需要设备密码', en: 'Device credentials required'),
          deviceCredentialsSetupDescription: pick(
            zh: '请在系统设置中设置设备密码',
            en: 'Set up device credentials in Settings.',
          ),
          goToSettingsButton: pick(zh: '去设置', en: 'Go to settings'),
          goToSettingsDescription: pick(
            zh: '本机尚未设置生物识别,请在系统设置中开启',
            en: 'Biometrics are not set up on this device. Set them up in Settings.',
          ),
        ),
        IOSAuthMessages(
          cancelButton: pick(zh: '取消', en: 'Cancel'),
          goToSettingsButton: pick(zh: '去设置', en: 'Go to settings'),
          goToSettingsDescription: pick(
            zh: '本机尚未设置生物识别,请在系统设置中开启',
            en: 'Biometrics are not set up on this device. Set them up in Settings.',
          ),
          lockOut: pick(
            zh: '生物识别已锁定,请用设备密码解锁后重试',
            en: 'Biometrics are locked. Unlock with your device passcode and try again.',
          ),
          localizedFallbackTitle: pick(zh: '使用设备密码', en: 'Use passcode'),
        ),
      ];
}
