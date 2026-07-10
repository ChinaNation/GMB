import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/widgets/profile_kebab_menu.dart';

Future<void> _openMenu(WidgetTester tester, {required bool isSelf}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        appBar: AppBar(
          actions: [
            ProfileKebabMenu(
              isSelf: isSelf,
              onQrCode: () {},
              onEditProfile: () {},
              onReport: () {},
              onDeleteAccount: () {},
            ),
          ],
        ),
      ),
    ),
  );
  await tester.tap(find.byIcon(Icons.more_vert));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('本人主页：显示「注销用户」「编辑资料」，不显示「举报」', (tester) async {
    await _openMenu(tester, isSelf: true);
    expect(find.text('注销用户'), findsOneWidget);
    expect(find.text('编辑资料'), findsOneWidget);
    expect(find.text('举报'), findsNothing);
  });

  testWidgets('他人主页：不显示「注销用户」「编辑资料」，显示「举报」', (tester) async {
    await _openMenu(tester, isSelf: false);
    expect(find.text('注销用户'), findsNothing);
    expect(find.text('编辑资料'), findsNothing);
    expect(find.text('举报'), findsOneWidget);
  });
}
