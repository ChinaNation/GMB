import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'dart:convert';
import 'package:citizenwallet/wallet/secret_cipher.dart';

void main() {
  const associatedData = 'wallet.master.test.seed_hex.v1';

  group('SecretCipher', () {
    setUp(() {
      FlutterSecureStorage.setMockInitialValues({});
      SecretCipher.clearCache();
    });

    tearDown(() {
      SecretCipher.clearCache();
    });

    test('首次并发加密只创建并复用一个可持久化 AEK', () async {
      final encrypted = await Future.wait([
        SecretCipher.encrypt('first', associatedData: associatedData),
        SecretCipher.encrypt('second', associatedData: associatedData),
      ]);
      SecretCipher.clearCache();

      expect(
        await SecretCipher.decrypt(
          encrypted[0],
          associatedData: associatedData,
        ),
        'first',
      );
      expect(
        await SecretCipher.decrypt(
          encrypted[1],
          associatedData: associatedData,
        ),
        'second',
      );
    });

    test('解密路径在 AEK 缺失时绝不创建新 AEK', () async {
      expect(
        () => SecretCipher.decrypt(
          base64Encode(List<int>.filled(29, 1)),
          associatedData: associatedData,
        ),
        throwsA(isA<FormatException>()),
      );
      const storage = FlutterSecureStorage();
      expect(await storage.read(key: 'wallet.internal.aek.v1'), isNull);
    });

    test('存在钱包密文但 AEK 缺失时禁止创建', () async {
      FlutterSecureStorage.setMockInitialValues({
        'wallet.master.0x${'11' * 32}.seed_hex.v1': 'orphaned',
      });
      SecretCipher.clearCache();
      expect(
        () => SecretCipher.encrypt(
          'must fail closed',
          associatedData: associatedData,
        ),
        throwsA(isA<FormatException>()),
      );
    });

    test('非法 AEK 不会被覆盖', () async {
      FlutterSecureStorage.setMockInitialValues({
        'wallet.internal.aek.v1': 'invalid',
      });
      SecretCipher.clearCache();
      expect(
        () => SecretCipher.encrypt(
          'must fail closed',
          associatedData: associatedData,
        ),
        throwsA(isA<FormatException>()),
      );
      const storage = FlutterSecureStorage();
      expect(await storage.read(key: 'wallet.internal.aek.v1'), 'invalid');
    });

    test('加密后解密得到原文', () async {
      const mnemonic =
          'bottom drive obey lake curtain smoke basket hold race lonely fit walk';
      final encrypted = await SecretCipher.encrypt(
        mnemonic,
        associatedData: associatedData,
      );
      final decrypted = await SecretCipher.decrypt(
        encrypted,
        associatedData: associatedData,
      );
      expect(decrypted, mnemonic);
    });

    test('主种子 hex 也能加密解密（与助记词同威胁模型）', () async {
      const seedHex =
          '46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a';
      final encrypted = await SecretCipher.encrypt(
        seedHex,
        associatedData: associatedData,
      );
      // 密文不得是明文种子。
      expect(encrypted, isNot(equals(seedHex)));
      final decrypted = await SecretCipher.decrypt(
        encrypted,
        associatedData: associatedData,
      );
      expect(decrypted, seedHex);
    });

    test('每次加密产生不同密文（IV 不同）', () async {
      const mnemonic = 'abandon abandon abandon abandon abandon about';
      final e1 =
          await SecretCipher.encrypt(mnemonic, associatedData: associatedData);
      final e2 =
          await SecretCipher.encrypt(mnemonic, associatedData: associatedData);
      expect(e1, isNot(equals(e2)));

      // 两个不同密文解密后都得到相同明文
      final d1 = await SecretCipher.decrypt(e1, associatedData: associatedData);
      final d2 = await SecretCipher.decrypt(e2, associatedData: associatedData);
      expect(d1, mnemonic);
      expect(d2, mnemonic);
    });

    test('解密被篡改的密文抛出异常', () async {
      const mnemonic = 'test mnemonic words here only for testing';
      final encrypted = await SecretCipher.encrypt(
        mnemonic,
        associatedData: associatedData,
      );

      // 先解码再翻转认证标签最后一个字节，确保每次都真实篡改。
      final bytes = base64Decode(encrypted);
      bytes[bytes.length - 1] ^= 0x01;
      final tampered = base64Encode(bytes);
      expect(
        () => SecretCipher.decrypt(
          tampered,
          associatedData: associatedData,
        ),
        throwsA(isA<FormatException>()),
      );
    });

    test('解密过短的数据抛出异常', () {
      expect(
        () => SecretCipher.decrypt(
          'AAAA',
          associatedData: associatedData,
        ),
        throwsA(isA<FormatException>()),
      );
    });

    test('clearCache 后重新加密仍可解密', () async {
      const mnemonic = 'abandon abandon abandon abandon about';
      final encrypted = await SecretCipher.encrypt(
        mnemonic,
        associatedData: associatedData,
      );
      SecretCipher.clearCache();

      // clearCache 清掉了内存中的 AEK，重新 encrypt 会从 SecureStorage 重读
      // 测试环境下 SecureStorage 有 mock，AEK 应该已被写入 mock
      final decrypted = await SecretCipher.decrypt(
        encrypted,
        associatedData: associatedData,
      );
      expect(decrypted, mnemonic);
    });

    test('加密单字符', () async {
      final encrypted =
          await SecretCipher.encrypt('a', associatedData: associatedData);
      final decrypted = await SecretCipher.decrypt(
        encrypted,
        associatedData: associatedData,
      );
      expect(decrypted, 'a');
    });

    test('加密超长字符串（模拟异常输入）', () async {
      final longText = 'word ' * 500; // 2500 字符
      final encrypted = await SecretCipher.encrypt(
        longText,
        associatedData: associatedData,
      );
      final decrypted = await SecretCipher.decrypt(
        encrypted,
        associatedData: associatedData,
      );
      expect(decrypted, longText);
    });

    test('支持中文和特殊字符', () async {
      const text = '测试助记词 with émojis 🔐';
      final encrypted = await SecretCipher.encrypt(
        text,
        associatedData: associatedData,
      );
      final decrypted = await SecretCipher.decrypt(
        encrypted,
        associatedData: associatedData,
      );
      expect(decrypted, text);
    });

    test('24 词助记词加密解密', () async {
      const mnemonic =
          'abandon abandon abandon abandon abandon abandon abandon abandon '
          'abandon abandon abandon abandon abandon abandon abandon abandon '
          'abandon abandon abandon abandon abandon abandon abandon art';
      final encrypted = await SecretCipher.encrypt(
        mnemonic,
        associatedData: associatedData,
      );
      final decrypted = await SecretCipher.decrypt(
        encrypted,
        associatedData: associatedData,
      );
      expect(decrypted, mnemonic);
    });

    test('关联数据不一致时拒绝解密', () async {
      final encrypted = await SecretCipher.encrypt(
        'secret',
        associatedData: associatedData,
      );
      expect(
        () => SecretCipher.decrypt(
          encrypted,
          associatedData: 'wallet.master.other.mnemonic.v1',
        ),
        throwsA(isA<FormatException>()),
      );
    });
  });
}
