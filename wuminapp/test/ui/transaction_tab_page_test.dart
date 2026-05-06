import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/transaction_tab_page.dart';
import 'package:wuminapp_mobile/ui/widgets/chain_progress_banner.dart';

void main() {
  testWidgets('交易页保留扫码支付 + 多签交易两个统一入口', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.lightTheme,
        home: const TransactionTabPage(),
      ),
    );
    await tester.pump();

    // 中文注释:交易页采用「单层入口」设计 —— 多签交易合一入口,点击进
    // `DuoqianAccountListPage` 后再分流个人多签 / 机构多签;扫码支付独立入口。
    // 链上支付主体字段(收款地址 / 金额 / 签名交易)由 `OnchainPaymentPanel`
    // 在选中钱包后渲染,本测试只校验顶层入口结构。
    expect(find.text('交易'), findsOneWidget);
    expect(find.byTooltip('我的通讯录'), findsOneWidget);
    expect(find.byTooltip('选择交易钱包'), findsOneWidget);
    expect(find.byType(ChainProgressBanner), findsOneWidget);
    expect(find.text('扫码支付'), findsOneWidget);
    expect(find.text('多签交易'), findsOneWidget);
    // 旧版分流入口已合并,不应再独立出现。
    expect(find.text('个人多签'), findsNothing);
    expect(find.text('机构多签'), findsNothing);
  });
}
