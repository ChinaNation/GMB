import 'package:citizenapp/chat/chat_flow.dart';
import 'package:citizenapp/chat/chat_media_limits.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/chat_payload.dart';
import 'package:citizenapp/chat/crypto/mls_boundary.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/chat/transport/chat_transport.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../support/isar_test_env.dart';

class _FakeMls implements MlsCrypto {
  @override
  Future<MlsKeyPackage> createKeyPackage(ChatDevice identity) async =>
      throw UnimplementedError();

  @override
  Future<MlsOutboundMessage> encrypt({
    required String conversationId,
    required String recipientAccount,
    MlsKeyPackage? recipientKeyPackage,
    required List<int> plaintext,
  }) async {
    return MlsOutboundMessage(
      conversationId: conversationId,
      applicationMessage: MlsWireMessage(
        wireBytes: plaintext,
        cipherSuite: '',
        conversationId: conversationId,
        messageKind: MlsMessageKind.application,
      ),
    );
  }

  @override
  Future<List<int>> decrypt(MlsWireMessage message) async =>
      throw UnimplementedError();

  @override
  Future<MlsInboundMessage> processIncoming(MlsWireMessage message) async =>
      throw UnimplementedError();
}

ChatMediaDraft _draft(int byteSize) => ChatMediaDraft(
      kind: ChatMessageKind.video,
      fileName: 'v.mp4',
      contentType: 'video/mp4',
      sourcePath: '/dev/null',
      byteSize: byteSize,
    );

void main() {
  useIsolatedIsar();

  setUp(() => ChatMediaLimits.applyMembershipLevel('spark')); // 5GB 档,门① 放行大文件
  tearDown(() => ChatMediaLimits.applyMembershipLevel(null));

  ChatFlow buildFlow(ChatStore store, {required List<ChatEnvelope> delivered}) {
    return ChatFlow(
      crypto: _FakeMls(),
      store: store,
      deliverer: (envelope, bytes) async {
        delivered.add(envelope);
        return ChatDeliveryResult(
          envelopeId: envelope.envelopeId,
          transportType: ChatTransportType.cloudflare,
          state: ChatMessageDeliveryState.sent,
        );
      },
    );
  }

  test('>100MB → 走中转:描述子入 E2E 载荷,不触 WebRTC', () async {
    final store = ChatStore();
    final flow = buildFlow(store, delivered: []);
    var webrtcCalls = 0;
    var uploaderCalls = 0;

    await flow.sendMedia(
      conversationId: 'dm:a:b',
      senderAccount: 'a',
      recipientAccount: 'b',
      senderDeviceId: 'devA',
      media: _draft(200 * 1024 * 1024),
      sendDeviceAttachment: ({
        required recipientAccount,
        required conversationId,
        required attachmentId,
        required fileName,
        required contentType,
        required sourcePath,
        required byteSize,
      }) async {
        webrtcCalls++;
      },
      uploadRelayMedia: ({
        required conversationId,
        required attachmentId,
        required media,
        int recipientCount = 1,
      }) async {
        uploaderCalls++;
        return const ChatRelayDescriptor(
          relayObjectKey: 'chat-relay/xyz',
          contentKeyB64: 'a2V5',
          chunkSize: 1048576,
          encSize: 200 * 1024 * 1024 + 8192,
        );
      },
    );

    expect(uploaderCalls, 1);
    expect(webrtcCalls, 0); // 中转路径绝不走 WebRTC
    final messages = await store.readMessages('dm:a:b');
    final content = ChatPayloadCodec.decode(messages.single.plaintext ?? '');
    expect(content.isRelayMedia, isTrue);
    expect(content.relayObjectKey, 'chat-relay/xyz');
  });

  test('≤100MB → 走 WebRTC,不触中转', () async {
    final store = ChatStore();
    final flow = buildFlow(store, delivered: []);
    var webrtcCalls = 0;
    var uploaderCalls = 0;

    await flow.sendMedia(
      conversationId: 'dm:a:b',
      senderAccount: 'a',
      recipientAccount: 'b',
      senderDeviceId: 'devA',
      media: _draft(50 * 1024 * 1024),
      sendDeviceAttachment: ({
        required recipientAccount,
        required conversationId,
        required attachmentId,
        required fileName,
        required contentType,
        required sourcePath,
        required byteSize,
      }) async {
        webrtcCalls++;
      },
      uploadRelayMedia: ({
        required conversationId,
        required attachmentId,
        required media,
        int recipientCount = 1,
      }) async {
        uploaderCalls++;
        return const ChatRelayDescriptor(
          relayObjectKey: '',
          contentKeyB64: '',
          chunkSize: 0,
          encSize: 0,
        );
      },
    );

    expect(uploaderCalls, 0);
    expect(webrtcCalls, 1);
    final messages = await store.readMessages('dm:a:b');
    expect(ChatPayloadCodec.decode(messages.single.plaintext ?? '').isRelayMedia,
        isFalse);
  });

  test('>100MB 但中转未配置 → 拒发(绝不降级 WebRTC)', () async {
    final store = ChatStore();
    final flow = buildFlow(store, delivered: []);
    await expectLater(
      flow.sendMedia(
        conversationId: 'dm:a:b',
        senderAccount: 'a',
        recipientAccount: 'b',
        senderDeviceId: 'devA',
        media: _draft(200 * 1024 * 1024),
        sendDeviceAttachment: ({
          required recipientAccount,
          required conversationId,
          required attachmentId,
          required fileName,
          required contentType,
          required sourcePath,
          required byteSize,
        }) async {},
        // uploadRelayMedia 未提供
      ),
      throwsA(isA<StateError>()),
    );
  });
}
