import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/chat/open_direct_chat.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/user/contact_book_page.dart';
import 'package:citizenapp/my/user/contact_service.dart';
import 'package:citizenapp/ui/app_theme.dart';

const _accountId =
    '0x2222222222222222222222222222222222222222222222222222222222222222';
const _contactAddress = 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT';
const _contactAccountId =
    '0x1111111111111111111111111111111111111111111111111111111111111111';

/// 联系人身份主键 CID 号。通讯录关系按 cid_number 建立，页面直接用它索引公开资料。
const _contactCidNumber = 'CN220-CTZN2-100000001-2026';
const _contact = UserContact(
  cidNumber: _contactCidNumber,
  accountId: _contactAccountId,
  ss58Address: _contactAddress,
  contactRemark: '张三',
  createdAt: 1,
  updatedAt: 2,
);
const _profile = CitizenProfile(
  accountId: _contactAccountId,
  displayName: 'Rhett',
  bio: '建设一个可信、自由的社会',
  avatarObjectKey: null,
  bannerObjectKey: null,
  cidNumber: _contactCidNumber,
  isCertified: true,
  identityLevel: 'voting',
  membershipLevel: 'democracy',
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
  Future<List<UserContact>> refreshContactBindings() async => contacts;

  @override
  Future<UserContact> resolveCurrentContact(String cidNumber) async =>
      contacts.singleWhere((contact) => contact.cidNumber == cidNumber);

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
    String cidNumber,
    String contactRemark,
  ) async {
    contacts = <UserContact>[
      _contact.copyWith(contactRemark: contactRemark, updatedAt: 3),
    ];
    return contacts;
  }

  @override
  Future<List<UserContact>> deleteContact(String cidNumber) async {
    contacts = const <UserContact>[];
    return contacts;
  }
}

class _PendingContacts extends _FakeContacts {
  final Completer<List<UserContact>> completer = Completer<List<UserContact>>();

  @override
  Future<List<UserContact>> getContacts() => completer.future;
}

class _FakeProfileApi extends CitizenProfileApi {
  _FakeProfileApi(this.profile);

  final CitizenProfile profile;

  @override
  Future<CitizenProfile> fetchProfile(
    String cidNumber, {
    SquareSession? session,
  }) async =>
      profile;
}

class _FakeSessionProvider extends SquareSessionProvider {
  @override
  Future<SquareSession?> ensureSession() async => SquareSession(
        sessionToken: 'token',
        cidNumber: "CN220-CTZN2-198805200-2026",
        bindingRevision: 1,
        accountId: _accountId,
        expiresAt: DateTime.now().millisecondsSinceEpoch + 60000,
      );
}

Widget _page({
  ContactPickMode mode = ContactPickMode.browse,
  CitizenProfile profile = _profile,
  UserContactService? service,
  DirectChatOpener? directChatOpener,
  Future<void> Function(
    BuildContext context, {
    required String toSs58Address,
  })? transferOpener,
}) =>
    MaterialApp(
      home: ContactBookPage(
        mode: mode,
        service: service ?? _FakeContacts(),
        profileApi: _FakeProfileApi(profile),
        sessionProvider: _FakeSessionProvider(),
        initialProfiles: {_contactCidNumber: profile},
        directChatOpener: directChatOpener,
        transferOpener: transferOpener,
      ),
    );

/// 身份账户缓存 fake：resolve/accountId 返回 null，让点开他人主页时 _resolveOwnAccount
/// 回退成「非本人」（行为与迁移前一致）；避免 instance 触发真链读/真 Isar。
class _NullIdentityCache extends IdentityAccountCache {
  @override
  Future<ResolvedIdentity?> resolve({bool allowChainRead = true}) async => null;
  @override
  Future<String?> accountId({bool allowChainRead = true}) async => null;
}

