import 'package:flutter/services.dart';

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

  /// 千分位格式化，只输出数字字符串（不带单位/符号）。
  ///
  /// 与 [format] 的区别：本方法专为列表余额行使用，
  /// 不带 `GMB` / `元` 等任何后缀，便于横向卡片紧凑展示。
  ///
  /// 异常输入（null / NaN / Infinity）返回 `--`，调用方无需再做判断。
  ///
  /// ```dart
  /// AmountFormat.formatThousands(1234567.89)        // "1,234,567.89"
  /// AmountFormat.formatThousands(0.0)               // "0.00"
  /// AmountFormat.formatThousands(-1000.5)           // "-1,000.50"
  /// AmountFormat.formatThousands(null)              // "--"
  /// AmountFormat.formatThousands(double.nan)        // "--"
  /// AmountFormat.formatThousands(double.infinity)   // "--"
  /// ```
  static String formatThousands(double? value, {int decimals = 2}) {
    if (value == null || value.isNaN || value.isInfinite) {
      return '--';
    }
    final fixed = value.toStringAsFixed(decimals);
    return _addThousandSeparator(fixed);
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

  /// 解析带千分位逗号的金额字符串为 double。
  ///
  /// 先去掉逗号再 parse，用于配合 [ThousandSeparatorFormatter] 使用。
  /// 返回 null 表示解析失败。
  ///
  /// ```dart
  /// AmountFormat.tryParse("1,234,567.89")  // 1234567.89
  /// AmountFormat.tryParse("100.00")        // 100.0
  /// AmountFormat.tryParse("")              // null
  /// ```
  static double? tryParse(String text) {
    final cleaned = text.replaceAll(',', '').trim();
    if (cleaned.isEmpty) return null;
    return double.tryParse(cleaned);
  }

  /// 去掉千分位逗号，返回纯数字字符串。
  ///
  /// 用于需要原始数字文本的场景（如 QR 码金额嵌入）。
  static String stripCommas(String text) => text.replaceAll(',', '').trim();

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

/// 金额输入框实时千分位格式化器。
///
/// 用户输入数字时自动插入千分位逗号，光标位置自动跟随。
/// 支持小数点（最多 [decimalDigits] 位小数）。
///
/// ```dart
/// TextField(
///   inputFormatters: [ThousandSeparatorFormatter()],
/// )
/// ```
class ThousandSeparatorFormatter extends TextInputFormatter {
  ThousandSeparatorFormatter({this.decimalDigits = 2});

  final int decimalDigits;

  @override
  TextEditingValue formatEditUpdate(
    TextEditingValue oldValue,
    TextEditingValue newValue,
  ) {
    final text = newValue.text;
    if (text.isEmpty) return newValue;

    // 只允许数字、小数点、逗号
    final cleaned = text.replaceAll(',', '');
    if (!RegExp(r'^\d*\.?\d*$').hasMatch(cleaned)) {
      return oldValue;
    }

    // 限制小数位数
    final dotIndex = cleaned.indexOf('.');
    if (dotIndex >= 0 && cleaned.length - dotIndex - 1 > decimalDigits) {
      return oldValue;
    }

    // 拆整数部分和小数部分
    final parts = cleaned.split('.');
    final intPart = parts[0];
    final decPart = parts.length > 1 ? '.${parts[1]}' : (text.endsWith('.') ? '.' : '');

    // 添加千分位
    final formattedInt = intPart.replaceAllMapped(
      RegExp(r'(\d)(?=(\d{3})+(?!\d))'),
      (m) => '${m[1]},',
    );
    final formatted = '$formattedInt$decPart';

    // 计算新光标位置：数光标左边有多少位数字/小数点，在新字符串中找到同样位置
    final oldCursor = newValue.selection.baseOffset.clamp(0, text.length);
    // 光标左侧在原文本中有多少"有效字符"（非逗号）
    int rawCount = 0;
    for (int i = 0; i < oldCursor && i < text.length; i++) {
      if (text[i] != ',') rawCount++;
    }
    // 在格式化后文本中找对应位置
    int newCursor = 0;
    int count = 0;
    for (int i = 0; i < formatted.length; i++) {
      if (count >= rawCount) break;
      if (formatted[i] != ',') count++;
      newCursor = i + 1;
    }
    newCursor = newCursor.clamp(0, formatted.length);

    return TextEditingValue(
      text: formatted,
      selection: TextSelection.collapsed(offset: newCursor),
    );
  }
}
