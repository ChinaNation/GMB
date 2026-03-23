import 'package:flutter_test/flutter_test.dart';
import 'package:wumin/wallet/wallet_manager.dart';

void main() {
  group('WalletProfile', () {
    WalletProfile _make({
      List<String> groupNames = const [],
      String signMode = 'local',
    }) {
      return WalletProfile(
        walletIndex: 1,
        walletName: '测试钱包',
        walletIcon: 'wallet',
        balance: 0,
        address: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        pubkeyHex: 'aabbccdd' * 8,
        alg: 'sr25519',
        ss58: 2027,
        createdAtMillis: 1000000,
        source: 'created',
        signMode: signMode,
        groupNames: groupNames,
      );
    }

    test('inGroup 对"全部"始终返回 true', () {
      final wallet = _make();
      expect(wallet.inGroup('全部'), isTrue);
    });

    test('inGroup 检查实际所属分组', () {
      final wallet = _make(groupNames: ['分组一', '分组二']);
      expect(wallet.inGroup('分组一'), isTrue);
      expect(wallet.inGroup('分组二'), isTrue);
      expect(wallet.inGroup('分组三'), isFalse);
    });

    test('inGroup 空分组列表返回 false', () {
      final wallet = _make();
      expect(wallet.inGroup('分组一'), isFalse);
    });

    test('isHotWallet 和 isColdWallet 互斥', () {
      final hot = _make(signMode: 'local');
      expect(hot.isHotWallet, isTrue);
      expect(hot.isColdWallet, isFalse);

      final cold = _make(signMode: 'external');
      expect(cold.isHotWallet, isFalse);
      expect(cold.isColdWallet, isTrue);
    });

    test('未知 signMode 两者均为 false', () {
      final unknown = _make(signMode: 'unknown');
      expect(unknown.isHotWallet, isFalse);
      expect(unknown.isColdWallet, isFalse);
    });
  });
}
