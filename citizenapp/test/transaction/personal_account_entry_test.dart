import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/transaction/personal-manage/personal_account_entry.dart';

/// 交易页顶部唯一入口卡片。扫一扫已收进收款地址框,这里不得再出现扫码入口。
void main() {
  testWidgets('渲染多签账户标题与右向 chevron', (tester) async {
    await tester.pumpWidget(const MaterialApp(
      home: Scaffold(body: PersonalAccountEntryCard()),
    ));

    expect(find.text('多签账户'), findsOneWidget);
    expect(find.text('扫一扫'), findsNothing);

    final chevron = tester.widget<Icon>(find.byIcon(Icons.chevron_right));
    // 与链上交易状态行右侧同一颗:线性 chevron,22,不设颜色。
    expect(chevron.size, 22);
    expect(chevron.color, isNull);
  });

  testWidgets('整张卡片是点击区', (tester) async {
    var taps = 0;
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: PersonalAccountEntryCard(onTap: () => taps += 1),
      ),
    ));

    // 点标题与点箭头都应命中同一个点击区(箭头只作指示,不单独包 InkWell)。
    await tester.tap(find.text('多签账户'));
    await tester.tap(find.byIcon(Icons.chevron_right));
    await tester.pumpAndSettle();

    expect(taps, 2);
  });
}
