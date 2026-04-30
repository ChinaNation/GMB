import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/pages/wallet_page.dart';

/// 中文注释:WalletListTile v6 渲染契约 ——
/// - 不渲染「当前」标签(active 概念已废)
/// - 不渲染扫码按钮(扫码功能彻底移除)
/// - 钱包图标按冷热配色(热=AppTheme.primaryDark / 冷=AppTheme.info)
/// - 三点菜单只有 重命名/删除钱包 2 项
/// - InkWell 整卡点击触发 onTap
/// - showActions=false 时隐藏三点菜单
void main() {
  WalletProfile makeWallet({
    required String signMode,
    int walletIndex = 1,
    String walletName = '我的钱包',
    double balance = 1234567.89,
  }) {
    return WalletProfile(
      walletIndex: walletIndex,
      walletName: walletName,
      walletIcon: 'wallet',
      balance: balance,
      address: 'addr_$walletIndex',
      pubkeyHex: 'pub_$walletIndex',
      alg: 'sr25519',
      ss58: 2027,
      createdAtMillis: 0,
      source: 'test',
      signMode: signMode,
    );
  }

  Future<void> pumpTile(
    WidgetTester tester, {
    required WalletProfile wallet,
    bool showActions = true,
    VoidCallback? onTap,
    VoidCallback? onRename,
    VoidCallback? onDelete,
  }) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: WalletListTile(
            wallet: wallet,
            showActions: showActions,
            onTap: onTap ?? () {},
            onRename: onRename ?? () {},
            onDelete: onDelete ?? () {},
          ),
        ),
      ),
    );
  }

  testWidgets('渲染钱包名 + 余额千分位文本', (tester) async {
    await pumpTile(tester,
        wallet: makeWallet(signMode: 'local', walletName: '我的钱包'));
    expect(find.text('我的钱包'), findsOneWidget);
    expect(find.text('1,234,567.89'), findsOneWidget);
  });

  testWidgets('热钱包不渲染「当前」文本(active 概念已废)', (tester) async {
    await pumpTile(tester, wallet: makeWallet(signMode: 'local'));
    expect(find.text('当前'), findsNothing);
  });

  testWidgets('冷钱包不渲染「当前」文本(active 概念已废)', (tester) async {
    await pumpTile(tester, wallet: makeWallet(signMode: 'external'));
    expect(find.text('当前'), findsNothing);
  });

  testWidgets('热钱包不渲染扫码按钮(扫码功能已删)', (tester) async {
    await pumpTile(tester, wallet: makeWallet(signMode: 'local'));
    expect(find.byIcon(Icons.qr_code_scanner), findsNothing);
  });

  testWidgets('冷钱包不渲染扫码按钮(扫码功能已删)', (tester) async {
    await pumpTile(tester, wallet: makeWallet(signMode: 'external'));
    expect(find.byIcon(Icons.qr_code_scanner), findsNothing);
  });

  testWidgets('热钱包图标 Icon 颜色为 AppTheme.primaryDark', (tester) async {
    await pumpTile(tester, wallet: makeWallet(signMode: 'local'));
    final iconWidget = tester.widget<Icon>(
      find.byIcon(Icons.account_balance_wallet_rounded).first,
    );
    expect(iconWidget.color, AppTheme.primaryDark);
  });

  testWidgets('冷钱包图标 Icon 颜色为 AppTheme.info', (tester) async {
    await pumpTile(tester, wallet: makeWallet(signMode: 'external'));
    final iconWidget = tester.widget<Icon>(
      find.byIcon(Icons.account_balance_wallet_rounded).first,
    );
    expect(iconWidget.color, AppTheme.info);
  });

  testWidgets('三点菜单只有「重命名」和「删除钱包」2 项,无「钱包详情」', (tester) async {
    await pumpTile(tester, wallet: makeWallet(signMode: 'local'));
    // 点开三点菜单
    await tester.tap(find.byIcon(Icons.more_vert));
    await tester.pumpAndSettle();

    expect(find.text('重命名'), findsOneWidget);
    expect(find.text('删除钱包'), findsOneWidget);
    // 关键防回归:不允许残留「钱包详情」菜单项
    expect(find.text('钱包详情'), findsNothing);
  });

  testWidgets('三点菜单点击「重命名」触发 onRename', (tester) async {
    var renamed = false;
    await pumpTile(tester,
        wallet: makeWallet(signMode: 'local'), onRename: () => renamed = true);
    await tester.tap(find.byIcon(Icons.more_vert));
    await tester.pumpAndSettle();
    await tester.tap(find.text('重命名'));
    await tester.pumpAndSettle();
    expect(renamed, isTrue);
  });

  testWidgets('三点菜单点击「删除钱包」触发 onDelete', (tester) async {
    var deleted = false;
    await pumpTile(tester,
        wallet: makeWallet(signMode: 'local'), onDelete: () => deleted = true);
    await tester.tap(find.byIcon(Icons.more_vert));
    await tester.pumpAndSettle();
    await tester.tap(find.text('删除钱包'));
    await tester.pumpAndSettle();
    expect(deleted, isTrue);
  });

  testWidgets('整卡 InkWell 点击触发 onTap', (tester) async {
    var tapped = false;
    await pumpTile(tester,
        wallet: makeWallet(signMode: 'local'), onTap: () => tapped = true);
    // 点钱包名所在区域(整卡 InkWell 范围内)。
    await tester.tap(find.text('我的钱包'));
    await tester.pumpAndSettle();
    expect(tapped, isTrue);
  });

  testWidgets('showActions=false 时不显示三点菜单', (tester) async {
    await pumpTile(tester,
        wallet: makeWallet(signMode: 'local'), showActions: false);
    expect(find.byIcon(Icons.more_vert), findsNothing);
  });
}
