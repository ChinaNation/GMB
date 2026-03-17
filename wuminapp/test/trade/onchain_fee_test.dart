import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart';

void main() {
  group('OnchainRpc.estimateTransferFeeYuan', () {
    // 链上常量：费率 0.1%（Perbill(1_000_000)），最低手续费 10 fen = 0.10 元

    test('small amount → minimum fee 0.10 yuan', () {
      // 1 元 → 1 fen by rate → min(1,10) → 10 fen = 0.10 元
      expect(OnchainRpc.estimateTransferFeeYuan(1.0), 0.10);
    });

    test('50 yuan → below minimum → 0.10 yuan', () {
      // 50 元 → 5000 fen * 1_000_000 / 1_000_000_000 = 5 fen → min fee
      expect(OnchainRpc.estimateTransferFeeYuan(50.0), 0.10);
    });

    test('100 yuan → exactly at minimum → 0.10 yuan', () {
      // 100 元 = 10000 fen → 10000 * 1_000_000 / 1_000_000_000 = 10 fen
      expect(OnchainRpc.estimateTransferFeeYuan(100.0), 0.10);
    });

    test('500 yuan → 0.50 yuan', () {
      // 500 元 = 50000 fen → 50 fen = 0.50 元
      expect(OnchainRpc.estimateTransferFeeYuan(500.0), 0.50);
    });

    test('10000 yuan → 10.00 yuan', () {
      // 10000 元 = 1000000 fen → 1000 fen = 10.00 元
      expect(OnchainRpc.estimateTransferFeeYuan(10000.0), 10.00);
    });

    test('0 yuan → minimum fee 0.10 yuan', () {
      expect(OnchainRpc.estimateTransferFeeYuan(0.0), 0.10);
    });

    test('0.01 yuan → minimum fee 0.10 yuan', () {
      // 0.01 元 = 1 fen → 0 by rate → min fee
      expect(OnchainRpc.estimateTransferFeeYuan(0.01), 0.10);
    });

    test('99.99 yuan → below minimum → 0.10 yuan', () {
      // 99.99 元 = 9999 fen → 9999 * 1_000_000 / 1_000_000_000
      // = 9_999_000_000 / 1_000_000_000 ≈ 9.999 → half-up → 10 fen
      // (9999 * 1_000_000 + 500_000_000) ~/ 1_000_000_000
      // = 10_499_000_000 ~/ 1_000_000_000 = 10 fen
      expect(OnchainRpc.estimateTransferFeeYuan(99.99), 0.10);
    });

    test('large amount 1000000 yuan → 1000.00 yuan', () {
      // 1_000_000 元 = 100_000_000 fen → 100_000 fen = 1000.00 元
      expect(OnchainRpc.estimateTransferFeeYuan(1000000.0), 1000.00);
    });

    test('fractional amount 123.45 yuan', () {
      // 123.45 元 = 12345 fen
      // byRate = (12345 * 1_000_000 + 500_000_000) ~/ 1_000_000_000
      //        = (12_345_000_000 + 500_000_000) ~/ 1_000_000_000
      //        = 12_845_000_000 ~/ 1_000_000_000 = 12 fen = 0.12 元
      expect(OnchainRpc.estimateTransferFeeYuan(123.45), 0.12);
    });
  });
}
