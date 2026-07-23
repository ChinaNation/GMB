import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_home_page.dart';
import 'package:citizenapp/8964/profile/widgets/local_identity_avatar.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:shared_preferences/shared_preferences.dart';

class _FakeWalletManager extends WalletManager {
  _FakeWalletManager(this.wallet);

  final WalletProfile? wallet;

  @override
  Future<WalletProfile?> getWallet() async => wallet;

  @override
  Future<WalletProfile?> getDefaultWallet() async => wallet;
}

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

/// 记录最近一次请求的分类，用于断言分类切换真的按 feedKind 重新拉流。
class _RecordingFeedSource implements SquareFeedSource {
  SquareFeedKind? lastFeedKind;

  @override
  Future<List<SquarePost>> fetchFeed({
    required SquareFeedKind feedKind,
    int limit = 20,
    SquareSession? session,
  }) async {
    lastFeedKind = feedKind;
    return const <SquarePost>[];
  }
}

/// 按分类返回不同夹具的假数据源；模拟 Worker 对每个 feed 端点各自过滤
/// （关注流走 `/v1/square/feed/following` 的 JOIN 结果）。
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

    // 中央坦克水印 + 左上默认用户头像 + 保留的发布按钮。
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
    final chainService = _FakeSquareChainService(null);
    final identityService = SquareIdentityService(
      walletManager: _FakeWalletManager(
        const WalletProfile(
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
        ),
      ),
      chainService: chainService,
    );

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

    // 无订阅时服务入口立即阻断，不打开类型选择或编辑器，也不触发链身份查询。
    expect(find.text('需要有效会员才能发布广场内容'), findsOneWidget);
    expect(find.text('发动态'), findsNothing);
    expect(find.text('发文章'), findsNothing);
    expect(chainService.fetchIdentityCount, 0);
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
}
