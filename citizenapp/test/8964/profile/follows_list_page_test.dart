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
    cidNumber: "CN220-CTZN2-198805200-2026",
    bindingRevision: 1,
    accountId:
        '0x6666666666666666666666666666666666666666666666666666666666666666',
    expiresAt: 9999999999999,
  );
  testWidgets('renders follow entries as rows', (tester) async {
    const cidA = 'CN001-CTZN-000000001-2026';
    const cidB = 'CN001-CTZN-000000002-2026';
    final api = FakeProfileApi(
      sampleProfile(),
      follows: const [
        SquareFollowEntry(cidNumber: cidA, createdAt: 200),
        SquareFollowEntry(cidNumber: cidB, createdAt: 100),
      ],
      throwOnProfile: true,
    );
    await tester.pumpWidget(
      MaterialApp(
        home: FollowsListPage(
          cidNumber: kOwner,
          type: FollowsType.following,
          session: session,
          api: api,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byType(ListTile), findsNWidgets(2));
    expect(find.text('关注'), findsOneWidget);
    // 缺公开资料时按身份主键 cid_number 稳定派生默认昵称。
    expect(
      find.text(ProfilePresentation.forIdentityKey(cidA).fallbackName),
      findsOneWidget,
    );
    // 副标题展示身份主键 cid_number（完整）。
    expect(find.text(cidA), findsOneWidget);
    expect(find.text(cidB), findsOneWidget);
  });

  testWidgets('shows empty state when there are no followers', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: FollowsListPage(
          cidNumber: kOwner,
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
