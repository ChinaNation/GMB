import 'package:citizenapp/8964/services/square_api_client.dart'
    show SquareSession;
import 'package:citizenapp/my/creator/creator_api.dart';
import 'package:citizenapp/my/creator/models/creator_plan.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  const session = SquareSession(
    sessionToken: 't',
    ownerAccount: 'creator-acc',
    expiresAt: 9999999999999,
  );

  const tier = CreatorTier(
    tierId: 't1',
    name: '铁杆粉丝',
    pricesFen: {BillingPeriod.monthly: 990},
  );

  test('FakeCreatorApi 保存必调签名（编辑=核心操作过生物识别）', () async {
    final api = FakeCreatorApi();
    var signed = false;

    final plan = await api.saveMyPlan(
      session: session,
      ownerAccount: 'creator-acc',
      tiers: const [tier],
      signAction: (message) async {
        signed = true;
        return '0x00';
      },
    );

    expect(signed, isTrue, reason: '保存档位必须触发主钥签名（生物识别）');
    expect(api.lastSaveSigned, isTrue);
    expect(plan.creatorAccount, 'creator-acc');
    expect(plan.tiers, hasLength(1));
    expect(await api.fetchMyPlan(session), isNotNull);
  });

  test('FakeCreatorApi 概览默认按档位数', () async {
    final api = FakeCreatorApi(
      initialPlan: const CreatorPlan(
        creatorAccount: 'creator-acc',
        tiers: [tier],
        updatedAt: 0,
      ),
    );
    final overview = await api.fetchOverview(session);
    expect(overview.tierCount, 1);
    expect(overview.subscriberCount, 0);
  });
}
