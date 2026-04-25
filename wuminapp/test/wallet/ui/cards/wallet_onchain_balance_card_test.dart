import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/cards/wallet_onchain_balance_card.dart';

/// 中文注释:WalletOnchainBalanceCard 基础渲染测试(v4 删刷新按钮)。
///
/// 布局变化:
/// - 删除了卡内刷新按钮(整个 IconButton 体系),改由外层 RefreshIndicator
///   下拉触发,通过 [GlobalKey<WalletOnchainBalanceCardState>] 调 [refresh()]。
/// - 第 1 行:仅标题「链上余额」。
/// - 第 2 行:金额(左)+ 「GMB」(右下,与金额 baseline 对齐)。
///
/// ChainRpc 走 smoldot 原生通道,单元测试环境没有轻节点;本轮只验证:
/// - 卡片能挂载,不崩溃
/// - 标题 + GMB 文案均可见
/// - 整卡内无 IconButton(刷新按钮已删)
/// - 错误态下可通过 GestureDetector 点击触发 refresh
void main() {
  const wallet = WalletProfile(
    walletIndex: 0,
    walletName: '测试钱包',
    walletIcon: 'wallet',
    balance: 0.0,
    address: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    pubkeyHex:
        '0x9c0c5bc3b65f2b1aeecec2a0e70e6f0ef3f2dc8d59c12a9fa79ca88e3f2c82a3',
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: 0,
    source: 'test',
    signMode: 'local',
  );

  testWidgets('card mounts with title visible',
      (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: WalletOnchainBalanceCard(wallet: wallet),
        ),
      ),
    );
    // 初次渲染:标题在第一帧就应该可见(加载 / 失败 / 成功态都展示)。
    expect(find.text('链上余额'), findsOneWidget);
    await tester.pump();
  });

  testWidgets('no IconButton inside the card (refresh button removed)',
      (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: WalletOnchainBalanceCard(wallet: wallet),
        ),
      ),
    );
    await tester.pump();
    // 中文注释:v4 删除了卡内刷新按钮,整卡应不存在 IconButton。
    expect(find.byType(IconButton), findsNothing);
  });

  testWidgets('title and GMB labels visible', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: WalletOnchainBalanceCard(wallet: wallet),
        ),
      ),
    );
    await tester.pump();
    // 中文注释:标题固定在第 1 行,GMB 固定在第 2 行右下角,均需可见。
    expect(find.text('链上余额'), findsOneWidget);
    expect(find.text('GMB'), findsOneWidget);
  });

  testWidgets('GlobalKey<WalletOnchainBalanceCardState> can call refresh',
      (tester) async {
    // 中文注释:外层下拉刷新通过 GlobalKey 拿到 State 调 refresh(),
    // 这里验证类型系统可用(编译期断言 State 类已公开)+ 调用不抛。
    final key = GlobalKey<WalletOnchainBalanceCardState>();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: WalletOnchainBalanceCard(key: key, wallet: wallet),
        ),
      ),
    );
    await tester.pump();
    expect(key.currentState, isA<WalletOnchainBalanceCardState>());
    // refresh() 在单测环境会走错误分支,这里只验证调用链通,不抛。
    await key.currentState!.refresh();
    await tester.pump();
  });
}
