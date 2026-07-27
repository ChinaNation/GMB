// Lv3 账户详情 widget 测试:公开信息 + 账户名可改 + 私钥区默认隐藏。
// model B 后 C-1 反转:每账户私钥独立隔离,展示单账户私钥安全(默认隐藏,验证后显示)。
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/ui/account_detail_page.dart';
import 'package:citizenwallet/wallet/wallet_manager.dart';

void main() {
  const account = Account(
    masterId:
        '0x2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972',
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

    // 需求1:账户详情不再显示派生路径。
    expect(find.text('派生路径'), findsNothing);
    expect(find.text('//1'), findsNothing);

    // 需求2:账户名展示且可点击改名(编辑图标在场)。
    expect(find.text('账户1'), findsOneWidget);
    expect(find.byIcon(Icons.edit_outlined), findsOneWidget);

    // 需求3(model B):私钥区在场,但**默认隐藏**——只显示入口,不显示明文私钥。
    expect(find.text('私钥'), findsOneWidget);
    expect(find.text('点击查看私钥'), findsOneWidget);
    // 默认态:不得把任何私钥/公钥指纹明文泄露到界面(未点查看)。
    expect(find.textContaining(account.masterId.substring(2)), findsNothing);
  });
}
