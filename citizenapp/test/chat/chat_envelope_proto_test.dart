import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('ChatEnvelope 只往返 MLS 瞬时投递字段', () {
    final envelope = ChatEnvelope(
      protocolVersion: 1,
      envelopeId: 'env-1',
      conversationId: 'conv-1',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
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
    expect(restored.recipientAccount, 'bob-wallet');
  });

  test('ChatKeyPackage 不包含消费状态', () {
    final keyPackage = ChatKeyPackage(
      protocolVersion: 1,
      ownerAccount: 'bob-wallet',
      deviceId: 'bob-phone',
      devicePublicKeyHex: 'aabbcc',
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
      peerAccount: 'bob-wallet',
      routeDisplayName: 'Bob',
      deviceId: 'bob-phone',
      devicePublicKeyHex: 'aabbcc',
      safetyNumber: '12 34',
      nearbyPeerHint: 'bob-nearby',
      createdAtMillis: Int64(1),
      expiresAtMillis: Int64(2),
    );

    final restored = ChatRoute.fromBuffer(route.writeToBuffer());
    expect(restored.peerAccount, 'bob-wallet');
    expect(restored.nearbyPeerHint, 'bob-nearby');
  });
}
