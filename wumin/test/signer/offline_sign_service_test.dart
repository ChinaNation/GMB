import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:sr25519/sr25519.dart' as sr25519;
import 'package:substrate_bip39/crypto_scheme.dart';
import 'package:wumin/signer/offline_sign_service.dart';
import 'package:wumin/signer/pallet_registry.dart';
import 'package:wumin/signer/qr_signer.dart';
import 'package:wumin/wallet/wallet_manager.dart';

void main() {
  group('OfflineSignService', () {
    late _FakeWalletManager walletManager;
    late OfflineSignService service;
    late WalletProfile hotWallet;

    setUp(() async {
      walletManager = _FakeWalletManager();
      service = OfflineSignService(walletManager: walletManager);
      hotWallet = (await walletManager.getWalletByIndex(1))!;
    });

    test('signParsedRequest should sign matching request with hot wallet',
        () async {
      final request = QrSigner().buildRequest(
        requestId: 'offline-req-test-0001',
        account: hotWallet.address,
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: '0x01020304',
        display: const <String, dynamic>{
          'action': 'test',
          'summary': 'test payload',
        },
      );

      final response = await service.signParsedRequest(
        walletIndex: hotWallet.walletIndex,
        request: request,
        acknowledgeDecodeFailed: true,
      );

      expect(response.requestId, request.requestId);
      expect(response.pubkey, '0x${hotWallet.pubkeyHex}');
      expect(
        _verifySr25519(
          pubkeyHex: response.pubkey,
          message: Uint8List.fromList(<int>[1, 2, 3, 4]),
          signatureHex: response.signature,
        ),
        isTrue,
      );
    });

    test('verifyPayload returns decodeFailed for unknown specVersion', () {
      final request = QrSigner().buildRequest(
        requestId: 'offline-req-test-spec',
        account: hotWallet.address,
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: '0x0203000102030405060708091011121314151617181920212223242526272829303132330401',
        specVersion: 999,
        display: const <String, dynamic>{
          'action': 'transfer',
          'summary': 'test transfer',
        },
      );

      final verification = service.verifyPayload(request);
      expect(verification.displayMatch, DisplayMatchStatus.decodeFailed);
      expect(verification.decoded, isNull);
    });

    test('verifyPayload decodes known specVersion', () {
      // Balances::transfer_keep_alive: pallet=2, call=3
      // MultiAddress::Id prefix=0x00, then 32 bytes dest, then compact amount
      final knownSpecVersion = PalletRegistry.supportedSpecVersions.first;
      final request = QrSigner().buildRequest(
        requestId: 'offline-req-test-known',
        account: hotWallet.address,
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: '0x020300aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0491',
        specVersion: knownSpecVersion,
        display: const <String, dynamic>{
          'action': 'transfer',
          'action_label': '转账',
          'summary': 'test transfer',
          'fields': [
            {'key': 'amount_yuan', 'label': '金额', 'value': '0.01 GMB', 'format': 'currency'},
          ],
        },
      );

      final verification = service.verifyPayload(request);
      // Should decode successfully (matched or at least not null)
      expect(verification.decoded, isNotNull);
      expect(verification.decoded!.action, 'transfer');
    });

    test('signParsedRequest should reject mismatched pubkey', () async {
      final request = QrSigner().buildRequest(
        requestId: 'offline-req-test-0002',
        account: hotWallet.address,
        pubkey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x0102',
        display: const <String, dynamic>{
          'action': 'login',
          'summary': 'test login',
        },
      );

      expect(
        () => service.signParsedRequest(
          walletIndex: hotWallet.walletIndex,
          request: request,
        ),
        throwsA(
          isA<OfflineSignException>().having(
            (e) => e.code,
            'code',
            OfflineSignErrorCode.walletMismatch,
          ),
        ),
      );
    });
  });
}

bool _verifySr25519({
  required String pubkeyHex,
  required Uint8List message,
  required String signatureHex,
}) {
  try {
    final publicKey = sr25519.PublicKey.newPublicKey(_hexToBytes(pubkeyHex));
    final signature = sr25519.Signature.fromBytes(
      Uint8List.fromList(_hexToBytes(signatureHex)),
    );
    final (verified, _) =
        sr25519.Sr25519.verify(publicKey, signature, message);
    return verified;
  } catch (_) {
    return false;
  }
}

List<int> _hexToBytes(String input) {
  final text =
      (input.startsWith('0x') || input.startsWith('0X'))
          ? input.substring(2)
          : input;
  if (text.isEmpty || text.length.isOdd) return const <int>[];
  return List<int>.generate(
    text.length ~/ 2,
    (i) => int.parse(text.substring(i * 2, i * 2 + 2), radix: 16),
    growable: false,
  );
}

String _toHex(List<int> bytes) {
  const chars = '0123456789abcdef';
  final buf = StringBuffer();
  for (final b in bytes) {
    buf
      ..write(chars[(b >> 4) & 0x0f])
      ..write(chars[b & 0x0f]);
  }
  return buf.toString();
}

class _FakeWalletManager extends WalletManager {
  static const int _ss58 = 2027;
  static const String _mnemonic =
      'bottom drive obey lake curtain smoke basket hold race lonely fit walk';

  _WalletFixture? _hotFixture;

  Future<_WalletFixture> _ensureHotFixture() async {
    final existing = _hotFixture;
    if (existing != null) {
      return existing;
    }

    final entropy =
        bip39m.Mnemonic.fromSentence(_mnemonic, bip39m.Language.english)
            .entropy;
    final miniSecret = await CryptoScheme.miniSecretFromEntropy(entropy);
    final pair = Keyring.sr25519.fromSeed(Uint8List.fromList(miniSecret));
    pair.ss58Format = _ss58;
    final pubkeyHex = _toHex(pair.bytes().toList(growable: false));

    _hotFixture = _WalletFixture(
      profile: WalletProfile(
        walletIndex: 1,
        walletName: '离线测试热钱包',
        walletIcon: 'wallet',
        balance: 0,
        address: pair.address,
        pubkeyHex: pubkeyHex,
        alg: 'sr25519',
        ss58: _ss58,
        createdAtMillis: DateTime.now().millisecondsSinceEpoch,
        source: 'test',
        signMode: 'local',
      ),
      seedHex: _toHex(miniSecret),
    );
    return _hotFixture!;
  }

  @override
  Future<WalletProfile?> getWalletByIndex(int walletIndex) async {
    final hot = await _ensureHotFixture();
    if (walletIndex == hot.profile.walletIndex) {
      return hot.profile;
    }
    return null;
  }

  @override
  Future<Uint8List> signWithWallet(int walletIndex, Uint8List payload) async {
    final hot = await _ensureHotFixture();
    if (walletIndex != hot.profile.walletIndex) {
      throw const WalletAuthException('未找到指定钱包');
    }
    final seedBytes = _hexToBytes(hot.seedHex);
    final pair = Keyring.sr25519.fromSeed(Uint8List.fromList(seedBytes));
    pair.ss58Format = _ss58;
    return Uint8List.fromList(pair.sign(payload));
  }
}

class _WalletFixture {
  const _WalletFixture({
    required this.profile,
    required this.seedHex,
  });

  final WalletProfile profile;
  final String seedHex;
}
