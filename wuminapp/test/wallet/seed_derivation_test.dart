import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:substrate_bip39/crypto_scheme.dart';

/// 关键测试：验证 mnemonic → seed → fromSeed() 与 fromMnemonic() 产出相同公钥。
///
/// 这确保 WalletManager 存储 seed 而非助记词后，
/// 仍然能派生出与原始助记词相同的密钥对。
void main() {
  group('seed derivation consistency', () {
    const testMnemonics = <String>[
      'legal winner thank year wave sausage worth useful legal winner thank yellow',
      'bottom drive obey lake curtain smoke basket hold race lonely fit walk',
      'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about',
    ];

    for (final mnemonic in testMnemonics) {
      test('fromSeed and fromMnemonic produce same pubkey for: "${mnemonic.substring(0, 30)}..."',
          () async {
        // 路径 A：fromMnemonic（polkadart_keyring 内部完整派生）。
        final pairFromMnemonic = await Keyring.sr25519.fromMnemonic(mnemonic);
        pairFromMnemonic.ss58Format = 2027;
        final pubkeyFromMnemonic =
            pairFromMnemonic.bytes().toList(growable: false);
        final addressFromMnemonic = pairFromMnemonic.address;

        // 路径 B：mnemonic → entropy → miniSecret → fromSeed。
        final entropy = bip39m
            .Mnemonic.fromSentence(mnemonic, bip39m.Language.english)
            .entropy;
        final miniSecret =
            await CryptoScheme.miniSecretFromEntropy(entropy);
        final pairFromSeed =
            Keyring.sr25519.fromSeed(Uint8List.fromList(miniSecret));
        pairFromSeed.ss58Format = 2027;
        final pubkeyFromSeed =
            pairFromSeed.bytes().toList(growable: false);
        final addressFromSeed = pairFromSeed.address;

        // 验证两条路径产出一致。
        expect(pubkeyFromSeed, equals(pubkeyFromMnemonic));
        expect(addressFromSeed, equals(addressFromMnemonic));
        expect(miniSecret.length, 32);
      });
    }
  });
}
