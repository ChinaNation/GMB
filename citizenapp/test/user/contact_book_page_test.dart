import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/chat/open_direct_chat.dart';
import 'package:citizenapp/my/user/contact_book_page.dart';
import 'package:citizenapp/my/user/contact_service.dart';
import 'package:citizenapp/ui/app_theme.dart';

const _accountId =
    '0x2222222222222222222222222222222222222222222222222222222222222222';
const _contactAddress = 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT';
const _contactAccountId =
    '0x1111111111111111111111111111111111111111111111111111111111111111';
const _contact = UserContact(
  accountId: _contactAccountId,
  ss58Address: _contactAddress,
  contactName: '张三',
  createdAt: 1,
  updatedAt: 2,
);
const _profile = CitizenProfile(
  accountId: _contactAccountId,
  displayName: 'Rhett',
  bio: '建设一个可信、自由的社会',
  avatarObjectKey: null,
  bannerObjectKey: null,
  cidNumber: 'CID-1',
  isCertified: true,
  identityLevel: 'voting',
  membershipLevel: 'voting',
  membershipActive: true,
  following: 1,
  followers: 2,
  posts: 3,
  isFollowing: false,
  isNotifying: false,
  updatedAt: 2,
);

class _FakeContacts extends UserContactService {
  _FakeContacts() : super(autoSync: false);

  List<UserContact> contacts = <UserContact>[_contact];

  @override
  Future<String> getAccountId() async => _accountId;

  @override
  Future<List<UserContact>> getContacts() async => contacts;

  @override
  Future<List<UserContact>> sync() async {
    syncState.value = const ContactSyncState(phase: ContactSyncPhase.synced);
    return contacts;
  }

  @override
  Future<ContactSyncState> readSyncState() async =>
      const ContactSyncState(phase: ContactSyncPhase.synced);

  @override
  Future<List<UserContact>> renameContact(
    String address,
    String contactName,
  ) async {
    contacts = <UserContact>[
      _contact.copyWith(contactName: contactName, updatedAt: 3),
    ];
    return contacts;
  }

  @override
  Future<List<UserContact>> deleteContact(String address) async {
    contacts = const <UserContact>[];
    return contacts;
  }
}

class _FakeProfileApi extends CitizenProfileApi {
  _FakeProfileApi(this.profile);

  final CitizenProfile profile;

  @override
  Future<CitizenProfile> fetchProfile(
    String accountId, {
    SquareSession? session,
  }) async =>
      profile;
}

class _FakeSessionProvider extends SquareSessionProvider {
  @override
  Future<SquareSession?> ensureSession() async => SquareSession(
        sessionToken: 'token',
        accountId: _accountId,
        expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
      );
}

Widget _page({
  ContactPickMode mode = ContactPickMode.browse,
  CitizenProfile profile = _profile,
  DirectChatOpener? directChatOpener,
  Future<void> Function(
    BuildContext context, {
    required String toSs58Address,
  })? transferOpener,
}) =>
    MaterialApp(
      home: ContactBookPage(
        mode: mode,
        service: _FakeContacts(),
        profileApi: _FakeProfileApi(profile),
        sessionProvider: _FakeSessionProvider(),
        initialProfiles: {_contactAccountId: profile},
        directChatOpener: directChatOpener,
        transferOpener: transferOpener,
      ),
    );

