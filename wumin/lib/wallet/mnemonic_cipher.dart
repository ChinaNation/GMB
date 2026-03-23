import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:pointycastle/export.dart';

/// 助记词 AES-256-GCM 加密/解密。
///
/// 使用应用级随机加密密钥（AEK）对助记词进行 AES-256-GCM 加密。
/// AEK 在首次使用时自动生成并存储在 SecureStorage 的独立键下，
/// 与助记词条目分离存储，防止部分 SecureStorage 泄露导致助记词暴露。
///
/// 存储格式：Base64(iv[12] + ciphertext[...] + tag[16])
class MnemonicCipher {
  const MnemonicCipher._();

  static const FlutterSecureStorage _secure = FlutterSecureStorage();
  static const String _aekKey = 'wallet.internal.aek.v1';
  static const int _ivLen = 12;
  static const int _tagLen = 16;
  static const int _keyLen = 32;

  /// 缓存的 AEK，避免每次读写都访问 SecureStorage。
  static Uint8List? _cachedAek;

  /// 用 AEK 加密助记词，返回 Base64 密文。
  static Future<String> encrypt(String mnemonic) async {
    final key = await _ensureAek();
    final iv = _randomBytes(_ivLen);
    final plaintext = Uint8List.fromList(utf8.encode(mnemonic));

    try {
      final cipher = GCMBlockCipher(AESEngine())
        ..init(
          true,
          AEADParameters(
            KeyParameter(key),
            _tagLen * 8,
            iv,
            Uint8List(0),
          ),
        );

      final output = Uint8List(cipher.getOutputSize(plaintext.length));
      final len =
          cipher.processBytes(plaintext, 0, plaintext.length, output, 0);
      final totalLen = len + cipher.doFinal(output, len);

      // 拼接：iv + 实际输出（ciphertext + tag）
      final result = Uint8List(_ivLen + totalLen);
      result.setRange(0, _ivLen, iv);
      result.setRange(_ivLen, result.length, output.sublist(0, totalLen));

      return base64Encode(result);
    } finally {
      plaintext.fillRange(0, plaintext.length, 0);
    }
  }

  /// 用 AEK 解密助记词。数据损坏或 AEK 不匹配时抛出异常。
  static Future<String> decrypt(String cipherBase64) async {
    final data = base64Decode(cipherBase64);
    if (data.length < _ivLen + _tagLen + 1) {
      throw const FormatException('助记词密文数据损坏');
    }

    final key = await _ensureAek();
    final iv = Uint8List.sublistView(data, 0, _ivLen);
    final ciphertextAndTag = Uint8List.sublistView(data, _ivLen);

    try {
      final cipher = GCMBlockCipher(AESEngine())
        ..init(
          false,
          AEADParameters(
            KeyParameter(key),
            _tagLen * 8,
            iv,
            Uint8List(0),
          ),
        );

      final output = Uint8List(cipher.getOutputSize(ciphertextAndTag.length));
      final len = cipher.processBytes(
        ciphertextAndTag, 0, ciphertextAndTag.length, output, 0,
      );
      final totalLen = len + cipher.doFinal(output, len);

      return utf8.decode(output.sublist(0, totalLen));
    } on InvalidCipherTextException {
      throw const FormatException('助记词密文已损坏或密钥不匹配');
    }
  }

  /// 判断存储值是否为加密格式。
  ///
  /// 明文助记词是空格分隔的英文单词，加密密文是 Base64。
  /// 用于透明迁移：旧版本存储的明文可自动升级为加密格式。
  static bool isEncrypted(String value) {
    // 明文助记词含空格，Base64 不含空格
    if (value.contains(' ')) return false;
    if (value.length < 40) return false;
    try {
      final decoded = base64Decode(value);
      return decoded.length >= _ivLen + _tagLen + 1;
    } catch (_) {
      return false;
    }
  }

  /// 获取或生成 AEK。
  static Future<Uint8List> _ensureAek() async {
    final cached = _cachedAek;
    if (cached != null) return cached;

    final stored = await _secure.read(key: _aekKey);
    if (stored != null && stored.length == _keyLen * 2) {
      final key = _hexToBytes(stored);
      _cachedAek = key;
      return key;
    }

    // 首次使用，生成随机 AEK
    final newKey = _randomBytes(_keyLen);
    await _secure.write(key: _aekKey, value: _toHex(newKey));
    _cachedAek = newKey;
    return newKey;
  }

  /// 清除缓存（仅用于数据清空场景）。
  static void clearCache() {
    final cached = _cachedAek;
    if (cached != null) {
      cached.fillRange(0, cached.length, 0);
      _cachedAek = null;
    }
  }

  static Uint8List _randomBytes(int length) {
    final random = Random.secure();
    return Uint8List.fromList(
      List<int>.generate(length, (_) => random.nextInt(256)),
    );
  }

  static Uint8List _hexToBytes(String hex) {
    return Uint8List.fromList(List<int>.generate(
      hex.length ~/ 2,
      (i) => int.parse(hex.substring(i * 2, i * 2 + 2), radix: 16),
    ));
  }

  static String _toHex(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
