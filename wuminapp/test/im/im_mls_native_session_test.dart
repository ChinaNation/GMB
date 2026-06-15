import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/im/crypto/im_mls_boundary.dart';
import 'package:wuminapp_mobile/im/crypto/im_mls_native.dart';
import 'package:wuminapp_mobile/im/crypto/im_mls_state_store.dart';

void main() {
  test('native OpenMLS creates, persists, and resumes a two-party session',
      () async {
    final root = await Directory.systemTemp.createTemp('gmb-im-native-mls-');
    addTearDown(() async {
      if (root.existsSync()) {
        await root.delete(recursive: true);
      }
    });

    final aliceStore = ImMlsStateStore(Directory('${root.path}/alice'));
    final bobStore = ImMlsStateStore(Directory('${root.path}/bob'));
    const aliceIdentity = ImMlsDeviceIdentity(
      walletChatAccount: 'alice-wallet',
      deviceId: 'alice-phone',
      devicePublicKeyHex: 'aabbcc',
    );
    const bobIdentity = ImMlsDeviceIdentity(
      walletChatAccount: 'bob-wallet',
      deviceId: 'bob-phone',
      devicePublicKeyHex: 'ddeeff',
    );

    final bobCrypto = NativeImMlsCrypto(
      identity: bobIdentity,
      stateStore: bobStore,
    );
    final bobKeyPackage = await bobCrypto.createKeyPackage(bobIdentity);

    final aliceCrypto = NativeImMlsCrypto(
      identity: aliceIdentity,
      stateStore: aliceStore,
    );
    final firstOutbound = await aliceCrypto.encrypt(
      conversationId: 'conv-alice-bob',
      recipientChatAccount: 'bob-wallet',
      recipientKeyPackage: bobKeyPackage,
      plaintext: utf8.encode('第一条消息'),
    );

    expect(firstOutbound.createdNewSession, isTrue);
    expect(firstOutbound.welcomeMessage, isNotNull);
    expect(
      firstOutbound.applicationMessage.messageKind,
      ImMlsMessageKind.application,
    );

    final bobAfterRestart = NativeImMlsCrypto(
      identity: bobIdentity,
      stateStore: bobStore,
    );
    final welcomeResult = await bobAfterRestart.processIncoming(
      firstOutbound.welcomeMessage!,
    );
    expect(welcomeResult.messageKind, ImMlsMessageKind.welcome);
    expect(welcomeResult.hasPlaintext, isFalse);

    final firstPlaintext = await bobAfterRestart.decrypt(
      firstOutbound.applicationMessage,
    );
    expect(utf8.decode(firstPlaintext), '第一条消息');

    final aliceAfterRestart = NativeImMlsCrypto(
      identity: aliceIdentity,
      stateStore: aliceStore,
    );
    final followUp = await aliceAfterRestart.encrypt(
      conversationId: 'conv-alice-bob',
      recipientChatAccount: 'bob-wallet',
      plaintext: utf8.encode('重启后的第二条消息'),
    );
    expect(followUp.createdNewSession, isFalse);

    final bobSecondRestart = NativeImMlsCrypto(
      identity: bobIdentity,
      stateStore: bobStore,
    );
    final secondPlaintext = await bobSecondRestart.decrypt(
      followUp.applicationMessage,
    );
    expect(utf8.decode(secondPlaintext), '重启后的第二条消息');
  });
}
