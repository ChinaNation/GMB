import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/transaction/onchain-topup/topup_api.dart';
import 'package:citizenapp/transaction/onchain-topup/topup_erc20.dart';
import 'package:citizenapp/transaction/onchain-topup/topup_models.dart';

void main() {
  group('encodeErc20Transfer', () {
    test('按 selector + 32B 地址 + 32B 金额编码', () {
      final data = encodeErc20Transfer('0x${'ab' * 20}', BigInt.from(15000000));
      final expected = '0xa9059cbb'
          '${'0' * 24}${'ab' * 20}'
          '${'0' * 58}e4e1c0';
      expect(data, expected);
      expect(data.length, 2 + 8 + 64 + 64);
    });

    test('非法地址抛错', () {
      expect(() => encodeErc20Transfer('0x1234', BigInt.one), throwsArgumentError);
    });

    test('负数金额抛错', () {
      expect(
        () => encodeErc20Transfer('0x${'ab' * 20}', BigInt.from(-1)),
        throwsArgumentError,
      );
    });
  });

  group('TopupConfig 解析', () {
    test('解析币轨与套餐', () {
      final config = TopupConfig.fromJson({
        'network': 'testnet',
        'recv_address': '0x${'cd' * 20}',
        'rails': [
          {'token': 'USDC', 'chain_id': 84532, 'token_contract': '0x${'11' * 20}', 'token_decimals': 6, 'label': 'USDC · Base Sepolia'},
        ],
        'packages': [
          {'package_id': 'pkg_15', 'pay_display': '15', 'pay_amount': '15000000', 'coin_display': '10,000.00', 'coin_fen': '1000000'},
        ],
      });
      expect(config.rails.single.chainId, 84532);
      expect(config.rails.single.caip2, 'eip155:84532');
      expect(config.packages.single.payAmountValue, BigInt.from(15000000));
    });
  });

  group('TopupApi', () {
    TopupApi apiWith(MockClient client) =>
        TopupApi(baseUrl: 'https://x.test/api', httpClient: client);

    test('fetchConfig 走 /v1/square/topup/config', () async {
      final api = apiWith(MockClient((request) async {
        expect(request.url.path, '/api/v1/square/topup/config');
        return http.Response(
          jsonEncode({'ok': true, 'network': 'testnet', 'recv_address': '0x${'cd' * 20}', 'rails': [], 'packages': []}),
          200,
        );
      }));
      final config = await api.fetchConfig();
      expect(config.network, 'testnet');
    });

    test('submit 已确认 → pending(待支付)', () async {
      final api = apiWith(MockClient((request) async {
        expect(request.method, 'POST');
        return http.Response(jsonEncode({'ok': true, 'status': 'pending', 'order_id': 'top_x'}), 200);
      }));
      final result = await api.submit(
        token: 'USDC', packageId: 'pkg_15', gmbAddress: 'gmbaddr', evmTxHash: '0x${'11' * 32}',
      );
      expect(result.status, TopupOrderStatus.pending);
      expect(result.orderId, 'top_x');
    });

    test('submit 未确认 → confirming', () async {
      final api = apiWith(MockClient((request) async =>
          http.Response(jsonEncode({'ok': true, 'status': 'confirming'}), 200)));
      final result = await api.submit(
        token: 'USDT', packageId: 'pkg_1400', gmbAddress: 'gmbaddr', evmTxHash: '0x${'22' * 32}',
      );
      expect(result.status, TopupOrderStatus.confirming);
    });

    test('status 解析已支付', () async {
      final api = apiWith(MockClient((request) async {
        expect(request.url.query, contains('chain_id=84532'));
        return http.Response(jsonEncode({'ok': true, 'status': 'paid'}), 200);
      }));
      final status = await api.status(chainId: 84532, evmTxHash: '0x${'11' * 32}');
      expect(status, TopupOrderStatus.paid);
    });

    test('非 2xx 抛 TopupApiException 带 error_code', () async {
      final api = apiWith(MockClient((request) async => http.Response(
            jsonEncode({'ok': false, 'error_code': 'topup_payment_invalid', 'message': '未确认到有效到账'}),
            400,
            headers: {'content-type': 'application/json; charset=utf-8'},
          )));
      expect(
        () => api.submit(token: 'USDC', packageId: 'pkg_15', gmbAddress: 'g', evmTxHash: '0x${'11' * 32}'),
        throwsA(isA<TopupApiException>().having((e) => e.errorCode, 'errorCode', 'topup_payment_invalid')),
      );
    });
  });
}
