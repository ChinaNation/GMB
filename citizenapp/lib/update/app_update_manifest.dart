class AppVersionInfo {
  const AppVersionInfo({
    required this.packageName,
    required this.versionName,
    required this.versionCode,
  });

  final String packageName;
  final String versionName;
  final int versionCode;

  String get label => 'v$versionName';
}

class AppUpdateManifest {
  const AppUpdateManifest({
    required this.app,
    required this.platform,
    required this.packageName,
    required this.versionName,
    required this.versionCode,
    required this.apkAsset,
    required this.apkSha256,
    required this.publishedAt,
    required this.notes,
  });

  final String app;
  final String platform;
  final String packageName;
  final String versionName;
  final int versionCode;
  final String apkAsset;
  final String apkSha256;
  final DateTime? publishedAt;
  final String notes;

  factory AppUpdateManifest.fromJson(Map<String, dynamic> json) {
    final versionCode = _intValue(json['version_code']);
    final sha256 = _stringValue(json['apk_sha256']).toLowerCase();
    if (versionCode == null || versionCode <= 0) {
      throw const FormatException('更新清单 version_code 无效');
    }
    if (!RegExp(r'^[0-9a-f]{64}$').hasMatch(sha256)) {
      throw const FormatException('更新清单 apk_sha256 无效');
    }

    return AppUpdateManifest(
      app: _stringValue(json['app']),
      platform: _stringValue(json['platform']),
      packageName: _stringValue(json['package_name']),
      versionName: _stringValue(json['version_name']),
      versionCode: versionCode,
      apkAsset: _stringValue(json['apk_asset']),
      apkSha256: sha256,
      publishedAt: DateTime.tryParse(_stringValue(json['published_at'])),
      notes: _stringValue(json['notes']),
    );
  }

  bool matchesAndroidPackage(String expectedPackageName) {
    return app == 'citizenapp' &&
        platform == 'android' &&
        packageName == expectedPackageName &&
        apkAsset.trim().isNotEmpty &&
        versionName.trim().isNotEmpty;
  }
}

class AppUpdateInfo {
  const AppUpdateInfo({
    required this.manifest,
    required this.apkDownloadUrl,
  });

  final AppUpdateManifest manifest;
  final Uri apkDownloadUrl;
}

class AppUpdateCheckResult {
  const AppUpdateCheckResult({
    required this.currentVersion,
    required this.update,
  });

  final AppVersionInfo currentVersion;
  final AppUpdateInfo? update;
}

String _stringValue(Object? value) {
  return (value as String?)?.trim() ?? '';
}

int? _intValue(Object? value) {
  if (value is int) return value;
  if (value is num) return value.toInt();
  if (value is String) return int.tryParse(value.trim());
  return null;
}
