import 'dart:convert';
import 'dart:math';

import 'package:crypto/crypto.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:isar/isar.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../isar/wallet_isar.dart';
import '../wallet/mnemonic_cipher.dart';

/// 应用锁（6 位 PIN）服务。
///
/// PIN 以 SHA-256(pin + salt) 形式存储在 SecureStorage 中。
/// 连续 5 次验证错误锁定 24 小时，累计 3 次锁定则清空全部应用数据。
class AppLockService {
  static const FlutterSecureStorage _secure = FlutterSecureStorage();
  static const String _keyPinHash = 'pin_hash';
  static const String _keyPinSalt = 'pin_salt';
  static const String _keyFailCount = 'pin_fail_count';
  static const String _keyLockUntil = 'pin_lock_until';
  static const String _keyLockCount = 'pin_lock_count';

  static const int maxFailAttempts = 5;
  static const int maxLockCount = 3;
  static const Duration lockDuration = Duration(hours: 24);

  // ---------------------------------------------------------------------------
  // PIN 管理
  // ---------------------------------------------------------------------------

  static Future<void> setPin(String pin) async {
    final salt = _generateSalt();
    final hash = _hash(pin, salt);
    await _secure.write(key: _keyPinSalt, value: salt);
    await _secure.write(key: _keyPinHash, value: hash);
    await _secure.write(key: _keyFailCount, value: '0');
    await _secure.delete(key: _keyLockUntil);
    await _secure.write(key: _keyLockCount, value: '0');
  }

  static Future<bool> verifyPin(String pin) async {
    if (await isLocked()) return false;

    final storedHash = await _secure.read(key: _keyPinHash);
    final storedSalt = await _secure.read(key: _keyPinSalt);
    if (storedHash == null || storedSalt == null) return false;

    final inputHash = _hash(pin, storedSalt);
    if (inputHash == storedHash) {
      await _secure.write(key: _keyFailCount, value: '0');
      return true;
    }

    final failCount = await _readInt(_keyFailCount) + 1;
    await _secure.write(key: _keyFailCount, value: failCount.toString());

    if (failCount >= maxFailAttempts) {
      final lockCount = await _readInt(_keyLockCount) + 1;
      await _secure.write(key: _keyLockCount, value: lockCount.toString());
      await _secure.write(key: _keyFailCount, value: '0');

      if (lockCount >= maxLockCount) {
        await wipeAllData();
        return false;
      }

      final lockUntil =
          DateTime.now().add(lockDuration).millisecondsSinceEpoch;
      await _secure.write(key: _keyLockUntil, value: lockUntil.toString());
    }

    return false;
  }

  static Future<void> removePin() async {
    await _secure.delete(key: _keyPinHash);
    await _secure.delete(key: _keyPinSalt);
    await _secure.delete(key: _keyFailCount);
    await _secure.delete(key: _keyLockUntil);
    await _secure.delete(key: _keyLockCount);
  }

  static Future<bool> isPinSet() async {
    final hash = await _secure.read(key: _keyPinHash);
    return hash != null && hash.isNotEmpty;
  }

  // ---------------------------------------------------------------------------
  // 锁定状态
  // ---------------------------------------------------------------------------

  static Future<bool> isLocked() async {
    final lockUntilStr = await _secure.read(key: _keyLockUntil);
    if (lockUntilStr == null) return false;
    final lockUntil = int.tryParse(lockUntilStr);
    if (lockUntil == null) return false;
    return DateTime.now().millisecondsSinceEpoch < lockUntil;
  }

  static Future<int> getRemainingLockSeconds() async {
    final lockUntilStr = await _secure.read(key: _keyLockUntil);
    if (lockUntilStr == null) return 0;
    final lockUntil = int.tryParse(lockUntilStr);
    if (lockUntil == null) return 0;
    final remaining = lockUntil - DateTime.now().millisecondsSinceEpoch;
    return remaining > 0 ? remaining ~/ 1000 : 0;
  }

  static Future<int> getFailCount() async => _readInt(_keyFailCount);
  static Future<int> getLockCount() async => _readInt(_keyLockCount);

  // ---------------------------------------------------------------------------
  // 数据清空
  // ---------------------------------------------------------------------------

  static Future<void> wipeAllData() async {
    // 先清除内存中的加密密钥缓存
    MnemonicCipher.clearCache();

    try {
      final isar = await WalletIsar.instance.db();
      await isar.close(deleteFromDisk: true);
    } catch (_) {}

    await _secure.deleteAll();

    final prefs = await SharedPreferences.getInstance();
    await prefs.clear();
  }

  // ---------------------------------------------------------------------------
  // 内部工具
  // ---------------------------------------------------------------------------

  static String _generateSalt() {
    final random = Random.secure();
    final bytes = List<int>.generate(16, (_) => random.nextInt(256));
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  /// PBKDF2-HMAC-SHA256，100 万次迭代。
  ///
  /// 6 位 PIN 仅有 100 万种组合，高迭代次数确保 GPU 暴力穷举
  /// 需要数天至数周。在中端手机上单次验证约 1–1.5 秒，可接受。
  static String _hash(String pin, String salt) {
    final saltBytes = utf8.encode(salt);
    final pinBytes = utf8.encode(pin);
    return _pbkdf2HmacSha256(pinBytes, saltBytes, 1000000, 32);
  }

  static String _pbkdf2HmacSha256(
    List<int> password,
    List<int> salt,
    int iterations,
    int keyLength,
  ) {
    final numBlocks = (keyLength + 31) ~/ 32;
    final result = <int>[];
    for (var block = 1; block <= numBlocks; block++) {
      final blockBytes = [
        ...salt,
        (block >> 24) & 0xff,
        (block >> 16) & 0xff,
        (block >> 8) & 0xff,
        block & 0xff,
      ];
      final hmac = Hmac(sha256, password);
      var u = hmac.convert(blockBytes).bytes;
      var xor = List<int>.from(u);
      for (var i = 1; i < iterations; i++) {
        u = Hmac(sha256, password).convert(u).bytes;
        for (var j = 0; j < xor.length; j++) {
          xor[j] ^= u[j];
        }
      }
      result.addAll(xor);
    }
    return result
        .sublist(0, keyLength)
        .map((b) => b.toRadixString(16).padLeft(2, '0'))
        .join();
  }

  static Future<int> _readInt(String key) async {
    final str = await _secure.read(key: key);
    if (str == null) return 0;
    return int.tryParse(str) ?? 0;
  }
}
