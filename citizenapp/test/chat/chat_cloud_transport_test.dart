import 'dart:convert';

import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:citizenapp/chat/transport/chat_cloud_transport.dart';
import 'package:citizenapp/chat/transport/chat_transport.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('未配置服务时密文保留在发送设备队列', () async {
    final envelope = _sampleEnvelope();
    final transport = ChatCloudTransport(
      ownerAccount: 'alice-wallet',
      ownerDeviceId: 'alice-phone',
    );

    final result = await transport.sendEncryptedEnvelope(
      envelopeId: envelope.envelopeId,
      envelopeBytes: envelope.writeToBuffer(),
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
        'recipient_account',
        'recipient_device_id',
        'envelope',
      });
      expect(body['recipient_account'], 'bob-wallet');
      return _json({'ok': true, 'delivery_state': 'sent'});
    });

    final result = await transport.sendEncryptedEnvelope(
      envelopeId: envelope.envelopeId,
      envelopeBytes: envelope.writeToBuffer(),
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
      devicePublicKeyHex: 'aabb',
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
      return _json({'ok': true, 'key_package': _keyPackageJson()});
    });

    final package = await transport.consumeKeyPackage(
      ownerAccount: 'bob-wallet',
      keyPackageId: 'kp-bob',
      requesterAccount: 'alice-wallet',
    );

    expect(package.keyPackageBytes, [1, 2, 3]);
  });

  test('TURN 与 WebRTC 信令使用独立瞬时接口', () async {
    final paths = <String>[];
    final transport = _transport((request) async {
      paths.add(request.url.path);
      if (request.url.path == '/v1/chat/turn') {
        return _json({
          'ok': true,
          'ice_servers': [
            {
              'urls': ['turn:turn.example:3478'],
              'username': 'user',
              'credential': 'secret',
            }
          ],
        });
      }
      final body = jsonDecode(request.body) as Map<String, dynamic>;
      expect(body['signal'], {'kind': 'offer'});
      return _json({'ok': true, 'delivery_state': 'sent'});
    });

    final servers = await transport.createIceServers();
    final sent = await transport.sendSignal(
      recipientAccount: 'bob-wallet',
      signal: const {'kind': 'offer'},
    );

    expect(servers.single.urls, ['turn:turn.example:3478']);
    expect(sent, isTrue);
    expect(paths, ['/v1/chat/turn', '/v1/chat/signals']);
  });
}

ChatCloudTransport _transport(
  Future<http.Response> Function(http.Request request) handler,
) {
  return ChatCloudTransport(
    ownerAccount: 'alice-wallet',
    ownerDeviceId: 'alice-phone',
    serviceBaseUrl: Uri.parse('https://worker.example'),
    sessionToken: 'session-token',
    httpClient: MockClient(handler),
  );
}

ChatEnvelope _sampleEnvelope() => ChatEnvelope(
      protocolVersion: 1,
      envelopeId: 'env-1',
      conversationId: 'dm:alice:bob',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      mlsWireMessage: [1, 2, 3],
      createdAtMillis: Int64(1),
      ttlMillis: Int64(60000),
      mlsMessageKind: MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION,
    );

Map<String, dynamic> _keyPackageJson() => {
      'owner_account': 'bob-wallet',
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
