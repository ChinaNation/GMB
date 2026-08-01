import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/qr/bodies/sign_request_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/generated/qr_action_registry.g.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/citizen_occupy_sign_service.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/signer/signing.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:flutter_test/flutter_test.dart';

const _cid = 'CN220-CTZN2-198805200-2026';
const _expiresAt = 1900000000;

Account _account({int index = 0, int accountByte = 0xab}) => Account(
      masterId: '0x${_hexByte(accountByte) * 32}',
      accountIndex: index,
      accountId: '0x${_hexByte(accountByte) * 32}',
      ss58Address: 'w5FhTestAddress',
      accountName: '账户$index',
    );

class _FakeWalletManager extends WalletManager {
  _FakeWalletManager({this.currentAccount});

  final Account? currentAccount;
  String? signedAccountId;
  Uint8List? signedPayload;
  final List<String> signedAccountIds = <String>[];
  final List<Uint8List> signedPayloads = <Uint8List>[];

  @override
  Future<Account?> getAccountByAccountId(String accountId) async =>
      currentAccount?.accountId == accountId ? currentAccount : null;

  @override
  Future<Uint8List> signForAccountId(
      String accountId, Uint8List payload) async {
    signedAccountId = accountId;
    signedPayload = Uint8List.fromList(payload);
    signedAccountIds.add(accountId);
    signedPayloads.add(Uint8List.fromList(payload));
    return Uint8List(64);
  }
}

String _hexByte(int value) => value.toRadixString(16).padLeft(2, '0');

List<int> _u64Le(int value) =>
    List<int>.generate(8, (index) => (value >> (index * 8)) & 0xff);

List<int> _occupyTemplate({int revision = 0, int expiresAt = _expiresAt}) => [
      ...List<int>.filled(32, 0x44),
      _cid.length << 2,
      ..._cid.codeUnits,
      ...List<int>.filled(32, 0),
      ..._u64Le(revision),
      ..._u64Le(expiresAt),
    ];

List<int> _rebindTemplate({int revision = 7, int expiresAt = _expiresAt}) => [
      ...List<int>.filled(32, 0x44),
      _cid.length << 2,
      ..._cid.codeUnits,
      ...List<int>.filled(32, 0x55),
      ...List<int>.filled(32, 0),
      ..._u64Le(revision),
      ..._u64Le(expiresAt),
    ];

/// 造注册局占号/换绑域签名 QR：b.u 留空，d 是带零账户槽的完整授权模板。
String _domainRaw({
  int? action,
  List<int>? payload,
  int outerExpiresAt = _expiresAt,
}) {
  final actualAction = action ?? QrActions.citizenOccupy;
  final authorization = payload ??
      (actualAction == QrActions.citizenOccupy
          ? _occupyTemplate()
          : _rebindTemplate());
  final payloadB64 = base64Url.encode(authorization).replaceAll('=', '');
  return QrSigner().encodeRequest(QrEnvelope<SignRequestBody>(
    kind: QrKind.signRequest,
    id: 'citizen-occupy-req-000001',
    issuedAt: 1800000000,
    expiresAt: outerExpiresAt,
    body: SignRequestBody(
      action: actualAction,
      signerPublicKey: '', // 占号/换绑 b.u 留空
      payload: payloadB64,
    ),
  ));
}

