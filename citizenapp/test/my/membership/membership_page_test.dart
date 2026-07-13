import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/membership/membership_page.dart';

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

SquareMembershipState _state({
  required String identityLevel,
  bool active = false,
  bool subscriptionActive = false,
  bool cancelAtPeriodEnd = false,
  bool frozen = false,
  String? membershipLevel,
  int currentPeriodStart = 0,
  String? subscriptionSource,
}) {
  return SquareMembershipState(
    active: active,
    expiresAt: active ? DateTime.now().millisecondsSinceEpoch + 600000 : 0,
    membershipLevel: membershipLevel,
    subscriptionActive: subscriptionActive,
    cancelAtPeriodEnd: cancelAtPeriodEnd,
    frozen: frozen,
    currentPeriodStart: currentPeriodStart,
    subscriptionSource: subscriptionSource,
    identityLevel: identityLevel,
    plans: const [],
  );
}

Future<void> _pump(WidgetTester tester, SquareMembershipState state) async {
  await tester.pumpWidget(
    MaterialApp(
      home: MembershipPage(
        apiClient: _FakeApiClient(state),
        sessionProvider: _FakeSessionProvider(),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('renders the three identity tier cards', (tester) async {
    await _pump(tester, _state(identityLevel: 'voting'));

    expect(find.text('访客轻节点'), findsOneWidget);
    expect(find.text('公民轻节点 · 投票'), findsOneWidget);
    expect(find.text('公民轻节点 · 竞选'), findsOneWidget);
  });

  testWidgets('brings the wallet identity tier card to the front',
      (tester) async {
    await _pump(tester, _state(identityLevel: 'candidate'));

    // 命中身份档卡在最上层（front-card key），且带「你的身份」。
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('membership-front-card')),
        matching: find.text('公民轻节点 · 竞选'),
      ),
      findsOneWidget,
    );
    expect(find.text('你的身份'), findsOneWidget);
  });

  testWidgets('visitor identity fronts the visitor card', (tester) async {
    await _pump(tester, _state(identityLevel: 'visitor'));

    expect(
      find.descendant(
        of: find.byKey(const ValueKey('membership-front-card')),
        matching: find.text('访客轻节点'),
      ),
      findsOneWidget,
    );
  });

  testWidgets('enables 订阅 only on the wallet identity card, locks the rest',
      (tester) async {
    await _pump(tester, _state(identityLevel: 'voting'));

    // 精确匹配：仅本人(投票)身份卡可订阅；访客/竞选两卡置灰。
    expect(find.text('订阅'), findsOneWidget);
    expect(find.text('仅本档身份可订阅'), findsNWidgets(2));
  });

  testWidgets('shows 取消订阅 on the active tier when auto-renewing',
      (tester) async {
    await _pump(
      tester,
      _state(
        identityLevel: 'voting',
        active: true,
        subscriptionActive: true,
        membershipLevel: 'voting',
      ),
    );

    expect(find.text('取消订阅'), findsOneWidget);
    // 另外两张非本档卡置灰。
    expect(find.text('仅本档身份可订阅'), findsNWidgets(2));
  });

  testWidgets('shows 续订会员 when cancelled but not yet expired',
      (tester) async {
    await _pump(
      tester,
      _state(
        identityLevel: 'voting',
        active: true,
        subscriptionActive: true,
        cancelAtPeriodEnd: true,
        membershipLevel: 'voting',
      ),
    );

    expect(find.text('续订会员'), findsOneWidget);
    expect(find.text('取消订阅'), findsNothing);
  });

  testWidgets('USDC 预付会员显示订阅起止 + 预付路线横幅', (tester) async {
    await _pump(
      tester,
      _state(
        identityLevel: 'voting',
        active: true,
        subscriptionActive: true,
        membershipLevel: 'voting',
        currentPeriodStart: DateTime.now().millisecondsSinceEpoch,
        subscriptionSource: 'usdc_prepaid',
      ),
    );

    expect(find.textContaining('预付 · 到期失效'), findsOneWidget);
    expect(find.textContaining('订阅 '), findsOneWidget);
  });

  testWidgets('卡连续订阅会员显示自动续费横幅', (tester) async {
    await _pump(
      tester,
      _state(
        identityLevel: 'voting',
        active: true,
        subscriptionActive: true,
        membershipLevel: 'voting',
        currentPeriodStart: DateTime.now().millisecondsSinceEpoch,
        subscriptionSource: 'stripe',
      ),
    );

    expect(find.textContaining('自动续费'), findsOneWidget);
  });

  testWidgets('卡已发起到期取消 → 横幅标签「到期终止」而非自动续费', (tester) async {
    await _pump(
      tester,
      _state(
        identityLevel: 'voting',
        active: true,
        subscriptionActive: true,
        cancelAtPeriodEnd: true,
        membershipLevel: 'voting',
        currentPeriodStart: DateTime.now().millisecondsSinceEpoch,
        subscriptionSource: 'stripe',
      ),
    );

    expect(find.textContaining('已取消 · 到期终止'), findsOneWidget);
    expect(find.textContaining('自动续费'), findsNothing);
  });

  testWidgets('冻结态只显示冻结横幅，不叠加订阅起止横幅', (tester) async {
    await _pump(
      tester,
      _state(
        identityLevel: 'voting',
        active: true,
        subscriptionActive: true,
        frozen: true,
        membershipLevel: 'voting',
        currentPeriodStart: DateTime.now().millisecondsSinceEpoch,
        subscriptionSource: 'usdc_prepaid',
      ),
    );

    // 冻结横幅在（雪花图标），起止横幅（"订阅 "文案）不显示。
    expect(find.byIcon(Icons.ac_unit), findsOneWidget);
    expect(find.textContaining('订阅 '), findsNothing);
  });

  test('SquareMembershipState 路线 / 订阅窗口 getter', () {
    const prepaid = SquareMembershipState(
      active: true,
      expiresAt: 2000,
      subscriptionActive: true,
      currentPeriodStart: 1000,
      subscriptionSource: 'usdc_prepaid',
    );
    expect(prepaid.isPrepaid, isTrue);
    expect(prepaid.hasSubscriptionWindow, isTrue);

    const noWindow = SquareMembershipState(
      active: true,
      expiresAt: 2000,
      subscriptionActive: true,
      subscriptionSource: 'stripe',
    );
    // 缺 current_period_start（=0）→ 无可展示窗口。
    expect(noWindow.isPrepaid, isFalse);
    expect(noWindow.hasSubscriptionWindow, isFalse);
  });

  testWidgets('visitor card toggles between 自由 and 民主 plans', (tester) async {
    await _pump(tester, _state(identityLevel: 'visitor'));

    Finder frontPrice(String price) => find.descendant(
          of: find.byKey(const ValueKey('membership-front-card')),
          matching: find.text(price),
        );

    // 访客卡有自由/民主分段，默认自由(￥2.99)。
    expect(find.text('自由会员'), findsOneWidget);
    expect(find.text('民主会员'), findsOneWidget);
    expect(frontPrice('\$2.99 / 月'), findsOneWidget);

    await tester.tap(find.text('民主会员'));
    await tester.pumpAndSettle();

    // 切到民主后价格变 ￥9.99。
    expect(frontPrice('\$9.99 / 月'), findsOneWidget);
  });
}
