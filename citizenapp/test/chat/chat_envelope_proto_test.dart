import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('ChatEnvelope 只往返 MLS 瞬时投递字段', () {
    final envelope = ChatEnvelope(
      protocolVersion: 1,
      envelopeId: 'env-1',
      conversationId: 'conv-1',
      senderAccountId:
          '0x1111111111111111111111111111111111111111111111111111111111111111',
      recipientAccountId:
          '0x2222222222222222222222222222222222222222222222222222222222222222',
      senderDeviceId: 'alice-phone',
      mlsWireMessage: [0xaa, 0xbb, 0xcc],
      encryptedMetadata: [0x01, 0x02],
      createdAtMillis: Int64(1),
      ttlMillis: Int64(60000),
      mlsMessageKind: MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION,
      ratchetTree: [0x0a, 0x0b],
    );

    final restored = ChatEnvelope.fromBuffer(envelope.writeToBuffer());
    expect(restored.envelopeId, 'env-1');
    expect(restored.mlsWireMessage, [0xaa, 0xbb, 0xcc]);
    expect(restored.ratchetTree, [0x0a, 0x0b]);
    expect(restored.recipientAccountId,
        '0x2222222222222222222222222222222222222222222222222222222222222222');
  });

  test('ChatKeyPackage 不包含消费状态', () {
    final keyPackage = ChatKeyPackage(
      protocolVersion: 1,
      accountId:
          '0x2222222222222222222222222222222222222222222222222222222222222222',
      deviceId: 'bob-phone',
      devicePublicKey: 'aabbcc',
      keyPackageId: 'kp-1',
      keyPackage: [0xde, 0xad, 0xbe, 0xef],
      cipherSuite: 'MLS_128',
      createdAtMillis: Int64(1),
      expiresAtMillis: Int64(2),
    );

    final restored = ChatKeyPackage.fromBuffer(keyPackage.writeToBuffer());
    expect(restored.keyPackageId, 'kp-1');
    expect(restored.keyPackage, [0xde, 0xad, 0xbe, 0xef]);
  });

  test('ChatRoute 只保存设备和近场路由', () {
    final route = ChatRoute(
      protocolVersion: 1,
      peerAccountId:
          '0x2222222222222222222222222222222222222222222222222222222222222222',
      routeDisplayName: 'Bob',
      deviceId: 'bob-phone',
      devicePublicKey: 'aabbcc',
      safetyNumber: '12 34',
      nearbyPeerHint: 'bob-nearby',
      createdAtMillis: Int64(1),
      expiresAtMillis: Int64(2),
    );

    final restored = ChatRoute.fromBuffer(route.writeToBuffer());
    expect(restored.peerAccountId,
        '0x2222222222222222222222222222222222222222222222222222222222222222');
    expect(restored.nearbyPeerHint, 'bob-nearby');
  });
}
