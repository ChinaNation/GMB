import 'package:biometric_storage/biometric_storage.dart';
import 'package:flutter/services.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';

/// [SecureSeedStore] 的 `biometric_storage` 后端。
///
/// 每个钱包在两个金库里各占一个 storage 文件（= 一把 Keystore/Keychain 密钥，
/// 每次读/写解密都触发系统认证，密钥永不出硬件）：
/// - seedVault（`wallet_seed_$id`）：seed。
/// - recoveryVault（`wallet_recovery_$id`）：助记词，作为 seed 读不出时的
///   自愈来源。
///
/// 两个金库均 `androidBiometricOnly: false` / `darwinBiometricOnly: false`
/// （允许生物识别**或**设备密码：图案/数字/PIN），以覆盖无生物识别的机型，
/// 与「有设备锁屏即可创建钱包」一致；iOS 对应 `.userPresence`（生物或密码）。
/// biometric_storage 每次读写都重新弹验证，故仍是「每次操作一次验证」。
///
/// 注（biometric_storage 5.0.1 约束，均已踩坑验证）：
/// - `androidBiometricOnly: true` 会禁用 PIN/图案（纯生物），且
///   `validityDurationSeconds: -1` 强制要求 biometricOnly；二者在无指纹/无人脸
///   的设备上无法创建密钥。
/// - `validityDurationSeconds: 0` = 认证令牌 0 秒过期，加密前就失效 →
///   keystore2 报 KEY_USER_NOT_AUTHENTICATED（表现为写入即 SecureStoreUnavailable）。
/// 故用正数 [_authTokenTtlSeconds] + `biometricOnly: false`：密钥锚定设备凭证、
/// 随生物变更不失效（seed 读失败的自愈路径仍保留为兜底）。
class BiometricSecureSeedStore implements SecureSeedStore {
  BiometricSecureSeedStore({BiometricStorage? plugin})
      : _plugin = plugin ?? BiometricStorage();

  final BiometricStorage _plugin;

  /// 已初始化的 storage 句柄按文件名缓存，避免每次操作重复 `init`。
  final Map<String, BiometricStorageFile> _handles =
      <String, BiometricStorageFile>{};

  static String _seedName(int walletIndex) => 'wallet_seed_$walletIndex';
  static String _recoveryName(int walletIndex) =>
      'wallet_recovery_$walletIndex';

  /// Keystore 认证令牌有效期（秒）。biometric_storage 每次读写都重新弹验证，
  /// 此值仅是「验证成功 → 那一次 AES 解/加密」之间的令牌存活缓冲；不是免验证
  /// 会话（签名/RPC 用派生密钥在 Dart 里做，不碰 Keystore，不受此窗口约束）。
  /// **不可为 0**：0 秒令牌会在加密操作前就过期 → keystore2 报
  /// KEY_USER_NOT_AUTHENTICATED（=「SecureStoreUnavailable: Unexpected Error」）。
  static const int _authTokenTtlSeconds = 10;

  /// seed 金库：每次操作一次身份验证，允许生物识别或设备密码（图案/数字/PIN）。
  static final StorageFileInitOptions _seedOptions = StorageFileInitOptions(
    authenticationValidityDurationSeconds: _authTokenTtlSeconds,
    androidBiometricOnly: false,
    darwinBiometricOnly: false,
  );

  /// 助记词金库：与 seed 金库同款——每次验证、允许设备密码、随生物变更不失效。
  static final StorageFileInitOptions _recoveryOptions = StorageFileInitOptions(
    authenticationValidityDurationSeconds: _authTokenTtlSeconds,
    androidBiometricOnly: false,
    darwinBiometricOnly: false,
  );

  static const PromptInfo _seedPrompt = PromptInfo(
    androidPromptInfo: AndroidPromptInfo(
      title: '验证身份以访问钱包密钥',
      negativeButton: '取消',
    ),
    iosPromptInfo: IosPromptInfo(
      saveTitle: '验证身份以保存钱包密钥',
      accessTitle: '验证身份以访问钱包密钥',
    ),
  );

  static const PromptInfo _recoveryPrompt = PromptInfo(
    androidPromptInfo: AndroidPromptInfo(
      title: '验证身份以访问助记词',
      negativeButton: '取消',
    ),
    iosPromptInfo: IosPromptInfo(
      saveTitle: '验证身份以保存助记词',
      accessTitle: '验证身份以访问助记词',
    ),
  );