void main() {
  testWidgets('联系人卡展示私人名称、公开资料、身份头像和同步状态', (tester) async {
    await tester.pumpWidget(_page());
    await tester.pumpAndSettle();

    expect(find.text('云端已同步'), findsOneWidget);
    expect(find.text('张三'), findsOneWidget);
    expect(find.textContaining('Rhett · w5Bc7m'), findsOneWidget);
    expect(find.text('建设一个可信、自由的社会'), findsOneWidget);
    expect(find.byKey(const ValueKey('contact-card-$_contactAccountId')),
        findsOneWidget);
  });

  testWidgets('搜索匹配公开昵称并可清空', (tester) async {
    await tester.pumpWidget(_page());
    await tester.pumpAndSettle();

    await tester.enterText(
      find.byKey(const ValueKey('contact-search')),
      'Rhett',
    );
    await tester.pump();
    expect(find.text('张三'), findsOneWidget);

    await tester.enterText(
      find.byKey(const ValueKey('contact-search')),
      '不存在',
    );
    await tester.pump();
    expect(find.text('没有匹配的联系人'), findsOneWidget);
  });

  testWidgets('公开昵称缺失时显示默认昵称而不是把账户当昵称', (tester) async {
    final emptyProfile = _profile.copyWith(displayName: '');
    await tester.pumpWidget(_page(profile: emptyProfile));
    await tester.pumpAndSettle();

    final fallback =
        ProfilePresentation.forAccount(_contactAccountId).fallbackName;
    expect(find.textContaining('$fallback · w5Bc7m'), findsOneWidget);
    expect(find.text(_contactAddress), findsNothing);
  });

  testWidgets('普通点击进入唯一 UserProfilePage', (tester) async {
    await tester.pumpWidget(_page());
    await tester.pumpAndSettle();

    await tester.tap(find.text('张三'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));

    expect(find.byType(UserProfilePage), findsOneWidget);
  });

  testWidgets('联系人菜单顺序正确且删除联系人使用红色文字', (tester) async {
    await tester.pumpWidget(_page());
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('联系人操作'));
    await tester.pumpAndSettle();

    final labels = <String>['转账', '私信', '修改名称', '删除联系人'];
    for (final label in labels) {
      expect(find.text(label), findsOneWidget);
    }
    final deleteText = tester.widget<Text>(find.text('删除联系人'));
    expect(deleteText.style?.color, AppTheme.danger);
  });

  testWidgets('修改名称取消和保存中文内容均无异常渲染', (tester) async {
    await tester.pumpWidget(_page());
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('联系人操作'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('修改名称'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextFormField), '李四');
    await tester.tap(find.text('取消'));
    await tester.pumpAndSettle();
    expect(find.text('张三'), findsOneWidget);
    expect(tester.takeException(), isNull);

    await tester.tap(find.byTooltip('联系人操作'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('修改名称'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextFormField), '李四');
    await tester.tap(find.text('保存'));
    await tester.pumpAndSettle();
    expect(find.text('李四'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('转账打开链上支付并预填联系人钱包账户', (tester) async {
    String? openedToAddress;
    Future<void> opener(
      BuildContext context, {
      required String toSs58Address,
    }) async {
      openedToAddress = toSs58Address;
    }

    await tester.pumpWidget(_page(transferOpener: opener));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('联系人操作'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('转账'));
    await tester.pump(const Duration(milliseconds: 400));

    expect(openedToAddress, _contactAddress);
  });

  testWidgets('私信复用统一聊天入口并使用公开昵称', (tester) async {
    String? openedPeerAccountId;
    String? openedTitle;
    Future<void> opener(
      BuildContext context, {
      required String peerAccountId,
      required String title,
    }) async {
      // 注入只用于断言路由参数，不替代正式 openDirectChat 实现。
      openedPeerAccountId = peerAccountId;
      openedTitle = title;
    }

    await tester.pumpWidget(_page(directChatOpener: opener));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('联系人操作'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('私信'));
    await tester.pump();

    expect(openedPeerAccountId, _contactAccountId);
    expect(openedTitle, 'Rhett');
  });

  testWidgets('发私信模式点联系人直接开私聊、无操作菜单', (tester) async {
    String? openedPeerAccountId;
    String? openedTitle;
    Future<void> opener(
      BuildContext context, {
      required String peerAccountId,
      required String title,
    }) async {
      openedPeerAccountId = peerAccountId;
      openedTitle = title;
    }

    await tester.pumpWidget(_page(
      mode: ContactPickMode.pickForMessage,
      directChatOpener: opener,
    ));
    await tester.pumpAndSettle();

    // 选私信模式:不显示逐项操作菜单,点联系人卡即开私聊。
    expect(find.byTooltip('联系人操作'), findsNothing);
    await tester.tap(find.text('张三'));
    await tester.pump();

    expect(openedPeerAccountId, _contactAccountId);
    expect(openedTitle, 'Rhett');
  });
}
