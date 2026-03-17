import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/qr/qr_router.dart';

void main() {
  late QrRouter router;

  setUp(() {
    router = QrRouter();
  });

  group('QrRouter', () {
    test('should route login challenge', () {
      final raw = jsonEncode({
        'proto': 'WUMINAPP_LOGIN_V1',
        'system': 'sfid',
        'request_id': 'req-1',
        'challenge': 'abc123',
        'nonce': 'n1',
        'issued_at': 1000,
        'expires_at': 1090,
        'sys_pubkey': '0xaabb',
        'sys_sig': '0xccdd',
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.login);
      expect(result.jsonData, isNotNull);
    });

    test('should route transfer QR', () {
      final raw = jsonEncode({
        'proto': 'WUMINAPP_TRANSFER_V1',
        'to': '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        'amount': '100.50',
        'symbol': 'GMB',
        'memo': '房租',
        'bank': '',
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.transfer);
    });

    test('should route contact QR (new protocol)', () {
      final raw = jsonEncode({
        'proto': 'WUMINAPP_CONTACT_V1',
        'address': '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        'name': '张三',
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.contact);
    });

    test('should route legacy user card QR', () {
      final raw = jsonEncode({
        'type': 'WUMINAPP_USER_CARD_V1',
        'nickname': '张三',
        'account_pubkey': '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d',
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.contact);
    });

    test('should route qr sign request', () {
      final raw = jsonEncode({
        'proto': 'WUMINAPP_QR_SIGN_V1',
        'type': 'sign_request',
        'scope': 'onchain_tx',
        'request_id': 'req-1',
        'account': '5Grw...',
        'pubkey': '0xaabb',
        'sig_alg': 'sr25519',
        'payload_hex': '0xccdd',
        'issued_at': 1000,
        'expires_at': 1090,
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.qrSign);
    });

    test('should route gmb:// address', () {
      final raw =
          'gmb://account/5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
      final result = router.route(raw);
      expect(result.type, QrRouteType.legacyAddress);
      expect(result.extractedAddress,
          '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');
    });

    test('should route bare SS58 address', () {
      final raw = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
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
