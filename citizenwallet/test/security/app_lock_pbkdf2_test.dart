import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';

/// PBKDF2-HMAC-SHA256 标准测试向量验证。
///
/// 与 AppLockService 中的 _pbkdf2HmacSha256 实现逻辑相同，
/// 此处独立实现以交叉验证正确性。
/// 测试向量来源：RFC 6070 / NIST SP 800-132。
void main() {
  group('PBKDF2-HMAC-SHA256', () {
    test('RFC 6070 向量: password="password", salt="salt", c=1, dkLen=32',
        () {
      final result = _pbkdf2HmacSha256(
        utf8.encode('password'),
        utf8.encode('salt'),
        1,
        32,
      );
      expect(
        result,
        '120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b',
      );
    });

    test('RFC 6070 向量: password="password", salt="salt", c=2, dkLen=32',
        () {
      final result = _pbkdf2HmacSha256(
        utf8.encode('password'),
        utf8.encode('salt'),
        2,
        32,
      );
      expect(
        result,
        'ae4d0c95af6b46d32d0adff928f06dd02a303f8ef3c251dfd6e2d85a95474c43',
      );
    });

    test('RFC 6070 向量: password="password", salt="salt", c=4096, dkLen=32',
        () {
      final result = _pbkdf2HmacSha256(
        utf8.encode('password'),
        utf8.encode('salt'),
        4096,
        32,
      );
      expect(
        result,
        'c5e478d59288c841aa530db6845c4c8d962893a001ce4e11a4963873aa98134a',
      );
    });

    test('短密码短盐', () {
      final result = _pbkdf2HmacSha256(
        utf8.encode('1'),
        utf8.encode('s'),
        1,
        32,
      );
      // 确保不会崩溃，且产出 64 字符 hex
      expect(result.length, 64);
      expect(RegExp(r'^[0-9a-f]{64}$').hasMatch(result), isTrue);
    });

    test('输出长度可指定', () {
      final r16 = _pbkdf2HmacSha256(utf8.encode('p'), utf8.encode('s'), 1, 16);
      final r64 = _pbkdf2HmacSha256(utf8.encode('p'), utf8.encode('s'), 1, 64);
      expect(r16.length, 32); // 16 字节 = 32 hex
      expect(r64.length, 128); // 64 字节 = 128 hex
      // r16 应是 r64 的前缀（PBKDF2 块拼接特性）
      // 注意：仅当 dkLen <= 32 时第一块相同，dkLen > 32 时会有第二块
      expect(r64.startsWith(r16), isTrue);
    });
  });
}

/// 与 AppLockService._pbkdf2HmacSha256 完全相同的实现。
String _pbkdf2HmacSha256(
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
