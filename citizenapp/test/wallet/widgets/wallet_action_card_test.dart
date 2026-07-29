import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/wallet/widgets/wallet_action_card.dart';

/// WalletActionCard 渲染 + 行为测试(步骤 4 三按钮重排版)。
///
/// 验证点:
/// - 三列 label:充值 / 提现 / 零钱包 全部渲染,图标全部渲染。
/// - 未绑定时零钱包列小字 `未绑定` 可见。
/// - 充值不再受清算行绑定门槛(不属于清算行);提现 / 零钱包未绑定点击 → 提示先绑定。
/// - 三列现在都可点击(充值 + 提现 + 零钱包),整卡 3 个 InkWell。
void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  const accountId =
      '0x0000000000000000000000000000000000000000000000000000000000000000';
  const ss58Address = '5DummyAddress';

  Future<void> pumpCard(WidgetTester tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: WalletActionCard(
            accountId: accountId,
            ss58Address: ss58Address,
            finalizedBalanceLoader: (_) async => 1280.35,
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
  }

  testWidgets('renders 充值 / 提现 / 零钱包 three columns with expected icons',
      (tester) async {
    await pumpCard(tester);
    expect(find.text('充值'), findsOneWidget);
    expect(find.text('提现'), findsOneWidget);
    expect(find.text('零钱包'), findsOneWidget);
    expect(find.byIcon(Icons.arrow_circle_down_outlined), findsOneWidget);
    expect(find.byIcon(Icons.arrow_circle_up_outlined), findsOneWidget);
    expect(find.byIcon(Icons.account_balance_wallet_outlined), findsOneWidget);
  });

  testWidgets('零钱包 column shows unbound state', (tester) async {
    await pumpCard(tester);
    expect(find.text('未绑定'), findsOneWidget);
  });

  testWidgets('充值列复用链上 finalized total 余额', (tester) async {
    await pumpCard(tester);
    expect(find.text('1,280.35 元'), findsOneWidget);
  });

  testWidgets('tapping 提现 unbound asks user to bind clearing bank',
      (tester) async {
    await pumpCard(tester);
    await tester.tap(find.byIcon(Icons.arrow_circle_up_outlined));
    await tester.pump();
    expect(find.text('请先在“清算行”页面绑定清算行'), findsOneWidget);
  });

  testWidgets('tapping 零钱包 unbound asks user to bind clearing bank',
      (tester) async {
    await pumpCard(tester);
    // 零钱包现在可点击:未绑定时进入前提示先绑定(不再是静态展示)。
    await tester.tap(find.byIcon(Icons.account_balance_wallet_outlined));
    await tester.pump();
    expect(find.text('请先在“清算行”页面绑定清算行'), findsOneWidget);
  });

  testWidgets('all three columns are clickable: exactly 3 InkWells in card',
      (tester) async {
    await pumpCard(tester);
    // 步骤 4:零钱包改为可点击进详情页,整卡 3 个 InkWell(充值 + 提现 + 零钱包)。
    // 充值不再弹「请先绑定」(它是链上充值,与清算行无关),故此处只做结构断言。
    expect(
      tester.widgetList(find.byType(InkWell)),
      hasLength(3),
      reason: '三列都可点击,整卡应有 3 个 InkWell',
    );
  });
}
