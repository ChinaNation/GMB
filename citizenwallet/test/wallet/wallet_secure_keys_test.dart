import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/wallet/wallet_secure_keys.dart';

void main() {
  const masterA =
      '0x2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972';
  const masterB =
      '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48';

  group('WalletSecureKeys', () {
    test('masterSeedHexV1 / masterMnemonicV1 键格式', () {
      expect(WalletSecureKeys.masterSeedHexV1(masterA),
          'wallet.master.$masterA.seed_hex');
      expect(WalletSecureKeys.masterMnemonicV1(masterA),
          'wallet.master.$masterA.mnemonic');
    });

    test('不同 masterId 生成不同键', () {
      expect(
        WalletSecureKeys.masterSeedHexV1(masterA) !=
            WalletSecureKeys.masterSeedHexV1(masterB),
        isTrue,
      );
    });

    test('同 masterId 下 seed 与 mnemonic 键不冲突', () {
      expect(
        WalletSecureKeys.masterSeedHexV1(masterA) !=
            WalletSecureKeys.masterMnemonicV1(masterA),
        isTrue,
      );
    });

    test('拒绝非规范 masterId', () {
      expect(
          () => WalletSecureKeys.masterSeedHexV1('nope'), throwsArgumentError);
      expect(
          () => WalletSecureKeys.masterSeedHexV1('0xABC'), throwsArgumentError);
      expect(
        () => WalletSecureKeys.masterMnemonicV1('${masterA}extra'),
        throwsArgumentError,
      );
    });
  });
}
