import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/qr/widgets/address_scan_button.dart';
import 'package:citizenapp/transaction/onchain-transaction/onchain_payment_page.dart'
    show transactionFieldDecoration;

/// 交易表单输入框布局约束。
///
/// 收款地址框内嵌扫码按钮后必须与旁边的金额框**等高** —— `InputDecoration` 默认给
/// suffix 至少 48 的盒子,若内容高度低于它,输入框会被图标撑高、与金额框错位。本装饰
/// 靠 `isDense: true` + `contentPadding vertical 14` 把内容压到同高来避免。这条约束
/// 一旦被后续调 padding 的改动打破,肉眼很难在真机上察觉,故由本测试钉住。
void main() {
  Future<Size> fieldSize(WidgetTester tester, {Widget? suffixIcon}) async {
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 320,
          child: TextField(
            decoration: transactionFieldDecoration(
              hintText: '请输入账户',
              suffixIcon: suffixIcon,
            ),
          ),
        ),
      ),
    ));
    await tester.pumpAndSettle();
    return tester.getSize(find.byType(TextField));
  }

  testWidgets('收款地址框内嵌扫码按钮后与无 suffix 的金额框等高', (tester) async {
    final plain = await fieldSize(tester);
    final withScan = await fieldSize(
      tester,
      suffixIcon: AddressScanButton(onAddressScanned: (_) {}),
    );

    expect(withScan.height, plain.height);
  });

  testWidgets('suffixIcon 不传时不渲染任何尾部图标', (tester) async {
    await fieldSize(tester);

    // 装饰被收款地址、金额、币种、备注四个字段共用,尾部图标只能逐字段传入。
    expect(find.byType(IconButton), findsNothing);
  });
}
