import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:citizenapp/transaction/onchain-topup/topup_api.dart';
import 'package:citizenapp/transaction/onchain-topup/topup_erc20.dart';
import 'package:citizenapp/transaction/onchain-topup/topup_models.dart';

void main() {
  group('WalletConnect WebView CSP', () {
    final html = File('assets/topup/walletconnect.html').readAsStringSync();

    test('只给钱包图片开放 blob 并固定 Reown 字体域名', () {
      expect(
        html,
        contains("img-src 'self' data: blob: https:;"),
      );
      expect(
        html,
        contains('font-src https://fonts.reown.com;'),
      );
    });

    test('不向脚本或框架开放 blob，正式官网元数据保持不变', () {
      expect(
        html,
        contains("script-src 'self' 'unsafe-inline';"),
      );
      expect(
        html,
        isNot(contains('script-src blob:')),
      );
      expect(
        html,
        isNot(contains('frame-src blob:')),
      );
      expect(
        html,
        contains("url: 'https://www.crcfrcn.com'"),
      );
    });
  });

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
      expect(
          () => encodeErc20Transfer('0x1234', BigInt.one), throwsArgumentError);
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
          {
            'token': 'USDC',
            'chain_id': 84532,
            'token_contract': '0x${'11' * 20}',
            'token_decimals': 6,
            'label': 'USDC · Base Sepolia'
          },
        ],
        'packages': [
          {
            'package_id': 'pkg_15',
            'pay_display': '15',
            'pay_amount': '15000000',
            'coin_display': '10,000.00',
            'coin_fen': '1000000'
          },
        ],
      });
      expect(config.rails.single.chainId, 84532);
      expect(config.rails.single.caip2, 'eip155:84532');
      expect(config.packages.single.payAmountValue, BigInt.from(15000000));
    });
  });

  group('TopupApi', () {
    const accountId =
        '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    // 充值已与广场会话解耦:客户端不再持有 / 传递任何 session。
    TopupApi apiWith(MockClient client) => TopupApi(
          baseUrl: 'https://x.test/api',
          httpClient: client,
        );

    test('fetchConfig 走 /v1/square/topup/config', () async {
      final api = apiWith(MockClient((request) async {
        expect(request.url.path, '/api/v1/square/topup/config');
        return http.Response(
          jsonEncode({
            'ok': true,
            'network': 'testnet',
            'recv_address': '0x${'cd' * 20}',
            'rails': [],
            'packages': []
          }),
          200,
        );
      }));
      final config = await api.fetchConfig();
      expect(config.network, 'testnet');
    });

    test('createIntent 上传充值目标 account_id 且不带任何会话头', () async {
      final api = apiWith(MockClient((request) async {
        expect(request.method, 'POST');
        expect(request.url.path, '/api/v1/square/topup/intent');
        expect(request.headers.containsKey('authorization'), isFalse);
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['account_id'], accountId);
        return http.Response(
            jsonEncode({
              'ok': true,
              'payment_intent': 'signed-intent',
              'expires_at': 123
            }),
            200);
      }));
      final result = await api.createIntent(
        token: 'USDC',
        packageId: 'pkg_15',
        accountId: accountId,
        payerAddress: '0x${'22' * 20}',
      );
      expect(result.token, 'signed-intent');
    });

    test('confirm 未确认 → confirming，且不带会话头', () async {
      final api = apiWith(MockClient((request) async {
        expect(request.headers.containsKey('authorization'), isFalse);
        return http.Response(
            jsonEncode({'ok': true, 'status': 'confirming'}), 200);
      }));
      final result = await api.confirm(
        paymentIntent: 'signed-intent',
        evmTxHash: '0x${'22' * 32}',
      );
      expect(result.status, TopupOrderStatus.confirming);
    });

    test('status 走 POST，凭付款意图查单，订单号不入 URL', () async {
      final api = apiWith(MockClient((request) async {
        expect(request.method, 'POST');
        expect(request.url.path, '/api/v1/square/topup/status');
        expect(request.headers.containsKey('authorization'), isFalse);
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['order_id'], 'top_123');
        expect(body['payment_intent'], 'signed-intent');
        return http.Response(jsonEncode({'ok': true, 'status': 'paid'}), 200);
      }));
      final status = await api.status(
        orderId: 'top_123',
        paymentIntent: 'signed-intent',
      );
      expect(status, TopupOrderStatus.paid);
    });

    test('非 2xx 抛 TopupApiException 带 error_code', () async {
      final api = apiWith(MockClient((request) async => http.Response(
            jsonEncode({
              'ok': false,
              'error_code': 'topup_payment_invalid',
              'message': '未确认到有效到账'
            }),
            400,
            headers: {'content-type': 'application/json; charset=utf-8'},
          )));
      expect(
        () => api.confirm(
            paymentIntent: 'signed-intent', evmTxHash: '0x${'11' * 32}'),
        throwsA(isA<TopupApiException>()
            .having((e) => e.errorCode, 'errorCode', 'topup_payment_invalid')),
      );
    });
  });
}
