import 'dart:typed_data';

import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:sr25519/sr25519.dart' as sr25519;
import 'package:substrate_bip39/crypto_scheme.dart';
import 'package:citizenwallet/signer/offline_sign_service.dart';
import 'package:citizenwallet/qr/qr_protocols.dart';
import 'package:citizenwallet/qr/envelope.dart';
import 'package:citizenwallet/qr/bodies/sign_request_body.dart';
import 'package:citizenwallet/signer/qr_signer.dart';
import 'package:citizenwallet/wallet/wallet_manager.dart';

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
  required String pubkey,
  required String payloadHex,
  required int action,
}) {
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  return QrEnvelope<SignRequestBody>(
    kind: QrKind.signRequest,
    id: requestId,
    issuedAt: now,
    expiresAt: now + 90,
    body: SignRequestBody.fromHex(
      action: action,
      pubkeyHex: pubkey,
      payloadHex: payloadHex,
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

    test('signParsedRequest should sign normal internal_vote (统一入口)', () async {
      // 所有管理员投票走 InternalVote(20).cast(0)
      // payload = [0x14][0x00][u64 LE proposal_id=1][bool approve=1] + 扩展尾
      final payloadHex = _withSigningTailHex('0x1400010000000000000001');
      final request = _buildTestRequest(
        requestId: 'offline-req-test-0001',
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: payloadHex,
        action: QrActions.internalVote,
      );

      final payloadBytes = _hexToBytes(payloadHex);

      final response = await service.signParsedRequest(
        walletIndex: hotWallet.walletIndex,
        request: request,
      );

      expect(walletManager.signCallCount, 1);
      expect(response.id, request.id);
      expect(response.body.pubkeyHex, '0x${hotWallet.pubkeyHex}');
      expect(
        _verifySr25519(
          pubkeyHex: response.body.pubkeyHex,
          message: Uint8List.fromList(payloadBytes),
          signatureHex: response.body.signatureHex,
        ),
        isTrue,
      );
    });

    test('signParsedRequest 拒绝 action 与 payload 不一致', () async {
      // decode 成功但 QR action 和 decoded.action 不一致 → 红色拒签。
      final payloadHex = _withSigningTailHex('0x1400070000000000000001');
      final request = _buildTestRequest(
        requestId: 'offline-req-test-action-mismatch',
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: payloadHex,
        action: QrActions.jointVote,
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
            OfflineSignErrorCode.contentMismatch,
          ),
        ),
      );
      expect(walletManager.signCallCount, 0);
    });

    test('verifyPayload decodes transfer payload', () {
      // OnchainTransaction::transfer_with_remark: pallet=4, call=0。
      // beneficiary 32B, amount u128_le, remark 空 Vec。
      final request = _buildTestRequest(
        requestId: 'offline-req-test-known',
        pubkey: '0x${hotWallet.pubkeyHex}',
        // call_data: [04][00][dest 32B][u128_le(1)][Vec(0)] → 0.01 GMB
        payloadHex: _withSigningTailHex(
            '0x0400aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0100000000000000000000000000000000'),
        action: QrActions.transferWithRemark,
      );

      final verification = service.verifyPayload(request);
      expect(verification.status, SignDecisionStatus.normal);
      expect(verification.canSign, isTrue);
      expect(verification.actionLabel, '转账');
      expect(verification.decoded, isNotNull);
      expect(verification.decoded!.action, 'transfer');
    });

    test('verifyPayload accepts exact SquarePost platform price action', () {
      const cid = 'GZ018-SFGYR-201206100-2026';
      final cidBytes = cid.codeUnits;
      const role = 'GENESIS_PRODUCT_MANAGER';
      final roleBytes = role.codeUnits;
      final price = List<int>.filled(16, 0)..[0] = 100;
      final payloadHex = '0x${_toHex([
            34,
            5,
            cidBytes.length << 2,
            ...cidBytes,
            roleBytes.length << 2,
            ...roleBytes,
            2,
            ...price,
          ])}';
      final request = _buildTestRequest(
        requestId: 'offline-platform-price',
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: payloadHex,
        action: QrActions.proposeSetPlatformPrice,
      );

      final verification = service.verifyPayload(request);
      expect(verification.status, SignDecisionStatus.normal);
      expect(verification.actionLabel, '发起平台会员调价提案');
      expect(verification.decoded!.fields['membership_level'], '薪火会员');
    });

    test('verifyPayload rejects platform price payload with mismatched action',
        () {
      const cid = 'GZ018-SFGYR-201206100-2026';
      final cidBytes = cid.codeUnits;
      const role = 'GENESIS_PRODUCT_MANAGER';
      final roleBytes = role.codeUnits;
      final price = List<int>.filled(16, 0)..[0] = 100;
      final request = _buildTestRequest(
        requestId: 'offline-platform-price-mismatch',
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: '0x${_toHex([
              34,
              5,
              cidBytes.length << 2,
              ...cidBytes,
              roleBytes.length << 2,
              ...roleBytes,
              0,
              ...price,
            ])}',
        action: QrActions.transferWithRemark,
      );

      final verification = service.verifyPayload(request);
      expect(verification.status, SignDecisionStatus.reject);
      expect(verification.rejectReason, contains('不匹配'));
    });

    test('verifyPayload 拒绝普通链交易 32 字节 hash-only payload', () {
      final request = _buildTestRequest(
        requestId: 'offline-req-test-hash-only-reject',
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        action: QrActions.privateInstitutionGovernance,
      );

      final verification = service.verifyPayload(request);

      expect(verification.status, SignDecisionStatus.reject);
      expect(verification.canSign, isFalse);
      expect(verification.actionLabel, '发起私权机构治理');
      expect(verification.rejectReason, contains('普通链交易不能只签 32 字节哈希'));
    });

    test('verifyPayload 拒绝未登记 action', () {
      final request = _buildTestRequest(
        requestId: 'offline-req-test-unknown-action',
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: _withSigningTailHex('0x1400010000000000000001'),
        action: 0x7fff,
      );

      final verification = service.verifyPayload(request);

      expect(verification.status, SignDecisionStatus.reject);
      expect(verification.actionLabel, isNull);
      expect(verification.rejectReason, contains('未登记的签名动作'));
    });

    test('verifyPayload 识别广场动作中文名但钱包端拒绝签名', () {
      final request = _buildTestRequest(
        requestId: 'offline-req-test-square-action',
        pubkey: '0x${hotWallet.pubkeyHex}',
        payloadHex: '0x01020304',
        action: QrActions.squareAccountAction,
      );

      final verification = service.verifyPayload(request);

      expect(verification.status, SignDecisionStatus.reject);
      expect(verification.canSign, isFalse);
      expect(verification.actionLabel, '广场账户动作签名');
      expect(verification.rejectReason, contains('签名载荷无法解码'));
    });

    test('signParsedRequest should reject wrong pubkey', () async {
      final request = _buildTestRequest(
        requestId: 'offline-req-test-0002',
        pubkey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x0102',
        action: QrActions.login,
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
  int signCallCount = 0;

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
        address: pair.address,
        pubkeyHex: pubkeyHex,
        alg: 'sr25519',
        ss58Prefix: _ss58,
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
    signCallCount += 1;
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
