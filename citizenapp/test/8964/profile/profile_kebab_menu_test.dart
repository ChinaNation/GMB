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
  testWidgets('本人主页显示注销用户和编辑资料', (tester) async {
    await _openMenu(tester, isSelf: true);
    expect(find.text('注销用户'), findsOneWidget);
    expect(find.text('编辑资料'), findsOneWidget);
  });

  testWidgets('他人主页只显示二维码', (tester) async {
    await _openMenu(tester, isSelf: false);
    expect(find.text('二维码'), findsOneWidget);
    expect(find.text('注销用户'), findsNothing);
    expect(find.text('编辑资料'), findsNothing);
  });
}
