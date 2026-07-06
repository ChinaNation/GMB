import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/qr/bodies/sign_request_body.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/qr/qr_router.dart';

void main() {
  late QrRouter router;

  setUp(() {
    router = QrRouter();
  });

  group('QrRouter QR_V1', () {
    test('should route login sign_request', () {
      final raw = jsonEncode({
        'p': QrProtocols.v1,
        'k': QrKind.signRequest.code,
        'i': 'ch-0123456789abcdef',
        'e': 1090,
        'b': SignRequestBody.fromHex(
          action: QrActions.login,
          pubkeyHex:
              '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
          payloadHex: '0x6369647c736967',
        ).toJson(),
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.signRequest);
      expect(result.envelope, isNotNull);
    });

    test('should route user_transfer', () {
      final raw = jsonEncode({
        'p': QrProtocols.v1,
        'k': QrKind.userTransfer.code,
        'i': 'tx-0123456789abcdef',
        'e': 1600,
        'b': {
          'address': '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
          'recipient_name': '张三',
          'amount': '100.50',
          'symbol': 'GMB',
          'memo': '房租',
          'bank': '',
        },
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.userTransfer);
    });

    test('should route user_contact fixed code', () {
      final raw = jsonEncode({
        'p': QrProtocols.v1,
        'k': QrKind.userContact.code,
        'b': {
          'address': '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
          'contact_name': '张三',
        },
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.userContact);
    });

    test('should route sign_request', () {
      final raw = jsonEncode({
        'p': QrProtocols.v1,
        'k': QrKind.signRequest.code,
        'i': 'req-0123456789abcdef',
        'e': 1090,
        'b': SignRequestBody.fromHex(
          action: QrActions.transferWithRemark,
          pubkeyHex:
              '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
          payloadHex: '0xccdd',
        ).toJson(),
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
      final raw = jsonEncode({'p': 'UNKNOWN_V99', 'foo': 'bar'});
      final result = router.route(raw);
      expect(result.type, QrRouteType.unknown);
    });

    test('should reject removed QR kind 5', () {
      final raw = jsonEncode({
        'p': QrProtocols.v1,
        'k': 5,
        'b': {'removed': true},
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.unknown);
    });
  });
}
