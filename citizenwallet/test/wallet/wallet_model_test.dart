import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/wallet/wallet_manager.dart';

void main() {
  Wallet wallet({List<String> groupNames = const []}) => Wallet(
        walletIndex: 1,
        walletName: '测试钱包',
        masterId:
            '0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a',
        createdAtMillis: 1000000,
        source: 'created',
        groupNames: groupNames,
      );

  Account account(int index) => Account(
        masterId:
            '0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a',
        accountIndex: index,
        accountId: '0x${'aabbccdd' * 8}',
        ss58Address: 'w5DBnqoUytARopdnyWhmBq7ZPr74cJJewugoafJJynKLrirdE',
        accountName: '账户$index',
        createdAtMillis: 1000000,
      );

  group('Wallet.inGroup', () {
    test('"全部"始终 true', () {
      expect(wallet().inGroup('全部'), isTrue);
    });

    test('检查实际所属分组', () {
      final w = wallet(groupNames: ['分组一', '分组二']);
      expect(w.inGroup('分组一'), isTrue);
      expect(w.inGroup('分组三'), isFalse);
    });

    test('空分组列表非"全部"返回 false', () {
      expect(wallet().inGroup('分组一'), isFalse);
    });
  });

  group('Account.derivationPath', () {
    test('账户0 = 根', () {
      expect(account(0).derivationPath, '根');
    });

    test('账户N = //N', () {
      expect(account(1).derivationPath, '//1');
      expect(account(7).derivationPath, '//7');
    });
  });
}
