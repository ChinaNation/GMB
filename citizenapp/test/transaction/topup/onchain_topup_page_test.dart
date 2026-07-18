import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/transaction/onchain-topup/onchain_topup_page.dart';
import 'package:citizenapp/transaction/onchain-topup/topup_api.dart';
import 'package:citizenapp/transaction/onchain-topup/topup_models.dart';

class _FakeApi extends TopupApi {
  _FakeApi() : super(baseUrl: 'https://x.test/api');

  @override
  Future<TopupConfig> fetchConfig() async => const TopupConfig(
        network: 'testnet',
        recvAddress: '0xabababababababababababababababababababab',
        rails: [
          TopupRail(token: 'USDC', chainId: 84532, tokenContract: '0x1111111111111111111111111111111111111111', tokenDecimals: 6, label: 'USDC · Base Sepolia'),
          TopupRail(token: 'USDT', chainId: 84532, tokenContract: '0x2222222222222222222222222222222222222222', tokenDecimals: 6, label: 'USDT · Base Sepolia'),
        ],
        packages: [
          TopupPackage(packageId: 'pkg_15', payDisplay: '15', payAmount: '15000000', coinDisplay: '10,000.00', coinFen: '1000000'),
          TopupPackage(packageId: 'pkg_1400', payDisplay: '1400', payAmount: '1400000000', coinDisplay: '1,000,000.00', coinFen: '100000000'),
        ],
      );
}

void main() {
  testWidgets('加载后渲染两条币轨与说明', (tester) async {
    await tester.pumpWidget(
      MaterialApp(home: OnchainTopupPage(gmbAddress: 'gmbaddr', api: _FakeApi())),
    );
    await tester.pumpAndSettle();
    expect(find.text('USDC'), findsOneWidget);
    expect(find.text('USDT'), findsOneWidget);
    expect(find.textContaining('用稳定币购买公民币'), findsOneWidget);
  });

  testWidgets('点币轨弹出套餐弹窗', (tester) async {
    await tester.pumpWidget(
      MaterialApp(home: OnchainTopupPage(gmbAddress: 'gmbaddr', api: _FakeApi())),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('USDC'));
    await tester.pumpAndSettle();
    expect(find.textContaining('选择充值套餐'), findsOneWidget);
    expect(find.text('10,000.00 公民币'), findsOneWidget);
    expect(find.text('连接钱包并支付'), findsOneWidget);
  });
}
