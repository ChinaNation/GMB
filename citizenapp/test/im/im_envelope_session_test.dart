import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/im/crypto/im_mls_boundary.dart';
import 'package:citizenapp/im/crypto/im_mls_state_store.dart';
import 'package:citizenapp/im/im_message_flow.dart';
import 'package:citizenapp/im/im_runtime.dart';
import 'package:citizenapp/im/im_session_models.dart';
import 'package:citizenapp/im/proto/im_envelope.pb.dart';
import 'package:citizenapp/im/storage/im_isar_store.dart';
import 'package:citizenapp/im/transport/im_cloudflare_transport.dart';
import 'package:citizenapp/im/transport/im_transport.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

void main() {
  setUp(() async {
    SharedPreferences.setMockInitialValues({});
    await WalletIsar.instance.resetForTest();
  });

  tearDown(() async {
    await WalletIsar.instance.resetForTest();
  });

  test('MLS wire message round-trips through formal ImEnvelope fields', () {
    const wire = ImMlsWireMessage(
      wireBytes: [0x01, 0x02],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-formal',
      messageKind: ImMlsMessageKind.welcome,
      ratchetTreeBytes: [0x0a, 0x0b],
    );

    final envelope = wire.toEnvelope(
      envelopeId: 'env-formal',
      senderChatAccount: 'alice-wallet',
      recipientChatAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      createdAtMillis: 1,
      ttlMillis: 60000,
    );
    final restored = imMlsWireMessageFromEnvelope(envelope);

    expect(restored.messageKind, ImMlsMessageKind.welcome);
    expect(restored.wireBytes, [0x01, 0x02]);
    expect(restored.ratchetTreeBytes, [0x0a, 0x0b]);
  });

  test('message flow sends Welcome and application envelopes in order',
      () async {
    final store = ImIsarStore();
    final delivered = <String>[];
    final flow = ImMessageFlow(
      crypto: _FakeMlsCrypto(),
      store: store,
      deliverer: (envelope, _) async {
        delivered.add(envelope.mlsMessageKind.name);
        return ImDeliveryResult(
          envelopeId: envelope.envelopeId,
          transportType: ImTransportType.cloudflare,
          state: ImMessageDeliveryState.sent,
        );
      },
    );

    final results = await flow.sendText(
      conversationId: 'conv-alice-bob',
      senderChatAccount: 'alice-wallet',
      recipientChatAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      recipientKeyPackage: _dummyKeyPackage(),
      text: 'hello bob',
    );

    expect(results, hasLength(2));
    expect(delivered, [
      'IM_MLS_WIRE_MESSAGE_KIND_WELCOME',
      'IM_MLS_WIRE_MESSAGE_KIND_APPLICATION',
    ]);
    expect(await store.outboundQueueCount(), 2);

    final conversations = await store.readConversationPreviews();
    expect(conversations.single.lastMessage, 'hello bob');
    expect(conversations.single.deliveryState, ImMessageDeliveryState.sent);

    final messages = await store.readMessages('conv-alice-bob');
    expect(messages, hasLength(1));
    expect(messages.single.plaintext, 'hello bob');
    expect(messages.single.deliveryState, ImMessageDeliveryState.sent);
  });

  test('message flow queues application before Welcome and replays it',
      () async {
    final store = ImIsarStore();
    final flow = ImMessageFlow(
      crypto: _FakeMlsCrypto(),
      store: store,
      deliverer: (_, __) async => const ImDeliveryResult(
        envelopeId: 'unused',
        transportType: ImTransportType.cloudflare,
        state: ImMessageDeliveryState.sent,
      ),
    );

    final application = const ImMlsWireMessage(
      wireBytes: [0xe4, 0xbd, 0xa0, 0xe5, 0xa5, 0xbd],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-incoming',
      messageKind: ImMlsMessageKind.application,
    ).toEnvelope(
      envelopeId: 'env-app',
      senderChatAccount: 'alice-wallet',
      recipientChatAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      createdAtMillis: 2,
      ttlMillis: 60000,
    );
    final welcome = const ImMlsWireMessage(
      wireBytes: [0x01],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-incoming',
      messageKind: ImMlsMessageKind.welcome,
      ratchetTreeBytes: [0x02],
    ).toEnvelope(
      envelopeId: 'env-welcome',
      senderChatAccount: 'alice-wallet',
      recipientChatAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      createdAtMillis: 1,
      ttlMillis: 60000,
    );

    final pendingResult =
        await flow.processIncomingEnvelopeBytes(application.writeToBuffer());
    expect(pendingResult.queuedPending, isTrue);
    expect(await store.pendingInboundCount(), 1);

    final welcomeResult =
        await flow.processIncomingEnvelopeBytes(welcome.writeToBuffer());
    expect(welcomeResult.accepted, isTrue);
    expect(await store.pendingInboundCount(), 0);

    final messages = await store.readMessages('conv-incoming');
    expect(messages.single.plaintext, '你好');
    expect(messages.single.direction, 'incoming');
  });

  test('message flow fetches mailbox pending, saves incoming message, and acks',
      () async {
    final mailbox = _MemoryMailbox();
    final aliceStore = ImIsarStore();
    final aliceFlow = ImMessageFlow(
      crypto: _FakeMlsCrypto(),
      store: aliceStore,
      deliverer: mailbox.submit,
    );

    final sendResults = await aliceFlow.sendText(
      conversationId: 'conv-mailbox-e2e',
      senderChatAccount: 'alice-wallet',
      recipientChatAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      recipientKeyPackage: _dummyKeyPackage(),
      text: 'mailbox 闭环',
    );

    expect(sendResults, hasLength(2));
    expect(mailbox.pendingCount('bob-wallet'), 2);
    final aliceMessages = await aliceStore.readMessages('conv-mailbox-e2e');
    expect(aliceMessages.single.direction, 'outgoing');
    expect(aliceMessages.single.deliveryState, ImMessageDeliveryState.sent);

    // 模拟 Bob 手机的独立本地数据库:远程 mailbox 保留,Alice 本地库不共享给 Bob。
    await WalletIsar.instance.resetForTest();
    final bobStore = ImIsarStore();
    final bobFlow = ImMessageFlow(
      crypto: _FakeMlsCrypto(),
      store: bobStore,
      deliverer: (_, __) {
        throw StateError('接收 pending 不应触发二次投递');
      },
    );

    final processed = await bobFlow.fetchAndProcessPending(
      fetchPending: () => mailbox.fetchPending('bob-wallet'),
      ackEnvelope: mailbox.ack,
    );

    expect(processed, 2);
    expect(mailbox.pendingCount('bob-wallet'), 0);
    expect(mailbox.ackedEnvelopeIds, hasLength(2));
    final bobMessages = await bobStore.readMessages('conv-mailbox-e2e');
    expect(bobMessages.single.direction, 'incoming');
    expect(bobMessages.single.plaintext, 'mailbox 闭环');
    final previews =
        await bobStore.readConversationPreviews(ownerChatAccount: 'bob-wallet');
    expect(previews.single.lastMessage, 'mailbox 闭环');
    expect(previews.single.unreadCount, 1);
  });

  test('message flow encrypts attachment objects and stores attachment message',
      () async {
    final mailbox = _MemoryMailbox();
    final uploadedObjects = <Uri, List<int>>{};
    late ImAttachmentCompleteRequest completedAttachment;
    final aliceStore = ImIsarStore();
    final aliceFlow = ImMessageFlow(
      crypto: _FakeMlsCrypto(),
      store: aliceStore,
      deliverer: mailbox.submit,
    );

    final results = await aliceFlow.sendAttachment(
      conversationId: 'conv-attachment',
      senderChatAccount: 'alice-wallet',
      recipientChatAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      recipientKeyPackage: _dummyKeyPackage(),
      attachment: const ImAttachmentDraft(
        fileName: 'photo.txt',
        contentType: 'text/plain',
        bytes: [0x70, 0x68, 0x6f, 0x74, 0x6f],
      ),
      prepareAttachmentUpload: ({
        required String conversationId,
        required String attachmentId,
        required int manifestByteSize,
        required List<ImAttachmentChunkDraft> chunks,
      }) async {
        expect(conversationId, 'conv-attachment');
        expect(attachmentId, startsWith('att-'));
        expect(chunks.single.byteSize, greaterThan(0));
        return ImAttachmentUploadPlan(
          attachmentId: attachmentId,
          manifestObjectKey:
              'chat/alice/conv-attachment/$attachmentId/manifest.enc',
          manifestUploadUrl: Uri.parse('https://worker.example/manifest'),
          chunks: [
            ImAttachmentUploadTarget(
              chunkId: chunks.single.chunkId,
              objectKey:
                  'chat/alice/conv-attachment/$attachmentId/chunk_001.bin',
              uploadUrl: Uri.parse('https://worker.example/chunk'),
            ),
          ],
        );
      },
      uploadAttachmentObject: ({
        required Uri uploadUrl,
        required List<int> bytes,
        required String contentType,
      }) async {
        uploadedObjects[uploadUrl] = bytes;
      },
      completeAttachmentUpload: (input) async {
        completedAttachment = input;
      },
    );

    expect(results, hasLength(2));
    expect(uploadedObjects, hasLength(2));
    expect(uploadedObjects[Uri.parse('https://worker.example/chunk')],
        isNot([0x70, 0x68, 0x6f, 0x74, 0x6f]));
    expect(completedAttachment.manifestHash, hasLength(64));
    final sentEnvelope = ImEnvelope.fromBuffer(
      mailbox.rows
          .singleWhere((row) => row.envelopeId.endsWith('-1'))
          .envelopeBytes,
    );
    expect(
        sentEnvelope.attachmentManifestHash, completedAttachment.manifestHash);
    expect(sentEnvelope.chunkRefs.first, completedAttachment.manifestObjectKey);

    final outgoing = await aliceStore.readMessages('conv-attachment');
    expect(outgoing.single.messageKind, ImMessageKind.attachment);
    expect(outgoing.single.plaintext, contains('gmb_im_attachment_v1'));

    await WalletIsar.instance.resetForTest();
    final bobStore = ImIsarStore();
    final bobFlow = ImMessageFlow(
      crypto: _FakeMlsCrypto(),
      store: bobStore,
      deliverer: (_, __) {
        throw StateError('接收附件 pending 不应触发二次投递');
      },
    );
    final tempDir = await Directory.systemTemp.createTemp('gmb-im-attachment-');
    try {
      await bobFlow.fetchAndProcessPending(
        fetchPending: () => mailbox.fetchPending('bob-wallet'),
        ackEnvelope: mailbox.ack,
        cacheIncomingAttachment: (conversationId, controlPlaintext) =>
            ImMessageFlow.downloadAttachment(
          conversationId: conversationId,
          controlPlaintext: controlPlaintext,
          cacheDirectory: tempDir,
          prepareAttachmentDownload: (input) async {
            expect(
                input.manifestObjectKey, completedAttachment.manifestObjectKey);
            expect(input.manifestHash, completedAttachment.manifestHash);
            return ImAttachmentDownloadPlan(
              attachmentId: input.attachmentId,
              manifestObjectKey: input.manifestObjectKey,
              manifestDownloadUrl: Uri.parse('https://worker.example/manifest'),
              chunks: [
                ImAttachmentDownloadTarget(
                  objectKey: input.chunkObjectKeys.single,
                  downloadUrl: Uri.parse('https://worker.example/chunk'),
                ),
              ],
            );
          },
          downloadAttachmentObject: (downloadUrl) async {
            return uploadedObjects[downloadUrl]!;
          },
        ),
      );
      final incoming = await bobStore.readMessages('conv-attachment');
      expect(incoming.single.messageKind, ImMessageKind.attachment);
      expect(incoming.single.plaintext, contains('photo.txt'));

      final downloaded = await ImMessageFlow.downloadAttachment(
        conversationId: 'conv-attachment',
        controlPlaintext: incoming.single.plaintext!,
        cacheDirectory: tempDir,
        prepareAttachmentDownload: (input) async {
          expect(
              input.manifestObjectKey, completedAttachment.manifestObjectKey);
          expect(input.manifestHash, completedAttachment.manifestHash);
          return ImAttachmentDownloadPlan(
            attachmentId: input.attachmentId,
            manifestObjectKey: input.manifestObjectKey,
            manifestDownloadUrl: Uri.parse('https://worker.example/manifest'),
            chunks: [
              ImAttachmentDownloadTarget(
                objectKey: input.chunkObjectKeys.single,
                downloadUrl: Uri.parse('https://worker.example/chunk'),
              ),
            ],
          );
        },
        downloadAttachmentObject: (downloadUrl) async {
          return uploadedObjects[downloadUrl]!;
        },
      );

      expect(downloaded.fileName, 'photo.txt');
      expect(downloaded.bytes, [0x70, 0x68, 0x6f, 0x74, 0x6f]);
      expect(await File(downloaded.filePath).readAsBytes(), downloaded.bytes);
    } finally {
      await tempDir.delete(recursive: true);
    }
  });

  test('runtime prepares Cloudflare mailbox automatically before send',
      () async {
    final requestPaths = <String>[];
    final store = ImIsarStore();
    late final MockClient httpClient;
    httpClient = MockClient((request) async {
      requestPaths.add('${request.method} ${request.url.path}');
      if (request.url.path == '/v1/square/auth/challenge') {
        return http.Response(
          jsonEncode({
            'ok': true,
            'challenge_id': 'challenge-1',
            'owner_account': 'alice-wallet',
            'op_tag': 0x1b,
            'signing_payload_hex': '6c6f67696e',
            'expires_at': DateTime.now()
                .add(const Duration(minutes: 5))
                .millisecondsSinceEpoch,
          }),
          200,
        );
      }
      if (request.url.path == '/v1/square/auth/session') {
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['signature'], '0xlogin');
        return http.Response(
          jsonEncode({
            'ok': true,
            'session_token': 'session-token',
            'expires_at': DateTime.now()
                .add(const Duration(hours: 1))
                .millisecondsSinceEpoch,
          }),
          200,
        );
      }
      expect(request.headers['authorization'], 'Bearer session-token');
      if (request.url.path == '/v1/chat/devices/register') {
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['owner_account'], 'alice-wallet');
        expect(body['device_id'], startsWith('im-'));
        expect(body['device_public_key_hex'], 'aabb');
        expect(body['binding_signature'], '0xbinding');
        return http.Response(jsonEncode({'ok': true}), 200);
      }
      if (request.url.path == '/v1/chat/keypackages' &&
          request.method == 'POST') {
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['owner_account'], 'alice-wallet');
        expect(body['device_public_key_hex'], 'aabb');
        return http.Response(jsonEncode({'ok': true}), 200);
      }
      if (request.url.path == '/v1/chat/keypackages/bob-wallet') {
        return http.Response(
          jsonEncode({
            'ok': true,
            'key_packages': [
              {
                'owner_account': 'bob-wallet',
                'device_id': 'bob-phone',
                'device_public_key_hex': 'ccdd',
                'key_package_id': 'kp-bob',
                'key_package': _base64UrlEncode([1, 2, 3]),
                'cipher_suite': 'MLS_128',
                'created_at': 1,
                'expires_at': 9999999999999,
                'consumed_at': null,
              }
            ],
          }),
          200,
        );
      }
      if (request.url.path == '/v1/chat/keypackages/consume') {
        return http.Response(
          jsonEncode({
            'ok': true,
            'key_package': {
              'owner_account': 'bob-wallet',
              'device_id': 'bob-phone',
              'device_public_key_hex': 'ccdd',
              'key_package_id': 'kp-bob',
              'key_package': _base64UrlEncode([1, 2, 3]),
              'cipher_suite': 'MLS_128',
              'created_at': 1,
              'expires_at': 9999999999999,
              'consumed_at': 2,
            },
          }),
          200,
        );
      }
      if (request.url.path == '/v1/chat/envelopes') {
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        expect(body['sender_account'], 'alice-wallet');
        expect(body['recipient_account'], 'bob-wallet');
        return http.Response(jsonEncode({'ok': true}), 200);
      }
      return http.Response(
        jsonEncode({'ok': false, 'message': 'unexpected'}),
        500,
      );
    });
    final api = SquareApiClient(
      baseUrl: 'https://worker.example',
      httpClient: httpClient,
    );
    final runtime = ImRuntime(
      store: store,
      walletManager: _FakeWalletManager(),
      preferences: await SharedPreferences.getInstance(),
      squareApiClient: api,
      cloudflareTransportFactory: ({
        required String ownerChatAccount,
        required String ownerDeviceId,
        Uri? mailboxBaseUrl,
        String? sessionToken,
      }) =>
          ImCloudflareTransport(
        ownerChatAccount: ownerChatAccount,
        ownerDeviceId: ownerDeviceId,
        mailboxBaseUrl: mailboxBaseUrl,
        sessionToken: sessionToken,
        httpClient: httpClient,
      ),
      squareLoginPayloadSigner: ({
        required int walletIndex,
        required String ownerAccount,
        required Uint8List loginMessage,
      }) async {
        expect(walletIndex, 7);
        expect(ownerAccount, 'alice-wallet');
        expect(loginMessage.length, 32);
        return '0xlogin';
      },
      walletPayloadSigner: ({
        required int walletIndex,
        required String ownerAccount,
        required Uint8List payload,
      }) async {
        expect(walletIndex, 7);
        expect(ownerAccount, 'alice-wallet');
        expect(payload.length, 32);
        return '0xbinding';
      },
      stateStoreFactory: (walletAccount, deviceId) async => ImMlsStateStore(
        Directory.systemTemp.createTempSync('im-runtime-test-'),
      ),
      cryptoFactory: (_, __) => _FakeMlsCrypto(),
    );

    final results = await runtime.sendText(
      peerWalletAddress: 'bob-wallet',
      conversationId: 'dm:alice-wallet:bob-wallet',
      text: 'hello',
    );

    expect(results, hasLength(2));
    expect(requestPaths, contains('POST /v1/square/auth/challenge'));
    expect(requestPaths, contains('POST /v1/square/auth/session'));
    expect(requestPaths, contains('POST /v1/chat/devices/register'));
    expect(requestPaths, contains('POST /v1/chat/keypackages'));
    expect(requestPaths, contains('GET /v1/chat/keypackages/bob-wallet'));
    expect(requestPaths, contains('POST /v1/chat/keypackages/consume'));
    expect(
      requestPaths.where((item) => item == 'POST /v1/chat/envelopes'),
      hasLength(2),
    );
    expect(
        await store.readMessages('dm:alice-wallet:bob-wallet'), hasLength(1));
  });
}

