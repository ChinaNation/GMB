import 'dart:convert';

import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:citizenapp/chat/transport/chat_cloud_transport.dart';
import 'package:citizenapp/chat/transport/chat_transport.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

/// CID 是 MLS 名册与投递的唯一身份键；当前账户只建立绑定会话。
const _bobAccountId =
    '0x2222222222222222222222222222222222222222222222222222222222222222';
const _bobCidNumber = 'CN220-CTZN2-100000002-2026';

void main() {
  test('未配置服务时密文保留在发送设备队列', () async {
    final envelope = _sampleEnvelope();
    final transport = ChatCloudTransport(
      accountId:
          '0x1111111111111111111111111111111111111111111111111111111111111111',
      localDeviceId: 'alice-phone',
    );

    final result = await transport.sendEncryptedEnvelope(
      envelopeId: envelope.envelopeId,
      envelopeBytes: envelope.writeToBuffer(),
      recipientCidNumber: _bobCidNumber,
    );

    expect(result.transportType, ChatTransportType.cloudflare);
    expect(result.state, ChatMessageDeliveryState.queued);
    expect(result.errorMessage, contains('瞬时转发尚未配置'));
  });

  test('密文只提交瞬时转发接口', () async {
    final envelope = _sampleEnvelope();
    final transport = _transport((request) async {
      expect(request.url.path, '/v1/chat/envelopes');
      final body = jsonDecode(request.body) as Map<String, dynamic>;
      expect(body.keys, {
        'envelope_id',
        'sender_device_id',
        'recipient_cid_number',
        'recipient_device_id',
        'envelope',
      });
      // 请求只携带收件人 CID，不允许另带账户路由。
      expect(body['recipient_cid_number'], _bobCidNumber);
      return _json({'ok': true, 'delivery_state': 'sent'});
    });

    final result = await transport.sendEncryptedEnvelope(
      envelopeId: envelope.envelopeId,
      envelopeBytes: envelope.writeToBuffer(),
      recipientCidNumber: _bobCidNumber,
    );

    expect(result.state, ChatMessageDeliveryState.sent);
  });

  test('接收设备离线时返回 queued 供本机重试', () async {
    final envelope = _sampleEnvelope();
    final transport = _transport(
      (_) async => _json({'ok': true, 'delivery_state': 'queued'}),
    );

    final result = await transport.sendEncryptedEnvelope(
      envelopeId: envelope.envelopeId,
      envelopeBytes: envelope.writeToBuffer(),
      recipientCidNumber: _bobCidNumber,
    );

    expect(result.state, ChatMessageDeliveryState.queued);
  });

  test('设备登记只提交公钥与无内容推送 token', () async {
    final transport = _transport((request) async {
      expect(request.url.path, '/v1/chat/devices/register');
      final body = jsonDecode(request.body) as Map<String, dynamic>;
      expect(body['push_provider'], 'fcm');
      expect(body['push_token'], 'fcm-token-1234567890');
      expect(body.containsKey('message'), isFalse);
      expect(body.containsKey('attachment'), isFalse);
      return _json({'ok': true});
    });

    await transport.registerDevice(
      devicePublicKey: 'aabb',
      pushProvider: 'fcm',
      pushToken: 'fcm-token-1234567890',
      bindingSignature: '0xsig',
      expiresAtMillis: 999999,
      nonce: 'nonce-123456',
    );
  });

  test('KeyPackage 消费响应不保留消费状态', () async {
    final transport = _transport((request) async {
      expect(request.url.path, '/v1/chat/keypackages/consume');
      final body = jsonDecode(request.body) as Map<String, dynamic>;
      // 领取按目标身份主键 CID 号寻址，不带领取方账户。
      expect(body['cid_number'], _bobCidNumber);
      return _json({'ok': true, 'key_package': _keyPackageJson()});
    });

    final package = await transport.consumeKeyPackage(
      targetCidNumber: _bobCidNumber,
      keyPackageId: 'kp-bob',
    );

    expect(package.keyPackageBytes, [1, 2, 3]);
    // CID 同时是 MLS 名册身份和投递主键；钱包账户不进入 KeyPackage。
    expect(package.cidNumber, _bobCidNumber);
  });

  test('WebRTC 只向 Worker 发送瞬时信令', () async {
    final paths = <String>[];
    final transport = _transport((request) async {
      paths.add(request.url.path);
      final body = jsonDecode(request.body) as Map<String, dynamic>;
      expect(body['signal'], {'kind': 'offer'});
      expect(body['recipient_cid_number'], _bobCidNumber);
      return _json({'ok': true, 'delivery_state': 'sent'});
    });

    final sent = await transport.sendSignal(
      recipientCidNumber: _bobCidNumber,
      signal: const {'kind': 'offer'},
    );

    expect(sent, isTrue);
    expect(paths, ['/v1/chat/signals']);
  });
}

ChatCloudTransport _transport(
  Future<http.Response> Function(http.Request request) handler,
) {
  return ChatCloudTransport(
    accountId:
        '0x1111111111111111111111111111111111111111111111111111111111111111',
    localDeviceId: 'alice-phone',
    serviceBaseUrl: Uri.parse('https://worker.example'),
    sessionToken: 'session-token',
    httpClient: MockClient(handler),
  );
}

ChatEnvelope _sampleEnvelope() => ChatEnvelope(
      protocolVersion: 1,
      envelopeId: 'env-1',
      conversationId: 'dm:alice:bob',
      senderCidNumber:
          '0x1111111111111111111111111111111111111111111111111111111111111111',
      recipientCidNumber: _bobAccountId,
      senderDeviceId: 'alice-phone',
      mlsWireMessage: [1, 2, 3],
      createdAtMillis: Int64(1),
      ttlMillis: Int64(60000),
      mlsMessageKind: MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION,
    );

/// Worker 真实响应形状：同时下发 MLS 名册身份 account_id 与寻址主键 cid_number。
Map<String, dynamic> _keyPackageJson() => {
      'account_id': _bobAccountId,
      'cid_number': _bobCidNumber,
      'device_id': 'bob-phone',
      'device_public_key_hex': 'aabb',
      'key_package_id': 'kp-bob',
      'key_package': base64Url.encode([1, 2, 3]).replaceAll('=', ''),
      'cipher_suite': 'MLS_128',
      'created_at': 1,
      'expires_at': 999999,
    };

http.Response _json(Map<String, dynamic> body) =>
    http.Response(jsonEncode(body), 200);
