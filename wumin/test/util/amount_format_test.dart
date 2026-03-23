import 'package:flutter_test/flutter_test.dart';
import 'package:wumin/util/amount_format.dart';

void main() {
  group('AmountFormat.format', () {
    test('基本金额格式化', () {
      expect(AmountFormat.format(100), '100.00 GMB');
      expect(AmountFormat.format(0), '0.00 GMB');
      expect(AmountFormat.format(99.99), '99.99 GMB');
    });

    test('千分位分隔', () {
      expect(AmountFormat.format(1234567.89), '1,234,567.89 GMB');
      expect(AmountFormat.format(1000), '1,000.00 GMB');
      expect(AmountFormat.format(1000000), '1,000,000.00 GMB');
    });

    test('自定义小数位', () {
      expect(AmountFormat.format(100, decimals: 0), '100 GMB');
      expect(AmountFormat.format(100, decimals: 4), '100.0000 GMB');
    });

    test('无币种后缀', () {
      expect(AmountFormat.format(100, symbol: ''), '100.00');
      expect(AmountFormat.format(1234, symbol: ''), '1,234.00');
    });

    test('自定义币种', () {
      expect(AmountFormat.format(50, symbol: 'BTC'), '50.00 BTC');
    });
  });

  group('AmountFormat.formatString', () {
    test('已有金额字符串添加千分位', () {
      expect(AmountFormat.formatString('1234567.89 GMB'), '1,234,567.89 GMB');
      expect(AmountFormat.formatString('100.00'), '100.00');
    });

    test('已有千分位不重复处理', () {
      // 纯数字 + 后缀
      expect(AmountFormat.formatString('999.99 GMB'), '999.99 GMB');
    });

    test('非数字输入原样返回', () {
      expect(AmountFormat.formatString('abc'), 'abc');
      expect(AmountFormat.formatString(''), '');
    });

    test('无小数部分', () {
      expect(AmountFormat.formatString('1000 GMB'), '1,000 GMB');
    });
  });
}
