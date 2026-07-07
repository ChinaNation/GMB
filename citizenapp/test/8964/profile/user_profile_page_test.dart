import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/user_profile_page.dart';

import 'profile_test_doubles.dart';

Widget _wrap({required bool isSelf}) => MaterialApp(
      home: UserProfilePage(
        ownerAccount: kOwner,
        isSelf: isSelf,
        api: FakeProfileApi(sampleProfile()),
        cache: FakeProfileCache(),
        sessionProvider: FakeSessionProvider(null),
      ),
    );

void main() {
  testWidgets('renders 5 category tabs with back and more actions',
      (tester) async {
    await tester.pumpWidget(_wrap(isSelf: true));
    await tester.pumpAndSettle();

    for (final label in ['帖子', '竞选', '照片', '视频', '文章']) {
      expect(find.text(label), findsOneWidget);
    }
    expect(find.byIcon(Icons.arrow_back), findsOneWidget);
    expect(find.byIcon(Icons.more_vert), findsOneWidget);
    expect(find.text('还没有帖子'), findsOneWidget);
  });

  testWidgets('switching category shows the matching tab body', (tester) async {
    await tester.pumpWidget(_wrap(isSelf: true));
    await tester.pumpAndSettle();

    await tester.tap(find.text('竞选'));
    await tester.pumpAndSettle();

    expect(find.text('还没有竞选内容'), findsOneWidget);
  });

  testWidgets('builds another user profile without exceptions', (tester) async {
    await tester.pumpWidget(_wrap(isSelf: false));
    await tester.pumpAndSettle();

    expect(find.byType(UserProfilePage), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
