import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/membership/membership_page.dart';
import 'package:citizenapp/my/membership/subscription_service.dart';
import 'package:citizenapp/rpc/chain_rpc.dart' show TxPoolWatchCallback;
import 'package:citizenapp/rpc/subscription_rpc.dart';

const String _owner = '5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U';

class _FakeSessionProvider extends SquareSessionProvider {
  _FakeSessionProvider() : super();

  @override
  Future<SquareSession?> ensureSession() async => SquareSession(
        sessionToken: 'tok',
        ownerAccount: _owner,
        expiresAt: DateTime.now().millisecondsSinceEpoch + 600000,
      );
}

class _FakeApiClient extends SquareApiClient {
  _FakeApiClient(this._state) : super(baseUrl: 'https://membership.test');

  final SquareMembershipState _state;

  @override
  Future<SquareMembershipState> fetchMembership(SquareSession session) async =>
      _state;
}

class _FailingApiClient extends SquareApiClient {
  _FailingApiClient() : super(baseUrl: 'https://membership.test');

  @override
  Future<SquareMembershipState> fetchMembership(SquareSession session) =>
      Future.error(StateError('Cloudflare unavailable'));
}

/// 平台档价格链上单源（`PlatformPrice[level]`，分）；测试直接注入 mock 价表。
class _FakeChainService extends SquareChainService {
  _FakeChainService(this._prices);

  final Map<String, int> _prices;

  @override
  Future<Map<String, int>> fetchAllPlatformPrices() async => _prices;
}

/// 记录订阅 / 取消动作的假编排：不触发真钱包与真上链。
class _RecordingSubscriptionService extends SubscriptionService {
  final List<String> subscribed = [];
  final List<String> changed = [];
  int cancelCount = 0;
  SquareMembershipState? mirror;

  @override
  Future<FinalizedSubscriptionSnapshot> fetchFinalizedState(
      String ownerAccount) async {
    final source = mirror!;
    final status = source.subscriptionStatus ??
        (source.subscriptionActive ? 'active' : null);
    final level = source.membershipLevel;
    final now = DateTime.now().millisecondsSinceEpoch;
    return FinalizedSubscriptionSnapshot(
      state: status == null || level == null
          ? null
          : ChainSubscriptionState(
              plan: ChainSubscriptionPlan.platform(level),
              pendingPlan: null,
              startedAt: source.currentPeriodStart == 0
                  ? now - 1000
                  : source.currentPeriodStart,
              lastChargedAt: source.currentPeriodStart == 0
                  ? now - 1000
                  : source.currentPeriodStart,
              lastChargedPriceFen: BigInt.one,
              paidUntil:
                  source.expiresAt == 0 ? now + 600000 : source.expiresAt,
              status: status,
            ),
      chainNowMs: now,
      blockHashHex: '0x${List.filled(64, '0').join()}',
    );
  }

  @override
  Future<void> subscribe(String level, int expectedPriceFen,
      {TxPoolWatchCallback? onWatchEvent}) async {
    subscribed.add('$level:$expectedPriceFen');
  }

  @override
  Future<void> cancel({TxPoolWatchCallback? onWatchEvent}) async {
    cancelCount++;
  }

  @override
  Future<void> changePlan(String level, int expectedPriceFen,
      {TxPoolWatchCallback? onWatchEvent}) async {
    changed.add('$level:$expectedPriceFen');
  }
}

/// 会员与身份彻底解耦（ADR-036）：会员卡只描述订阅，不含任何身份字段。
/// plans 留空 → 页面用三档兜底套餐（自由 / 民主 / 薪火）渲染。
SquareMembershipState _state({
  bool active = false,
  bool subscriptionActive = false,
  String? membershipLevel,
  String? subscriptionStatus,
  int currentPeriodStart = 0,
}) {
  return SquareMembershipState(
    active: active,
    expiresAt: active ? DateTime.now().millisecondsSinceEpoch + 600000 : 0,
    membershipLevel: membershipLevel,
    subscriptionStatus: subscriptionStatus,
    subscriptionActive: subscriptionActive,
    currentPeriodStart: currentPeriodStart,
    plans: const [],
  );
}