class _MemoryMailbox {
  final List<_MemoryMailboxRow> _rows = <_MemoryMailboxRow>[];
  final Set<String> ackedEnvelopeIds = <String>{};

  List<_MemoryMailboxRow> get rows => List.unmodifiable(_rows);

  Future<ImDeliveryResult> submit(
    ImEnvelope envelope,
    List<int> envelopeBytes,
  ) async {
    _rows.add(
      _MemoryMailboxRow(
        envelopeId: envelope.envelopeId,
        recipientChatAccount: envelope.recipientChatAccount,
        envelopeBytes: envelopeBytes,
      ),
    );
    return ImDeliveryResult(
      envelopeId: envelope.envelopeId,
      transportType: ImTransportType.cloudflare,
      state: ImMessageDeliveryState.sent,
    );
  }

  Future<List<ImPendingEncryptedEnvelope>> fetchPending(
    String ownerChatAccount,
  ) async {
    return _rows
        .where(
          (row) => row.recipientChatAccount == ownerChatAccount && !row.acked,
        )
        .map(
          (row) => ImPendingEncryptedEnvelope(
            envelopeId: row.envelopeId,
            envelopeBytes: row.envelopeBytes,
          ),
        )
        .toList(growable: false);
  }

  Future<void> ack(String envelopeId) async {
    for (final row in _rows) {
      if (row.envelopeId == envelopeId) {
        row.acked = true;
        ackedEnvelopeIds.add(envelopeId);
        return;
      }
    }
    throw StateError('未知 mailbox envelope: $envelopeId');
  }

