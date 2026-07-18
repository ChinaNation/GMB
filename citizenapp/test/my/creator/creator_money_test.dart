import 'package:citizenapp/my/creator/creator_money.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('fenToYuanLabel（分→元展示，去尾零）', () {
    test('整元不带小数', () {
      expect(fenToYuanLabel(2700), '27');
      expect(fenToYuanLabel(29900), '299');
      expect(fenToYuanLabel(100), '1');
    });

    test('带小数去尾零', () {
      expect(fenToYuanLabel(990), '9.9');
      expect(fenToYuanLabel(150), '1.5');
      expect(fenToYuanLabel(1), '0.01');
    });
  });

  group('yuanTextToFen（元输入→分，非法返回 null）', () {
    test('合法元转分', () {
      expect(yuanTextToFen('9.9'), 990);
      expect(yuanTextToFen('27'), 2700);
      expect(yuanTextToFen('1.5'), 150);
      expect(yuanTextToFen('￥9.9'), 990);
    });

    test('非法输入返回 null', () {
      expect(yuanTextToFen(''), isNull);
      expect(yuanTextToFen('abc'), isNull);
      expect(yuanTextToFen('0'), isNull);
      expect(yuanTextToFen('-1'), isNull);
      expect(yuanTextToFen('9.999'), isNull); // 超过两位小数
    });
  });
}
