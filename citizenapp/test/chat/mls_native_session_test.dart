import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/crypto/mls_boundary.dart';
import 'package:citizenapp/chat/crypto/mls_native.dart';
import 'package:citizenapp/chat/crypto/mls_state_store.dart';
import 'package:citizenapp/chat/chat_flow.dart';
import 'package:citizenapp/chat/chat_models.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/chat/transport/chat_transport.dart';
import 'package:citizenapp/isar/app_isar.dart';

import '../support/smoldot_native_probe.dart';

void main() {
  final skip = smoldotNativeSkipReason();

  setUp(() async {
    await WalletIsar.instance.resetForTest();
  });

  tearDown(() async {
    await WalletIsar.instance.resetForTest();
  });

  test('native OpenMLS creates, persists, and resumes a two-party session',
      () async {
    final root = await Directory.systemTemp.createTemp('gmb-im-native-mls-');
    addTearDown(() async {
      if (root.existsSync()) {
        await root.delete(recursive: true);
      }
    });

    final aliceStore = MlsStateStore(Directory('${root.path}/alice'));
    final bobStore = MlsStateStore(Directory('${root.path}/bob'));
    const aliceIdentity = ChatDevice(
      ownerAccount: 'alice-wallet',
      deviceId: 'alice-phone',
      devicePublicKeyHex: 'aabbcc',
    );
    const bobIdentity = ChatDevice(
      ownerAccount: 'bob-wallet',
      deviceId: 'bob-phone',
      devicePublicKeyHex: 'ddeeff',
    );

    final bobCrypto = NativeMlsCrypto(
      identity: bobIdentity,
      stateStore: bobStore,
    );
    final bobKeyPackage = await bobCrypto.createKeyPackage(bobIdentity);

    final aliceCrypto = NativeMlsCrypto(
      identity: aliceIdentity,
      stateStore: aliceStore,
    );
    final firstOutbound = await aliceCrypto.encrypt(
      conversationId: 'conv-alice-bob',
      recipientAccount: 'bob-wallet',
      recipientKeyPackage: bobKeyPackage,
      plaintext: utf8.encode('第一条消息'),
    );

    expect(firstOutbound.createdNewSession, isTrue);
    expect(firstOutbound.welcomeMessage, isNotNull);
    expect(
      firstOutbound.applicationMessage.messageKind,
      MlsMessageKind.application,
    );

    final bobAfterRestart = NativeMlsCrypto(
      identity: bobIdentity,
      stateStore: bobStore,
    );
    final welcomeResult = await bobAfterRestart.processIncoming(
      firstOutbound.welcomeMessage!,
    );
    expect(welcomeResult.messageKind, MlsMessageKind.welcome);
    expect(welcomeResult.hasPlaintext, isFalse);

    final firstPlaintext = await bobAfterRestart.decrypt(
      firstOutbound.applicationMessage,
    );
    expect(utf8.decode(firstPlaintext), '第一条消息');

    final aliceAfterRestart = NativeMlsCrypto(
      identity: aliceIdentity,
      stateStore: aliceStore,
    );
    final followUp = await aliceAfterRestart.encrypt(
      conversationId: 'conv-alice-bob',
      recipientAccount: 'bob-wallet',
      plaintext: utf8.encode('重启后的第二条消息'),
    );
    expect(followUp.createdNewSession, isFalse);

    final bobSecondRestart = NativeMlsCrypto(
      identity: bobIdentity,
      stateStore: bobStore,
    );
    final secondPlaintext = await bobSecondRestart.decrypt(
      followUp.applicationMessage,
    );
    expect(utf8.decode(secondPlaintext), '重启后的第二条消息');
    // libsmoldot 不可用(纯 Dart CI 无宿主 .so)则跳过;真机/集成构建照跑。
  }, skip: skip);

  test('native OpenMLS delivers through mailbox pull, ack, and Isar save',
      () async {
    final root = await Directory.systemTemp.createTemp('gmb-im-native-e2e-');
    addTearDown(() async {
      if (root.existsSync()) {
        await root.delete(recursive: true);
      }
    });

    final aliceMlsStore = MlsStateStore(Directory('${root.path}/alice'));
    final bobMlsStore = MlsStateStore(Directory('${root.path}/bob'));
    const aliceIdentity = ChatDevice(
      ownerAccount: 'alice-wallet',
      deviceId: 'alice-phone',
      devicePublicKeyHex: 'aabbcc',
    );
    const bobIdentity = ChatDevice(
      ownerAccount: 'bob-wallet',
      deviceId: 'bob-phone',
      devicePublicKeyHex: 'ddeeff',
    );

    final bobCryptoForKeyPackage = NativeMlsCrypto(
      identity: bobIdentity,
      stateStore: bobMlsStore,
    );
    final bobKeyPackage =
        await bobCryptoForKeyPackage.createKeyPackage(bobIdentity);
    final mailbox = _MemoryMailbox();
    final aliceLocalStore = ChatStore();
    final aliceFlow = ChatFlow(
      crypto: NativeMlsCrypto(
        identity: aliceIdentity,
        stateStore: aliceMlsStore,
      ),
      store: aliceLocalStore,
      deliverer: mailbox.submit,
    );

    final sendResults = await aliceFlow.sendText(
      conversationId: 'conv-internet-alice-bob',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      recipientKeyPackage: bobKeyPackage,
      text: '互联网私聊真实闭环',
    );

    expect(sendResults, hasLength(2));
    expect(
        sendResults.map((item) => item.state),
        everyElement(
          ChatMessageDeliveryState.sent,
        ));
    expect(mailbox.pendingCount('bob-wallet'), 2);
    final aliceMessages =
        await aliceLocalStore.readMessages('conv-internet-alice-bob');
    expect(aliceMessages.single.direction, 'outgoing');
    expect(aliceMessages.single.plaintext, '互联网私聊真实闭环');

    // A/B 两台手机各有自己的 Isar。本测试用 reset 模拟 Bob 端独立本地库，
    // mailbox 和 MLS stateStore 保留，验证远程密文拉取后的真实落库路径。
    await WalletIsar.instance.resetForTest();
    final bobLocalStore = ChatStore();
    final bobFlow = ChatFlow(
      crypto: NativeMlsCrypto(
        identity: bobIdentity,
        stateStore: bobMlsStore,
      ),
      store: bobLocalStore,
      deliverer: (_, __) {
        throw StateError('接收端处理 pending 时不应重新投递密文');
      },
    );

    final processed = await bobFlow.fetchAndProcessPending(
      fetchPending: () => mailbox.fetchPending('bob-wallet'),
      ackEnvelope: mailbox.ack,
    );

    expect(processed, 2);
    expect(mailbox.pendingCount('bob-wallet'), 0);
    expect(mailbox.ackedEnvelopeIds, hasLength(2));
    final bobMessages =
        await bobLocalStore.readMessages('conv-internet-alice-bob');
    expect(bobMessages.single.direction, 'incoming');
    expect(bobMessages.single.senderAccount, 'alice-wallet');
    expect(bobMessages.single.recipientAccount, 'bob-wallet');
    expect(bobMessages.single.plaintext, '互联网私聊真实闭环');
    final bobConversations = await bobLocalStore.readConversationPreviews(
        ownerAccount: 'bob-wallet');
    expect(bobConversations.single.lastMessage, '互联网私聊真实闭环');
    expect(bobConversations.single.unreadCount, 1);
  }, skip: skip);
}

