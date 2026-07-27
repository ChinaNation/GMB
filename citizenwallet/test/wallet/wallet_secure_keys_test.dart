import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/wallet/wallet_secure_keys.dart';

void main() {
  const masterA =
      '0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a';
  const masterB =
      '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48';

  group('WalletSecureKeys', () {
    test('masterSeedHexV1 / masterMnemonicV1 键格式', () {
      expect(WalletSecureKeys.masterSeedHexV1(masterA),
          'wallet.master.$masterA.seed_hex.v1');
      expect(WalletSecureKeys.masterMnemonicV1(masterA),
          'wallet.master.$masterA.mnemonic.v1');
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
      expect(() => WalletSecureKeys.masterSeedHexV1('nope'), throwsArgumentError);
      expect(() => WalletSecureKeys.masterSeedHexV1('0xABC'), throwsArgumentError);
      expect(
        () => WalletSecureKeys.masterMnemonicV1('${masterA}extra'),
        throwsArgumentError,
      );
    });
  });
}