  int pendingCount(String ownerChatAccount) {
    return _rows
        .where(
          (row) => row.recipientChatAccount == ownerChatAccount && !row.acked,
        )
        .length;
  }
}

class _MemoryMailboxRow {
  _MemoryMailboxRow({
    required this.envelopeId,
    required this.recipientChatAccount,
    required this.envelopeBytes,
  });

  final String envelopeId;
  final String recipientChatAccount;
  final List<int> envelopeBytes;
  bool acked = false;
}

class _FakeMlsCrypto implements ImMlsCryptoBoundary {
  final Set<String> _readyConversations = <String>{};

  @override
  Future<ImMlsKeyPackage> createKeyPackage(ImMlsDeviceIdentity identity) async {
    return ImMlsKeyPackage(
      ownerChatAccount: identity.walletChatAccount,
      deviceId: identity.deviceId,
      devicePublicKeyHex: 'aabb',
      keyPackageId: 'kp-${identity.deviceId}',
      keyPackageBytes: const [9, 8, 7],
      cipherSuite: 'MLS_128',
      createdAtMillis: 1,
      expiresAtMillis: 9999999999999,
    );
  }

  @override
  Future<ImMlsOutboundMessage> encrypt({
    required String conversationId,
    required String recipientChatAccount,
    ImMlsKeyPackage? recipientKeyPackage,
    required List<int> plaintext,
  }) async {
    if (recipientKeyPackage == null) {
      throw StateError('首次 MLS 会话必须提供对方 KeyPackage');
    }
    final application = ImMlsWireMessage(
      wireBytes: plaintext,
      cipherSuite: 'MLS_128',
      conversationId: conversationId,
      messageKind: ImMlsMessageKind.application,
    );
    _readyConversations.add(conversationId);
    return ImMlsOutboundMessage(
      conversationId: conversationId,
      welcomeMessage: ImMlsWireMessage(
        wireBytes: const [0x01],
        cipherSuite: 'MLS_128',
        conversationId: conversationId,
        messageKind: ImMlsMessageKind.welcome,
        ratchetTreeBytes: const [0x02],
      ),
      applicationMessage: application,
    );
  }

