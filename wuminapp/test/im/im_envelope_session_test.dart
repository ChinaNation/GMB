import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/im/crypto/im_mls_boundary.dart';
import 'package:wuminapp_mobile/im/im_message_flow.dart';
import 'package:wuminapp_mobile/im/im_session_models.dart';
import 'package:wuminapp_mobile/im/storage/im_isar_store.dart';
import 'package:wuminapp_mobile/im/transport/im_transport.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

void main() {
  setUp(() async {
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
          transportType: ImTransportType.privateNode,
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
        transportType: ImTransportType.privateNode,
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
}

class _FakeMlsCrypto implements ImMlsCryptoBoundary {
  final Set<String> _readyConversations = <String>{};

  @override
  Future<ImMlsKeyPackage> createKeyPackage(ImMlsDeviceIdentity identity) {
    throw UnimplementedError();
  }

  @override
  Future<ImMlsOutboundMessage> encrypt({
    required String conversationId,
    required String recipientChatAccount,
    ImMlsKeyPackage? recipientKeyPackage,
    required List<int> plaintext,
  }) async {
    final application = ImMlsWireMessage(
      wireBytes: plaintext,
      cipherSuite: 'MLS_128',
      conversationId: conversationId,
      messageKind: ImMlsMessageKind.application,
    );
    if (recipientKeyPackage == null) {
      return ImMlsOutboundMessage(
        conversationId: conversationId,
        applicationMessage: application,
      );
    }
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
