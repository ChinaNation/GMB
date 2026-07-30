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
import 'package:citizenwallet/isar/wallet_isar.dart';
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
  required String signerPublicKey,
  required String payloadHex,
  required int action,
  int? expiresAt,
}) {
  final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
  return QrEnvelope<SignRequestBody>(
    kind: QrKind.signRequest,
    id: requestId,
    expiresAt: expiresAt ?? now + 90,
    body: SignRequestBody.fromHex(
      action: action,
      signerPublicKeyHex: signerPublicKey,
      payloadHex: payloadHex,
    ),
  );
}

List<int> _u64Le(int value) =>
    List<int>.generate(8, (index) => (value >> (index * 8)) & 0xff);

List<int> _occupyAuthorizationTemplate(String cid, int expiresAt) => [
      ...List<int>.filled(32, 0x44),
      cid.length << 2,
      ...cid.codeUnits,
      ...List<int>.filled(32, 0),
      ..._u64Le(0),
      ..._u64Le(expiresAt),
    ];

List<int> _rebindAuthorizationTemplate(String cid, int expiresAt) => [
      ...List<int>.filled(32, 0x44),
      cid.length << 2,
      ...cid.codeUnits,
      ...List<int>.filled(32, 0x55),
      ...List<int>.filled(32, 0),
      ..._u64Le(7),
      ..._u64Le(expiresAt),
    ];

