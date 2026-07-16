import 'dart:convert';
import 'dart:io';

import 'package:citizenapp/chat/chat_flow.dart';
import 'package:citizenapp/chat/chat_media_limits.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/chat_payload.dart';
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

  test('媒体经设备通道流式发送、控制不含云端引用,且路径/自存以同一 attachmentId 关联', () async {
    final root = await Directory.systemTemp.createTemp('gmb-chat-send-');
    addTearDown(() => root.delete(recursive: true));
    final source = File('${root.path}/photo.jpg');
    await source.writeAsBytes(const [1, 2, 3, 4], flush: true);

    String? sentSourcePath;
    int? sentByteSize;
    String? sentAttachmentId;
    String? sentFileName;
    String? savedSourcePath;
    String? savedAttachmentId;
    int? savedByteSize;
    final flow = ChatFlow(
      crypto: _FakeMlsCrypto(),
      store: ChatStore(),
      deliverer: (envelope, _) async => ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.sent,
      ),
    );

    await flow.sendMedia(
      conversationId: 'conv-attachment',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      recipientKeyPackage: _dummyKeyPackage(),
      media: ChatMediaDraft(
        kind: ChatMessageKind.image,
        fileName: 'photo.jpg',
        contentType: 'image/jpeg',
        sourcePath: source.path,
        byteSize: 4,
      ),
      sendDeviceAttachment: ({
        required recipientAccount,
        required conversationId,
        required attachmentId,
        required fileName,
        required contentType,
        required sourcePath,
        required byteSize,
      }) async {
        sentSourcePath = sourcePath;
        sentByteSize = byteSize;
        sentAttachmentId = attachmentId;
        sentFileName = fileName;
      },
      saveLocalAttachment: ({
        required conversationId,
        required attachmentId,
        required fileName,
        required contentType,
        required sourcePath,
        required byteSize,
      }) async {
        savedSourcePath = sourcePath;
        savedAttachmentId = attachmentId;
        savedByteSize = byteSize;
      },
    );

    expect(sentSourcePath, source.path);
    expect(sentByteSize, 4);
    final message =
        (await ChatStore().readMessages('conv-attachment')).single.plaintext!;
    // 控制载荷是端到端明文,只带元数据,绝无任何云端对象引用。
    expect(message, contains('gmb.chat.msg'));
    expect(message, contains('"kind":"image"'));
    expect(message, contains('"byte_size":4'));
    expect(message, isNot(contains('object_key')));
    expect(message, isNot(contains('manifest')));
    // WebRTC 字节与 MLS 控制消息必须以同一 attachmentId 关联,否则接收端按控制
    // 里的 id 找不到设备通道存下的字节。
    final controlAttachmentId = ChatPayloadCodec.decode(message).attachmentId;
    expect(controlAttachmentId, isNotNull);
    expect(sentAttachmentId, controlAttachmentId);
    expect(sentFileName, 'photo.jpg');
    // 发送方本机自存副本用同一 id/源/大小,发送方才能在会话里看到自己发出的媒体。
    expect(savedAttachmentId, controlAttachmentId);
    expect(savedSourcePath, source.path);
    expect(savedByteSize, 4);
  });

  test('门①:sendMedia 超限文件抛 ChatMediaTooLargeException 且不发字节', () async {
    var deviceSendCalls = 0;
    final flow = ChatFlow(
      crypto: _FakeMlsCrypto(),
      store: ChatStore(),
      deliverer: (envelope, _) async => ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.sent,
      ),
    );

    await expectLater(
      flow.sendMedia(
        conversationId: 'conv-oversize',
        senderAccount: 'alice-wallet',
        recipientAccount: 'bob-wallet',
        senderDeviceId: 'alice-phone',
        recipientKeyPackage: _dummyKeyPackage(),
        // byteSize 超出图片 100MB 上限;门控看 byteSize 字段,发前即拦,不触碰源文件。
        media: const ChatMediaDraft(
          kind: ChatMessageKind.image,
          fileName: 'huge.jpg',
          contentType: 'image/jpeg',
          sourcePath: '/nonexistent',
          byteSize: ChatMediaLimits.imageMaxBytes + 1,
        ),
        sendDeviceAttachment: ({
          required recipientAccount,
          required conversationId,
          required attachmentId,
          required fileName,
          required contentType,
          required sourcePath,
          required byteSize,
        }) async {
          deviceSendCalls += 1;
        },
      ),
      throwsA(isA<ChatMediaTooLargeException>()),
    );
    expect(deviceSendCalls, 0);
  });

  test('sendMedia 控制消息加密失败时绝不先发 WebRTC 字节(零泄漏顺序)', () async {
    final root = await Directory.systemTemp.createTemp('gmb-chat-leak-');
    addTearDown(() => root.delete(recursive: true));
    final source = File('${root.path}/photo.jpg');
    await source.writeAsBytes(const [9, 9, 9], flush: true);
    var deviceSendCalls = 0;
    final flow = ChatFlow(
      crypto: _FakeMlsCrypto(),
      store: ChatStore(),
      deliverer: (envelope, _) async => ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.sent,
      ),
    );

    // 全新会话且不提供 KeyPackage:_FakeMlsCrypto.encrypt 抛错,模拟首次会话缺
    // KeyPackage。字节必须在加密成功之后才发,否则会泄漏一条永远送不达的媒体。
    await expectLater(
      flow.sendMedia(
        conversationId: 'conv-no-keypackage',
        senderAccount: 'alice-wallet',
        recipientAccount: 'bob-wallet',
        senderDeviceId: 'alice-phone',
        media: ChatMediaDraft(
          kind: ChatMessageKind.image,
          fileName: 'photo.jpg',
          contentType: 'image/jpeg',
          sourcePath: source.path,
          byteSize: 3,
        ),
        sendDeviceAttachment: ({
          required recipientAccount,
          required conversationId,
          required attachmentId,
          required fileName,
          required contentType,
          required sourcePath,
          required byteSize,
        }) async {
          deviceSendCalls += 1;
        },
      ),
      throwsA(isA<StateError>()),
    );
    expect(deviceSendCalls, 0);
  });

  test('sendMedia 在线送达:登记待投递后随即标记已送达(同一 attachmentId,净零)', () async {
    final root = await Directory.systemTemp.createTemp('gmb-online-');
    addTearDown(() => root.delete(recursive: true));
    final source = File('${root.path}/photo.jpg');
    await source.writeAsBytes(const [1, 2, 3, 4], flush: true);
    final recorded = <String>[];
    final delivered = <String>[];
    final flow = ChatFlow(
      crypto: _FakeMlsCrypto(),
      store: ChatStore(),
      deliverer: (envelope, _) async => ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.sent,
      ),
    );

    await flow.sendMedia(
      conversationId: 'conv-online',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      recipientKeyPackage: _dummyKeyPackage(),
      media: ChatMediaDraft(
        kind: ChatMessageKind.image,
        fileName: 'photo.jpg',
        contentType: 'image/jpeg',
        sourcePath: source.path,
        byteSize: 4,
      ),
      // sendDeviceAttachment 成功(对方在线)。
      sendDeviceAttachment: ({
        required recipientAccount,
        required conversationId,
        required attachmentId,
        required fileName,
        required contentType,
        required sourcePath,
        required byteSize,
      }) async {},
      recordPendingMedia: (id) async => recorded.add(id),
      onDeviceDelivered: (id) async => delivered.add(id),
    );

    // 先登记待投递、字节送达后随即标记已送达:同一 attachmentId,净零残留。
    expect(recorded, hasLength(1));
    expect(delivered, hasLength(1));
    expect(recorded.single, delivered.single);
  });

  test('sendMedia 对离线对端:控制消息仍成立、登记待投递、不抛错', () async {
    final root = await Directory.systemTemp.createTemp('gmb-offline-');
    addTearDown(() => root.delete(recursive: true));
    final source = File('${root.path}/photo.jpg');
    await source.writeAsBytes(const [1, 2, 3, 4], flush: true);
    final recorded = <String>[];
    final delivered = <String>[];
    final flow = ChatFlow(
      crypto: _FakeMlsCrypto(),
      store: ChatStore(),
      deliverer: (envelope, _) async => ChatDeliveryResult(
        envelopeId: envelope.envelopeId,
        transportType: ChatTransportType.cloudflare,
        state: ChatMessageDeliveryState.queued,
      ),
    );

    // sendDeviceAttachment 抛错模拟对方离线(WebRTC 连不上);sendMedia 必须吞掉
    // 该异常:控制消息已成立,字节留待上线补发。
    final results = await flow.sendMedia(
      conversationId: 'conv-offline',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      recipientKeyPackage: _dummyKeyPackage(),
      media: ChatMediaDraft(
        kind: ChatMessageKind.image,
        fileName: 'photo.jpg',
        contentType: 'image/jpeg',
        sourcePath: source.path,
        byteSize: 4,
      ),
      sendDeviceAttachment: ({
        required recipientAccount,
        required conversationId,
        required attachmentId,
        required fileName,
        required contentType,
        required sourcePath,
        required byteSize,
      }) async {
        throw const SocketException('offline');
      },
      recordPendingMedia: (id) async => recorded.add(id),
      onDeviceDelivered: (id) async => delivered.add(id),
    );

    // 控制消息仍落库成立,sendMedia 未抛错。
    expect(results, isNotEmpty);
    final message = (await ChatStore().readMessages('conv-offline')).single;
    expect(message.messageKind, ChatMessageKind.image);
    // 登记了待投递,但未标记已送达(离线,字节没发出去)。
    expect(recorded, hasLength(1));
    expect(delivered, isEmpty);
  });

  test('downloadAttachment 拒绝非媒体控制消息', () async {
    final root = await Directory.systemTemp.createTemp('gmb-chat-neg-');
    addTearDown(() => root.delete(recursive: true));
    for (final control in [
      ChatPayloadCodec.encode(ChatContent.text('hi')),
      ChatPayloadCodec.encode(
        ChatContent.sticker(packId: 'fluent3d', stickerId: 'grinning_face'),
      ),
    ]) {
      await expectLater(
        ChatFlow.downloadAttachment(
          conversationId: 'c',
          controlPlaintext: control,
          cacheDirectory: root,
        ),
        throwsA(isA<FormatException>()),
      );
    }
  });

  test('downloadAttachment 在字节未到达或截断时报错,不返回半成品', () async {
    final root = await Directory.systemTemp.createTemp('gmb-chat-partial-');
    addTearDown(() => root.delete(recursive: true));
    final control = ChatPayloadCodec.encode(
      ChatContent.media(
        kind: ChatMessageKind.file,
        attachmentId: 'att-x',
        fileName: 'note.txt',
        mime: 'text/plain',
        byteSize: 10,
      ),
    );

    // 缓存缺失 → 未完成传输。
    await expectLater(
      ChatFlow.downloadAttachment(
        conversationId: 'conv-x',
        controlPlaintext: control,
        cacheDirectory: root,
      ),
      throwsA(isA<StateError>()),
    );

    // 缓存只有 3 字节,控制声明 10 字节 → 视为截断/损坏,拒绝返回。
    final partial = File('${root.path}/partial.txt');
    await partial.writeAsBytes(const [1, 2, 3], flush: true);
    await ChatFlow.importAttachmentFileToCache(
      conversationId: 'conv-x',
      attachmentId: 'att-x',
      fileName: 'note.txt',
      contentType: 'text/plain',
      sourcePath: partial.path,
      byteSize: 3,
      moveSource: false,
      cacheDirectory: root,
    );
    await expectLater(
      ChatFlow.downloadAttachment(
        conversationId: 'conv-x',
        controlPlaintext: control,
        cacheDirectory: root,
      ),
      throwsA(isA<StateError>()),
    );
  });

  test('附件下载只读取设备本地缓存,返回路径(不载入整块字节)', () async {
    final root = await Directory.systemTemp.createTemp('gmb-chat-device-');
    addTearDown(() => root.delete(recursive: true));
    final src = File('${root.path}/src-note.txt');
    await src.writeAsBytes(utf8.encode('local only'), flush: true);
    final saved = await ChatFlow.importAttachmentFileToCache(
      conversationId: 'conv-cache',
      attachmentId: 'attachment-1',
      fileName: 'note.txt',
      contentType: 'text/plain',
      sourcePath: src.path,
      byteSize: 10,
      moveSource: false,
      cacheDirectory: root,
    );
    final control = ChatPayloadCodec.encode(
      ChatContent.media(
        kind: ChatMessageKind.file,
        attachmentId: 'attachment-1',
        fileName: 'note.txt',
        mime: 'text/plain',
        byteSize: 10,
      ),
    );

    final loaded = await ChatFlow.downloadAttachment(
      conversationId: 'conv-cache',
      controlPlaintext: control,
      cacheDirectory: root,
    );

    expect(loaded.filePath, saved.filePath);
    expect(await File(loaded.filePath).readAsString(), 'local only');
  });

  test('importAttachmentFileToCache moveSource 把临时文件移入缓存并删源', () async {
    final root = await Directory.systemTemp.createTemp('gmb-chat-move-');
    addTearDown(() => root.delete(recursive: true));
    final temp = File('${root.path}/incoming.part');
    await temp.writeAsBytes(utf8.encode('moved bytes'), flush: true);

    final saved = await ChatFlow.importAttachmentFileToCache(
      conversationId: 'conv-move',
      attachmentId: 'att-move',
      fileName: 'clip.bin',
      contentType: 'application/octet-stream',
      sourcePath: temp.path,
      byteSize: 11,
      moveSource: true,
      cacheDirectory: root,
    );

    expect(await temp.exists(), isFalse); // 源(临时)已移走,不留残余
    expect(await File(saved.filePath).readAsString(), 'moved bytes');
  });

  test('门③:接收落盘二次门控——超限删临时不入缓存,达标则移入', () async {
    final root = await Directory.systemTemp.createTemp('gmb-chat-gate3-');
    addTearDown(() => root.delete(recursive: true));

    // 超限:byteSize 声明超图片 100MB 上限 → 删临时,返回 null,缓存为空。
    final big = File('${root.path}/big.part');
    await big.writeAsBytes(const [1, 2, 3], flush: true);
    final rejected = await ChatFlow.acceptReceivedMediaToCache(
      conversationId: 'conv-g3',
      attachmentId: 'att-big',
      fileName: 'p.jpg',
      contentType: 'image/jpeg',
      tempFilePath: big.path,
      byteSize: ChatMediaLimits.imageMaxBytes + 1,
      cacheDirectory: root,
    );
    expect(rejected, isNull);
    expect(await big.exists(), isFalse);

    // 达标:临时文件移入缓存并可读回。
    final ok = File('${root.path}/ok.part');
    await ok.writeAsBytes(utf8.encode('ok'), flush: true);
    final accepted = await ChatFlow.acceptReceivedMediaToCache(
      conversationId: 'conv-g3',
      attachmentId: 'att-ok',
      fileName: 'ok.txt',
      contentType: 'text/plain',
      tempFilePath: ok.path,
      byteSize: 2,
      cacheDirectory: root,
    );
    expect(accepted, isNotNull);
    expect(await ok.exists(), isFalse);
    expect(await File(accepted!.filePath).readAsString(), 'ok');
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
