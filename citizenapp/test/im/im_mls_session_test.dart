import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/im/crypto/im_mls_session.dart';
import 'package:citizenapp/im/crypto/im_mls_state_store.dart';

void main() {
  test('outbound message yields Welcome before application', () {
    const outbound = ImMlsOutboundMessage(
      conversationId: 'conv-1',
      welcomeMessage: ImMlsWireMessage(
        wireBytes: [0x01],
        cipherSuite: 'MLS_128',
        conversationId: 'conv-1',
        messageKind: ImMlsMessageKind.welcome,
        ratchetTreeBytes: [0x02],
      ),
      applicationMessage: ImMlsWireMessage(
        wireBytes: [0x03],
        cipherSuite: 'MLS_128',
        conversationId: 'conv-1',
        messageKind: ImMlsMessageKind.application,
      ),
    );

    expect(outbound.createdNewSession, isTrue);
    expect(
      outbound.wireMessages.map((message) => message.messageKind).toList(),
      [ImMlsMessageKind.welcome, ImMlsMessageKind.application],
    );
  });

  test('state store persists pending inbound messages', () async {
    final dir = await Directory.systemTemp.createTemp('gmb-im-mls-state-');
    addTearDown(() async {
      if (dir.existsSync()) {
        await dir.delete(recursive: true);
      }
    });
    final store = ImMlsStateStore(dir);

    const pending = ImMlsWireMessage(
      wireBytes: [0xaa, 0xbb],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-pending',
      messageKind: ImMlsMessageKind.application,
    );

    await store.queuePendingInbound(pending);
    final restored = await store.readPendingInbound();

    expect(restored, hasLength(1));
    expect(restored.single.conversationId, 'conv-pending');
    expect(restored.single.wireBytes, [0xaa, 0xbb]);

    await store.clearPendingInbound();
    expect(await store.readPendingInbound(), isEmpty);
  });
}
