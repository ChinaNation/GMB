import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// 全 App 单源的加固 SecureStorage 实例。
///
/// 冷钱包的密钥(master seed / 助记词密文 / AEK)、PIN 哈希、锁设置都存这里,
/// 硬化选项集中一处,避免各文件各自 `FlutterSecureStorage()` 时选项漂移。
///
/// - Android:启用 EncryptedSharedPreferences(AES-256)。需 minSdk ≥ 23
///   (见 android/app/build.gradle.kts `minSdk = maxOf(23, ...)`)。
/// - iOS:钥匙串可访问性设为 first_unlock_this_device —— 不随 iCloud 备份/
///   迁移外泄,仅本机首次解锁后可读,契合冷钱包只在前台解锁态使用的场景。
const FlutterSecureStorage appSecureStorage = FlutterSecureStorage(
  aOptions: AndroidOptions(
    encryptedSharedPreferences: true,
  ),
  iOptions: IOSOptions(
    accessibility: KeychainAccessibility.first_unlock_this_device,
  ),
);
