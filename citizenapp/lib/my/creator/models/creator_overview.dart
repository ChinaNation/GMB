/// 创作者概览（Cloudflare 聚合）。收入为**当月真实扣到的公民币**，非估计/摊算。
///
/// 金额一律以「分」为后端口径；展示层换算「元」（分÷100，全仓约定）。
class CreatorOverview {
  const CreatorOverview({
    required this.subscriberCount,
    required this.monthIncomeFen,
    required this.tierCount,
  });

  /// 订阅人数（活跃订阅者去重计数）。
  final int subscriberCount;

  /// 本月已收入（分）：当月实际扣款到账的公民币合计（真实，非按月摊算）。
  final int monthIncomeFen;

  /// 档位数（= 当前计划档位数量）。
  final int tierCount;

  static const CreatorOverview zero = CreatorOverview(
    subscriberCount: 0,
    monthIncomeFen: 0,
    tierCount: 0,
  );

  factory CreatorOverview.fromJson(Map<String, dynamic> json) {
    int asInt(Object? value) => value is int ? value : 0;
    return CreatorOverview(
      subscriberCount: asInt(json['subscriber_count']),
      monthIncomeFen: asInt(json['month_income_fen']),
      tierCount: asInt(json['tier_count']),
    );
  }
}
