import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:sr25519/sr25519.dart' as sr25519;
import 'package:substrate_bip39/crypto_scheme.dart';
import 'package:wumin/signer/offline_sign_service.dart';
import 'package:wumin/qr/qr_protocols.dart';
import 'package:wumin/qr/envelope.dart';
import 'package:wumin/qr/bodies/sign_request_body.dart';
import 'package:wumin/signer/qr_signer.dart';
import 'package:wumin/wallet/wallet_manager.dart';

/// 给纯 call_data 拼上真实 SigningPayload 扩展尾(与节点端 build_signing_payload
/// 布局一致)。decoder 的两色识别要求链上 payload 必带合法尾,裸 call_data 拒签。
String _withSigningTailHex(String callDataHex) {
  final genesis = List<int>.generate(32, (i) => 0x49 ^ i);
  final tail = <int>[
    0x00, // era: immortal
    0x04, // Compact(nonce=1)
    0x00, // Compact(tip=0)
    0x00, // CheckMetadataHash mode=Disabled
    1, 0, 0, 0, // spec_version u32 LE
    1, 0, 0, 0, // tx_version u32 LE
    ...genesis,
    ...genesis, // immortal: birth hash = genesis hash
    0x00, // CheckMetadataHash Option::None
  ];
  return '0x${_toHex([..._hexToBytes(callDataHex), ...tail])}';
}

SignRequestEnvelope _buildTestRequest({
  required String requestId,
  required String address,
  required String pubkey,
  required String payloadHex,
  required SignDisplay display,
}) {
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  return QrEnvelope<SignRequestBody>(
    kind: QrKind.signRequest,
    id: requestId,
    issuedAt: now,
    expiresAt: now + 90,
    body: SignRequestBody(
      address: address,
      pubkey: pubkey,
      sigAlg: 'sr25519',
      payloadHex: payloadHex,
      display: display,
    ),
  );
}

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

    test('signParsedRequest should sign matched internal_vote (统一入口)',
        () async {
      // 所有管理员投票走 InternalVote(22).cast(0)
      // payload = [0x16][0x00][u64 LE proposal_id=1][bool approve=1] + 扩展尾
      final payloadHex = _withSigningTailHex('0x1600010000000000000001');
      final request = _buildTestRequest(
        requestId: 'offline-req-test-0001',
        address: hotWallet.address,
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: payloadHex,
        display: const SignDisplay(
          action: 'internal_vote',
          summary: '管理员投票 提案 #1：赞成',
          fields: [
            // 两色识别模型要求 display.fields 的 key 与 decoder 输出逐字对齐。
            SignDisplayField(key: 'proposal_id', label: '提案', value: '1'),
            SignDisplayField(key: 'approve', label: '投票', value: 'true'),
          ],
        ),
      );

      final payloadBytes = _hexToBytes(payloadHex);

      final response = await service.signParsedRequest(
        walletIndex: hotWallet.walletIndex,
        request: request,
      );

      expect(response.id, request.id);
      expect(response.body.pubkey, '0x${hotWallet.pubkeyHex}');
      expect(
        _verifySr25519(
          pubkeyHex: response.body.pubkey,
          message: Uint8List.fromList(payloadBytes),
          signatureHex: response.body.signature,
        ),
        isTrue,
      );
    });

    test('signParsedRequest 拒绝 mismatched(action 不一致)', () async {
      // decode 成功但 display.action 和 decoded.action 不一致 → 红色拒签。
      final payloadHex = _withSigningTailHex('0x1600070000000000000001');
      final request = _buildTestRequest(
        requestId: 'offline-req-test-action-mismatch',
        address: hotWallet.address,
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: payloadHex,
        display: const SignDisplay(
          action: 'joint_vote', // decoder 会解码为 'internal_vote'
          summary: '恶意伪造',
          fields: [
            SignDisplayField(key: 'proposal_id', label: '提案', value: '7'),
            SignDisplayField(key: 'approve', label: '投票', value: 'true'),
          ],
        ),
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
            OfflineSignErrorCode.displayMismatch,
          ),
        ),
      );
    });

    test('verifyPayload decodes transfer payload', () {
      // Balances::transfer_keep_alive: pallet=2, call=3
      // MultiAddress::Id prefix=0x00, then 32 bytes dest, then compact amount
      final request = _buildTestRequest(
        requestId: 'offline-req-test-known',
        address: hotWallet.address,
        pubkey: '0x${hotWallet.pubkeyHex}',
        // call_data: [02][03][00][dest 32B][Compact(1) = 0x04] → 0.01 GMB
        payloadHex: _withSigningTailHex(
            '0x020300aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa04'),
        display: const SignDisplay(
          action: 'transfer',
          summary: 'test transfer',
          fields: [
            SignDisplayField(
                key: 'amount_yuan', label: '金额', value: '0.01 GMB'),
          ],
        ),
      );

      final verification = service.verifyPayload(request);
      // Should decode successfully (matched or at least not null)
      expect(verification.decoded, isNotNull);
      expect(verification.decoded!.action, 'transfer');
    });

    test('signParsedRequest should reject mismatched pubkey', () async {
      final request = _buildTestRequest(
        requestId: 'offline-req-test-0002',
        address: hotWallet.address,
        pubkey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x0102',
        display: const SignDisplay(
          action: 'login',
          summary: 'test login',
        ),
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
    final (verified, _) = sr25519.Sr25519.verify(publicKey, signature, message);
    return verified;
  } catch (_) {
    return false;
  }
}

List<int> _hexToBytes(String input) {
  final text = (input.startsWith('0x') || input.startsWith('0X'))
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
