import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

/// 本地静止态统一加密器（AES-256-GCM）。
///
/// 全 App 落盘密文只允许经本类产生，禁止各模块自行拼 nonce / 选算法。密钥一律
/// 来自 `local_data_key.dart` 的用途子钥，本类不接触账户密钥、不做密钥派生。
///
/// 密文格式（单串）：`base64( nonce[12] || cipherText || mac[16] )`。
/// nonce 每次随机 12 字节；GCM tag 即完整性校验，不再另加 sha256。
///
/// **AAD 必填**：调用方必须传入能唯一标识该记录的字符串（用途域 + 记录主键），
/// 把密文钉死在它的位置上，防止把 A 记录的密文搬到 B 记录位置的重放/串位。
class LocalCipher {
  const LocalCipher._();

  static final AesGcm _algorithm = AesGcm.with256bits();
  static final Random _random = Random.secure();

  /// GCM 推荐 nonce 长度；固定 12 字节。
  static const int nonceLength = 12;

  /// GCM tag 长度（128 bit）。
  static const int macLength = 16;

  /// 加密字符串明文，返回单串密文。
  static Future<String> encryptString({
    required List<int> key,
    required String plaintext,
    required String aad,
  }) =>
      encryptBytes(key: key, plaintext: utf8.encode(plaintext), aad: aad);

  /// 解密单串密文并按 UTF-8 还原字符串。
  static Future<String> decryptString({
    required List<int> key,
    required String blob,
    required String aad,
  }) async =>
      utf8.decode(await decryptBytes(key: key, blob: blob, aad: aad));

  /// 加密任意字节明文，返回单串密文。
  static Future<String> encryptBytes({
    required List<int> key,
    required List<int> plaintext,
    required String aad,
  }) async {
    _requireKey(key);
    final nonce = randomBytes(nonceLength);
    final box = await _algorithm.encrypt(
      plaintext,
      secretKey: SecretKey(key),
      nonce: nonce,
      aad: utf8.encode(aad),
    );
    final out = Uint8List(nonceLength + box.cipherText.length + macLength)
      ..setAll(0, nonce)
      ..setAll(nonceLength, box.cipherText)
      ..setAll(nonceLength + box.cipherText.length, box.mac.bytes);
    return base64Encode(out);
  }

  /// 解密单串密文。密钥错误、AAD 不符或密文被篡改一律抛
  /// [LocalCipherException]，**绝不静默返回空值**。
  static Future<Uint8List> decryptBytes({
    required List<int> key,
    required String blob,
    required String aad,
  }) async {
    _requireKey(key);
    final Uint8List raw;
    try {
      raw = base64Decode(blob);
    } on FormatException catch (error) {
      throw LocalCipherException('本地密文不是合法 base64：${error.message}');
    }
    if (raw.length < nonceLength + macLength) {
      throw LocalCipherException(
        '本地密文长度无效：至少 ${nonceLength + macLength} 字节，实际 ${raw.length}',
      );
    }
    final nonce = raw.sublist(0, nonceLength);
    final cipherText = raw.sublist(nonceLength, raw.length - macLength);
    final mac = raw.sublist(raw.length - macLength);
    try {
      final clear = await _algorithm.decrypt(
        SecretBox(cipherText, nonce: nonce, mac: Mac(mac)),
        secretKey: SecretKey(key),
        aad: utf8.encode(aad),
      );
      return Uint8List.fromList(clear);
    } on SecretBoxAuthenticationError {
      throw const LocalCipherException('本地密文校验失败：密钥不匹配或密文被篡改');
    }
  }

  /// 生成密码学安全随机字节（nonce / 随机密钥共用）。
  static Uint8List randomBytes(int length) {
    final out = Uint8List(length);
    for (var i = 0; i < length; i += 1) {
      out[i] = _random.nextInt(256);
    }
    return out;
  }

  static void _requireKey(List<int> key) {
    if (key.length != 32) {
      throw LocalCipherException('本地加密密钥必须为 32 字节，实际 ${key.length}');
    }
  }
}

/// 本地加解密失败。调用方必须显式处理，不允许吞掉。
class LocalCipherException implements Exception {
  const LocalCipherException(this.message);

  final String message;

  @override
  String toString() => 'LocalCipherException: $message';
}
