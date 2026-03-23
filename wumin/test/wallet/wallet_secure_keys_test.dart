import 'package:flutter_test/flutter_test.dart';
import 'package:wumin/wallet/wallet_secure_keys.dart';

void main() {
  group('WalletSecureKeys', () {
    test('seedHexV1 生成正确的键格式', () {
      expect(WalletSecureKeys.seedHexV1(1), 'wallet.secret.1.seed_hex.v1');
      expect(WalletSecureKeys.seedHexV1(99), 'wallet.secret.99.seed_hex.v1');
    });

    test('mnemonicV1 生成正确的键格式', () {
      expect(WalletSecureKeys.mnemonicV1(1), 'wallet.secret.1.mnemonic.v1');
      expect(WalletSecureKeys.mnemonicV1(42), 'wallet.secret.42.mnemonic.v1');
    });

    test('seedHexV1 拒绝非正数 walletId', () {
      expect(() => WalletSecureKeys.seedHexV1(0), throwsArgumentError);
      expect(() => WalletSecureKeys.seedHexV1(-1), throwsArgumentError);
    });

    test('mnemonicV1 拒绝非正数 walletId', () {
      expect(() => WalletSecureKeys.mnemonicV1(0), throwsArgumentError);
      expect(() => WalletSecureKeys.mnemonicV1(-5), throwsArgumentError);
    });

    test('不同 walletId 生成不同键', () {
      expect(
        WalletSecureKeys.seedHexV1(1) != WalletSecureKeys.seedHexV1(2),
        isTrue,
      );
      expect(
        WalletSecureKeys.mnemonicV1(1) != WalletSecureKeys.mnemonicV1(2),
        isTrue,
      );
    });

    test('seedHexV1 和 mnemonicV1 同 walletId 键不冲突', () {
      expect(
        WalletSecureKeys.seedHexV1(1) != WalletSecureKeys.mnemonicV1(1),
        isTrue,
      );
    });
  });
}
