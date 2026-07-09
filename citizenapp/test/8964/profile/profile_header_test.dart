import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/profile/user_qr_page.dart';

import 'profile_test_doubles.dart';

Widget _wrap({
  required bool isSelf,
  required CitizenProfileApi api,
  CitizenProfileCache? cache,
  SquareSessionProvider? sessionProvider,
}) {
  return MaterialApp(
    home: UserProfilePage(
      ownerAccount: kOwner,
      isSelf: isSelf,
      api: api,
      cache: cache ?? FakeProfileCache(),
      sessionProvider: sessionProvider ?? FakeSessionProvider(null),
    ),
  );
}

void main() {
  testWidgets('self profile shows three icons and edit-profile in kebab',
      (tester) async {
    await tester.pumpWidget(
      _wrap(isSelf: true, api: FakeProfileApi(sampleProfile())),
    );
    await tester.pumpAndSettle();

    expect(find.byIcon(Icons.notifications_outlined), findsOneWidget);
    expect(find.byIcon(Icons.chat_bubble_outline), findsOneWidget);
    expect(find.byIcon(Icons.people_outline), findsOneWidget);
    // 认证以头像角的勾号呈现（推特式，去掉旧「认证公民」胶囊）。
    expect(find.byIcon(Icons.verified), findsOneWidget);

    await tester.tap(find.byIcon(Icons.more_vert));
    await tester.pumpAndSettle();
    expect(find.text('二维码'), findsOneWidget);
    expect(find.text('编辑资料'), findsOneWidget);
    expect(find.text('举报'), findsNothing);
  });

  testWidgets('other profile shows follow + message and report in kebab',
      (tester) async {
    await tester.pumpWidget(
      _wrap(
          isSelf: false, api: FakeProfileApi(sampleProfile(following: false))),
    );
    await tester.pumpAndSettle();

    expect(find.byIcon(Icons.person_add_alt), findsOneWidget);
    expect(find.byIcon(Icons.chat_bubble_outline), findsOneWidget);
    expect(find.byIcon(Icons.notifications_outlined), findsNothing);
    expect(find.byIcon(Icons.people_outline), findsNothing);

    await tester.tap(find.byIcon(Icons.more_vert));
    await tester.pumpAndSettle();
    expect(find.text('举报'), findsOneWidget);
    expect(find.text('编辑资料'), findsNothing);
  });

  testWidgets('renders an avatar image when the profile has an avatar key',
      (tester) async {
    await tester.pumpWidget(
      _wrap(
        isSelf: false,
        api: FakeProfileApi(
          sampleProfile(avatarKey: 'profile/acct/avatar.webp'),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byType(Image), findsWidgets);
    expect(tester.takeException(), isNull);
  });

  testWidgets('uncertified profile hides the verified badge', (tester) async {
    await tester.pumpWidget(
      _wrap(
          isSelf: false, api: FakeProfileApi(sampleProfile(certified: false))),
    );
    await tester.pumpAndSettle();

    expect(find.byIcon(Icons.verified), findsNothing);
  });

  testWidgets('cache-first renders the fetched profile and writes cache',
      (tester) async {
    final api = FakeProfileApi(sampleProfile(displayName: '刷新名'));
    final cache = FakeProfileCache();
    await tester.pumpWidget(_wrap(isSelf: true, api: api, cache: cache));
    await tester.pumpAndSettle();

    expect(api.calls, 1);
    expect(cache.wrote, isTrue);
    expect(find.text('刷新名'), findsWidgets);
  });

  testWidgets('following a user optimistically flips the icon', (tester) async {
    final api = FakeProfileApi(sampleProfile(following: false));
    await tester.pumpWidget(
      _wrap(
        isSelf: false,
        api: api,
        sessionProvider: FakeSessionProvider(fakeSession()),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byIcon(Icons.person_add_alt), findsOneWidget);
    await tester.tap(find.byIcon(Icons.person_add_alt));
    await tester.pumpAndSettle();

    expect(find.byIcon(Icons.how_to_reg), findsOneWidget);
    expect(api.followCalls, 1);
  });

  testWidgets('a failed follow rolls the icon back', (tester) async {
    final api = FakeProfileApi(
      sampleProfile(following: false),
      throwOnFollow: true,
    );
    await tester.pumpWidget(
      _wrap(
        isSelf: false,
        api: api,
        sessionProvider: FakeSessionProvider(fakeSession()),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.person_add_alt));
    await tester.pumpAndSettle();

    expect(find.byIcon(Icons.person_add_alt), findsOneWidget);
    expect(api.followCalls, 1);
  });

  testWidgets('message on another profile opens a direct chat with that user',
      (tester) async {
    String? peer;
    String? chatTitle;
    await tester.pumpWidget(
      MaterialApp(
        home: UserProfilePage(
          ownerAccount: kOwner,
          isSelf: false,
          api: FakeProfileApi(sampleProfile(displayName: '轻节点')),
          cache: FakeProfileCache(),
          sessionProvider: FakeSessionProvider(null),
          onOpenDirectChat: (context, {required peerAddress, required title}) {
            peer = peerAddress;
            chatTitle = title;
            return Future<void>.value();
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.chat_bubble_outline));
    await tester.pumpAndSettle();

    expect(peer, kOwner);
    expect(chatTitle, '轻节点');
  });

  testWidgets('self notifications opens the placeholder page', (tester) async {
    await tester
        .pumpWidget(_wrap(isSelf: true, api: FakeProfileApi(sampleProfile())));
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.notifications_outlined));
    await tester.pumpAndSettle();

    expect(find.text('通知功能即将上线'), findsOneWidget);
  });

  testWidgets('kebab QR code opens the user QR page', (tester) async {
    await tester.pumpWidget(
      _wrap(isSelf: true, api: FakeProfileApi(sampleProfile())),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.more_vert));
    await tester.pumpAndSettle();
    await tester.tap(find.text('二维码'));
    await tester.pumpAndSettle();

    expect(find.byType(UserQrPage), findsOneWidget);
  });
}
