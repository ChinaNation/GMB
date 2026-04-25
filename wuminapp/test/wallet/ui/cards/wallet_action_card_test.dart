import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/cards/wallet_action_card.dart';

/// 中文注释:WalletActionCard 渲染 + SnackBar 行为测试(v2 三列版)。
///
/// 验证点:
/// - 三列 label:充值 / 提现 / 余额 全部渲染。
/// - 三列图标:arrow_circle_down_outlined / arrow_circle_up_outlined /
///   account_balance_wallet_outlined 全部渲染。
/// - 充值 / 提现点击 → SnackBar「功能开发中」。
/// - 余额列下方小字 `0.00 元` 可见。
/// - 整卡只有 2 个 InkWell(充值 + 提现),余额列不可点击。
void main() {
  // 构造一个最小可用的 WalletProfile 用于 widget 入参。
  const wallet = WalletProfile(
    walletIndex: 0,
    walletName: '测试钱包',
    walletIcon: 'wallet',
    balance: 0.0,
    address: '5DummyAddress',
    pubkeyHex: '0x00',
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: 0,
    source: 'test',
    signMode: 'local',
  );

  Future<void> pumpCard(WidgetTester tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: WalletActionCard(wallet: wallet),
        ),
      ),
    );
  }

  testWidgets('renders 充值 / 提现 / 余额 three columns with expected icons',
      (tester) async {
    await pumpCard(tester);
    expect(find.text('充值'), findsOneWidget);
    expect(find.text('提现'), findsOneWidget);
    expect(find.text('余额'), findsOneWidget);
    expect(find.byIcon(Icons.arrow_circle_down_outlined), findsOneWidget);
    expect(find.byIcon(Icons.arrow_circle_up_outlined), findsOneWidget);
    expect(find.byIcon(Icons.account_balance_wallet_outlined), findsOneWidget);
  });

  testWidgets('balance column shows 0.00 元 placeholder', (tester) async {
    await pumpCard(tester);
    expect(find.text('0.00 元'), findsOneWidget);
  });

  testWidgets('tapping 充值 shows 功能开发中 snackbar', (tester) async {
    await pumpCard(tester);
    // 中文注释:定位图标而非 label,避免点到占位 Text('\u00A0') 或 label 文本
    // 这些非 InkWell 区域。InkWell 包住图标圆圈,点图标最稳。
    await tester.tap(find.byIcon(Icons.arrow_circle_down_outlined));
    await tester.pump(); // trigger snackbar
    expect(find.text('功能开发中'), findsOneWidget);
  });

  testWidgets('tapping 提现 shows 功能开发中 snackbar', (tester) async {
    await pumpCard(tester);
    await tester.tap(find.byIcon(Icons.arrow_circle_up_outlined));
    await tester.pump();
    expect(find.text('功能开发中'), findsOneWidget);
  });

  testWidgets('balance column is non-interactive: exactly 2 InkWells in card',
      (tester) async {
    await pumpCard(tester);
    // 中文注释:整卡只有 2 个 InkWell(充值 + 提现)。余额列是静态展示,不包
    // InkWell / GestureDetector / onTap 回调。这条是硬规则。
    expect(
      tester.widgetList(find.byType(InkWell)),
      hasLength(2),
      reason: '余额列不可点击,整卡只允许有 2 个 InkWell(充值 + 提现)',
    );
  });
}