void main() {
  final service = CitizenOccupySignService();

  test('citizenOccupy/citizenRebind 硬编码常量与 registry 一致(防漂移)', () {
    expect(QrActions.citizenOccupy,
        GeneratedQrActionRegistry.actionCodeByKey['citizen_occupy']);
    expect(QrActions.citizenRebind,
        GeneratedQrActionRegistry.actionCodeByKey['citizen_rebind']);
    for (final field in const <String>[
      'genesis_hash',
      'cid_number',
      'current_account_id',
      'expected_binding_revision',
      'expires_at',
    ]) {
      expect(
        GeneratedQrActionRegistry.hasFieldLabel(field),
        isTrue,
        reason: '$field 必须由共享 QR registry 生成中文字段名',
      );
    }
  });

  test('prepare 严格解出占号完整授权模板并展示全部防重放字段', () async {
    final prep = await service.prepare(_domainRaw(), _account());
    expect(prep.cidNumber, _cid);
    expect(prep.isOccupy, isTrue);
    expect(prep.genesisHash, '0x${'44' * 32}');
    expect(prep.currentAccountId, isNull);
    expect(prep.expectedBindingRevision, BigInt.zero);
    expect(prep.expiresAt, BigInt.from(_expiresAt));
    expect(prep.account.accountId, '0x${'ab' * 32}');
  });

  test('prepare 严格解出换绑当前账户、非零 revision 与 expires', () async {
    final prep = await service.prepare(
        _domainRaw(action: QrActions.citizenRebind), _account());
    expect(prep.isOccupy, isFalse);
    expect(prep.cidNumber, _cid);
    expect(prep.genesisHash, '0x${'44' * 32}');
    expect(prep.currentAccountId, '0x${'55' * 32}');
    expect(prep.expectedBindingRevision, BigInt.from(7));
    expect(prep.expiresAt, BigInt.from(_expiresAt));
  });

  test('非占号/换绑动作即拒', () async {
    final raw = QrSigner().encodeRequest(QrSigner().buildRequest(
      requestId: 'citizen-identity-req-0001',
      signerPublicKey: '0x${'11' * 32}',
      payloadHex: '0x01020304',
      action: QrActions.citizenIdentity,
    ));
    await expectLater(
      service.prepare(raw, _account()),
      throwsA(isA<CitizenOccupySignException>()),
    );
  });

  test('旧 CID-only 载荷即拒，不恢复末尾追加账户协议', () async {
    await expectLater(
      service.prepare(
        _domainRaw(payload: [_cid.length << 2, ..._cid.codeUnits]),
        _account(),
      ),
      throwsA(isA<CitizenOccupySignException>()),
    );
  });

  test('外层 e 与内层 expires_at 不一致即拒', () async {
    await expectLater(
      service.prepare(
        _domainRaw(outerExpiresAt: _expiresAt + 1),
        _account(),
      ),
      throwsA(
        isA<CitizenOccupySignException>().having(
          (error) => error.message,
          'message',
          contains('过期时间'),
        ),
      ),
    );
  });

  test('零槽污染、尾字节与错误 revision 全部 fail-closed', () async {
    final nonZeroSlot = _occupyTemplate();
    nonZeroSlot[32 + 1 + _cid.length] = 1;
    final malformed = <List<int>>[
      nonZeroSlot,
      [..._occupyTemplate(), 0xff],
      _occupyTemplate(revision: 1),
      _rebindTemplate(revision: 0),
    ];
    for (var index = 0; index < malformed.length; index++) {
      await expectLater(
        service.prepare(
          _domainRaw(
            action: index == malformed.length - 1
                ? QrActions.citizenRebind
                : QrActions.citizenOccupy,
            payload: malformed[index],
          ),
          _account(),
        ),
        throwsA(isA<CitizenOccupySignException>()),
        reason: 'malformed template #$index must reject',
      );
    }
  });

  test('换绑选择账户与 current_account_id 相同即拒', () async {
    await expectLater(
      service.prepare(
        _domainRaw(action: QrActions.citizenRebind),
        _account(accountByte: 0x55),
      ),
      throwsA(
        isA<CitizenOccupySignException>().having(
          (error) => error.message,
          'message',
          contains('不得与当前绑定账户相同'),
        ),
      ),
    );
  });

  test('账户卡锁定的子账户原位填入占号零槽后签名', () async {
    final account = _account(index: 5);
    final manager = _FakeWalletManager();
    final prep = await service.prepare(_domainRaw(), account);
    await service.sign(prep, manager);
    expect(prep.account.accountIndex, 5);
    expect(manager.signedAccountId, account.accountId);
    final exactAuthorization = _occupyTemplate()
      ..setRange(
        32 + 1 + _cid.length,
        32 + 1 + _cid.length + 32,
        List<int>.filled(32, 0xab),
      );
    expect(
      manager.signedPayload,
      signingMessage(
        opTag: kOpSignCidOccupy,
        scalePayload: exactAuthorization,
      ),
    );
  });

  test('注册局换绑在同一次扫码中收集当前与新账户对同一授权的双签名', () async {
    final newAccount = _account(accountByte: 0xab);
    final currentAccount = _account(accountByte: 0x55);
    final manager = _FakeWalletManager(currentAccount: currentAccount);
    final prep = await service.prepare(
      _domainRaw(action: QrActions.citizenRebind),
      newAccount,
      manager,
    );

    expect(prep.currentAccount?.accountId, currentAccount.accountId);
    final raw = await service.sign(prep, manager);
    expect(manager.signedAccountIds, <String>[
      newAccount.accountId,
      currentAccount.accountId,
    ]);

    final exactAuthorization = _rebindTemplate()
      ..setRange(
        32 + 1 + _cid.length + 32,
        32 + 1 + _cid.length + 64,
        List<int>.filled(32, 0xab),
      );
    expect(
      manager.signedPayloads[0],
      QrSigner.signingBytesForHex(
        payloadHex: prep.request.body.payloadHex,
        action: QrActions.citizenRebind,
        selfAccountId: Uint8List.fromList(List<int>.filled(32, 0xab)),
      ),
    );
    expect(
      manager.signedPayloads[1],
      signingMessage(
        opTag: kOpSignCidRebind,
        scalePayload: exactAuthorization,
      ),
    );

    final response = QrSigner().parseResponse(
      raw,
      expectedRequestId: prep.request.id!,
    );
    expect(response.body.signerPublicKeyHex, newAccount.accountId);
    expect(response.body.currentAccountIdHex, currentAccount.accountId);
    expect(response.body.currentAccountSignatureHex, '0x${'00' * 64}');
  });

  test('当前钱包不在本机时注册局仍可强制换绑，但响应不伪造当前账户签名', () async {
    final manager = _FakeWalletManager();
    final prep = await service.prepare(
      _domainRaw(action: QrActions.citizenRebind),
      _account(accountByte: 0xab),
      manager,
    );
    expect(prep.currentAccount, isNull);

    final response = QrSigner().parseResponse(
      await service.sign(prep, manager),
      expectedRequestId: prep.request.id!,
    );
    expect(manager.signedAccountIds, <String>['0x${'ab' * 32}']);
    expect(response.body.currentAccountIdHex, isNull);
    expect(response.body.currentAccountSignatureHex, isNull);
  });
}
