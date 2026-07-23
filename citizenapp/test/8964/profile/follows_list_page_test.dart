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
    accountId:
        '0x6666666666666666666666666666666666666666666666666666666666666666',
    expiresAt: 9999999999999,
  );
  testWidgets('renders follow entries as rows', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      follows: const [
        SquareFollowEntry(
            accountId:
                '0x0101010101010101010101010101010101010101010101010101010101010101',
            createdAt: 200),
        SquareFollowEntry(
            accountId:
                '0x0202020202020202020202020202020202020202020202020202020202020202',
            createdAt: 100),
      ],
      throwOnProfile: true,
    );
    await tester.pumpWidget(
      MaterialApp(
        home: FollowsListPage(
          accountId: kOwner,
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
      find.text(ProfilePresentation.forAccount(
              '0x0101010101010101010101010101010101010101010101010101010101010101')
          .fallbackName),
      findsOneWidget,
    );
    expect(find.text('0x0101...010101'), findsOneWidget);
    expect(find.text('0x0202...020202'), findsOneWidget);
  });

  testWidgets('shows empty state when there are no followers', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: FollowsListPage(
          accountId: kOwner,
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
