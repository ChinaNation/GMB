// 每钱包扫码作用域谓词单测(安全边界:跨钱包签名请求必须拒绝)。
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/ui/scan_page.dart';

void main() {
  const walletA =
      '0x2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972';
  const walletB =
      '0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48';

  group('accountBelongsToWallet', () {
    test('同钱包账户放行', () {
      expect(accountBelongsToWallet(walletA, walletA), isTrue);
    });

    test('跨钱包账户拒绝', () {
      expect(accountBelongsToWallet(walletB, walletA), isFalse);
      expect(accountBelongsToWallet(walletA, walletB), isFalse);
    });
  });
}