class _MemoryMailbox {
  final List<_MemoryMailboxRow> _rows = <_MemoryMailboxRow>[];
  final Set<String> ackedEnvelopeIds = <String>{};

  Future<ChatDeliveryResult> submit(
      ChatEnvelope envelope, List<int> envelopeBytes) async {
    _rows.add(
      _MemoryMailboxRow(
        envelopeId: envelope.envelopeId,
        recipientAccount: envelope.recipientAccount,
        envelopeBytes: envelopeBytes,
      ),
    );
    return ChatDeliveryResult(
      envelopeId: envelope.envelopeId,
      transportType: ChatTransportType.cloudflare,
      state: ChatMessageDeliveryState.sent,
    );
  }

  Future<List<ChatPendingEncryptedEnvelope>> fetchPending(
    String ownerAccount,
  ) async {
    return _rows
        .where(
          (row) => row.recipientAccount == ownerAccount && !row.acked,
        )
        .map(
          (row) => ChatPendingEncryptedEnvelope(
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

  int pendingCount(String ownerAccount) {
    return _rows
        .where(
          (row) => row.recipientAccount == ownerAccount && !row.acked,
        )
        .length;
  }
}

class _MemoryMailboxRow {
  _MemoryMailboxRow({
    required this.envelopeId,
    required this.recipientAccount,
    required this.envelopeBytes,
  });

  final String envelopeId;
  final String recipientAccount;
  final List<int> envelopeBytes;
  bool acked = false;
}
