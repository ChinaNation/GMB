import 'package:flutter_test/flutter_test.dart';
import 'package:fixnum/fixnum.dart';
import 'package:citizenapp/chat/proto/chat_envelope.pb.dart';

void main() {
  test('ChatEnvelope protobuf round-trips MLS wire bytes', () {
    final envelope = ChatEnvelope(
      protocolVersion: 1,
      envelopeId: 'env-1',
      conversationId: 'conv-1',
      senderAccount: 'alice-wallet',
      recipientAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      mlsWireMessage: [0xaa, 0xbb, 0xcc],
      encryptedMetadata: [0x01, 0x02],
      attachmentManifestHash: '0xhash',
      chunkRefs: ['chunk-1'],
      createdAtMillis: Int64(1),
      ttlMillis: Int64(60000),
      ackPolicy: 'account_ack',
      mlsMessageKind: MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION,
      ratchetTree: [0x0a, 0x0b],
    );

    final restored = ChatEnvelope.fromBuffer(envelope.writeToBuffer());
    expect(restored.envelopeId, 'env-1');
    expect(restored.mlsWireMessage, [0xaa, 0xbb, 0xcc]);
    expect(
      restored.mlsMessageKind,
      MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION,
    );
    expect(restored.ratchetTree, [0x0a, 0x0b]);
    expect(restored.recipientAccount, 'bob-wallet');
  });

  test('ChatKeyPackage protobuf round-trips key package bytes', () {
    final keyPackage = ChatKeyPackage(
      protocolVersion: 1,
      ownerAccount: 'bob-wallet',
      deviceId: 'bob-phone',
      devicePublicKeyHex: 'aabbcc',
      keyPackageId: 'kp-1',
      keyPackage: [0xde, 0xad, 0xbe, 0xef],
      cipherSuite: 'MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519',
      createdAtMillis: Int64(1),
      expiresAtMillis: Int64(2),
      consumedAtMillis: Int64(0),
    );

    final restored = ChatKeyPackage.fromBuffer(keyPackage.writeToBuffer());
    expect(restored.keyPackageId, 'kp-1');
    expect(restored.devicePublicKeyHex, 'aabbcc');
    expect(restored.keyPackage, [0xde, 0xad, 0xbe, 0xef]);
  });

  test('ChatRoute protobuf keeps wallet address and mailbox hints', () {
    final route = ChatRoute(
      protocolVersion: 1,
      peerAccount: 'bob-wallet',
      routeDisplayName: 'Bob',
      deviceId: 'bob-phone',
      devicePublicKeyHex: 'aabbcc',
      safetyNumber: '12 34',
      cloudflareMailboxId: 'bob-wallet',
      nearbyPeerHint: 'bob-nearby',
      createdAtMillis: Int64(1),
      expiresAtMillis: Int64(2),
    );

    final restored = ChatRoute.fromBuffer(route.writeToBuffer());
    expect(restored.peerAccount, 'bob-wallet');
    expect(restored.cloudflareMailboxId, 'bob-wallet');
    expect(restored.nearbyPeerHint, 'bob-nearby');
  });
}
