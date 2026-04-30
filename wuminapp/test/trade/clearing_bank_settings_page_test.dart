import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/offchain/pages/clearing_bank_settings_page.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 中文注释:`ClearingBankSettingsPage` 基础渲染测试。
///
/// 默认状态不主动调 SFID / 链上 API;测试只断言:
/// - AppBar 标题「设置清算行」可见
/// - 顶部搜索框(TextField)存在,hint 为「搜索清算行」
/// - 空态提示「暂无结果」可见
void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

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

  testWidgets('renders AppBar title, search field and empty hint',
      (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: ClearingBankSettingsPage(wallet: wallet),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('设置清算行'), findsOneWidget);
    expect(find.byType(TextField), findsOneWidget);
    expect(find.text('搜索清算行'), findsOneWidget);
    expect(find.text('暂无结果'), findsOneWidget);
  });
}
