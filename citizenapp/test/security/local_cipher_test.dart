import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/security/local_cipher.dart';

void main() {
  final key = Uint8List.fromList(List<int>.generate(32, (i) => i));
  final otherKey = Uint8List.fromList(List<int>.generate(32, (i) => 255 - i));

  group('LocalCipher 加解密往返', () {
    test('字符串明文往返一致', () async {
      const clear = '你好，公民 · Hello 😀';
      final blob = await LocalCipher.encryptString(
        key: key,
        plaintext: clear,
        aad: 'citizenapp.local/chat|msg-1',
      );
      final back = await LocalCipher.decryptString(
        key: key,
        blob: blob,
        aad: 'citizenapp.local/chat|msg-1',
      );
      expect(back, clear);
    });

    test('空明文也能往返', () async {
      final blob = await LocalCipher.encryptBytes(
        key: key,
        plaintext: const <int>[],
        aad: 'citizenapp.local/chat|empty',
      );
      final back = await LocalCipher.decryptBytes(
        key: key,
        blob: blob,
        aad: 'citizenapp.local/chat|empty',
      );
      expect(back, isEmpty);
    });

    test('密文不含明文子串', () async {
      const clear = 'SUPER_SECRET_MARKER';
      final blob = await LocalCipher.encryptString(
        key: key,
        plaintext: clear,
        aad: 'citizenapp.local/chat|m',
      );
      expect(blob.contains(clear), isFalse);
      expect(utf8.decode(base64Decode(blob), allowMalformed: true),
          isNot(contains(clear)));
    });
  });

  group('LocalCipher 拒绝路径（必须抛错，不得静默返回空）', () {
    late String blob;

    setUp(() async {
      blob = await LocalCipher.encryptString(
        key: key,
        plaintext: '正文',
        aad: 'citizenapp.local/chat|msg-1',
      );
    });

    test('错误密钥被拒', () async {
      await expectLater(
        LocalCipher.decryptBytes(
          key: otherKey,
          blob: blob,
          aad: 'citizenapp.local/chat|msg-1',
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });

    test('AAD 不符被拒（防串位重放）', () async {
      await expectLater(
        LocalCipher.decryptBytes(
          key: key,
          blob: blob,
          aad: 'citizenapp.local/chat|msg-2',
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });

    test('密文被篡改被拒', () async {
      final raw = base64Decode(blob);
      raw[raw.length - 1] ^= 0xFF;
      await expectLater(
        LocalCipher.decryptBytes(
          key: key,
          blob: base64Encode(raw),
          aad: 'citizenapp.local/chat|msg-1',
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });

    test('非法 base64 被拒', () async {
      await expectLater(
        LocalCipher.decryptBytes(
          key: key,
          blob: '不是base64!!!',
          aad: 'citizenapp.local/chat|msg-1',
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });

    test('长度不足被拒', () async {
      await expectLater(
        LocalCipher.decryptBytes(
          key: key,
          blob: base64Encode(List<int>.filled(10, 0)),
          aad: 'citizenapp.local/chat|msg-1',
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });

    test('密钥长度非 32 字节被拒', () async {
      await expectLater(
        LocalCipher.encryptString(
          key: Uint8List(16),
          plaintext: 'x',
          aad: 'a',
        ),
        throwsA(isA<LocalCipherException>()),
      );
    });
  });

  test('nonce 每次随机：同明文同密钥两次密文不同', () async {
    final a = await LocalCipher.encryptString(
      key: key,
      plaintext: '同样的话',
      aad: 'citizenapp.local/chat|same',
    );
    final b = await LocalCipher.encryptString(
      key: key,
      plaintext: '同样的话',
      aad: 'citizenapp.local/chat|same',
    );
    expect(a, isNot(b));
    expect(
      await LocalCipher.decryptString(
          key: key, blob: a, aad: 'citizenapp.local/chat|same'),
      await LocalCipher.decryptString(
          key: key, blob: b, aad: 'citizenapp.local/chat|same'),
    );
  });

  test('randomBytes 长度正确且不重复', () {
    final a = LocalCipher.randomBytes(32);
    final b = LocalCipher.randomBytes(32);
    expect(a.length, 32);
    expect(b.length, 32);
    expect(a, isNot(b));
  });
}
