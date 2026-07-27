import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/wallet/wallet_secure_keys.dart';

void main() {
  const accountA =
      '0x2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972';
  const accountB =
      '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48';

  group('WalletSecureKeys', () {
    test('accountMiniSecretV1 键格式', () {
      expect(WalletSecureKeys.accountMiniSecretV1(accountA),
          'wallet.account.$accountA.minisecret.v1');
    });

    test('不同 accountId 生成不同键', () {
      expect(
        WalletSecureKeys.accountMiniSecretV1(accountA) !=
            WalletSecureKeys.accountMiniSecretV1(accountB),
        isTrue,
      );
    });

    test('拒绝非规范 accountId', () {
      expect(() => WalletSecureKeys.accountMiniSecretV1('nope'),
          throwsArgumentError);
      expect(() => WalletSecureKeys.accountMiniSecretV1('0xABC'),
          throwsArgumentError);
      expect(
        () => WalletSecureKeys.accountMiniSecretV1('${accountA}extra'),
        throwsArgumentError,
      );
    });
  });
}
