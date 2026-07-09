import 'package:flutter/services.dart';

import 'app_update_manifest.dart';

class AppUpdateInstaller {
  static const MethodChannel _channel = MethodChannel('org.citizenapp/update');

  Future<AppVersionInfo> getPackageInfo() async {
    final result = await _channel.invokeMapMethod<String, Object?>(
      'getPackageInfo',
    );
    if (result == null) {
      throw StateError('无法读取当前应用版本');
    }
    final versionCode = result['versionCode'];
    return AppVersionInfo(
      packageName: result['packageName'] as String? ?? '',
      versionName: result['versionName'] as String? ?? '',
      versionCode: versionCode is int
          ? versionCode
          : int.tryParse(versionCode.toString()) ?? 0,
    );
  }

  Future<bool> installApk(String apkPath) async {
    final started = await _channel.invokeMethod<bool>(
      'installApk',
      <String, Object?>{'apkPath': apkPath},
    );
    return started ?? false;
  }
}
