import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/qr/qr_protocols.dart';
import 'package:wuminapp_mobile/qr/qr_router.dart';

void main() {
  late QrRouter router;

  setUp(() {
    router = QrRouter();
  });

  group('QrRouter WUMIN_QR_V1', () {
    test('should route login_challenge', () {
      final raw = jsonEncode({
        'proto': QrProtocols.v1,
        'kind': 'login_challenge',
        'id': 'ch_01',
        'issued_at': 1000,
        'expires_at': 1090,
        'body': {
          'system': 'sfid',
          'sys_pubkey': '0xaabb',
          'sys_sig': '0xccdd',
        },
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.loginChallenge);
      expect(result.envelope, isNotNull);
    });

    test('should route user_transfer', () {
      final raw = jsonEncode({
        'proto': QrProtocols.v1,
        'kind': 'user_transfer',
        'id': 'tx_01',
        'issued_at': 1000,
        'expires_at': 1600,
        'body': {
          'address': '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
          'name': '张三',
          'amount': '100.50',
          'symbol': 'GMB',
          'memo': '房租',
          'bank': '',
        },
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.userTransfer);
    });

    test('should route user_contact (fixed, no id/issued_at/expires_at)', () {
      final raw = jsonEncode({
        'proto': QrProtocols.v1,
        'kind': 'user_contact',
        'body': {
          'address': '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
          'name': '张三',
        },
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.userContact);
    });

    test('should route user_duoqian (fixed)', () {
      final raw = jsonEncode({
        'proto': QrProtocols.v1,
        'kind': 'user_duoqian',
        'body': {
          'address': '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
          'name': '多签账户',
          'proposal_id': 0,
        },
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.userDuoqian);
    });

    test('should route sign_request', () {
      final raw = jsonEncode({
        'proto': QrProtocols.v1,
        'kind': 'sign_request',
        'id': 'req_01',
        'issued_at': 1000,
        'expires_at': 1090,
        'body': {
          'address': '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
          'pubkey': '0xaabb',
          'sig_alg': 'sr25519',
          'payload_hex': '0xccdd',
          'spec_version': 100,
          'display': {
            'action': 'transfer',
            'summary': '转账',
            'fields': [],
          },
        },
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.signRequest);
    });

    test('should route gmb:// address', () {
      const raw =
          'gmb://account/5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
      final result = router.route(raw);
      expect(result.type, QrRouteType.legacyAddress);
      expect(result.extractedAddress,
          '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');
    });

    test('should route bare SS58 address', () {
      const raw = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
      final result = router.route(raw);
      expect(result.type, QrRouteType.legacyAddress);
      expect(result.extractedAddress, raw);
    });

    test('should return unknown for unrecognized content', () {
      final result = router.route('hello world');
      expect(result.type, QrRouteType.unknown);
    });

    test('should return unknown for empty string', () {
      final result = router.route('');
      expect(result.type, QrRouteType.unknown);
    });

    test('should return unknown for JSON with unknown proto', () {
      final raw = jsonEncode({'proto': 'UNKNOWN_V99', 'foo': 'bar'});
      final result = router.route(raw);
      expect(result.type, QrRouteType.unknown);
    });
  });
}
