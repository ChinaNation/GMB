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
        'p': QrProtocol.qrV1,
        'k': QrKind.signRequest.code,
        'i': 'ch-0123456789abcdef',
        'e': 1090,
        'b': SignRequestBody.fromHex(
          action: QrActions.login,
          signerPublicKeyHex:
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
        'p': QrProtocol.qrV1,
        'k': QrKind.userTransfer.code,
        'i': 'tx-0123456789abcdef',
        'e': 1600,
        'b': {
          'ss58_address': '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
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
        'p': QrProtocol.qrV1,
        'k': QrKind.userContact.code,
        'b': {
          'cid_number': 'CN001-CTZN-000000001-2026',
          'ss58_address': 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT',
          'display_name': '张三',
        },
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.userContact);
    });

    test('旧 contact_name 用户码和未知字段直接拒绝', () {
      final old = jsonEncode({
        'p': QrProtocol.qrV1,
        'k': QrKind.userContact.code,
        'b': {
          'ss58_address': 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT',
          'contact_name': '旧字段',
        },
      });
      final extra = jsonEncode({
        'p': QrProtocol.qrV1,
        'k': QrKind.userContact.code,
        'b': {
          'cid_number': 'CN001-CTZN-000000001-2026',
          'ss58_address': 'w5Bc7ma8qUcECfQDJmRyQM2wGmga5XSYtz7DvEengQ86xBWrT',
          'display_name': '张三',
          'name': '别名',
        },
      });

      expect(router.route(old).type, QrRouteType.unknown);
      expect(router.route(extra).type, QrRouteType.unknown);
    });

    test('should route sign_request', () {
      final raw = jsonEncode({
        'p': QrProtocol.qrV1,
        'k': QrKind.signRequest.code,
        'i': 'req-0123456789abcdef',
        'e': 1090,
        'b': SignRequestBody.fromHex(
          action: QrActions.transferWithRemark,
          signerPublicKeyHex:
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
      final raw = jsonEncode({'p': 'UNKNOWN_PROTO', 'foo': 'bar'});
      final result = router.route(raw);
      expect(result.type, QrRouteType.unknown);
    });

    test('should route wallet_code', () {
      final raw = jsonEncode({
        'p': QrProtocol.qrV1,
        'k': QrKind.walletCode.code,
        'b': {
          'account_id':
              '0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48',
        },
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.walletCode);
      expect(result.envelope, isNotNull);
      expect(result.envelope!.id, isNull);
      expect(result.envelope!.expiresAt, isNull);
    });

    test('should reject legacy chat_node_pairing payload on k=5', () {
      // k=5 已回收给钱包码；旧 chat_node_pairing 载荷靠 body 字段集精确匹配拒绝，
      // 不需要专门的拒绝分支。
      final raw = jsonEncode({
        'p': QrProtocol.qrV1,
        'k': 5,
        'b': {
          'node_peer_id': '12D3Koo',
          'node_multiaddr': '/ip4/1.2.3.4/tcp/30333',
          'endpoint_kind': 'ip4',
        },
      });
      final result = router.route(raw);
      expect(result.type, QrRouteType.unknown);
    });
  });
}
