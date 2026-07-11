import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/follows_list_page.dart';
import 'package:citizenapp/8964/profile/models/citizen_profile.dart';

import 'fake_profile.dart';

void main() {
  testWidgets('renders follow entries as rows', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(),
      follows: const [
        SquareFollowEntry(ownerAccount: 'a______________1', createdAt: 200),
        SquareFollowEntry(ownerAccount: 'a______________2', createdAt: 100),
      ],
    );
    await tester.pumpWidget(
      MaterialApp(
        home: FollowsListPage(
          ownerAccount: kOwner,
          type: FollowsType.following,
          api: api,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byType(ListTile), findsNWidgets(2));
    expect(find.text('关注'), findsOneWidget);
  });

  testWidgets('shows empty state when there are no followers', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: FollowsListPage(
          ownerAccount: kOwner,
          type: FollowsType.followers,
          api: FakeProfileApi(sampleProfile()),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('还没有关注者'), findsOneWidget);
  });
}
