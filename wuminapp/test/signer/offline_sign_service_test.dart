import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:substrate_bip39/crypto_scheme.dart';
import 'package:wuminapp_mobile/signer/offline_sign_service.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/signer/system_signature_verifier.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

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
        scope: QrSignScope.onchainTx,
        requestId: 'offline-req-1',
        account: hotWallet.address,
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: '0x01020304',
      );

      final response = await service.signParsedRequest(
        walletIndex: hotWallet.walletIndex,
        request: request,
      );

      expect(response.requestId, request.requestId);
      expect(response.pubkey, '0x${hotWallet.pubkeyHex}');
      expect(
        Sr25519MessageVerifier().verify(
          pubkeyHex: response.pubkey,
          message: Uint8List.fromList(<int>[1, 2, 3, 4]),
          signatureHex: response.signature,
        ),
        isTrue,
      );
    });

    test('signParsedRequest should reject mismatched pubkey', () async {
      final request = QrSigner().buildRequest(
        scope: QrSignScope.login,
        requestId: 'offline-req-2',
        account: hotWallet.address,
        pubkey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x0102',
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

    test('signParsedRequest should reject cold wallet signer', () async {
      final coldWallet = await walletManager.buildColdWalletProfile(9);
      final request = QrSigner().buildRequest(
        scope: QrSignScope.login,
        requestId: 'offline-req-3',
        account: coldWallet.address,
        pubkey: '0x${coldWallet.pubkeyHex}',
        payloadHex: '0x0102',
      );

      expect(
        () => service.signParsedRequest(
          walletIndex: coldWallet.walletIndex,
          request: request,
        ),
        throwsA(
          isA<OfflineSignException>().having(
            (e) => e.code,
            'code',
            OfflineSignErrorCode.coldWalletUnsupported,
          ),
        ),
      );
    });
  });
}

class _FakeWalletManager extends WalletManager {
  static const int _ss58 = 2027;
  static const String _mnemonic =
      'bottom drive obey lake curtain smoke basket hold race lonely fit walk';

  _WalletFixture? _hotFixture;
  WalletProfile? _coldWallet;

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

  Future<WalletProfile> buildColdWalletProfile(int walletIndex) async {
    final hot = await _ensureHotFixture();
    _coldWallet = WalletProfile(
      walletIndex: walletIndex,
      walletName: '离线测试冷钱包',
      walletIcon: 'wallet',
      balance: 0,
      address: hot.profile.address,
      pubkeyHex: hot.profile.pubkeyHex,
      alg: hot.profile.alg,
      ss58: hot.profile.ss58,
      createdAtMillis: hot.profile.createdAtMillis,
      source: 'test',
      signMode: 'external',
    );
    return _coldWallet!;
  }

  @override
  Future<WalletProfile?> getWalletByIndex(int walletIndex) async {
    final hot = await _ensureHotFixture();
    if (walletIndex == hot.profile.walletIndex) {
      return hot.profile;
    }
    if (_coldWallet != null && _coldWallet!.walletIndex == walletIndex) {
      return _coldWallet;
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

  List<int> _hexToBytes(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
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
}

class _WalletFixture {
  const _WalletFixture({
    required this.profile,
    required this.seedHex,
  });

  final WalletProfile profile;
  final String seedHex;
}
