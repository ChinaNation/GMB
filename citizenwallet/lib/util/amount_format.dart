/// 金额格式化工具。
///
/// 提供千分位分隔符和标准化金额显示。
class AmountFormat {
  AmountFormat._();

  /// 将数值格式化为带千分位逗号的字符串。
  ///
  /// [amount] 金额数值
  /// [decimals] 小数位数，默认 2
  /// [symbol] 货币符号后缀，默认 'GMB'，传空字符串则不加
  ///
  /// ```dart
  /// AmountFormat.format(1234567.89)       // "1,234,567.89 GMB"
  /// AmountFormat.format(100)              // "100.00 GMB"
  /// AmountFormat.format(100, symbol: '')  // "100.00"
  /// ```
  static String format(
    double amount, {
    int decimals = 2,
    String symbol = 'GMB',
  }) {
    final fixed = amount.toStringAsFixed(decimals);
    final formatted = _addThousandSeparator(fixed);
    return symbol.isEmpty ? formatted : '$formatted $symbol';
  }

  /// 将已有的金额字符串添加千分位逗号。
  ///
  /// 自动识别并保留末尾的币种后缀（如 " GMB"）。
  ///
  /// ```dart
  /// AmountFormat.formatString("1234567.89 GMB")  // "1,234,567.89 GMB"
  /// AmountFormat.formatString("100.00")           // "100.00"
  /// ```
  static String formatString(String value) {
    final match = RegExp(r'^([\d.]+)(.*)$').firstMatch(value.trim());
    if (match == null) return value;
    final numPart = match.group(1)!;
    final suffix = match.group(2)!;
    return '${_addThousandSeparator(numPart)}$suffix';
  }

  static String _addThousandSeparator(String numStr) {
    final parts = numStr.split('.');
    final intPart = parts[0].replaceAllMapped(
      RegExp(r'(\d)(?=(\d{3})+(?!\d))'),
      (m) => '${m[1]},',
    );
    final decimal = parts.length > 1 ? '.${parts[1]}' : '';
    return '$intPart$decimal';
  }
}
