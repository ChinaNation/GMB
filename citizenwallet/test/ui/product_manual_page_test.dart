import 'package:citizenwallet/ui/app_theme.dart';
import 'package:citizenwallet/ui/product_manual_page.dart';
import 'package:citizenwallet/ui/settings_page.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('设置页产品手册入口可以进入手册', (tester) async {
    FlutterSecureStorage.setMockInitialValues({});
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.darkTheme,
        home: const SettingsPage(),
      ),
    );
    await tester.pumpAndSettle();

    final entry = find.widgetWithText(ListTile, '产品手册');
    expect(entry, findsOneWidget);
    await tester.tap(entry);
    await tester.pumpAndSettle();
    expect(find.text('一个钱包，五层关系'), findsOneWidget);
  });

  testWidgets('产品手册展示完整密钥关系与公开边界', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.darkTheme,
        home: const ProductManualPage(),
      ),
    );

    expect(find.text('产品手册'), findsOneWidget);
    expect(find.text('一个钱包，五层关系'), findsOneWidget);
    expect(find.text('助记词'), findsOneWidget);
    expect(find.text('种子'), findsOneWidget);
    expect(find.text('私钥'), findsOneWidget);
    expect(find.text('公钥 / AccountId'), findsOneWidget);

    await tester.drag(
      find.byKey(const Key('product-manual-scroll')),
      const Offset(0, -700),
    );
    await tester.pumpAndSettle();

    expect(find.text('账户地址'), findsOneWidget);
    expect(find.textContaining('备份地址不能恢复钱包'), findsOneWidget);
    expect(find.text('必须\n保密'), findsNWidgets(3));
    expect(find.text('可以\n公开'), findsNWidgets(2));
  });
}
