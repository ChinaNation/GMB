// 卡B 公权机构导航 widget 测试。

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/citizen/public/public_page.dart';

import 'public_nav_harness.dart';

Widget _wrap(Widget child) => MaterialApp(home: Scaffold(body: child));

void main() {
  testWidgets('顶部显示"公权机构"标题 + 左栏 关注 + 规范省份(对称治理)', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['中枢省'],
      institutions: [seedDto('A', province: '中枢省', city: '中央')],
    );
    await tester.pumpWidget(_wrap(
      PublicPage(repository: repo, walletPubkeyProvider: () async => null),
    ));
    await tester.pumpAndSettle();

    // 标题与治理 tab"治理机构"对称。
    expect(find.text('公权机构'), findsWidgets);
    // 左栏:关注 + 省名展示**不带"省"**(中枢/岭南),匹配仍用全名。
    expect(find.text('关注'), findsOneWidget);
    expect(find.text('中枢'), findsOneWidget);
    expect(find.text('岭南'), findsOneWidget);
    expect(find.text('中枢省'), findsNothing);
    // 关注默认选中 + 无订阅空态。
    expect(find.text('还没有关注的公权机构'), findsOneWidget);
  });

  testWidgets('选中某省 → 右侧显示该省市列表', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['中枢省'],
      institutions: [
        seedDto('A', province: '中枢省', city: '中央'),
        seedDto('B', province: '中枢省', city: '北区'),
      ],
    );
    await tester.pumpWidget(_wrap(
      PublicPage(repository: repo, walletPubkeyProvider: () async => null),
    ));
    await tester.pumpAndSettle();

    await tester.tap(find.text('中枢'));
    await tester.pumpAndSettle();

    expect(find.text('中央'), findsOneWidget);
    expect(find.text('北区'), findsOneWidget);
  });

  testWidgets('点市 → 进入该市公权机构列表页', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['中枢省'],
      institutions: [
        seedDto('A', province: '中枢省', city: '中央', name: '中枢省人民政府'),
      ],
    );
    await tester.pumpWidget(_wrap(
      PublicPage(repository: repo, walletPubkeyProvider: () async => null),
    ));
    await tester.pumpAndSettle();

    await tester.tap(find.text('中枢'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('中央'));
    await tester.pumpAndSettle();

    expect(find.text('中央公权机构'), findsOneWidget); // AppBar 标题
    expect(find.text('中枢省人民政府'), findsOneWidget);
  });

  testWidgets('关注分组渲染我订阅的机构', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['中枢省'],
      institutions: [
        seedDto('A', province: '中枢省', city: '中央', name: '中枢省人民政府'),
      ],
      subscriptions: const {'aa': 'A'},
    );
    await tester.pumpWidget(_wrap(
      PublicPage(repository: repo, walletPubkeyProvider: () async => 'aa'),
    ));
    await tester.pumpAndSettle();

    expect(find.text('中枢省人民政府'), findsOneWidget);
    expect(find.text('中枢 · 中央'), findsOneWidget);
  });
}
