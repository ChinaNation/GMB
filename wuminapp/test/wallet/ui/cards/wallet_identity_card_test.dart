import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/cards/wallet_identity_card.dart';

/// 中文注释:WalletIdentityCard 渲染 + 钱包名编辑态 + 回调触发测试。
void main() {
  // 选一个长度超过 14 的模拟地址,保证短地址规则生效。
  const wallet = WalletProfile(
    walletIndex: 0,
    walletName: '我的钱包',
    walletIcon: 'wallet',
    balance: 0.0,
    address: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    pubkeyHex: '0x00',
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: 0,
    source: 'test',
    signMode: 'local',
  );

  Future<void> pumpCard(
    WidgetTester tester,
    Future<void> Function(String) onNameChanged,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: WalletIdentityCard(
            wallet: wallet,
            onNameChanged: onNameChanged,
          ),
        ),
      ),
    );
  }

  testWidgets('renders wallet name and short address', (tester) async {
    await pumpCard(tester, (_) async {});
    expect(find.text('我的钱包'), findsOneWidget);
    // 短地址规则:前 8 + ... + 后 6
    final shortAddr =
        '${wallet.address.substring(0, 8)}...${wallet.address.substring(wallet.address.length - 6)}';
    expect(find.text(shortAddr), findsOneWidget);
  });

  testWidgets('tap wallet name enters edit mode and submits new name',
      (tester) async {
    String? received;
    await pumpCard(tester, (name) async {
      received = name;
    });

    // 点击钱包名进入编辑态。
    await tester.tap(find.text('我的钱包'));
    await tester.pump();
    expect(find.byType(TextField), findsOneWidget);

    // 输入新名称并提交(通过 TextField onSubmitted)。
    await tester.enterText(find.byType(TextField), '新钱包名');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(received, '新钱包名');
  });

  testWidgets('empty name rollback without calling callback', (tester) async {
    var callCount = 0;
    await pumpCard(tester, (_) async {
      callCount += 1;
    });

    await tester.tap(find.text('我的钱包'));
    await tester.pump();
    // 清空后提交 → 回滚,不触发回调。
    await tester.enterText(find.byType(TextField), '   ');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(callCount, 0);
    // 回滚后应该回到展示态显示原钱包名。
    expect(find.text('我的钱包'), findsOneWidget);
  });
}
