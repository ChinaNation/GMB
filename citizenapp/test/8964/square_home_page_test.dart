import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_home_page.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/widgets/local_identity_avatar.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/my/myid/identity_account_cache.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// 身份账户缓存 fake:resolve 返回 null,让 loadCurrent 回退 wallet.accountId
/// (身份=账户0 常态),行为与迁移前一致;避免 instance 触发真链读/真 Isar。
class _NullIdentityCache extends IdentityAccountCache {
  @override
  Future<ResolvedIdentity?> resolve({bool allowChainRead = true}) async => null;
  @override
  Future<String?> accountId({bool allowChainRead = true}) async => null;
}

class _FakeWalletManager extends WalletManager {
  _FakeWalletManager(this.wallet);

  final WalletProfile? wallet;

  @override
  Future<WalletProfile?> getWallet() async => wallet;

  @override
  Future<WalletProfile?> getDefaultWallet() async => wallet;
}

const _registeredWallet = WalletProfile(
  walletIndex: 1,
  walletName: '测试钱包',
  walletIcon: '',
  balance: 0,
  ss58Address: 'gmb_test_account_id',
  accountId:
      '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
  alg: 'sr25519',
  ss58: 2027,
  createdAtMillis: 1,
  source: 'test',
  signMode: 'local',
);

SquareIdentityService _registeredIdentityService({
  _FakeSquareChainService? chainService,
}) =>
    SquareIdentityService(
      walletManager: _FakeWalletManager(_registeredWallet),
      chainService:
          chainService ?? _FakeSquareChainService('CN220-CTZN2-100000001-2026'),
    );

class _FakeSquareChainService extends SquareChainService {
  _FakeSquareChainService(this.cidNumber);

  final String? cidNumber;
  int fetchIdentityCount = 0;

  @override
  Future<String?> fetchNormalCitizenCidNumber(String accountId) async {
    return cidNumber;
  }

  @override
  Future<({String? cidNumber, String identityLevel})> fetchIdentity(
    String accountId,
  ) async {
    fetchIdentityCount += 1;
    return (
      cidNumber: cidNumber,
      identityLevel: cidNumber == null ? 'visitor' : 'voting',
    );
  }
}

class _FakeFeedSource implements SquareFeedSource {
  const _FakeFeedSource();

  @override
  Future<List<SquarePost>> fetchFeed({
    required SquareFeedKind feedKind,
    int limit = 20,
    SquareSession? session,
  }) async {
    return const <SquarePost>[];
  }
}

/// 模拟后台通知在创建会话时抛出非 Exception 错误，验证 unawaited 边界完整。
class _ThrowingSessionProvider extends SquareSessionProvider {
  @override
  Future<SquareSession?> ensureSession() async {
    throw StateError('session unavailable');
  }
}

/// 保持生产路径类型判断成立，但不访问真实 Worker。
class _FakeSquareApiClient extends SquareApiClient {
  @override
  Future<List<SquarePost>> fetchFeed({
    required SquareFeedKind feedKind,
    int limit = 20,
    SquareSession? session,
  }) async {
    return const <SquarePost>[];
  }
}

/// 记录最近一次请求的分类，用于断言分类切换真的按 feedKind 重新拉流。
class _RecordingFeedSource implements SquareFeedSource {
  SquareFeedKind? lastFeedKind;
  int calls = 0;

  @override
  Future<List<SquarePost>> fetchFeed({
    required SquareFeedKind feedKind,
    int limit = 20,
    SquareSession? session,
  }) async {
    calls++;
    lastFeedKind = feedKind;
    return const <SquarePost>[];
  }
}

/// 控制 feed 完成时机，验证网络未返回时页面结构仍已显示。
class _PendingFeedSource implements SquareFeedSource {
  final Completer<List<SquarePost>> completer = Completer<List<SquarePost>>();
  int calls = 0;

  @override
  Future<List<SquarePost>> fetchFeed({
    required SquareFeedKind feedKind,
    int limit = 20,
    SquareSession? session,
  }) {
    calls += 1;
    return completer.future;
  }
}

/// 按分类返回不同夹具的假数据源；模拟 Worker 对每个 feed 端点各自过滤
/// （关注流走 `/square/feed/following` 的 JOIN 结果）。
class _KindFeedSource implements SquareFeedSource {
  _KindFeedSource({this.following = const <SquarePost>[]});

  final List<SquarePost> following;

  @override
  Future<List<SquarePost>> fetchFeed({
    required SquareFeedKind feedKind,
    int limit = 20,
    SquareSession? session,
  }) async {
    if (feedKind == SquareFeedKind.following) return following;
    return const <SquarePost>[];
  }
}

SquareMediaItem _media(SquareMediaKind kind) =>
    SquareMediaItem(mediaKind: kind, url: '');

SquarePost _seedPost({
  required String id,
  required String text,
  String? title,
  SquarePostContentFormat format = SquarePostContentFormat.normal,
  List<SquareMediaItem> media = const [],
}) {
  return SquarePost(
    postId: id,
    author: const SquareAuthor(
      accountId:
          '0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd',
      displayName: '作者',
      identityLevel: 'voting',
    ),
    postCategory: SquarePostCategory.normal,
    contentFormat: format,
    text: text,
    title: title,
    createdAt: DateTime.fromMillisecondsSinceEpoch(1000),
    mediaItems: media,
  );
}

