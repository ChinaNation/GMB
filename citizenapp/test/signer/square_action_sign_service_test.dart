import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/qr/bodies/sign_response_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/signer/signing.dart';
import 'package:citizenapp/signer/square_action_sign_service.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

const _owner = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
final Uint8List _pubBytes =
    Uint8List.fromList(List.generate(32, (i) => (i + 7) & 0xff));
final String _pubHex =
    _pubBytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

Uint8List _payloadBytes() => Uint8List.fromList(<int>[
      ...scaleString('cancel_membership'),
      ...scaleString(_owner),
      ...scaleString('sqa_1'),
      ...u64Le(1700000000000),
    ]);

String _hex(List<int> b) =>
    b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

String _signRequestRaw({int action = QrActions.squareAccountAction}) {
  final signer = QrSigner();
  return signer.encodeRequest(
    signer.buildRequest(
      requestId: QrSigner.generateRequestId(prefix: 'sq-'),
      pubkey: '0x$_pubHex',
      payloadHex: '0x${_hex(_payloadBytes())}',
      action: action,
    ),
  );
}

WalletProfile _wallet(
    {required String pubkeyHex, String signMode = 'local', int index = 3}) {
  return WalletProfile(
    walletIndex: index,
    walletName: 'w',
    walletIcon: '',
    balance: 0,
    address: _owner,
    pubkeyHex: pubkeyHex,
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: 0,
    source: 'test',
    signMode: signMode,
  );
}

class _FakeWalletManager extends WalletManager {
  _FakeWalletManager(this._wallets);
  final List<WalletProfile> _wallets;

  Uint8List signature = Uint8List(64)..fillRange(0, 64, 0x5a);
  Uint8List? signedPayload;
  int? signedIndex;

  @override
  Future<List<WalletProfile>> getWallets() async => _wallets;

  @override
  Future<Uint8List> signWithWallet(int walletIndex, Uint8List payload) async {
    signedIndex = walletIndex;
    signedPayload = payload;
    return signature;
  }
}

void main() {
  final service = SquareActionSignService();

  test('prepare resolves owner wallet by QR u pubkey + decodes action',
      () async {
    final wm = _FakeWalletManager([_wallet(pubkeyHex: _pubHex)]);
    final prep = await service.prepare(_signRequestRaw(), wm);
    expect(prep.wallet.walletIndex, 3);
    expect(prep.actionLabel, '广场账户动作签名');
    expect(prep.decoded.action, 'cancel_membership');
    expect(prep.decoded.actionTypeLabel, '取消订阅');
    expect(prep.decoded.reviewFields, isNotNull);
  });

  test('prepare rejects unknown action before signing', () async {
    final wm = _FakeWalletManager([_wallet(pubkeyHex: _pubHex)]);
    await expectLater(
      service.prepare(_signRequestRaw(action: 0x7fff), wm),
      throwsA(
        isA<SquareActionSignException>()
            .having(
              (e) => e.error,
              'error',
              SquareActionSignError.unsupportedAction,
            )
            .having(
              (e) => e.message,
              'message',
              contains('未登记的签名动作'),
            ),
      ),
    );
  });

  test('prepare rejects registered but unsupported action before signing',
      () async {
    final wm = _FakeWalletManager([_wallet(pubkeyHex: _pubHex)]);
    await expectLater(
      service.prepare(_signRequestRaw(action: QrActions.login), wm),
      throwsA(
        isA<SquareActionSignException>()
            .having(
              (e) => e.error,
              'error',
              SquareActionSignError.unsupportedAction,
            )
            .having(
              (e) => e.message,
              'message',
              contains('登录确认 暂不支持在公民端签名'),
            ),
      ),
    );
  });

  test('prepare throws accountNotLocal when no wallet matches u', () async {
    final wm = _FakeWalletManager([_wallet(pubkeyHex: 'aa' * 32)]);
    await expectLater(
      service.prepare(_signRequestRaw(), wm),
      throwsA(
        isA<SquareActionSignException>().having(
          (e) => e.error,
          'error',
          SquareActionSignError.accountNotLocal,
        ),
      ),
    );
  });

  test('prepare rejects cold wallet', () async {
    final wm =
        _FakeWalletManager([_wallet(pubkeyHex: _pubHex, signMode: 'external')]);
    await expectLater(
      service.prepare(_signRequestRaw(), wm),
      throwsA(
        isA<SquareActionSignException>().having(
          (e) => e.error,
          'error',
          SquareActionSignError.coldWalletUnsupported,
        ),
      ),
    );
  });

  test(
      'sign signs signing_message(0x1D) with owner wallet and builds signResponse',
      () async {
    final wm = _FakeWalletManager([_wallet(pubkeyHex: _pubHex)]);
    final prep = await service.prepare(_signRequestRaw(), wm);

    final responseJson = await service.sign(prep, wm);

    // 用 owner 钱包（index 3）对 signing_message(0x1D, payload) 签名。
    expect(wm.signedIndex, 3);
    final expected = signingMessage(
        opTag: kOpSignSquareAction, scalePayload: _payloadBytes());
    expect(wm.signedPayload, expected);

    // signResponse envelope 携带该 64B 签名。
    final env = QrEnvelope.parse(responseJson);
    expect(env.kind, QrKind.signResponse);
    final body = env.body as SignResponseBody;
    expect(body.signatureBytes.length, 64);
    expect(body.signatureBytes, wm.signature);
    // 请求-响应由 id 绑定。
    expect(env.id, isNotNull);

    // 冗余校验 JSON 结构。
    final decoded = jsonDecode(responseJson) as Map<String, dynamic>;
    expect(decoded['k'], 2);
  });
}
