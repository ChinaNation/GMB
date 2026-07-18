import 'package:citizenapp/my/creator/models/creator_plan.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('CreatorTier JSON', () {
    test('toJson/fromJson 往返（分口径）', () {
      const tier = CreatorTier(
        tierId: 't1',
        name: '铁杆粉丝',
        pricesFen: {
          BillingPeriod.monthly: 990,
          BillingPeriod.yearly: 9900,
        },
      );
      final json = tier.toJson();
      expect(json['prices_fen'], {'monthly': 990, 'yearly': 9900});

      final back = CreatorTier.fromJson(json);
      expect(back.tierId, 't1');
      expect(back.name, '铁杆粉丝');
      expect(back.priceFenOf(BillingPeriod.monthly), 990);
      expect(back.priceFenOf(BillingPeriod.yearly), 9900);
      expect(back.hasPeriod(BillingPeriod.quarterly), isFalse);
    });

    test('fromJson 丢弃非法/非正价格', () {
      final tier = CreatorTier.fromJson({
        'tier_id': 't2',
        'name': 'x',
        'prices_fen': {'monthly': 0, 'quarterly': -5, 'yearly': 100, 'bad': 9},
      });
      expect(tier.hasPeriod(BillingPeriod.monthly), isFalse);
      expect(tier.hasPeriod(BillingPeriod.quarterly), isFalse);
      expect(tier.priceFenOf(BillingPeriod.yearly), 100);
    });
  });

  group('CreatorPlan', () {
    test('fromJson 解析档位列表', () {
      final plan = CreatorPlan.fromJson({
        'creator_account': 'acc',
        'updated_at': 123,
        'tiers': [
          {
            'tier_id': 'a',
            'name': '基础',
            'prices_fen': {'monthly': 500},
          },
        ],
      });
      expect(plan.creatorAccount, 'acc');
      expect(plan.updatedAt, 123);
      expect(plan.tiers, hasLength(1));
      expect(plan.tiers.first.priceFenOf(BillingPeriod.monthly), 500);
    });

    test('empty 构造无档位', () {
      final plan = CreatorPlan.empty('acc');
      expect(plan.isEmpty, isTrue);
      expect(CreatorPlan.maxTiers, 10);
    });
  });
}