Widget _wrap(Widget child) {
  return MaterialApp(
    theme: AppTheme.lightTheme,
    home: Scaffold(body: child),
  );
}

void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
    IdentityAccountCache.debugInstance = _NullIdentityCache();
  });

  tearDown(IdentityAccountCache.resetDebugInstance);

  testWidgets('feed 未返回时直接显示广场页面且不使用整页转圈', (tester) async {
    final feedSource = _PendingFeedSource();

    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: SquareIdentityService(
          walletManager: _FakeWalletManager(null),
        ),
        feedSource: feedSource,
        membershipLoader: () async => null,
      )),
    );
    await tester.pump();
    await tester.pump();

    expect(feedSource.calls, 1);
    expect(find.byTooltip('发布动态'), findsOneWidget);
    expect(find.byKey(const ValueKey('square-tank-watermark')), findsOneWidget);
    expect(find.byKey(const ValueKey('square-feed-progress')), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsNothing);
    expect(find.text('广场内容加载失败'), findsNothing);

    feedSource.completer.complete(const <SquarePost>[]);
    await tester.pumpAndSettle();

    expect(feedSource.calls, 1);
    expect(find.byKey(const ValueKey('square-feed-progress')), findsNothing);
    expect(find.text('广场内容加载失败'), findsNothing);
  });

  testWidgets('广场顶部删旧标题/空态字、显示坦克水印与左上头像并可切换分类', (tester) async {
    final identityService = SquareIdentityService(
      walletManager: _FakeWalletManager(null),
    );
    final feedSource = _RecordingFeedSource();

    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: identityService,
        feedSource: feedSource,
        membershipLoader: () async => const SquareMembershipState(
          active: true,
          paidUntil: 9999999999999,
          membershipLevel: 'freedom',
        ),
      )),
    );
    await tester.pumpAndSettle();

    // 顶部大小标题与空态图标/文字彻底删除。
    expect(find.text('广场'), findsNothing);
    expect(find.text('暂无推荐动态'), findsNothing);
    expect(find.text('暂无关注动态'), findsNothing);
    expect(find.text('暂无竞选动态'), findsNothing);

    // 中央坦克水印 + 保留的发布按钮（顶部头像入口已删）。
    expect(find.byKey(const ValueKey<String>('square-tank-watermark')),
        findsOneWidget);
    // 顶部头像入口已删除；发布改为右下角悬浮 FAB（仍带「发布动态」tooltip）。
    expect(find.byType(LocalIdentityAvatar), findsNothing);
    expect(find.byType(FloatingActionButton), findsOneWidget);
    expect(find.byTooltip('发布动态'), findsOneWidget);

    // 三分类可切换：点击后按对应 feedKind 重新拉流。
    expect(find.text('推荐'), findsOneWidget);
    expect(feedSource.lastFeedKind, SquareFeedKind.recommended);

    await tester.tap(find.text('关注'));
    await tester.pumpAndSettle();
    expect(feedSource.lastFeedKind, SquareFeedKind.following);

    await tester.tap(find.text('竞选'));
    await tester.pumpAndSettle();
    expect(feedSource.lastFeedKind, SquareFeedKind.campaign);

    await tester.tap(find.text('文章'));
    await tester.pumpAndSettle();
    expect(feedSource.lastFeedKind, SquareFeedKind.article);

    await tester.tap(find.text('照片'));
    await tester.pumpAndSettle();
    expect(feedSource.lastFeedKind, SquareFeedKind.photos);

    await tester.tap(find.text('视频'));
    await tester.pumpAndSettle();
    expect(feedSource.lastFeedKind, SquareFeedKind.videos);
  });

  testWidgets('广场按内容分类过滤 seedPosts（文章/照片/视频互不串档）', (tester) async {
    // 带媒体的卡片较高，默认 600 视口会让第二张之后懒加载不构建；用高视口保证三帖都渲染。
    tester.view.physicalSize = const Size(500, 3000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final seed = [
      _seedPost(id: 'a', text: '照片帖', media: [_media(SquareMediaKind.image)]),
      _seedPost(id: 'b', text: '视频帖', media: [_media(SquareMediaKind.video)]),
      _seedPost(
        id: 'c',
        text: '文章正文C',
        title: '文章帖',
        format: SquarePostContentFormat.article,
      ),
    ];
    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: SquareIdentityService(
          walletManager: _FakeWalletManager(null),
        ),
        feedSource: _RecordingFeedSource(),
        seedPosts: seed,
        membershipLoader: () async => const SquareMembershipState(
          active: true,
          paidUntil: 9999999999999,
          membershipLevel: 'freedom',
        ),
      )),
    );
    await tester.pumpAndSettle();

    // 推荐：三帖全在。
    expect(find.text('照片帖'), findsOneWidget);
    expect(find.text('视频帖'), findsOneWidget);
    expect(find.text('文章帖'), findsOneWidget);

    await tester.tap(find.text('文章'));
    await tester.pumpAndSettle();
    expect(find.text('文章帖'), findsOneWidget);
    expect(find.text('照片帖'), findsNothing);
    expect(find.text('视频帖'), findsNothing);

    await tester.tap(find.text('照片'));
    await tester.pumpAndSettle();
    expect(find.text('照片帖'), findsOneWidget);
    expect(find.text('视频帖'), findsNothing);
    expect(find.text('文章帖'), findsNothing);

    await tester.tap(find.text('视频'));
    await tester.pumpAndSettle();
    expect(find.text('视频帖'), findsOneWidget);
    expect(find.text('照片帖'), findsNothing);
    expect(find.text('文章帖'), findsNothing);
  });

  testWidgets('无订阅钱包禁止打开任何发布页', (tester) async {
    final chainService = _FakeSquareChainService('CN220-CTZN2-100000001-2026');
    final identityService =
        _registeredIdentityService(chainService: chainService);

    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: identityService,
        feedSource: const _FakeFeedSource(),
        membershipLoader: () async => const SquareMembershipState(
          active: false,
          paidUntil: 0,
        ),
      )),
    );
    await tester.pumpAndSettle();

    // 广场首页只读本地徽章快照，不读取链。
    expect(chainService.fetchIdentityCount, 0);

    await tester.tap(find.byTooltip('发布动态'));
    await tester.pumpAndSettle();

    // 发布动作先做一次真实链身份校验，再由会员门禁阻断，不打开编辑器。
    expect(find.text('需要有效会员才能发布广场内容'), findsOneWidget);
    expect(find.text('发动态'), findsNothing);
    expect(find.text('发文章'), findsNothing);
    expect(chainService.fetchIdentityCount, 1);
  });

  testWidgets('会员 finalized 验证异常时提示稍后重试，不误报无会员', (tester) async {
    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: _registeredIdentityService(),
        feedSource: const _FakeFeedSource(),
        membershipLoader: () async => throw const SquareApiException(
          '暂时无法验证会员状态，请稍后重试',
          statusCode: 503,
          errorCode: 'membership_verification_unavailable',
        ),
      )),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('发布动态'));
    await tester.pumpAndSettle();

    expect(find.text('暂时无法验证会员状态，请稍后重试'), findsOneWidget);
    expect(find.text('需要有效会员才能发布广场内容'), findsNothing);
  });

  testWidgets('会员状态缺失时提示验证失败，不把 null 当成无会员', (tester) async {
    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: _registeredIdentityService(),
        feedSource: const _FakeFeedSource(),
        membershipLoader: () async => null,
      )),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('发布动态'));
    await tester.pumpAndSettle();

    expect(find.text('暂时无法验证会员状态，请稍后重试'), findsOneWidget);
    expect(find.text('需要有效会员才能发布广场内容'), findsNothing);
  });

  testWidgets('会员检查会话失效时提示重新登录', (tester) async {
    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: _registeredIdentityService(),
        feedSource: const _FakeFeedSource(),
        membershipLoader: () async => throw const SquareApiException(
          '钱包登录态已过期',
          statusCode: 401,
          errorCode: 'expired_session',
        ),
      )),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('发布动态'));
    await tester.pumpAndSettle();

    expect(find.text('广场登录状态已失效，请重试'), findsOneWidget);
  });

  group('关注流', () {
    testWidgets('渲染服务端关注帖(动态+文章)，本地种子不混入', (tester) async {
      final feedSource = _KindFeedSource(
        following: [
          _seedPost(id: 'f1', text: '关注动态AA'),
          _seedPost(
            id: 'f2',
            text: '文章摘要',
            title: '关注文章BB',
            format: SquarePostContentFormat.article,
          ),
        ],
      );

      await tester.pumpWidget(
        _wrap(SquareHomePage(
          identityService: SquareIdentityService(
            walletManager: _FakeWalletManager(null),
          ),
          feedSource: feedSource,
          seedPosts: [_seedPost(id: 's1', text: '种子SS')],
          membershipLoader: () async => const SquareMembershipState(
            active: true,
            paidUntil: 9999999999999,
            membershipLevel: 'freedom',
          ),
        )),
      );
      await tester.pumpAndSettle();

      // 推荐流(默认)：种子帖在。
      expect(find.text('种子SS'), findsOneWidget);

      // 关注流：只服务端 following 结果(动态+文章)，种子不混入。
      await tester.tap(find.text('关注'));
      await tester.pumpAndSettle();
      expect(find.text('关注动态AA'), findsOneWidget);
      expect(find.text('关注文章BB'), findsOneWidget);
      expect(find.text('种子SS'), findsNothing);
    });
  });

  testWidgets('信息流与后台通知会话快速失败时不产生未捕获异步异常', (tester) async {
    await tester.pumpWidget(
      _wrap(SquareHomePage(
        identityService: SquareIdentityService(
          walletManager: _FakeWalletManager(null),
        ),
        feedSource: _FakeSquareApiClient(),
        sessionProvider: _ThrowingSessionProvider(),
        membershipLoader: () async => null,
      )),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(tester.takeException(), isNull);
  });
}
