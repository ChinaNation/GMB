import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/widgets/wallet_identity_card.dart';

/// WalletIdentityCard 渲染 + 钱包名编辑态 + 回调触发测试。
void main() {
  // 使用完整 SS58 地址验证两行展示与复制按钮布局。
  const wallet = WalletProfile(
    walletIndex: 0,
    walletName: '我的钱包',
    walletIcon: 'wallet',
    balance: 0.0,
    ss58Address: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    accountId:
        '0x0000000000000000000000000000000000000000000000000000000000000000',
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
          body: Align(
            alignment: Alignment.topCenter,
            child: SizedBox(
              width: 411,
              child: WalletIdentityCard(
                wallet: wallet,
                onNameChanged: onNameChanged,
              ),
            ),
          ),
        ),
      ),
    );
  }

  testWidgets('renders wallet name and full address', (tester) async {
    await pumpCard(tester, (_) async {});
    expect(find.text('我的钱包'), findsOneWidget);
    expect(find.textContaining('...'), findsNothing);
    final firstLine = tester.widget<Text>(
      find.byKey(const ValueKey('wallet-identity-address-line-1')),
    );
    final secondLine = tester.widget<Text>(
      find.byKey(const ValueKey('wallet-identity-address-line-2')),
    );
    expect('${firstLine.data}${secondLine.data}', wallet.ss58Address);
    expect(secondLine.data, isNotEmpty);
    // 测试字体下第一行可容纳 20 个字符；锁定“排满第一行后再换行”，
    // 防止恢复为优先塞满第二行而导致第一行过短的旧拆分逻辑。
    expect(firstLine.data, wallet.ss58Address.substring(0, 20));
    expect(find.byIcon(Icons.edit_outlined), findsOneWidget);
    expect(find.byIcon(Icons.copy_outlined), findsOneWidget);
    expect(find.byIcon(Icons.qr_code_rounded), findsOneWidget);
    expect(find.byIcon(Icons.account_balance_wallet_rounded), findsNothing);
  });

  testWidgets('方案 2 身份区高度和二维码竖分隔线固定', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Align(
            alignment: Alignment.topCenter,
            child: SizedBox(
              width: 411,
              child: WalletIdentityCard(
                wallet: wallet,
                onNameChanged: (_) async {},
              ),
            ),
          ),
        ),
      ),
    );

    expect(
      tester
          .getSize(
            find.byKey(const ValueKey('wallet-identity-section')),
          )
          .height,
      112,
    );
    expect(
      tester.getSize(
        find.byKey(const ValueKey('wallet-identity-qr-divider')),
      ),
      const Size(1, 48),
    );
    final copyButtonRect = tester.getRect(
      find.byKey(const ValueKey('wallet-identity-copy-button')),
    );
    final dividerRect = tester.getRect(
      find.byKey(const ValueKey('wallet-identity-qr-divider')),
    );
    final qrButtonRect = tester.getRect(
      find.byKey(const ValueKey('wallet-identity-qr-button')),
    );
    expect(dividerRect.left - copyButtonRect.right, 21.25);
    expect(qrButtonRect.left - dividerRect.right, 16);
    // 复制图标保持原 copy_outlined 原子与 12 尺寸；21.25 的按钮外间距是
    // Pixel 8a 真机可见图形边缘校准值，不用按钮盒理论留白代替视觉量取。

    final copyIcon = tester.widget<Icon>(
      find.byKey(const ValueKey('wallet-identity-copy-icon')),
    );
    expect(copyIcon.icon, Icons.copy_outlined);
    expect(copyIcon.size, 12);
    final copyIconRect = tester.getRect(
      find.byKey(const ValueKey('wallet-identity-copy-icon')),
    );
    expect(copyIconRect.center.dy - copyButtonRect.center.dy, 4);
    final qrMaterial = tester.widget<Material>(
      find.byKey(const ValueKey('wallet-identity-qr-button')),
    );
    expect(qrMaterial.color, Colors.transparent);

    final nameRect = tester.getRect(find.text('我的钱包'));
    final firstLineRect = tester.getRect(
      find.byKey(const ValueKey('wallet-identity-address-line-1')),
    );
    expect(copyIconRect.center.dy, greaterThan(firstLineRect.center.dy));
    expect(
      copyButtonRect.center.dy,
      isNot(closeTo(nameRect.center.dy, 1)),
    );
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