void main() {
  group('OfflineSignService', () {
    late _FakeWalletManager walletManager;
    late OfflineSignService service;
    late Account signingAccount;

    setUp(() async {
      await WalletIsar.instance.resetForTest();
      walletManager = _FakeWalletManager();
      service = OfflineSignService(walletManager: walletManager);
      signingAccount = await walletManager.ensureAccount();
    });

    tearDown(() => WalletIsar.instance.resetForTest());

    test('首次绑定/换绑仅接受完整授权模板且展示防重放字段', () {
      final expiresAt = DateTime.now().millisecondsSinceEpoch ~/ 1000 + 90;
      const cid = 'CN220-CTZN2-198805200-2026';
      final occupyRequest = _buildTestRequest(
        requestId: 'offline-cid-occupy-template',
        signerPublicKey: '',
        payloadHex: '0x${_toHex(_occupyAuthorizationTemplate(cid, expiresAt))}',
        action: QrActions.citizenOccupy,
        expiresAt: expiresAt,
      );
      final occupy = service.verifyPayload(occupyRequest);
      expect(occupy.status, SignDecisionStatus.normal);
      expect(occupy.decoded?.fields['genesis_hash'], '0x${'44' * 32}');
      expect(occupy.decoded?.fields['expected_binding_revision'], '0');
      expect(occupy.decoded?.fields['expires_at'], expiresAt.toString());

      final rebindRequest = _buildTestRequest(
        requestId: 'offline-cid-rebind-template',
        signerPublicKey: '',
        payloadHex: '0x${_toHex(_rebindAuthorizationTemplate(cid, expiresAt))}',
        action: QrActions.citizenRebind,
        expiresAt: expiresAt,
      );
      final rebind = service.verifyPayload(rebindRequest);
      expect(rebind.status, SignDecisionStatus.normal);
      expect(
          rebind.decoded?.fields['expected_old_account_id'], '0x${'55' * 32}');
      expect(rebind.decoded?.fields['expected_binding_revision'], '7');
    });

    test('域签名拒绝 envelope expiry 不一致和旧版 CID-only 载荷', () {
      final expiresAt = DateTime.now().millisecondsSinceEpoch ~/ 1000 + 90;
      const cid = 'CN220-CTZN2-198805200-2026';
      final mismatch = _buildTestRequest(
        requestId: 'offline-cid-expiry-mismatch',
        signerPublicKey: '',
        payloadHex: '0x${_toHex(_occupyAuthorizationTemplate(cid, expiresAt))}',
        action: QrActions.citizenOccupy,
        expiresAt: expiresAt + 1,
      );
      expect(service.verifyPayload(mismatch).status, SignDecisionStatus.reject);
      expect(service.verifyPayload(mismatch).rejectReason, contains('过期时间'));

      final legacy = _buildTestRequest(
        requestId: 'offline-cid-legacy-payload',
        signerPublicKey: '',
        payloadHex: '0x${_toHex([cid.length << 2, ...cid.codeUnits])}',
        action: QrActions.citizenOccupy,
        expiresAt: expiresAt,
      );
      expect(service.verifyPayload(legacy).status, SignDecisionStatus.reject);
    });

    test('首次绑定按所选账户签署精确授权并在响应带回 account_id', () async {
      final expiresAt = DateTime.now().millisecondsSinceEpoch ~/ 1000 + 90;
      const cid = 'CN220-CTZN2-198805200-2026';
      final request = _buildTestRequest(
        requestId: 'offline-cid-occupy-sign',
        signerPublicKey: '',
        payloadHex: '0x${_toHex(_occupyAuthorizationTemplate(cid, expiresAt))}',
        action: QrActions.citizenOccupy,
        expiresAt: expiresAt,
      );
      final response = await service.signParsedRequest(
        accountId: signingAccount.accountId,
        request: request,
      );
      expect(response.body.signerPublicKeyHex, signingAccount.accountId);
      final signingBytes = QrSigner.signingBytesFor(
        request.body,
        selfAccountId:
            Uint8List.fromList(_hexToBytes(signingAccount.accountId)),
      );
      expect(
        _verifySr25519(
          signerPublicKeyHex: response.body.signerPublicKeyHex,
          message: signingBytes,
          signatureHex: response.body.signatureHex,
        ),
        isTrue,
      );
    });

    test('signParsedRequest should sign normal internal_vote (统一入口)', () async {
      // 所有管理员投票走 InternalVote(20).cast(0)
      // payload = [0x14][0x00][u64 LE proposal_id=1][Personal=0][approve=1] + 扩展尾
      final payloadHex = _withSigningTailHex('0x140001000000000000000001');
      final request = _buildTestRequest(
        requestId: 'offline-req-test-0001',
        signerPublicKey: signingAccount.accountId,
        payloadHex: payloadHex,
        action: QrActions.internalVote,
      );

      final payloadBytes = _hexToBytes(payloadHex);

      final response = await service.signParsedRequest(
        accountId: signingAccount.accountId,
        request: request,
      );

      expect(walletManager.signCallCount, 1);
      expect(response.id, request.id);
      expect(
        response.body.signerPublicKeyHex,
        signingAccount.accountId,
      );
      expect(
        _verifySr25519(
          signerPublicKeyHex: response.body.signerPublicKeyHex,
          message: Uint8List.fromList(payloadBytes),
          signatureHex: response.body.signatureHex,
        ),
        isTrue,
      );
    });

    test('同一请求 id 到期前只能签名一次', () async {
      final payloadHex = _withSigningTailHex('0x140001000000000000000001');
      final request = _buildTestRequest(
        requestId: 'offline-replay-test-0001',
        signerPublicKey: signingAccount.accountId,
        payloadHex: payloadHex,
        action: QrActions.internalVote,
      );

      await service.signParsedRequest(
        accountId: signingAccount.accountId,
        request: request,
      );

      expect(
        () => service.signParsedRequest(
          accountId: signingAccount.accountId,
          request: request,
        ),
        throwsA(
          isA<OfflineSignException>().having(
            (e) => e.code,
            'code',
            OfflineSignErrorCode.replayed,
          ),
        ),
      );
      expect(walletManager.signCallCount, 1);
    });

    test('signParsedRequest 拒绝 action 与 payload 不一致', () async {
      // decode 成功但 QR action 和 decoded.action 不一致 → 红色拒签。
      final payloadHex = _withSigningTailHex('0x1400070000000000000001');
      final request = _buildTestRequest(
        requestId: 'offline-req-test-action-mismatch',
        signerPublicKey: signingAccount.accountId,
        payloadHex: payloadHex,
        action: QrActions.jointVote,
      );

      expect(
        () => service.signParsedRequest(
          accountId: signingAccount.accountId,
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
        signerPublicKey: signingAccount.accountId,
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
        signerPublicKey: signingAccount.accountId,
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
        signerPublicKey: signingAccount.accountId,
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
        signerPublicKey: signingAccount.accountId,
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
        signerPublicKey: signingAccount.accountId,
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
        signerPublicKey: signingAccount.accountId,
        payloadHex: '0x01020304',
        action: QrActions.squareAccountAction,
      );

      final verification = service.verifyPayload(request);

      expect(verification.status, SignDecisionStatus.reject);
      expect(verification.canSign, isFalse);
      expect(verification.actionLabel, '广场账户动作签名');
      expect(verification.rejectReason, contains('签名载荷无法解码'));
    });

    test('signParsedRequest should reject wrong signer public key', () async {
      final request = _buildTestRequest(
        requestId: 'offline-req-test-0002',
        signerPublicKey:
            '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        payloadHex: '0x0102',
        action: QrActions.login,
      );

      expect(
        () => service.signParsedRequest(
          accountId: signingAccount.accountId,
          request: request,
        ),
        throwsA(
          isA<OfflineSignException>().having(
            (e) => e.code,
            'code',
            OfflineSignErrorCode.accountMismatch,
          ),
        ),
      );
    });

    test('signParsedRequest should reject unknown account', () async {
      final request = _buildTestRequest(
        requestId: 'offline-req-test-unknown-account',
        signerPublicKey: signingAccount.accountId,
        payloadHex: _withSigningTailHex('0x140001000000000000000001'),
        action: QrActions.internalVote,
      );

      expect(
        () => service.signParsedRequest(
          accountId:
              '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb0',
          request: request,
        ),
        throwsA(
          isA<OfflineSignException>().having(
            (e) => e.code,
            'code',
            OfflineSignErrorCode.accountNotFound,
          ),
        ),
      );
    });
  });
}

bool _verifySr25519({
  required String signerPublicKeyHex,
  required Uint8List message,
  required String signatureHex,
}) {
  try {
    final publicKey = sr25519.PublicKey.newPublicKey(
      _hexToBytes(signerPublicKeyHex),
    );
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

/// 假 WalletManager:按账户提供签名（不触存储/生物识别）。
class _FakeWalletManager extends WalletManager {
  static const int _ss58 = 2027;
  static const String _mnemonic =
      'bottom drive obey lake curtain smoke basket hold race lonely fit walk';

  Account? _account;
  String? _seedHex;
  int signCallCount = 0;

  Future<Account> ensureAccount() async {
    final existing = _account;
    if (existing != null) {
      return existing;
    }
    final entropy =
        bip39m.Mnemonic.fromSentence(_mnemonic, bip39m.Language.english)
            .entropy;
    final miniSecret = await CryptoScheme.miniSecretFromEntropy(entropy);
    final pair = Keyring.sr25519.fromSeed(Uint8List.fromList(miniSecret));
    pair.ss58Format = _ss58;
    final accountId = '0x${_toHex(pair.bytes().toList(growable: false))}';
    _seedHex = _toHex(miniSecret);
    _account = Account(
      masterId: accountId,
      accountIndex: 0,
      accountId: accountId,
      ss58Address: pair.address,
      accountName: '账户0',
      createdAtMillis: 0,
    );
    return _account!;
  }

  @override
  Future<Account?> getAccountByAccountId(String accountId) async {
    final account = await ensureAccount();
    return account.accountId == accountId ? account : null;
  }

  @override
  Future<Uint8List> signForAccount(String accountId, Uint8List payload) async {
    signCallCount += 1;
    final account = await ensureAccount();
    if (accountId != account.accountId) {
      throw const WalletAuthException('未找到指定账户');
    }
    final pair =
        Keyring.sr25519.fromSeed(Uint8List.fromList(_hexToBytes(_seedHex!)));
    pair.ss58Format = _ss58;
    return Uint8List.fromList(pair.sign(payload));
  }
}
