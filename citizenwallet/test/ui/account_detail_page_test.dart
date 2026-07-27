// Lv3 账户详情 widget 测试:钉死 C-1 修复(账户页不展示私钥)+ 公开信息展示。
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/ui/account_detail_page.dart';
import 'package:citizenwallet/wallet/wallet_manager.dart';

void main() {
  const account = Account(
    masterId:
        '0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a',
    accountIndex: 1,
    accountId:
        '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48',
    ss58Address: 'w5FhUDLW4BxsE1QXK4sNjPZ8rqSnK2QeVpUfXzqczpWdxChxV',
    accountName: '账户1',
    createdAtMillis: 0,
  );

  testWidgets('账户详情展示公开信息,不展示私钥(C-1)', (tester) async {
    await tester.pumpWidget(const MaterialApp(
      home: AccountDetailPage(account: account, walletName: '钱包1'),
    ));
    await tester.pump();

    // 公开信息在。
    expect(find.text('公钥（账户 ID）'), findsOneWidget);
    expect(find.text('SS58 地址'), findsOneWidget);
    expect(find.text('派生路径'), findsOneWidget);
    expect(find.text('//1'), findsWidgets);

    // C-1:绝不出现任何私钥入口/展示。
    expect(find.textContaining('私钥'), findsNothing);
    expect(find.textContaining('查看私钥'), findsNothing);
    // 也不得把 master 种子 URI 泄露到界面。
    expect(find.textContaining(account.masterId.substring(2)), findsNothing);
  });
}
