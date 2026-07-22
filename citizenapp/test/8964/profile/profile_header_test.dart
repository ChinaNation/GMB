import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/profile/user_qr_page.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';

import 'fake_profile.dart';

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
      sessionProvider: sessionProvider ?? FakeSessionProvider(fakeSession()),
    ),
  );
}

void main() {
  testWidgets('missing public name uses local nickname and keeps account below',
      (tester) async {
    await tester.pumpWidget(
      _wrap(
        isSelf: false,
        api: FakeProfileApi(sampleProfile(displayName: '')),
      ),
    );
    await tester.pumpAndSettle();

    final fallback = ProfilePresentation.forAccount(kOwner).fallbackName;
    expect(find.text(fallback), findsWidgets);
    expect(find.textContaining(kOwner.substring(0, 6)), findsOneWidget);
    expect(find.text(kOwner), findsNothing);
  });

  testWidgets('self profile hides owner-directed action icons, edit in kebab',
      (tester) async {
    await tester.pumpWidget(
      _wrap(isSelf: true, api: FakeProfileApi(sampleProfile())),
    );
    await tester.pumpAndSettle();

    // 自己看自己：通知/聊天/关注是对主页主人的操作，一律不显示。
    expect(find.byIcon(Icons.notifications_outlined), findsNothing);
    expect(find.byIcon(Icons.chat_bubble_outline), findsNothing);
    expect(find.byIcon(Icons.person_add_alt), findsNothing);
    expect(find.byIcon(Icons.how_to_reg), findsNothing);
    // 认证以头像角的公民徽章呈现（链上身份分色 + 会员匹配带勾）。
    expect(find.byType(IdentityBadge), findsOneWidget);

    await tester.tap(find.byIcon(Icons.more_vert));
    await tester.pumpAndSettle();
    expect(find.text('二维码'), findsOneWidget);
    expect(find.text('编辑资料'), findsOneWidget);
  });

  testWidgets('other profile shows subscribe, chat and follow', (tester) async {
    await tester.pumpWidget(
      _wrap(
          isSelf: false, api: FakeProfileApi(sampleProfile(following: false))),
    );
    await tester.pumpAndSettle();

    // 看别人主页：通知(订阅)/聊天/关注 三个图标都在。
    expect(find.byIcon(Icons.notifications_outlined), findsOneWidget);
    expect(find.byIcon(Icons.chat_bubble_outline), findsOneWidget);
    expect(find.byIcon(Icons.person_add_alt), findsOneWidget);

    await tester.tap(find.byIcon(Icons.more_vert));
    await tester.pumpAndSettle();
    expect(find.text('二维码'), findsOneWidget);
    expect(find.text('编辑资料'), findsNothing);
  });

  testWidgets('renders an avatar image when the profile has an avatar key',
      (tester) async {
    await tester.pumpWidget(
      _wrap(
        isSelf: false,
        api: FakeProfileApi(
          sampleProfile(avatarKey: 'profile/acct/avatar'),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byType(Image), findsWidgets);
    final networkImage = tester
        .widgetList<Image>(find.byType(Image))
        .map((image) => image.image)
        .whereType<NetworkImage>()
        .single;
    expect(networkImage.headers?['authorization'], 'Bearer tok');
    expect(tester.takeException(), isNull);
  });

  testWidgets('pure visitor shows an orange person badge (no membership)',
      (tester) async {
    await tester.pumpWidget(
      _wrap(
          isSelf: false, api: FakeProfileApi(sampleProfile(certified: false))),
    );
    await tester.pumpAndSettle();

    final badge = tester.widget<IdentityBadge>(find.byType(IdentityBadge));
    expect(badge.style.color, AppTheme.identityVisitor);
    expect(badge.style.checked, isFalse);
  });

  testWidgets('voting identity, no membership -> blue ring, unchecked',
      (tester) async {
    await tester.pumpWidget(
      _wrap(
        isSelf: false,
        api: FakeProfileApi(sampleProfile(identityLevel: 'voting')),
      ),
    );
    await tester.pumpAndSettle();
    final badge = tester.widget<IdentityBadge>(find.byType(IdentityBadge));
    expect(badge.style.color, AppTheme.identityVoting);
    expect(badge.style.checked, isFalse);
  });

  testWidgets('voting identity + voting membership -> blue, checked',
      (tester) async {
    await tester.pumpWidget(
      _wrap(
        isSelf: false,
        api: FakeProfileApi(sampleProfile(
          identityLevel: 'voting',
          membershipLevel: 'voting',
        )),
      ),
    );
    await tester.pumpAndSettle();
    final badge = tester.widget<IdentityBadge>(find.byType(IdentityBadge));
    expect(badge.style.color, AppTheme.identityVoting);
    expect(badge.style.checked, isTrue);
  });

  testWidgets('candidate identity + candidate membership -> red, checked',
      (tester) async {
    await tester.pumpWidget(
      _wrap(
        isSelf: false,
        api: FakeProfileApi(sampleProfile(
          identityLevel: 'candidate',
          membershipLevel: 'candidate',
        )),
      ),
    );
    await tester.pumpAndSettle();
    final badge = tester.widget<IdentityBadge>(find.byType(IdentityBadge));
    expect(badge.style.color, AppTheme.identityCandidate);
    expect(badge.style.checked, isTrue);
  });

  testWidgets('candidate identity + any active membership -> red, checked',
      (tester) async {
    // 规则简化：买了会员（任意档）就带勾，颜色仍按链上身份=竞选红。
    await tester.pumpWidget(
      _wrap(
        isSelf: false,
        api: FakeProfileApi(sampleProfile(
          identityLevel: 'candidate',
          membershipLevel: 'voting',
        )),
      ),
    );
    await tester.pumpAndSettle();
    final badge = tester.widget<IdentityBadge>(find.byType(IdentityBadge));
    expect(badge.style.color, AppTheme.identityCandidate);
    expect(badge.style.checked, isTrue);
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
          sessionProvider: FakeSessionProvider(fakeSession()),
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

  testWidgets('从他人视角看的是自己账户时私信按钮置灰不触发', (tester) async {
    String? peer;
    await tester.pumpWidget(
      MaterialApp(
        home: UserProfilePage(
          ownerAccount: kOwner,
          isSelf: false,
          api: FakeProfileApi(sampleProfile(displayName: '轻节点')),
          cache: FakeProfileCache(),
          sessionProvider: FakeSessionProvider(fakeSession()),
          // 浏览者账户 == 主页账户 = 他人视角看自己 → 按钮应置灰。
          viewerAccountLoader: () async => kOwner,
          onOpenDirectChat: (context, {required peerAddress, required title}) {
            peer = peerAddress;
            return Future<void>.value();
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    // 按钮仍显示（保留他人视角版式）但禁用：点了不触发私信。
    expect(find.byIcon(Icons.chat_bubble_outline), findsOneWidget);
    await tester.tap(find.byIcon(Icons.chat_bubble_outline));
    await tester.pumpAndSettle();
    expect(peer, isNull);
  });

  testWidgets('other profile bell prompts to follow first when not following',
      (tester) async {
    final api = FakeProfileApi(sampleProfile(following: false));
    await tester.pumpWidget(_wrap(isSelf: false, api: api));
    await tester.pumpAndSettle();

    // 未关注时铃铛为空心；点它提示先关注（通知归属挂在关注关系上），不发通知请求。
    await tester.tap(find.byIcon(Icons.notifications_outlined));
    await tester.pumpAndSettle();

    expect(find.textContaining('请先关注'), findsOneWidget);
    expect(api.notifyCalls, 0);
  });

  testWidgets('other profile bell mutes notify when following and notifying',
      (tester) async {
    final api = FakeProfileApi(sampleProfile(following: true, notifying: true));
    await tester.pumpWidget(_wrap(isSelf: false, api: api));
    await tester.pumpAndSettle();

    // 已关注且开通知：铃铛为实心 active；点它静音（enabled=false）。
    expect(find.byIcon(Icons.notifications_active), findsOneWidget);
    await tester.tap(find.byIcon(Icons.notifications_active));
    await tester.pumpAndSettle();

    expect(api.notifyCalls, 1);
    expect(api.lastNotifyEnabled, isFalse);
    // 乐观更新后铃铛转为空心。
    expect(find.byIcon(Icons.notifications_outlined), findsOneWidget);
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
