import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:pointycastle/export.dart';

import '../security/secure_storage.dart';

/// 机密(助记词 / 主种子)AES-256-GCM 加密/解密。
///
/// 用应用级随机加密密钥(AEK)对机密做 AES-256-GCM 加密。AEK 在首次使用时
/// 自动生成并存进 SecureStorage 的独立键下,与被加密条目分离存放,防止部分
/// SecureStorage 泄露导致机密暴露。助记词与主种子共用同一 AEK/算法(同威胁模型)。
///
/// 存储格式:Base64(iv[12] + ciphertext[...] + tag[16])
class SecretCipher {
  const SecretCipher._();

  static const String _aekKey = 'wallet.internal.aek';
  static const int _ivLen = 12;
  static const int _tagLen = 16;
  static const int _keyLen = 32;

  /// 缓存的 AEK,避免每次读写都访问 SecureStorage。
  static Uint8List? _cachedAek;
  static Future<Uint8List>? _aekInitialization;
  static final RegExp _aekHexPattern = RegExp(r'^[0-9a-f]{64}$');
  static final RegExp _walletSecretKeyPattern =
      RegExp(r'^wallet\.master\.0x[0-9a-f]{64}\.(seed_hex|mnemonic)$');

  /// 用 AEK 加密明文机密,返回 Base64 密文。
  static Future<String> encrypt(
    String plaintextSecret, {
    required String associatedData,
  }) async {
    final key = await _loadAek(allowCreate: true);
    final iv = _randomBytes(_ivLen);
    final plaintext = Uint8List.fromList(utf8.encode(plaintextSecret));
    final aad = Uint8List.fromList(utf8.encode(associatedData));

    try {
      final cipher = GCMBlockCipher(AESEngine())
        ..init(
          true,
          AEADParameters(
            KeyParameter(key),
            _tagLen * 8,
            iv,
            aad,
          ),
        );

      final output = Uint8List(cipher.getOutputSize(plaintext.length));
      final len =
          cipher.processBytes(plaintext, 0, plaintext.length, output, 0);
      final totalLen = len + cipher.doFinal(output, len);

      // 拼接:iv + 实际输出(ciphertext + tag)
      final result = Uint8List(_ivLen + totalLen);
      result.setRange(0, _ivLen, iv);
      result.setRange(_ivLen, result.length, output.sublist(0, totalLen));

      return base64Encode(result);
    } finally {
      plaintext.fillRange(0, plaintext.length, 0);
    }
  }

  /// 用 AEK 解密机密。数据损坏或 AEK 不匹配时抛出 [FormatException]。
  static Future<String> decrypt(
    String cipherBase64, {
    required String associatedData,
  }) async {
    final data = base64Decode(cipherBase64);
    if (data.length < _ivLen + _tagLen + 1) {
      throw const FormatException('机密密文数据损坏');
    }

    // 解密路径绝不创建 AEK。AEK 缺失时必须显式失败，不能生成新密钥掩盖损坏。
    final key = await _loadAek(allowCreate: false);
    final iv = Uint8List.sublistView(data, 0, _ivLen);
    final ciphertextAndTag = Uint8List.sublistView(data, _ivLen);
    final aad = Uint8List.fromList(utf8.encode(associatedData));
    Uint8List? output;

    try {
      final cipher = GCMBlockCipher(AESEngine())
        ..init(
          false,
          AEADParameters(
            KeyParameter(key),
            _tagLen * 8,
            iv,
            aad,
          ),
        );

      output = Uint8List(cipher.getOutputSize(ciphertextAndTag.length));
      final len = cipher.processBytes(
        ciphertextAndTag,
        0,
        ciphertextAndTag.length,
        output,
        0,
      );
      final totalLen = len + cipher.doFinal(output, len);

      return utf8.decode(output.sublist(0, totalLen));
    } on InvalidCipherTextException {
      throw const FormatException('机密密文已损坏或密钥不匹配');
    } finally {
      output?.fillRange(0, output.length, 0);
    }
  }

  /// 串行读取 AEK；仅加密路径允许在“存储明确无 AEK 且无钱包机密”时创建。
  static Future<Uint8List> _loadAek({required bool allowCreate}) async {
    final cached = _cachedAek;
    if (cached != null) return cached;

    final inflight = _aekInitialization;
    if (inflight != null) {
      return inflight;
    }

    final task = _loadAekAtomic(allowCreate: allowCreate);
    _aekInitialization = task;
    try {
      return await task;
    } finally {
      if (identical(_aekInitialization, task)) {
        _aekInitialization = null;
      }
    }
  }

  static Future<Uint8List> _loadAekAtomic({
    required bool allowCreate,
  }) async {
    final stored = await appSecureStorage.read(key: _aekKey);
    if (stored != null) {
      if (!_aekHexPattern.hasMatch(stored)) {
        throw const FormatException('应用加密密钥格式异常');
      }
      final key = _hexToBytes(stored);
      _cachedAek = key;
      return key;
    }

    if (!allowCreate) {
      throw const FormatException('应用加密密钥不存在');
    }

    // 二次读取全部键，确认没有任何既有钱包密文；否则 AEK 丢失，禁止覆盖事故现场。
    final all = await appSecureStorage.readAll();
    if (all.keys.any(_walletSecretKeyPattern.hasMatch)) {
      throw const FormatException('检测到钱包密文但应用加密密钥不存在');
    }

    final newKey = _randomBytes(_keyLen);
    try {
      await appSecureStorage.write(key: _aekKey, value: _toHex(newKey));
      final persisted = await appSecureStorage.read(key: _aekKey);
      if (persisted != _toHex(newKey)) {
        throw const FormatException('应用加密密钥持久化校验失败');
      }
      _cachedAek = newKey;
      return newKey;
    } catch (_) {
      newKey.fillRange(0, newKey.length, 0);
      rethrow;
    }
  }

  /// 最后一只钱包删除后移除 AEK。只有确认不存在任何钱包密文时才允许删除。
  static Future<void> deleteAekIfNoWalletSecrets() async {
    final all = await appSecureStorage.readAll();
    if (all.keys.any(_walletSecretKeyPattern.hasMatch)) {
      throw StateError('仍存在钱包机密，不能删除应用加密密钥');
    }
    await appSecureStorage.delete(key: _aekKey);
    final persisted = await appSecureStorage.read(key: _aekKey);
    if (persisted != null) {
      throw StateError('应用加密密钥删除校验失败');
    }
    clearCache();
  }

  /// 清除缓存(仅用于数据清空场景)。
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