void main() {
  setUp(() {
    IdentityAccountCache.debugInstance = _NullIdentityCache();
  });

  tearDown(IdentityAccountCache.resetDebugInstance);

  testWidgets('本地通讯录未返回时直接显示页面结构且不使用整页转圈', (tester) async {
    final service = _PendingContacts();
    await tester.pumpWidget(_page(service: service));
    await tester.pump();

    expect(find.text('我的通讯录'), findsOneWidget);
    expect(find.byKey(const ValueKey('contact-search')), findsOneWidget);
    expect(
      find.byKey(const ValueKey('contacts-local-load-progress')),
      findsOneWidget,
    );
    expect(find.text('正在读取本地通讯录'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsNothing);

    service.completer.complete(const <UserContact>[]);
    await tester.pumpAndSettle();
    expect(
      find.byKey(const ValueKey('contacts-local-load-progress')),
      findsNothing,
    );
  });

  testWidgets('联系人卡以公开昵称为主并分别展示备注、CID、SS58', (tester) async {
    await tester.pumpWidget(_page());
    await tester.pumpAndSettle();

    expect(find.text('云端已同步'), findsOneWidget);
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('contact-card-$_contactCidNumber')),
        matching: find.text('Rhett'),
      ),
      findsOneWidget,
    );
    expect(find.text('备注：张三'), findsOneWidget);
    expect(find.text('CID：$_contactCidNumber'), findsOneWidget);
    expect(find.text('SS58：$_contactAddress'), findsOneWidget);
    expect(find.text('建设一个可信、自由的社会'), findsOneWidget);
    expect(find.byKey(const ValueKey('contact-card-$_contactCidNumber')),
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
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('contact-card-$_contactCidNumber')),
        matching: find.text('Rhett'),
      ),
      findsOneWidget,
    );

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
        ProfilePresentation.forIdentityKey(_contactCidNumber).fallbackName;
    expect(find.text(fallback), findsOneWidget);
    expect(find.text('SS58：$_contactAddress'), findsOneWidget);
  });

  testWidgets('普通点击进入唯一 UserProfilePage', (tester) async {
    await tester.pumpWidget(_page());
    await tester.pumpAndSettle();

    await tester.tap(find.text('Rhett'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));

    expect(find.byType(UserProfilePage), findsOneWidget);
  });

  testWidgets('联系人菜单顺序正确且删除联系人使用红色文字', (tester) async {
    await tester.pumpWidget(_page());
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('联系人操作'));
    await tester.pumpAndSettle();

    final labels = <String>['转账', '私信', '修改备注', '删除联系人'];
    for (final label in labels) {
      expect(find.text(label), findsOneWidget);
    }
    final deleteText = tester.widget<Text>(find.text('删除联系人'));
    expect(deleteText.style?.color, AppTheme.danger);
  });

  testWidgets('修改私人备注可取消、保存中文或清空', (tester) async {
    await tester.pumpWidget(_page());
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('联系人操作'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('修改备注'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextFormField), '李四');
    await tester.tap(find.text('取消'));
    await tester.pumpAndSettle();
    expect(find.text('备注：张三'), findsOneWidget);
    expect(tester.takeException(), isNull);

    await tester.tap(find.byTooltip('联系人操作'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('修改备注'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextFormField), '李四');
    await tester.tap(find.text('保存'));
    await tester.pumpAndSettle();
    expect(find.text('备注：李四'), findsOneWidget);
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
    String? openedPeerCidNumber;
    String? openedTitle;
    Future<void> opener(
      BuildContext context, {
      required String peerCidNumber,
      required String title,
    }) async {
      // 注入只用于断言路由参数，不替代正式 openDirectChat 实现。
      openedPeerCidNumber = peerCidNumber;
      openedTitle = title;
    }

    await tester.pumpWidget(_page(directChatOpener: opener));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('联系人操作'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('私信'));
    await tester.pump();

    expect(openedPeerCidNumber, _contactCidNumber);
    expect(openedTitle, 'Rhett');
  });

  testWidgets('发私信模式点联系人直接开私聊、无操作菜单', (tester) async {
    String? openedPeerCidNumber;
    String? openedTitle;
    Future<void> opener(
      BuildContext context, {
      required String peerCidNumber,
      required String title,
    }) async {
      openedPeerCidNumber = peerCidNumber;
      openedTitle = title;
    }

    await tester.pumpWidget(_page(
      mode: ContactPickMode.pickForMessage,
      directChatOpener: opener,
    ));
    await tester.pumpAndSettle();

    // 选私信模式:不显示逐项操作菜单,点联系人卡即开私聊。
    expect(find.byTooltip('联系人操作'), findsNothing);
    await tester.tap(find.text('Rhett'));
    await tester.pump();

    expect(openedPeerCidNumber, _contactCidNumber);
    expect(openedTitle, 'Rhett');
  });
}
