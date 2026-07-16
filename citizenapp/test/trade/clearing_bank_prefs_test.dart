import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/transaction/offchain-transaction/services/clearing_bank_prefs.dart';

const _mainAccount =
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _feeAccount =
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';

ClearingBankBindingSnapshot _snapshot({
  required String cidNumber,
  String mainAccount = _mainAccount,
  String feeAccount = _feeAccount,
}) {
  return ClearingBankBindingSnapshot(
    cidNumber: cidNumber,
    cidFullName: '测试清算行',
    cidShortName: '测试清算行',
    mainAccount: mainAccount,
    feeAccount: feeAccount,
    peerId: '12D3KooWTest',
    rpcDomain: '127.0.0.1',
    rpcPort: 9944,
    boundAtMs: 1,
    lastVerifiedAtMs: 2,
  );
}

void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  group('ClearingBankPrefs', () {
    test('loadSnapshot returns null when key absent', () async {
      expect(await ClearingBankPrefs.loadSnapshot(0), isNull);
    });

    test('完整快照按 walletIndex 隔离', () async {
      await ClearingBankPrefs.saveSnapshot(
        0,
        _snapshot(cidNumber: 'GD001-SCB05-000000001-2026'),
      );
      await ClearingBankPrefs.saveSnapshot(
        1,
        _snapshot(cidNumber: 'BJ001-SCB0U-000000002-2026'),
      );
      expect(
        (await ClearingBankPrefs.loadSnapshot(0))?.cidNumber,
        'GD001-SCB05-000000001-2026',
      );
      expect(
        (await ClearingBankPrefs.loadSnapshot(1))?.cidNumber,
        'BJ001-SCB0U-000000002-2026',
      );
    });

    test('clear removes only the specified walletIndex', () async {
      await ClearingBankPrefs.saveSnapshot(0, _snapshot(cidNumber: 'AAA'));
      await ClearingBankPrefs.saveSnapshot(1, _snapshot(cidNumber: 'BBB'));
      await ClearingBankPrefs.clear(0);
      expect(await ClearingBankPrefs.loadSnapshot(0), isNull);
      expect((await ClearingBankPrefs.loadSnapshot(1))?.cidNumber, 'BBB');
    });

    test('saveSnapshot overwrites previous value (切换清算行)', () async {
      await ClearingBankPrefs.saveSnapshot(0, _snapshot(cidNumber: 'OLD'));
      await ClearingBankPrefs.saveSnapshot(0, _snapshot(cidNumber: 'NEW'));
      expect((await ClearingBankPrefs.loadSnapshot(0))?.cidNumber, 'NEW');
    });

    test('saveSnapshot stores endpoint data', () async {
      await ClearingBankPrefs.saveSnapshot(
        0,
        _snapshot(
          cidNumber: 'GD001-SCB05-000000001-2026',
        ),
      );

      final snapshot = await ClearingBankPrefs.loadSnapshot(0);
      expect(snapshot, isNotNull);
      expect(snapshot!.cidNumber, 'GD001-SCB05-000000001-2026');
      expect(snapshot.wssUrl, 'ws://127.0.0.1:9944');
      expect(snapshot.mainAccount, _mainAccount);
      expect(snapshot.feeAccount, _feeAccount);
    });

    test('缺少费用账户的旧快照必须拒绝', () async {
      SharedPreferences.setMockInitialValues({
        'clearing_bank_binding_0': '{"cid_number":"GD001-SCB05-000000001-2026",'
            '"main_account":"$_mainAccount",'
            '"rpc_domain":"127.0.0.1","rpc_port":9944}',
      });
      expect(await ClearingBankPrefs.loadSnapshot(0), isNull);
    });
  });
}
