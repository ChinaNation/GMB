import 'package:citizenapp/rpc/subscription_rpc.dart' show ChainCreatorTier;

/// 订阅周期。创作者每档可只开其中部分周期。
enum BillingPeriod { monthly, quarterly, yearly }

/// 周期与后端/链一致的字符串键（后端 D1 / 链上统一用这些值，不用枚举序号）。
extension BillingPeriodKey on BillingPeriod {
  String get key => switch (this) {
        BillingPeriod.monthly => 'monthly',
        BillingPeriod.quarterly => 'quarterly',
        BillingPeriod.yearly => 'yearly',
      };

  /// 展示用中文短名（前端展示层用）。
  String get label => switch (this) {
        BillingPeriod.monthly => '每月',
        BillingPeriod.quarterly => '每季',
        BillingPeriod.yearly => '每年',
      };

  static BillingPeriod? tryParse(String value) => switch (value) {
        'monthly' => BillingPeriod.monthly,
        'quarterly' => BillingPeriod.quarterly,
        'yearly' => BillingPeriod.yearly,
        _ => null,
      };
}

/// 创作者单个会员档：tier_id 与周期价格以链上为真源，名称由 Cloudflare 保存。
///
/// 价格一律以「分」为后端/链单一口径存储；展示与输入的「元」换算只发生在 UI 边界。

class CreatorTier {
  const CreatorTier({
    required this.tierId,
    required this.name,
    required this.pricesFen,
  });

  /// 档位稳定 id（CreatorPlans 链上引用；用于编辑、删除定位与订阅关联）。
  final String tierId;

  /// 档名（创作者自定义，如「铁杆粉丝」）。
  final String name;

  /// 各周期价（分，公民币）；由 finalized CreatorPlans 读取。
  final Map<BillingPeriod, int> pricesFen;

  bool hasPeriod(BillingPeriod period) => pricesFen.containsKey(period);

  int? priceFenOf(BillingPeriod period) => pricesFen[period];

  CreatorTier copyWith({String? name, Map<BillingPeriod, int>? pricesFen}) {
    return CreatorTier(
      tierId: tierId,
      name: name ?? this.name,
      pricesFen: pricesFen ?? this.pricesFen,
    );
  }

  Map<String, Object?> toJson() => {
        'tier_id': tierId,
        'name': name,
        // 后端一律分：{'monthly': 990, 'yearly': 9900}
        'prices_fen': {
          for (final entry in pricesFen.entries) entry.key.key: entry.value,
        },
      };

  factory CreatorTier.fromJson(Map<String, dynamic> json) {
    final rawPrices = json['prices_fen'];
    final prices = <BillingPeriod, int>{};
    if (rawPrices is Map) {
      for (final entry in rawPrices.entries) {
        final period = BillingPeriodKey.tryParse(entry.key.toString());
        final fen = entry.value;
        if (period != null && fen is int && fen > 0) {
          prices[period] = fen;
        }
      }
    }
    return CreatorTier(
      tierId: json['tier_id']?.toString() ?? '',
      name: json['name']?.toString() ?? '',
      pricesFen: prices,
    );
  }
}

/// 将 Cloudflare 展示名与 finalized 链上档位合并；价格绝不从 Cloudflare 覆盖链上值。
CreatorPlan mergeCreatorPlanWithChain({
  required String creatorAccountId,
  required CreatorPlan? displayPlan,
  required List<ChainCreatorTier> chainTiers,
}) {
  final names = <String, String>{
    for (final tier in displayPlan?.tiers ?? const <CreatorTier>[])
      tier.tierId: tier.name,
  };
  final tiers = <CreatorTier>[];
  for (final tier in chainTiers) {
    final prices = <BillingPeriod, int>{};
    for (final entry in tier.pricesFen.entries) {
      final period = BillingPeriodKey.tryParse(entry.key);
      if (period != null) prices[period] = entry.value.toInt();
    }
    tiers.add(CreatorTier(
      tierId: tier.tierId,
      name: names[tier.tierId] ?? '',
      pricesFen: prices,
    ));
  }
  return CreatorPlan(
    creatorAccountId: creatorAccountId,
    tiers: List.unmodifiable(tiers),
    updatedAt: displayPlan?.updatedAt ?? 0,
  );
}

/// 创作者会员计划（一名创作者的全部档位，≤ [maxTiers]）。

class CreatorPlan {
  const CreatorPlan({
    required this.creatorAccountId,
    required this.tiers,
    required this.updatedAt,
  });

  /// 创作者钱包账户（SS58）。
  final String creatorAccountId;

  /// 有序档位集合，≤ [maxTiers]。
  final List<CreatorTier> tiers;

  /// Cloudflare 侧最近更新时间（unix 毫秒），仅展示与并发参考。
  final int updatedAt;

  /// 单创作者档位数量硬上限（客户端护栏，BFF 兜底）。
  static const int maxTiers = 10;

  bool get isEmpty => tiers.isEmpty;

  static CreatorPlan empty(String creatorAccountId) => CreatorPlan(
      creatorAccountId: creatorAccountId, tiers: const [], updatedAt: 0);

  List<Map<String, Object?>> tiersJson() =>
      tiers.map((tier) => tier.toJson()).toList(growable: false);

  factory CreatorPlan.fromJson(Map<String, dynamic> json) {
    final rawTiers = json['tiers'];
    final tiers = <CreatorTier>[];
    if (rawTiers is List) {
      for (final item in rawTiers) {
        if (item is Map<String, dynamic>) {
          tiers.add(CreatorTier.fromJson(item));
        }
      }
    }
    return CreatorPlan(
      creatorAccountId: json['creator_account_id']?.toString() ?? '',
      tiers: tiers,
      updatedAt: json['updated_at'] is int ? json['updated_at'] as int : 0,
    );
  }
}