  @override
  Future<List<int>> decrypt(ImMlsWireMessage message) async {
    final inbound = await processIncoming(message);
    return inbound.plaintext ?? const [];
  }

  @override
  Future<ImMlsInboundMessage> processIncoming(ImMlsWireMessage message) async {
    if (message.messageKind == ImMlsMessageKind.welcome) {
      _readyConversations.add(message.conversationId);
      return ImMlsInboundMessage(
        conversationId: message.conversationId,
        messageKind: ImMlsMessageKind.welcome,
      );
    }
    if (!_readyConversations.contains(message.conversationId)) {
      throw StateError('MLS group missing');
    }
    return ImMlsInboundMessage(
      conversationId: message.conversationId,
      messageKind: ImMlsMessageKind.application,
      plaintext: message.wireBytes,
    );
  }
}

class _FakeWalletManager extends WalletManager {
  static const _alice = WalletProfile(
    walletIndex: 7,
    walletName: 'Alice',
    walletIcon: 'person',
    balance: 0,
    address: 'alice-wallet',
    pubkeyHex: '00',
    alg: 'sr25519',
    ss58: 2027,
    createdAtMillis: 1,
    source: 'test',
    signMode: 'local',
  );

  @override
  Future<WalletProfile?> getWalletByIndex(int walletIndex) async => _alice;

  @override
  Future<WalletProfile?> getDefaultWallet() async => _alice;
}

ImMlsKeyPackage _dummyKeyPackage() {
  return const ImMlsKeyPackage(
    ownerChatAccount: 'bob-wallet',
    deviceId: 'bob-phone',
    keyPackageId: 'kp-1',
    keyPackageBytes: [0x01],
    cipherSuite: 'MLS_128',
    createdAtMillis: 1,
    expiresAtMillis: 2,
  );
}

String _base64UrlEncode(List<int> bytes) {
  return base64Url.encode(bytes).replaceAll('=', '');
}
