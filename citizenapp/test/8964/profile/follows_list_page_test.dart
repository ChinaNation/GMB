import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/follows_list_page.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';

import 'fake_profile.dart';

void main() {
  const session = SquareSession(
    sessionToken: 'test-session',
    ownerAccount: 'viewer',
    expiresAt: 9999999999999,
  );
  testWidgets('renders follow entries as rows', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      follows: const [
        SquareFollowEntry(ownerAccount: 'a______________1', createdAt: 200),
        SquareFollowEntry(ownerAccount: 'a______________2', createdAt: 100),
      ],
      throwOnProfile: true,
    );
    await tester.pumpWidget(
      MaterialApp(
        home: FollowsListPage(
          ownerAccount: kOwner,
          type: FollowsType.following,
          session: session,
          api: api,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byType(ListTile), findsNWidgets(2));
    expect(find.text('关注'), findsOneWidget);
    expect(
      find.text(
          ProfilePresentation.forAccount('a______________1').fallbackName),
      findsOneWidget,
    );
    expect(find.textContaining('a_____'), findsNWidgets(2));
  });

  testWidgets('shows empty state when there are no followers', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: FollowsListPage(
          ownerAccount: kOwner,
          type: FollowsType.followers,
          session: session,
          api: FakeProfileApi(sampleProfile()),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('还没有关注者'), findsOneWidget);
  });
}
