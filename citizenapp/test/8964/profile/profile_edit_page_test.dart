import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/profile_edit_page.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';

import 'fake_profile.dart';

Widget _wrap(FakeProfileApi api, {SquareSessionProvider? sessionProvider}) {
  return MaterialApp(
    home: CitizenProfileEditPage(
      accountId: kOwner,
      initialProfile: sampleProfile(displayName: '旧名', bio: '旧签名'),
      api: api,
      sessionProvider: sessionProvider ?? FakeSessionProvider(fakeSession()),
    ),
  );
}

void main() {
  testWidgets('prefills display name and bio from the profile', (tester) async {
    await tester.pumpWidget(_wrap(FakeProfileApi(sampleProfile())));
    await tester.pumpAndSettle();

    expect(find.text('旧名'), findsOneWidget);
    expect(find.text('旧签名'), findsOneWidget);
  });

  testWidgets('saving sends the edited fields to the api', (tester) async {
    final api = FakeProfileApi(sampleProfile());
    await tester.pumpWidget(_wrap(api));
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextField).at(0), '新名');
    await tester.enterText(find.byType(TextField).at(1), '新签名');
    await tester.tap(find.text('保存'));
    await tester.pumpAndSettle();

    expect(api.lastUpdate, {'display_name': '新名', 'bio': '新签名'});
  });

  testWidgets('save without a hot wallet shows guidance and skips the call',
      (tester) async {
    final api = FakeProfileApi(sampleProfile());
    await tester.pumpWidget(
      _wrap(api, sessionProvider: FakeSessionProvider(null)),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('保存'));
    await tester.pumpAndSettle();

    expect(api.lastUpdate, isNull);
    expect(find.text('请先在「我的 → 我的钱包」创建热钱包'), findsOneWidget);
  });
}
