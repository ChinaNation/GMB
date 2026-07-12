import 'dart:convert';
import 'dart:io';

import 'package:citizenapp/chat/chat_flow.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/crypto/mls_boundary.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/chat/transport/chat_transport.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/isar_test_env.dart';

void main() {
  useIsolatedIsar();

  test('MLS wire message 只写入目标 ChatEnvelope 字段', () {
    const wire = MlsWireMessage(
      wireBytes: [0x01, 0x02],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-formal',
      messageKind: MlsMessageKind.welcome,
      ratchetTreeBytes: [0x0a, 0x0b],
    );

    final restored = imMlsWireMessageFromEnvelope(
      wire.toEnvelope(
        envelopeId: 'env-formal',
        senderAccount: 'alice-wallet',
        recipientAccount: 'bob-wallet',
        senderDeviceId: 'alice-phone',
        createdAtMillis: 1,
        ttlMillis: 60000,
      ),
    );

    expect(restored.messageKind, MlsMessageKind.welcome);
    expect(restored.wireBytes, [0x01, 0x02]);
    expect(restored.ratchetTreeBytes, [0x0a, 0x0b]);
  });

  test('接收设备离线时密文只留发送设备本机队列', () async {
    final store = ChatStore();
    final flow = ChatFlow(
      crypto: _FakeMlsCrypto(),
      store: store,
      deliverer: (envelope, _) async => ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.queued,
      ),
    );

    await flow.sendText(
      conversationId: 'conv-alice-bob',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      recipientKeyPackage: _dummyKeyPackage(),
      text: 'hello bob',
    );

    final queued = await store.readQueuedEnvelopes();
    expect(queued, hasLength(2));
    expect(
        queued.every((item) => item.recipientAccount == 'bob-wallet'), isTrue);
    for (final item in queued) {
      await store.markOutgoingDelivery(
        envelopeId: item.envelopeId,
        state: ChatMessageDeliveryState.sent,
      );
    }
    expect(await store.outboundQueueCount(), 0);
  });

  test('在线设备收到密文后立即解密并写入本机', () async {
    final flow = ChatFlow(
      crypto: _FakeMlsCrypto(),
      store: ChatStore(),
      deliverer: (_, __) => throw StateError('接收端不得重新投递'),
    );
    final welcome = const MlsWireMessage(
      wireBytes: [0x01],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-incoming',
      messageKind: MlsMessageKind.welcome,
      ratchetTreeBytes: [0x02],
    ).toEnvelope(
      envelopeId: 'env-welcome',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      createdAtMillis: 1,
      ttlMillis: 60000,
    );
    final application = MlsWireMessage(
      wireBytes: utf8.encode('设备直收'),
      cipherSuite: 'MLS_128',
      conversationId: 'conv-incoming',
      messageKind: MlsMessageKind.application,
    ).toEnvelope(
      envelopeId: 'env-app',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      createdAtMillis: 2,
      ttlMillis: 60000,
    );

    await flow.processIncomingEnvelopeBytes(welcome.writeToBuffer());
    final result =
        await flow.processIncomingEnvelopeBytes(application.writeToBuffer());

    expect(result.plaintext, '设备直收');
    final messages = await ChatStore().readMessages('conv-incoming');
    expect(messages.single.plaintext, '设备直收');
    expect(messages.single.direction, 'incoming');
  });

  test('附件字节经设备通道发送且控制消息不含云端对象引用', () async {
    final sentBytes = <int>[];
    final flow = ChatFlow(
      crypto: _FakeMlsCrypto(),
      store: ChatStore(),
      deliverer: (envelope, _) async => ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.sent,
      ),
    );

    await flow.sendAttachment(
      conversationId: 'conv-attachment',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      recipientKeyPackage: _dummyKeyPackage(),
      attachment: const ChatAttachmentDraft(
        fileName: 'photo.jpg',
        contentType: 'image/jpeg',
        bytes: [1, 2, 3, 4],
      ),
      sendDeviceAttachment: ({
        required recipientAccount,
        required conversationId,
        required attachmentId,
        required fileName,
        required contentType,
        required bytes,
      }) async {
        sentBytes.addAll(bytes);
      },
    );

    expect(sentBytes, [1, 2, 3, 4]);
    final message =
        (await ChatStore().readMessages('conv-attachment')).single.plaintext!;
    expect(message, contains('gmb_chat_attachment_v2'));
    expect(message, isNot(contains('object_key')));
    expect(message, isNot(contains('manifest')));
  });

  test('附件下载只读取设备本地缓存', () async {
    final root = await Directory.systemTemp.createTemp('gmb-chat-device-');
    addTearDown(() => root.delete(recursive: true));
    final saved = await ChatFlow.saveAttachmentBytesToCache(
      conversationId: 'conv-cache',
      attachmentId: 'attachment-1',
      fileName: 'note.txt',
      contentType: 'text/plain',
      bytes: utf8.encode('local only'),
      cacheDirectory: root,
    );
    final control = jsonEncode({
      'type': 'gmb_chat_attachment_v2',
      'attachment_id': 'attachment-1',
      'file_name': 'note.txt',
      'content_type': 'text/plain',
      'clear_byte_size': 10,
    });

    final loaded = await ChatFlow.downloadAttachment(
      conversationId: 'conv-cache',
      controlPlaintext: control,
      cacheDirectory: root,
    );

    expect(loaded.filePath, saved.filePath);
    expect(utf8.decode(loaded.bytes), 'local only');
  });
}

