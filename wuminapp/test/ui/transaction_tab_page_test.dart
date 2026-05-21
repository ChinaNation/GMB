import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/transaction/onchain-transaction/onchain_payment_page.dart';
import 'package:wuminapp_mobile/transaction/shared/local_tx_store.dart';
import 'package:wuminapp_mobile/transaction/transaction_tab_page.dart';
import 'package:wuminapp_mobile/ui/widgets/chain_progress_banner.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

const _walletAPubkey =
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _walletBPubkey =
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';

WalletProfile _wallet({
  required int index,
  required String name,
  required String address,
  required String pubkeyHex,
}) {
  return WalletProfile(
    walletIndex: index,
    walletName: name,
    walletIcon: '',
    balance: 100,
    address: address,
    pubkeyHex: pubkeyHex,
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: index,
    source: 'test',
    signMode: 'local',
    sortOrder: index,
  );
}

LocalTxEntity _tx({
  required String recordKey,
  required String walletAddress,
  required String walletPubkeyHex,
  required String amountDeltaFen,
  required String status,
}) {
  return LocalTxEntity()
    ..recordKey = recordKey
    ..walletAddress = walletAddress
    ..walletPubkeyHex = LocalTxStore.normalizePubkey(walletPubkeyHex)
    ..type = 'transfer'
    ..amountDeltaFen = amountDeltaFen
    ..transferAmountFen = amountDeltaFen.replaceFirst('-', '')
    ..counterpartyAddress = 'counterparty'
    ..fromAddress = walletAddress
    ..toAddress = 'counterparty'
    ..status = status
    ..source = 'test'
    ..createdAtMillis = recordKey.hashCode;
}

Future<void> _pumpUntilFound(WidgetTester tester, Finder finder) async {
  for (var i = 0; i < 20; i++) {
    await tester.pump(const Duration(milliseconds: 50));
    if (finder.evaluate().isNotEmpty) {
      return;
    }
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('交易页保留扫码支付入口', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.lightTheme,
        home: const TransactionTabPage(),
      ),
    );
    await tester.pump();

    // 中文注释:多签账户列表已经迁入底部「多签」Tab，交易页只保留交易相关入口。
    // 链上支付主体字段(收款地址 / 金额 / 签名交易)由 `OnchainPaymentPanel`
    // 在选中钱包后渲染,本测试只校验顶层入口结构。
    expect(find.text('交易'), findsOneWidget);
    expect(find.byTooltip('我的通讯录'), findsOneWidget);
    expect(find.byTooltip('选择交易钱包'), findsOneWidget);
    expect(find.byType(ChainProgressBanner), findsOneWidget);
    expect(find.text('扫码支付'), findsOneWidget);
    expect(find.text('多签'), findsNothing);
    expect(find.text('个人多签'), findsNothing);
    expect(find.text('机构多签'), findsNothing);
  });

  testWidgets('链上交易状态跟随交易钱包切换刷新且只统计转出', (tester) async {
    final walletA = _wallet(
      index: 1,
      name: '钱包A',
      address: 'wallet_a',
      pubkeyHex: _walletAPubkey,
    );
    final walletB = _wallet(
      index: 2,
      name: '钱包B',
      address: 'wallet_b',
      pubkeyHex: _walletBPubkey,
    );
    var currentWallet = walletA;
    final records = [
      _tx(
        recordKey: 'a:pending',
        walletAddress: 'wallet_a',
        walletPubkeyHex: _walletAPubkey,
        amountDeltaFen: '-101',
        status: LocalTxStore.statusPending,
      ),
      _tx(
        recordKey: 'a:inBlock',
        walletAddress: 'wallet_a',
        walletPubkeyHex: _walletAPubkey,
        amountDeltaFen: '-202',
        status: LocalTxStore.statusInBlock,
      ),
      _tx(
        recordKey: 'a:finalized',
        walletAddress: 'wallet_a',
        walletPubkeyHex: _walletAPubkey,
        amountDeltaFen: '-303',
        status: LocalTxStore.statusFinalized,
      ),
      _tx(
        recordKey: 'b:incoming',
        walletAddress: 'wallet_b',
        walletPubkeyHex: _walletBPubkey,
        amountDeltaFen: '404',
        status: LocalTxStore.statusFinalized,
      ),
    ];

    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.lightTheme,
        home: OnchainPaymentPanel(
          title: '交易',
          enableDelayedLocalRecordRefresh: false,
          currentWalletLoader: () async => currentWallet,
          localRecordsLoader: (pubkeyHex, {limit = 100}) async {
            final pubkey = LocalTxStore.normalizePubkey(pubkeyHex);
            return records
                .where((record) => record.walletPubkeyHex == pubkey)
                .take(limit)
                .toList();
          },
          walletPicker: () async {
            currentWallet = walletB;
            return true;
          },
        ),
      ),
    );
    await _pumpUntilFound(tester, find.text('已提交 1'));

    expect(find.text('已提交 1'), findsOneWidget);
    expect(find.text('已出块 1'), findsOneWidget);
    expect(find.text('已确认 1'), findsOneWidget);
    expect(find.text('失败 0'), findsOneWidget);

    await tester.tap(find.byTooltip('选择交易钱包'));
    await _pumpUntilFound(tester, find.text('已提交 0'));

    expect(find.text('已提交 0'), findsOneWidget);
    expect(find.text('已出块 0'), findsOneWidget);
    expect(find.text('已确认 0'), findsOneWidget);
    expect(find.text('失败 0'), findsOneWidget);
  });
}
