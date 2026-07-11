import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:fixnum/fixnum.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/crypto/mls_boundary.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:citizenapp/chat/transport/chat_cloud_transport.dart';
import 'package:citizenapp/chat/transport/chat_transport.dart';

void main() {
  test('Cloudflare transport validates encrypted envelope bytes before config',
      () async {
    final transport = ChatCloudTransport(
      ownerAccount: 'alice-wallet',
      ownerDeviceId: 'alice-phone',
    );
    final envelope = ChatEnvelope(
      protocolVersion: 1,
      envelopeId: 'env-cloudflare',
      conversationId: 'dm:alice:bob',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      mlsWireMessage: [1, 2, 3],
      createdAtMillis: Int64(1),
      ttlMillis: Int64(60000),
      ackPolicy: 'device_ack',
      mlsMessageKind: MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION,
    );

    final result = await transport.sendEncryptedEnvelope(
      envelopeId: envelope.envelopeId,
      envelopeBytes: envelope.writeToBuffer(),
    );

    expect(result.transportType, ChatTransportType.cloudflare);
    expect(result.state, ChatMessageDeliveryState.failed);
    expect(result.errorMessage, contains('Cloudflare 密文 mailbox 尚未配置'));
  });

  test('Cloudflare transport rejects invalid envelope bytes', () async {
    final transport = ChatCloudTransport(
      ownerAccount: 'alice-wallet',
      ownerDeviceId: 'alice-phone',
    );

    final result = await transport.sendEncryptedEnvelope(
      envelopeId: 'bad-env',
      envelopeBytes: [0xff, 0xff, 0xff],
    );

    expect(result.transportType, ChatTransportType.cloudflare);
    expect(result.state, ChatMessageDeliveryState.failed);
    expect(result.errorMessage, contains('密文 envelope 格式无效'));
  });

  test('Cloudflare transport posts encrypted envelope to mailbox API',
      () async {
    final envelope = _sampleEnvelope();
    final transport = ChatCloudTransport(
      ownerAccount: 'alice-wallet',
      ownerDeviceId: 'alice-phone',
      mailboxBaseUrl: Uri.parse('https://worker.example'),
      sessionToken: 'session-token',
      httpClient: MockClient((request) async {
        expect(request.method, 'POST');
        expect(request.url.path, '/v1/chat/envelopes');
        expect(request.headers['authorization'], 'Bearer session-token');
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['envelope_id'], envelope.envelopeId);
        expect(body['sender_account'], envelope.senderAccount);
        expect(body['recipient_account'], envelope.recipientAccount);
        expect(body['mls_message_kind'], 'application');
        expect(body['envelope'], isNotEmpty);
        return http.Response(jsonEncode({'ok': true}), 200);
      }),
    );

    final result = await transport.sendEncryptedEnvelope(
      envelopeId: envelope.envelopeId,
      envelopeBytes: envelope.writeToBuffer(),
    );

    expect(result.state, ChatMessageDeliveryState.sent);
  });

  test('Cloudflare transport fetches and consumes KeyPackages', () async {
    final transport = ChatCloudTransport(
      ownerAccount: 'alice-wallet',
      ownerDeviceId: 'alice-phone',
      mailboxBaseUrl: Uri.parse('https://worker.example'),
      sessionToken: 'session-token',
      httpClient: MockClient((request) async {
        if (request.method == 'GET') {
          expect(request.url.path, '/v1/chat/keypackages/bob-wallet');
          return http.Response(
            jsonEncode({
              'ok': true,
              'key_packages': [_keyPackageJson(consumedAt: null)],
            }),
            200,
          );
        }
        expect(request.method, 'POST');
        expect(request.url.path, '/v1/chat/keypackages/consume');
        return http.Response(
          jsonEncode({
            'ok': true,
            'key_package': _keyPackageJson(consumedAt: 456),
          }),
          200,
        );
      }),
    );

    final packages = await transport.fetchKeyPackages(
      ownerAccount: 'bob-wallet',
      requesterAccount: 'alice-wallet',
    );
    final consumed = await transport.consumeKeyPackage(
      ownerAccount: 'bob-wallet',
      keyPackageId: packages.single.keyPackageId,
      requesterAccount: 'alice-wallet',
    );

    expect(packages.single.keyPackageBytes, [1, 2, 3]);
    expect(consumed.consumedAtMillis, 456);
  });

  test('Cloudflare transport fetches pending envelopes and acks them',
      () async {
    final envelope = _sampleEnvelope();
    var acked = false;
    final transport = ChatCloudTransport(
      ownerAccount: 'bob-wallet',
      ownerDeviceId: 'bob-phone',
      mailboxBaseUrl: Uri.parse('https://worker.example'),
      sessionToken: 'session-token',
      httpClient: MockClient((request) async {
        if (request.method == 'GET') {
          expect(request.url.path, '/v1/chat/envelopes/pending');
          expect(request.url.queryParameters['owner_account'], 'bob-wallet');
          expect(request.url.queryParameters['device_id'], 'bob-phone');
          return http.Response(
            jsonEncode({
              'ok': true,
              'envelopes': [
                {
                  'envelope_id': envelope.envelopeId,
                  'envelope': _base64UrlEncode(envelope.writeToBuffer()),
                }
              ],
            }),
            200,
          );
        }
        expect(request.method, 'POST');
        expect(request.url.path, '/v1/chat/envelopes/ack');
        acked = true;
        return http.Response(jsonEncode({'ok': true}), 200);
      }),
    );

    final pending = await transport.fetchPending();
    await transport.ackEnvelope(pending.single.envelopeId);

    expect(ChatEnvelope.fromBuffer(pending.single.envelopeBytes).envelopeId,
        envelope.envelopeId);
    expect(acked, isTrue);
  });

  test('Cloudflare transport publishes local KeyPackage', () async {
    final transport = ChatCloudTransport(
      ownerAccount: 'alice-wallet',
      ownerDeviceId: 'alice-phone',
      mailboxBaseUrl: Uri.parse('https://worker.example'),
      sessionToken: 'session-token',
      httpClient: MockClient((request) async {
        expect(request.method, 'POST');
        expect(request.url.path, '/v1/chat/keypackages');
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['owner_account'], 'alice-wallet');
        expect(body['key_package'], _base64UrlEncode([9, 8, 7]));
        return http.Response(jsonEncode({'ok': true}), 200);
      }),
    );

    await transport.publishKeyPackage(
      const MlsKeyPackage(
        ownerAccount: 'alice-wallet',
        deviceId: 'alice-phone',
        devicePublicKeyHex: 'aabb',
        keyPackageId: 'kp-alice',
        keyPackageBytes: [9, 8, 7],
        cipherSuite: 'MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519',
        createdAtMillis: 1,
        expiresAtMillis: 999,
      ),
    );
  });

  test('Cloudflare transport prepares, uploads, and completes attachment',
      () async {
    final requestPaths = <String>[];
    final transport = ChatCloudTransport(
      ownerAccount: 'alice-wallet',
      ownerDeviceId: 'alice-phone',
      mailboxBaseUrl: Uri.parse('https://worker.example'),
      sessionToken: 'session-token',
      httpClient: MockClient((request) async {
        requestPaths.add('${request.method} ${request.url.path}');
        if (request.url.path == '/v1/chat/attachments/prepare') {
          final body = jsonDecode(request.body) as Map<String, dynamic>;
          expect(body['owner_account'], 'alice-wallet');
          expect(body['device_id'], 'alice-phone');
          expect(body['conversation_id'], 'conv-1');
          return http.Response(
            jsonEncode({
              'ok': true,
              'attachment_id': body['attachment_id'],
              'manifest_object_key': 'chat/alice/conv-1/att-1/manifest.enc',
              'manifest_upload_url':
                  'https://worker.example/v1/chat/attachments/dev-put?object_key=manifest',
              'chunks': [
                {
                  'chunk_id': 'chunk-001',
                  'object_key': 'chat/alice/conv-1/att-1/chunk_001.bin',
                  'upload_url':
                      'https://worker.example/v1/chat/attachments/dev-put?object_key=chunk',
                }
              ],
            }),
            200,
          );
        }
        if (request.url.path == '/v1/chat/attachments/dev-put') {
          expect(request.method, 'PUT');
          expect(request.headers['authorization'], 'Bearer session-token');
          return http.Response(jsonEncode({'ok': true}), 200);
        }
        if (request.url.path == '/v1/chat/attachments/complete') {
          final body = jsonDecode(request.body) as Map<String, dynamic>;
          expect(body['manifest_hash'], 'a' * 64);
          expect(body['chunk_refs'], ['chat/alice/conv-1/att-1/chunk_001.bin']);
          return http.Response(jsonEncode({'ok': true}), 200);
        }
        return http.Response(jsonEncode({'ok': false}), 500);
      }),
    );

    final plan = await transport.prepareAttachmentUpload(
      conversationId: 'conv-1',
      attachmentId: 'att-1',
      manifestByteSize: 12,
      chunks: const [
        ChatAttachmentChunkDraft(chunkId: 'chunk-001', byteSize: 24),
      ],
    );
    await transport.uploadAttachmentObject(
      uploadUrl: plan.manifestUploadUrl,
      bytes: [1, 2, 3],
      contentType: 'application/octet-stream',
    );
    await transport.completeAttachmentUpload(
      ChatAttachmentCompleteRequest(
        attachmentId: 'att-1',
        conversationId: 'conv-1',
        manifestObjectKey: plan.manifestObjectKey,
        manifestHash: 'a' * 64,
        chunkObjectKeys: [plan.chunks.single.objectKey],
      ),
    );

    expect(requestPaths, [
      'POST /v1/chat/attachments/prepare',
      'PUT /v1/chat/attachments/dev-put',
      'POST /v1/chat/attachments/complete',
    ]);
  });

  test('Cloudflare transport prepares and downloads encrypted attachment',
      () async {
    final requestPaths = <String>[];
    final transport = ChatCloudTransport(
      ownerAccount: 'bob-wallet',
      ownerDeviceId: 'bob-phone',
      mailboxBaseUrl: Uri.parse('https://worker.example'),
      sessionToken: 'session-token',
      httpClient: MockClient((request) async {
        requestPaths.add('${request.method} ${request.url.path}');
        if (request.url.path == '/v1/chat/attachments/download') {
          final body = jsonDecode(request.body) as Map<String, dynamic>;
          expect(body['owner_account'], 'bob-wallet');
          expect(body['device_id'], 'bob-phone');
          expect(
              body['manifest_object_key'], 'chat/alice/conv/att/manifest.enc');
          return http.Response(
            jsonEncode({
              'ok': true,
              'attachment_id': body['attachment_id'],
              'manifest_object_key': body['manifest_object_key'],
              'manifest_download_url':
                  'https://worker.example/v1/chat/attachments/dev-get?object_key=manifest',
              'chunks': [
                {
                  'object_key': 'chat/alice/conv/att/chunk_001.bin',
                  'download_url':
                      'https://worker.example/v1/chat/attachments/dev-get?object_key=chunk',
                }
              ],
            }),
            200,
          );
        }
        if (request.url.path == '/v1/chat/attachments/dev-get') {
          expect(request.method, 'GET');
          expect(request.headers['authorization'], 'Bearer session-token');
          return http.Response.bytes([9, 8, 7], 200);
        }
        return http.Response(jsonEncode({'ok': false}), 500);
      }),
    );

    final plan = await transport.prepareAttachmentDownload(
      ChatAttachmentDownloadRequest(
        attachmentId: 'att-1',
        conversationId: 'conv',
        manifestObjectKey: 'chat/alice/conv/att/manifest.enc',
        manifestHash: 'b' * 64,
        chunkObjectKeys: ['chat/alice/conv/att/chunk_001.bin'],
      ),
    );
    final bytes = await transport.downloadAttachmentObject(
      plan.manifestDownloadUrl,
    );

    expect(plan.chunks.single.objectKey, 'chat/alice/conv/att/chunk_001.bin');
    expect(bytes, [9, 8, 7]);
    expect(requestPaths, [
      'POST /v1/chat/attachments/download',
      'GET /v1/chat/attachments/dev-get',
    ]);
  });
}

ChatEnvelope _sampleEnvelope() {
  return ChatEnvelope(
    protocolVersion: 1,
    envelopeId: 'env-cloudflare',
    conversationId: 'dm:alice:bob',
    senderAccount: 'alice-wallet',
    recipientAccount: 'bob-wallet',
    senderDeviceId: 'alice-phone',
    mlsWireMessage: [1, 2, 3],
    createdAtMillis: Int64(1),
    ttlMillis: Int64(60000),
    ackPolicy: 'device_ack',
    mlsMessageKind: MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION,
  );
}

Map<String, Object?> _keyPackageJson({required int? consumedAt}) {
  return {
    'owner_account': 'bob-wallet',
    'device_id': 'bob-phone',
    'device_public_key_hex': 'aabb',
    'key_package_id': 'kp-bob',
    'key_package': _base64UrlEncode([1, 2, 3]),
    'cipher_suite': 'MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519',
    'created_at': 123,
    'expires_at': 999,
    'consumed_at': consumedAt,
  };
}

String _base64UrlEncode(List<int> bytes) {
  return base64Url.encode(bytes).replaceAll('=', '');
}
