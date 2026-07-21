/// 金额元/分换算（唯一边界）：后端/模型一律「分」，前端展示与输入一律「元」。
///
/// 只有本文件做换算，其余 UI 直接用这两个函数，杜绝口径漂移。
library;

/// 分 → 展示用「元」字符串（去掉多余小数零）：990 → "9.9"，2700 → "27"，29900 → "299"。
String fenToYuanLabel(int fen) {
  final yuan = fen / 100.0;
  if (yuan == yuan.roundToDouble()) {
    return yuan.round().toString();
  }
  // 最多两位小数，去尾零。
  return yuan
      .toStringAsFixed(2)
      .replaceFirst(RegExp(r'0+$'), '')
      .replaceFirst(RegExp(r'\.$'), '');
}

/// 分 → 展示用「元」金额串，带千分号 + 固定两位小数 + 元：199900 → "1,999.00 元"。
/// 会员价等正式金额展示用（区别于 [fenToYuanLabel] 的去零简写）。
String fenToYuanMoneyLabel(int fen) {
  final parts = (fen / 100.0).toStringAsFixed(2).split('.');
  final intWithSep =
      parts[0].replaceAllMapped(RegExp(r'\B(?=(\d{3})+$)'), (_) => ',');
  return '$intWithSep.${parts[1]} 元';
}

/// 输入「元」文本 → 「分」整数；非法/≤0 返回 null（调用方按校验失败处理）。
///
/// 接受 "9.9" / "27" / "￥9.9"（容错前缀）；超过两位小数视为非法。
int? yuanTextToFen(String input) {
  final text = input.trim().replaceFirst(RegExp(r'^[¥￥]'), '').trim();
  if (text.isEmpty) return null;
  final yuan = double.tryParse(text);
  if (yuan == null || yuan <= 0) return null;
  // 两位小数上限：元×100 必须是整数分。
  final fen = (yuan * 100).round();
  if ((fen / 100 - yuan).abs() > 1e-9) return null;
  return fen > 0 ? fen : null;
}
