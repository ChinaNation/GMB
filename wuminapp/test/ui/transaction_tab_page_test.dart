import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/transaction_tab_page.dart';
import 'package:wuminapp_mobile/ui/widgets/chain_progress_banner.dart';

void main() {
  testWidgets('交易页恢复原交易 UI 并增加两个独立多签入口', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.lightTheme,
        home: const TransactionTabPage(),
      ),
    );
    await tester.pump();

    // 中文注释：交易页不能退化成四入口列表，原交易头部和链上支付表单必须保留。
    expect(find.text('交易'), findsOneWidget);
    expect(find.byTooltip('我的通讯录'), findsOneWidget);
    expect(find.byTooltip('选择交易钱包'), findsOneWidget);
    expect(find.byType(ChainProgressBanner), findsOneWidget);
    expect(find.text('扫码支付'), findsOneWidget);
    expect(find.text('个人多签'), findsOneWidget);
    expect(find.text('机构多签'), findsOneWidget);
    expect(find.text('收款地址'), findsOneWidget);
    expect(find.text('金额'), findsOneWidget);
    expect(find.text('签名交易'), findsOneWidget);
    expect(find.text('链上支付'), findsNothing);
  });
}
