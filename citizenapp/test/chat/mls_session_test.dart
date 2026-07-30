import 'dart:typed_data';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/chat/crypto/mls_session.dart';
import 'package:citizenapp/chat/crypto/mls_state_store.dart';

/// MLS 本地状态信封测试密钥（固定 32 字节，仅测试用）。
final Uint8List _testStateKey =
    Uint8List.fromList(List<int>.generate(32, (i) => i));

void main() {
  test('outbound message yields Welcome before application', () {
    const outbound = MlsOutboundMessage(
      conversationId: 'conv-1',
      welcomeMessage: MlsWireMessage(
        wireBytes: [0x01],
        cipherSuite: 'MLS_128',
        conversationId: 'conv-1',
        messageKind: MlsMessageKind.welcome,
        ratchetTreeBytes: [0x02],
      ),
      applicationMessage: MlsWireMessage(
        wireBytes: [0x03],
        cipherSuite: 'MLS_128',
        conversationId: 'conv-1',
        messageKind: MlsMessageKind.application,
      ),
    );

    expect(outbound.createdNewSession, isTrue);
    expect(
      outbound.wireMessages.map((message) => message.messageKind).toList(),
      [MlsMessageKind.welcome, MlsMessageKind.application],
    );
  });

  test('state store persists pending inbound messages', () async {
    final dir = await Directory.systemTemp.createTemp('gmb-im-mls-state-');
    addTearDown(() async {
      if (dir.existsSync()) {
        await dir.delete(recursive: true);
      }
    });
    final store = MlsStateStore(
      dir,
      ownerCidNumber: 'CN220-CTZN2-100000001-2026',
      stateKey: _testStateKey,
    );

    const pending = MlsWireMessage(
      wireBytes: [0xaa, 0xbb],
      cipherSuite: 'MLS_128',
      conversationId: 'conv-pending',
      messageKind: MlsMessageKind.application,
    );

    await store.queuePendingInbound(pending);
    final restored = await store.readPendingInbound();

    expect(restored, hasLength(1));
    expect(restored.single.conversationId, 'conv-pending');
    expect(restored.single.wireBytes, [0xaa, 0xbb]);

    final otherCidStore = MlsStateStore(
      dir,
      ownerCidNumber: 'CN220-CTZN2-999999999-2026',
      stateKey: _testStateKey,
    );
    await expectLater(
      otherCidStore.readPendingInbound(),
      throwsA(anything),
    );

    await store.clearPendingInbound();
    expect(await store.readPendingInbound(), isEmpty);
  });
}
