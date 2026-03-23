import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wumin/wallet/mnemonic_cipher.dart';

void main() {
  // flutter_secure_storage 在测试环境下无法真正读写，
  // 但 MnemonicCipher 内部会在首次调用 _ensureAek() 时生成 AEK
  // 并尝试写入 SecureStorage。为绕过此限制，先通过一次
  // encrypt 调用让 AEK 生成到内存缓存中（SecureStorage 写入会
  // 在测试环境中失败，但缓存有效即可）。
  //
  // 注意：此测试仅覆盖加密/解密逻辑正确性，不覆盖持久化。

  group('MnemonicCipher', () {
    // 手动触发 AEK 生成到缓存
    setUpAll(() async {
      FlutterSecureStorage.setMockInitialValues({});
      // 首次 encrypt 会在内存中缓存 AEK
      await MnemonicCipher.encrypt('init');
    });

    tearDownAll(() {
      MnemonicCipher.clearCache();
    });

    test('加密后解密得到原文', () async {
      const mnemonic =
          'bottom drive obey lake curtain smoke basket hold race lonely fit walk';
      final encrypted = await MnemonicCipher.encrypt(mnemonic);
      final decrypted = await MnemonicCipher.decrypt(encrypted);
      expect(decrypted, mnemonic);
    });

    test('每次加密产生不同密文（IV 不同）', () async {
      const mnemonic = 'abandon abandon abandon abandon abandon about';
      final e1 = await MnemonicCipher.encrypt(mnemonic);
      final e2 = await MnemonicCipher.encrypt(mnemonic);
      expect(e1, isNot(equals(e2)));

      // 两个不同密文解密后都得到相同明文
      final d1 = await MnemonicCipher.decrypt(e1);
      final d2 = await MnemonicCipher.decrypt(e2);
      expect(d1, mnemonic);
      expect(d2, mnemonic);
    });

    test('isEncrypted 正确识别加密密文', () async {
      const mnemonic = 'zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong';
      final encrypted = await MnemonicCipher.encrypt(mnemonic);
      expect(MnemonicCipher.isEncrypted(encrypted), isTrue);
    });

    test('isEncrypted 正确识别明文助记词', () {
      const plaintext =
          'bottom drive obey lake curtain smoke basket hold race lonely fit walk';
      expect(MnemonicCipher.isEncrypted(plaintext), isFalse);
    });

    test('isEncrypted 对空字符串返回 false', () {
      expect(MnemonicCipher.isEncrypted(''), isFalse);
    });

    test('isEncrypted 对短字符串返回 false', () {
      expect(MnemonicCipher.isEncrypted('abc'), isFalse);
    });

    test('解密被篡改的密文抛出异常', () async {
      const mnemonic = 'test mnemonic words here only for testing';
      final encrypted = await MnemonicCipher.encrypt(mnemonic);

      // 篡改密文中的一个字符
      final tampered = '${encrypted.substring(0, 10)}X${encrypted.substring(11)}';
      expect(
        () => MnemonicCipher.decrypt(tampered),
        throwsA(isA<FormatException>()),
      );
    });

    test('解密过短的数据抛出异常', () {
      expect(
        () => MnemonicCipher.decrypt('AAAA'),
        throwsA(isA<FormatException>()),
      );
    });

    test('clearCache 后重新加密仍可解密', () async {
      const mnemonic = 'abandon abandon abandon abandon about';
      final encrypted = await MnemonicCipher.encrypt(mnemonic);
      MnemonicCipher.clearCache();

      // clearCache 清掉了内存中的 AEK，重新 encrypt 会从 SecureStorage 重读
      // 测试环境下 SecureStorage 有 mock，AEK 应该已被写入 mock
      final decrypted = await MnemonicCipher.decrypt(encrypted);
      expect(decrypted, mnemonic);
    });

    test('支持中文和特殊字符', () async {
      const text = '测试助记词 with émojis 🔐';
      final encrypted = await MnemonicCipher.encrypt(text);
      final decrypted = await MnemonicCipher.decrypt(encrypted);
      expect(decrypted, text);
    });

    test('24 词助记词加密解密', () async {
      const mnemonic =
          'abandon abandon abandon abandon abandon abandon abandon abandon '
          'abandon abandon abandon abandon abandon abandon abandon abandon '
          'abandon abandon abandon abandon abandon abandon abandon art';
      final encrypted = await MnemonicCipher.encrypt(mnemonic);
      final decrypted = await MnemonicCipher.decrypt(encrypted);
      expect(decrypted, mnemonic);
    });
  });
}
