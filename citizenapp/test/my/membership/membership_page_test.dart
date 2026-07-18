import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/membership/membership_page.dart';
import 'package:citizenapp/my/membership/subscription_service.dart';
import 'package:citizenapp/rpc/chain_rpc.dart' show TxPoolWatchCallback;

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
  int cancelCount = 0;

  @override
  Future<void> subscribe(String level,
      {TxPoolWatchCallback? onWatchEvent}) async {
    subscribed.add(level);
  }

  @override
  Future<void> cancel({TxPoolWatchCallback? onWatchEvent}) async {
    cancelCount++;
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
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: MembershipPage(
        apiClient: _FakeApiClient(state),
        chainService: _FakeChainService(prices),
        sessionProvider: _FakeSessionProvider(),
        subscriptionService: service,
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
  testWidgets('renders the three subscription tier cards', (tester) async {
    await _pump(tester, _state());

    expect(find.text('自由会员'), findsOneWidget);
    expect(find.text('民主会员'), findsOneWidget);
    expect(find.text('薪火会员'), findsOneWidget);
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
    await _pump(tester, _state());
    // 三档解耦、无身份门槛：三张卡都可订阅。
    expect(find.text('订阅'), findsNWidgets(3));
    expect(find.text('当前会员'), findsNothing);
  });

  testWidgets('shows 取消订阅 on the active tier + 订阅 on the others',
      (tester) async {
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        membershipLevel: 'democracy',
      ),
    );

    expect(find.text('取消订阅'), findsOneWidget);
    // 另外两档仍可订阅。
    expect(find.text('订阅'), findsNWidgets(2));
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

  testWidgets('链上未设价 → 三档价签均显示占位「—」', (tester) async {
    await _pump(tester, _state());
    expect(find.text('—'), findsNWidgets(3));
  });

  testWidgets('点击订阅按钮走 App 内订阅并传对应档', (tester) async {
    final service = _RecordingSubscriptionService();
    await _pump(tester, _state(), service: service);

    final button = _frontButton('订阅');
    expect(button, findsOneWidget);
    await tester.tap(button);
    await tester.pumpAndSettle();

    // 默认前置档=自由 → 订阅传 'freedom'。
    expect(service.subscribed, ['freedom']);
    expect(service.cancelCount, 0);
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
  });

  testWidgets('订阅生效横幅显示订阅起止 + 自动续费', (tester) async {
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

    expect(find.textContaining('自动续费'), findsOneWidget);
    expect(find.textContaining('订阅 '), findsOneWidget);
  });

  testWidgets('已取消订阅 → 横幅标签「到期终止」而非自动续费', (tester) async {
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
    expect(find.textContaining('自动续费'), findsNothing);
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