  @override
  Future<SecureAuthStatus> authStatus() async {
    final CanAuthenticateResponse response;
    try {
      response = await _plugin.canAuthenticate();
    } on PlatformException {
      return SecureAuthStatus.unsupported;
    }
    return switch (response) {
      CanAuthenticateResponse.success => SecureAuthStatus.available,
      CanAuthenticateResponse.errorPasscodeNotSet =>
        SecureAuthStatus.noDeviceLock,
      _ => SecureAuthStatus.unsupported,
    };
  }

  @override
  Future<void> putSeed(int walletIndex, String seedHex) =>
      _write(_seedName(walletIndex), _seedOptions, _seedPrompt, seedHex);

  @override
  Future<String?> readSeed(int walletIndex) =>
      _readSeedEntry(_seedName(walletIndex), _seedOptions, _seedPrompt);

  @override
  Future<void> deleteSeed(int walletIndex) =>
      _delete(_seedName(walletIndex), _seedOptions, _seedPrompt);

  @override
  Future<void> putMnemonic(int walletIndex, String mnemonic) => _write(
      _recoveryName(walletIndex), _recoveryOptions, _recoveryPrompt, mnemonic);

  @override
  Future<String?> readMnemonic(int walletIndex) => _readWideEntry(
      _recoveryName(walletIndex), _recoveryOptions, _recoveryPrompt);

  @override
  Future<void> deleteMnemonic(int walletIndex) =>
      _delete(_recoveryName(walletIndex), _recoveryOptions, _recoveryPrompt);

  Future<BiometricStorageFile> _handle(
    String name,
    StorageFileInitOptions options,
    PromptInfo prompt,
  ) async {
    final cached = _handles[name];
    if (cached != null) {
      return cached;
    }
    final file = await _plugin.getStorage(
      name,
      options: options,
      promptInfo: prompt,
    );
    _handles[name] = file;
    return file;
  }

  /// 写入前先做 D3 fail-closed 校验：设备无锁屏则拒绝（不落密钥）。
  Future<void> _write(
    String name,
    StorageFileInitOptions options,
    PromptInfo prompt,
    String value,
  ) async {
    if (await authStatus() == SecureAuthStatus.noDeviceLock) {
      throw const NoDeviceCredential('设备未设置锁屏，无法安全存储钱包密钥');
    }
    final file = await _handle(name, options, prompt);
    try {
      await file.write(value);
    } on AuthException catch (e) {
      throw _mapCancelOr(e, () => SecureStoreUnavailable(e.message));
    } on PlatformException catch (e) {
      throw SecureStoreUnavailable(e.message ?? e.code);
    }
  }

  /// 严档读取：非用户取消的失败一律归为 [SeedKeyInvalidated]（含 KEK 失效），
  /// 交上层从助记词自愈。
  Future<String?> _readSeedEntry(
    String name,
    StorageFileInitOptions options,
    PromptInfo prompt,
  ) async {
    final file = await _handle(name, options, prompt);
    try {
      return await file.read();
    } on AuthException catch (e) {
      throw _mapCancelOr(e, () => SeedKeyInvalidated(e.message));
    } on PlatformException catch (e) {
      throw SeedKeyInvalidated(e.message ?? e.code);
    }
  }

  /// 宽档读取：宽档不应因生物变更失效，非取消失败归为 [SecureStoreUnavailable]，
  /// 不误判成需要自愈。
  Future<String?> _readWideEntry(
    String name,
    StorageFileInitOptions options,
    PromptInfo prompt,
  ) async {
    final file = await _handle(name, options, prompt);
    try {
      return await file.read();
    } on AuthException catch (e) {
      throw _mapCancelOr(e, () => SecureStoreUnavailable(e.message));
    } on PlatformException catch (e) {
      throw SecureStoreUnavailable(e.message ?? e.code);
    }
  }

  Future<void> _delete(
    String name,
    StorageFileInitOptions options,
    PromptInfo prompt,
  ) async {
    final file = await _handle(name, options, prompt);
    try {
      await file.delete();
    } on AuthException catch (e) {
      throw _mapCancelOr(e, () => SecureStoreUnavailable(e.message));
    } on PlatformException catch (e) {
      throw SecureStoreUnavailable(e.message ?? e.code);
    }
  }

  /// 用户主动取消/超时统一为 [AuthCancelled]；其余（`unknown`）交由
  /// [orElse] 决定（严档=失效自愈，宽档/写=不可用）。
  SecureSeedException _mapCancelOr(
    AuthException e,
    SecureSeedException Function() orElse,
  ) {
    return switch (e.code) {
      AuthExceptionCode.userCanceled ||
      AuthExceptionCode.canceled ||
      AuthExceptionCode.timeout =>
        AuthCancelled(e.message),
      // unknown / linuxAppArmorDenied / 未来新增码 → 交调用点决定语义。
      _ => orElse(),
    };
  }
}