Future<void> _pump(
  WidgetTester tester,
  SquareMembershipState state, {
  Map<String, int> prices = const {},
  SubscriptionService? service,
  SquareApiClient? apiClient,
}) async {
  final effectiveService = service ?? _RecordingSubscriptionService();
  if (effectiveService is _RecordingSubscriptionService) {
    effectiveService.mirror = state;
  }
  await tester.pumpWidget(
    MaterialApp(
      home: MembershipPage(
        apiClient: apiClient ?? _FakeApiClient(state),
        chainService: _FakeChainService(prices),
        sessionProvider: _FakeSessionProvider(),
        subscriptionService: effectiveService,
      ),
    ),
  );
  await tester.pumpAndSettle();
}

Finder _frontCard(String text) => find.descendant(
      of: find.byKey(const ValueKey('membership-front-card')),
      matching: find.text(text),
    );

Finder _frontButton(String label) => find.descendant(
      of: find.byKey(const ValueKey('membership-front-card')),
      matching: find.widgetWithText(FilledButton, label),
    );

void main() {
  test('平台 finalized 镜像回执不再产生设备签名', () async {
    var deviceSignCount = 0;
    final api = SquareApiClient(
      baseUrl: 'https://membership.test',
      httpClient: MockClient((request) async {
        expect(request.url.path, '/v1/square/membership/confirm');
        expect(request.headers['authorization'], 'Bearer tok');
        expect(request.headers, isNot(contains('x-device-signature')));
        return http.Response('{}', 200);
      }),
    );
    final session = SquareSession(
      sessionToken: 'tok',
      ownerAccount: _owner,
      expiresAt: 9999999999999,
      signRequest: (_) async {
        deviceSignCount++;
        return 'device-signature';
      },
    );

    await api.confirmPlatformSubscription(
      session: session,
      txHash: '0x${List.filled(64, 'a').join()}',
      level: 'freedom',
    );

    expect(deviceSignCount, 0);
  });

  testWidgets('renders the three subscription tier cards', (tester) async {
    await _pump(tester, _state());

    expect(find.text('自由会员'), findsOneWidget);
    expect(find.text('民主会员'), findsOneWidget);
    expect(find.text('薪火会员'), findsOneWidget);
  });

  testWidgets('Cloudflare 不可用时仍按链上真态和兜底名称展示', (tester) async {
    await _pump(
      tester,
      _state(),
      prices: const {'freedom': 29900},
      apiClient: _FailingApiClient(),
    );

    expect(find.text('自由会员'), findsOneWidget);
    expect(find.text('299 公民币/月'), findsOneWidget);
    expect(find.text('会员状态加载失败'), findsNothing);
  });

  testWidgets('defaults to the freedom card for a fresh account',
      (tester) async {
    await _pump(tester, _state());
    expect(_frontCard('自由会员'), findsOneWidget);
  });

  testWidgets('fronts the current membership tier card + 当前会员 marker',
      (tester) async {
    await _pump(
      tester,
      _state(active: true, subscriptionActive: true, membershipLevel: 'spark'),
    );

    expect(_frontCard('薪火会员'), findsOneWidget);
    expect(find.text('当前会员'), findsOneWidget);
  });

  testWidgets('any identity can subscribe any tier — all cards show 订阅',
      (tester) async {
    await _pump(
      tester,
      _state(),
      prices: const {'freedom': 1, 'democracy': 2, 'spark': 3},
    );
    // 三档解耦、无身份门槛：三张卡都可订阅。
    expect(find.text('订阅'), findsNWidgets(3));
    expect(find.text('当前会员'), findsNothing);
  });

  testWidgets('shows 取消订阅 on the active tier + 更换为此档 on the others',
      (tester) async {
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        membershipLevel: 'democracy',
      ),
      prices: const {'freedom': 1, 'democracy': 2, 'spark': 3},
    );

    expect(find.text('取消订阅'), findsOneWidget);
    expect(find.text('更换为此档'), findsNWidgets(2));
  });

  testWidgets('公民币月价来自链读，逐档展示', (tester) async {
    await _pump(
      tester,
      _state(),
      prices: const {'freedom': 29900, 'democracy': 99900, 'spark': 199900},
    );

    expect(find.text('299 公民币/月'), findsOneWidget);
    expect(find.text('999 公民币/月'), findsOneWidget);
    expect(find.text('1999 公民币/月'), findsOneWidget);
  });

  testWidgets('链上未设价 → 显示占位且禁止发起订阅', (tester) async {
    await _pump(tester, _state());
    expect(find.text('—'), findsNWidgets(3));
    final button = tester.widget<FilledButton>(
      _frontButton('链上价格未就绪'),
    );
    expect(button.onPressed, isNull);
  });

  testWidgets('点击订阅按钮走 App 内订阅并传对应档', (tester) async {
    final service = _RecordingSubscriptionService();
    await _pump(
      tester,
      _state(),
      prices: const {'freedom': 29900},
      service: service,
    );

    final button = _frontButton('订阅');
    expect(button, findsOneWidget);
    await tester.tap(button);
    await tester.pumpAndSettle();

    // 默认前置档=自由 → 订阅传 'freedom'。
    expect(service.subscribed, ['freedom:29900']);
    expect(service.cancelCount, 0);
    expect(service.changed, isEmpty);
  });

  testWidgets('点击另一档按钮走一次链上换档', (tester) async {
    final service = _RecordingSubscriptionService();
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        membershipLevel: 'freedom',
        subscriptionStatus: 'active',
      ),
      prices: const {'freedom': 1, 'democracy': 2, 'spark': 3},
      service: service,
    );

    // 在会员卡层叠手势区左滑，把民主档移到前层，再执行换档。
    await tester.drag(
      find.byKey(const ValueKey('membership-tier-stack-gesture')),
      const Offset(-360, 0),
    );
    await tester.pumpAndSettle();
    await tester.tap(_frontButton('更换为此档'));
    await tester.pumpAndSettle();

    expect(service.changed, ['democracy:2']);
    expect(service.subscribed, isEmpty);
  });

  testWidgets('点击取消订阅按钮走 App 内取消', (tester) async {
    final service = _RecordingSubscriptionService();
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        membershipLevel: 'democracy',
      ),
      service: service,
    );

    final button = _frontButton('取消订阅');
    expect(button, findsOneWidget);
    await tester.tap(button);
    await tester.pumpAndSettle();

    expect(service.cancelCount, 1);
    expect(service.subscribed, isEmpty);
    expect(service.changed, isEmpty);
  });

  testWidgets('订阅生效横幅显示订阅起止 + 链上自动续费口径', (tester) async {
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        membershipLevel: 'democracy',
        subscriptionStatus: 'active',
        currentPeriodStart: DateTime.now().millisecondsSinceEpoch,
      ),
    );

    expect(find.textContaining('链上到期自动续费'), findsOneWidget);
    expect(find.textContaining('订阅 '), findsOneWidget);
  });

  testWidgets('已取消订阅 → 横幅标签「到期终止」且不再续费', (tester) async {
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        membershipLevel: 'democracy',
        subscriptionStatus: 'cancelled',
        currentPeriodStart: DateTime.now().millisecondsSinceEpoch,
      ),
    );

    expect(find.textContaining('已取消 · 到期终止'), findsOneWidget);
    expect(find.textContaining('链上到期自动续费'), findsNothing);
  });

  test('SquareMembershipState 订阅窗口 getter', () {
    const withWindow = SquareMembershipState(
      active: true,
      expiresAt: 2000,
      subscriptionActive: true,
      currentPeriodStart: 1000,
    );
    expect(withWindow.hasSubscriptionWindow, isTrue);

    const noWindow = SquareMembershipState(
      active: true,
      expiresAt: 2000,
      subscriptionActive: true,
    );
    // 缺 current_period_start（=0）→ 无可展示窗口。
    expect(noWindow.hasSubscriptionWindow, isFalse);
  });
}