class _FakeMlsCrypto implements MlsCrypto {
  final Set<String> _ready = <String>{};

  @override
  Future<MlsKeyPackage> createKeyPackage(ChatDevice identity) async =>
      _dummyKeyPackage();

  @override
  Future<MlsOutboundMessage> encrypt({
    required String conversationId,
    required String recipientAccount,
    MlsKeyPackage? recipientKeyPackage,
    required List<int> plaintext,
  }) async {
    if (!_ready.add(conversationId) && recipientKeyPackage == null) {
      return MlsOutboundMessage(
        conversationId: conversationId,
        applicationMessage: MlsWireMessage(
          wireBytes: plaintext,
          cipherSuite: 'MLS_128',
          conversationId: conversationId,
          messageKind: MlsMessageKind.application,
        ),
      );
    }
    if (recipientKeyPackage == null) {
      throw StateError('首次 MLS 会话必须提供对方 KeyPackage');
    }
    return MlsOutboundMessage(
      conversationId: conversationId,
      welcomeMessage: MlsWireMessage(
        wireBytes: const [1],
        cipherSuite: 'MLS_128',
        conversationId: conversationId,
        messageKind: MlsMessageKind.welcome,
        ratchetTreeBytes: const [2],
      ),
      applicationMessage: MlsWireMessage(
        wireBytes: plaintext,
        cipherSuite: 'MLS_128',
        conversationId: conversationId,
        messageKind: MlsMessageKind.application,
      ),
    );
  }

  @override
  Future<List<int>> decrypt(MlsWireMessage message) async =>
      (await processIncoming(message)).plaintext ?? const [];

  @override
  Future<MlsInboundMessage> processIncoming(MlsWireMessage message) async {
    if (message.messageKind == MlsMessageKind.welcome) {
      _ready.add(message.conversationId);
      return MlsInboundMessage(
        conversationId: message.conversationId,
        messageKind: message.messageKind,
      );
    }
    if (!_ready.contains(message.conversationId)) {
      throw StateError('MLS group missing');
    }
    return MlsInboundMessage(
      conversationId: message.conversationId,
      messageKind: message.messageKind,
      plaintext: message.wireBytes,
    );
  }
}

MlsKeyPackage _dummyKeyPackage() => const MlsKeyPackage(
      ownerAccount: 'bob-wallet',
      deviceId: 'bob-phone',
      devicePublicKeyHex: 'aabb',
      keyPackageId: 'kp-bob',
      keyPackageBytes: [1],
      cipherSuite: 'MLS_128',
      createdAtMillis: 1,
      expiresAtMillis: 9999999999999,
    );
