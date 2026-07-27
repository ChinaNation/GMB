import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/widgets/wallet_onchain_balance_card.dart';

/// WalletOnchainBalanceCard 方案 2 基础渲染测试。
///
/// 布局变化:
/// - 删除了卡内刷新按钮(整个 IconButton 体系),改由外层 RefreshIndicator
///   下拉触发,通过 [GlobalKey<WalletOnchainBalanceCardState>] 调 [refresh()]。
/// - 第 1 行:标题「链上余额」。
/// - 第 2 行:金额 + 唯一单位「元」。
///
/// ChainRpc 走 smoldot 原生通道,单元测试环境没有轻节点;本轮只验证:
/// - 卡片能挂载,不崩溃
/// - 标题 + 单一金额单位均可见
/// - 整卡内无 IconButton(刷新按钮已删)
/// - 错误态下可通过 InkWell 点击触发 refresh
void main() {
  const wallet = WalletProfile(
    walletIndex: 0,
    walletName: '测试钱包',
    walletIcon: 'wallet',
    balance: 0.0,
    ss58Address: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
    accountId:
        '0x9c0c5bc3b65f2b1aeecec2a0e70e6f0ef3f2dc8d59c12a9fa79ca88e3f2c82a3',
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: 0,
    source: 'test',
    signMode: 'local',
  );

  testWidgets('card mounts with title visible', (tester) async {
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
    // v4 删除了卡内刷新按钮,整卡应不存在 IconButton。
    expect(find.byType(IconButton), findsNothing);
  });

  testWidgets('loading state shows title and exactly one yuan label',
      (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: WalletOnchainBalanceCard(wallet: wallet),
        ),
      ),
    );
    // 初始加载帧中单位与占位金额分开渲染，且单位只允许出现一次。
    expect(find.text('链上余额'), findsOneWidget);
    expect(find.text('元'), findsOneWidget);
  });

  testWidgets('方案 2 余额区保持紧凑高度', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: Align(
            alignment: Alignment.topCenter,
            child: WalletOnchainBalanceCard(wallet: wallet),
          ),
        ),
      ),
    );

    expect(
      tester
          .getSize(
            find.byKey(const ValueKey('wallet-onchain-balance-section')),
          )
          .height,
      123,
    );
  });

  testWidgets('GlobalKey<WalletOnchainBalanceCardState> can call refresh',
      (tester) async {
    // 外层下拉刷新通过 GlobalKey 拿到 State 调 refresh(),
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
