import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';

import 'fake_profile.dart';

/// 身份账户缓存 fake：resolve/accountId 返回 null，让 _resolveOwnAccount 回退成
/// 「非本人」（行为与迁移前一致）；避免 instance 触发真链读/真 Isar。
class _NullIdentityCache extends IdentityAccountCache {
  @override
  Future<ResolvedIdentity?> resolve({bool allowChainRead = true}) async => null;
  @override
  Future<String?> accountId({bool allowChainRead = true}) async => null;
}

Widget _wrap({required bool isSelf}) => MaterialApp(
      home: UserProfilePage(
        cidNumber: kOwner,
        isSelf: isSelf,
        api: FakeProfileApi(sampleProfile()),
        cache: FakeProfileCache(),
        sessionProvider: FakeSessionProvider(fakeSession()),
      ),
    );

void main() {
  setUp(() {
    IdentityAccountCache.debugInstance = _NullIdentityCache();
  });

  tearDown(IdentityAccountCache.resetDebugInstance);

  testWidgets('renders 5 category tabs with back and more actions',
      (tester) async {
    await tester.pumpWidget(_wrap(isSelf: true));
    await tester.pumpAndSettle();

    for (final label in ['帖子', '竞选', '照片', '视频', '文章']) {
      expect(find.text(label), findsOneWidget);
    }
    // 当前资料页统一使用细体左箭头返回，测试与已确认的正式 UI 保持一致。
    expect(find.byIcon(Icons.chevron_left), findsOneWidget);
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
