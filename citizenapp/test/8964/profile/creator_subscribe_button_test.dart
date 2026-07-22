import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/widgets/creator_subscribe_button.dart';
import 'package:citizenapp/8964/subscribe/creator_subscribe_service.dart';
import 'package:citizenapp/my/creator/creator_api.dart';
import 'package:citizenapp/rpc/subscription_rpc.dart';

import 'fake_profile.dart';

/// 只覆盖订阅按钮的显示门禁：有档 且 创作者本人平台会员有效才显示，其余一律隐藏。
class _FakeSubscribeService extends CreatorSubscribeService {
  _FakeSubscribeService({
    required this.tiers,
    required this.ownerPlatform,
    this.throwOwnerPlatform = false,
  }) : super();

  final List<ChainCreatorTier> tiers;
  final FinalizedSubscriptionSnapshot ownerPlatform;
  final bool throwOwnerPlatform;

  @override
  Future<List<ChainCreatorTier>> fetchCreatorPlans(
          String creatorAddress) async =>
      tiers;

  @override
  Future<FinalizedSubscriptionSnapshot> fetchFinalizedState({
    required String subscriberAddress,
    required String creatorAddress,
  }) async =>
      _snapshot(state: null); // 访客尚未订阅该创作者 → 按钮呈「订阅 TA」

  @override
  Future<FinalizedSubscriptionSnapshot> fetchPlatformSnapshot(
      String address) async {
    // 真实 fetchSubscriptionSnapshot 在链读/解码失败时抛 FormatException（Exception）。
    if (throwOwnerPlatform) throw const FormatException('chain read failed');
    return ownerPlatform;
  }
}

FinalizedSubscriptionSnapshot _snapshot(
        {required ChainSubscriptionState? state}) =>
    FinalizedSubscriptionSnapshot(
      state: state,
      chainNowMs: 1000,
      blockHashHex: '0x00',
    );

/// 平台会员快照：active → paidUntil 远大于 chainNowMs 且状态 active；否则 terminated 已到期。
FinalizedSubscriptionSnapshot _platform({required bool active}) => _snapshot(
      state: ChainSubscriptionState(
        plan: const ChainSubscriptionPlan.platform('freedom'),
        startedAt: 0,
        lastChargedAt: 0,
        lastChargedPriceFen: BigInt.zero,
        paidUntil: active ? 9999999999999 : 500,
        status: active ? 'active' : 'terminated',
        authorizedPriceFen: BigInt.zero,
        suspendReason: null,
      ),
    );

final _tiers = <ChainCreatorTier>[
  ChainCreatorTier(tierId: 't1', pricesFen: {'monthly': BigInt.from(299)}),
];

void main() {
  Future<void> pump(
    WidgetTester tester, {
    required bool ownerActive,
    List<ChainCreatorTier>? tiers,
    bool throwOwner = false,
  }) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: CreatorSubscribeButton(
            creatorAccount: kOwner,
            service: _FakeSubscribeService(
              tiers: tiers ?? _tiers,
              ownerPlatform: _platform(active: ownerActive),
              throwOwnerPlatform: throwOwner,
            ),
            api: FakeCreatorApi(),
            sessionProvider: FakeSessionProvider(fakeSession()),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
  }

  testWidgets('有档 且 创作者平台会员 active → 显示订阅按钮', (tester) async {
    await pump(tester, ownerActive: true);
    expect(find.text('订阅 TA'), findsOneWidget);
  });

  testWidgets('有档 但 创作者平台会员过期 → 隐藏', (tester) async {
    await pump(tester, ownerActive: false);
    expect(find.text('订阅 TA'), findsNothing);
  });

  testWidgets('创作者平台会员快照读失败 → 隐藏（fail-closed）', (tester) async {
    await pump(tester, ownerActive: true, throwOwner: true);
    expect(find.text('订阅 TA'), findsNothing);
  });

  testWidgets('无档 → 隐藏（既有行为不回归）', (tester) async {
    await pump(tester, ownerActive: true, tiers: const <ChainCreatorTier>[]);
    expect(find.text('订阅 TA'), findsNothing);
  });
}
