import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/offchain/services/clearing_bank_prefs.dart';

void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  group('ClearingBankPrefs', () {
    test('load returns null when key absent', () async {
      expect(await ClearingBankPrefs.load(0), isNull);
    });

    test('save then load roundtrip per walletIndex', () async {
      await ClearingBankPrefs.save(0, 'SFR-GD-SZ01-CB01-N9-D8');
      await ClearingBankPrefs.save(1, 'SFR-BJ-BJ01-CB01-N9-D8');
      expect(await ClearingBankPrefs.load(0), 'SFR-GD-SZ01-CB01-N9-D8');
      expect(await ClearingBankPrefs.load(1), 'SFR-BJ-BJ01-CB01-N9-D8');
    });

    test('save empty string clears entry', () async {
      await ClearingBankPrefs.save(0, 'SFR-GD-SZ01-CB01-N9-D8');
      expect(await ClearingBankPrefs.load(0), isNotNull);
      await ClearingBankPrefs.save(0, '   '); // 空白等价于清除
      expect(await ClearingBankPrefs.load(0), isNull);
    });

    test('clear removes only the specified walletIndex', () async {
      await ClearingBankPrefs.save(0, 'AAA');
      await ClearingBankPrefs.save(1, 'BBB');
      await ClearingBankPrefs.clear(0);
      expect(await ClearingBankPrefs.load(0), isNull);
      expect(await ClearingBankPrefs.load(1), 'BBB');
    });

    test('save overwrites previous value (切换清算行)', () async {
      await ClearingBankPrefs.save(0, 'OLD');
      await ClearingBankPrefs.save(0, 'NEW');
      expect(await ClearingBankPrefs.load(0), 'NEW');
    });

    test('saveSnapshot stores endpoint data', () async {
      await ClearingBankPrefs.saveSnapshot(
        0,
        const ClearingBankBindingSnapshot(
          sfidId: 'SFR-GD-SZ01-CB01-N9-D8',
          institutionName: '测试清算行',
          mainAccount: 'aa',
          feeAccount: 'bb',
          peerId: '12D3KooWTest',
          rpcDomain: '127.0.0.1',
          rpcPort: 9944,
          boundAtMs: 1,
          lastVerifiedAtMs: 2,
        ),
      );

      final snapshot = await ClearingBankPrefs.loadSnapshot(0);
      expect(snapshot, isNotNull);
      expect(snapshot!.sfidId, 'SFR-GD-SZ01-CB01-N9-D8');
      expect(snapshot.wssUrl, 'ws://127.0.0.1:9944');
      expect(await ClearingBankPrefs.load(0), 'SFR-GD-SZ01-CB01-N9-D8');
    });
  });
}
