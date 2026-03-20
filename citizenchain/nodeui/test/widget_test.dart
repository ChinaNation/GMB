import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:nodeui/main.dart';

void main() {
  // 新版 NodeUI 首先要保证迁移首页能稳定渲染，避免后续模块迁入前工程就失稳。
  testWidgets('显示新版 NodeUI 迁移首页', (WidgetTester tester) async {
    await tester.pumpWidget(const NodeUiApp());

    expect(find.text('CitizenChain NodeUI'), findsOneWidget);
    expect(find.text('新版节点桌面 UI 已建立 Flutter Desktop 工程'), findsOneWidget);
    expect(find.text('功能迁移路线'), findsOneWidget);
  });
}
