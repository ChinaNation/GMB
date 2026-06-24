// 卡B 公权机构导航 widget 测试。
//
// ADR-021:机构只存 code(中枢省=ZS,链上常量派生);省名走链上常量、市名走字典。

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/public/public_page.dart';

import 'public_nav_harness.dart';

Widget _wrap(Widget child) => MaterialApp(home: Scaffold(body: child));

void main() {
  testWidgets('字典延迟就绪:先回退 code,ensureSynced 完成后回刷成市名(时序回归)',
      (tester) async {
    // 回归 20260623 时序 bug:首装字典(4.2 万条)还在灌库时市名回退 code(001),
    // 同步完成后 _syncThenRefresh 须清脏缓存按就绪字典重 join,否则永远停在 001。
    final gate = Completer<void>();
    final repo = await buildLateDictRepo(
      provinceOrder: const ['ZS'],
      institutions: [seedDto('A', provinceCode: 'ZS', cityCode: '001')],
      lateCityNames: const {'ZS|001': '中央'},
      gate: gate.future,
    );
    await tester.pumpWidget(_wrap(
      PublicPage(repository: repo, walletPubkeyProvider: () async => null),
    ));
    await tester.pumpAndSettle();

    // 字典未就绪(gate 未开):进省看市,市名回退 code,且这种脏列表不入缓存。
    await tester.tap(find.text('中枢'));
    await tester.pumpAndSettle();
    expect(find.text('001'), findsOneWidget);
    expect(find.text('中央'), findsNothing);

    // 字典灌完(gate 打开)→ _syncThenRefresh 清脏缓存重 join → 显示市名。
    gate.complete();
    await tester.pumpAndSettle();
    expect(find.text('中央'), findsOneWidget);
    expect(find.text('001'), findsNothing);
  });

  testWidgets('顶部显示"公权机构"标题 + 左栏 关注 + 规范省份(对称治理)', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['ZS'],
      institutions: [seedDto('A', provinceCode: 'ZS', cityCode: '001')],
      cityNames: const {'ZS|001': '中央'},
    );
    await tester.pumpWidget(_wrap(
      PublicPage(repository: repo, walletPubkeyProvider: () async => null),
    ));
    await tester.pumpAndSettle();

    // 标题与治理 tab"治理机构"对称。
    expect(find.text('公权机构'), findsWidgets);
    // 左栏:关注 + 省名展示**不带"省"**(中枢/岭南),来自链上常量。
    expect(find.text('关注'), findsOneWidget);
    expect(find.text('中枢'), findsOneWidget);
    expect(find.text('岭南'), findsOneWidget);
    expect(find.text('中枢省'), findsNothing);
    // 关注默认选中 + 无订阅空态。
    expect(find.text('还没有关注的公权机构'), findsOneWidget);
  });

  testWidgets('选中某省 → 右侧显示该省市列表(市名来自字典)', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['ZS'],
      institutions: [
        seedDto('A', provinceCode: 'ZS', cityCode: '001'),
        seedDto('B', provinceCode: 'ZS', cityCode: '002'),
      ],
      cityNames: const {'ZS|001': '中央', 'ZS|002': '北区'},
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
      provinceOrder: const ['ZS'],
      institutions: [
        seedDto('A', provinceCode: 'ZS', cityCode: '001', name: '中枢省人民政府'),
      ],
      cityNames: const {'ZS|001': '中央'},
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

  testWidgets('关注分组渲染我订阅的机构(所属地 省名·市名 字典 join)', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['ZS'],
      institutions: [
        seedDto('A', provinceCode: 'ZS', cityCode: '001', name: '中枢省人民政府'),
      ],
      cityNames: const {'ZS|001': '中央'},
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
