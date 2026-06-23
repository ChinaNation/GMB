import 'package:flutter_test/flutter_test.dart';
import 'package:fixnum/fixnum.dart';
import 'package:citizenapp/im/proto/im_envelope.pb.dart';

void main() {
  test('ImEnvelope protobuf round-trips MLS wire bytes', () {
    final envelope = ImEnvelope(
      protocolVersion: 1,
      envelopeId: 'env-1',
      conversationId: 'conv-1',
      senderChatAccount: 'alice-wallet',
      recipientChatAccount: 'bob-wallet',
      senderDeviceId: 'alice-phone',
      mlsWireMessage: [0xaa, 0xbb, 0xcc],
      encryptedMetadata: [0x01, 0x02],
      attachmentManifestHash: '0xhash',
      chunkRefs: ['chunk-1'],
      createdAtMillis: Int64(1),
      ttlMillis: Int64(60000),
      ackPolicy: 'account_ack',
      mlsMessageKind: ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_APPLICATION,
      ratchetTree: [0x0a, 0x0b],
    );

    final restored = ImEnvelope.fromBuffer(envelope.writeToBuffer());
    expect(restored.envelopeId, 'env-1');
    expect(restored.mlsWireMessage, [0xaa, 0xbb, 0xcc]);
    expect(
      restored.mlsMessageKind,
      ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_APPLICATION,
    );
    expect(restored.ratchetTree, [0x0a, 0x0b]);
    expect(restored.recipientChatAccount, 'bob-wallet');
  });

  test('ImKeyPackage protobuf round-trips key package bytes', () {
    final keyPackage = ImKeyPackage(
      protocolVersion: 1,
      ownerWalletAccount: 'bob-wallet',
      deviceId: 'bob-phone',
      devicePublicKeyHex: 'aabbcc',
      keyPackageId: 'kp-1',
      keyPackage: [0xde, 0xad, 0xbe, 0xef],
      cipherSuite: 'MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519',
      createdAtMillis: Int64(1),
      expiresAtMillis: Int64(2),
      consumedAtMillis: Int64(0),
    );

    final restored = ImKeyPackage.fromBuffer(keyPackage.writeToBuffer());
    expect(restored.keyPackageId, 'kp-1');
    expect(restored.devicePublicKeyHex, 'aabbcc');
    expect(restored.keyPackage, [0xde, 0xad, 0xbe, 0xef]);
  });

  test('ImRouteRecord protobuf keeps wallet address and IPv6 endpoint', () {
    final route = ImRouteRecord(
      protocolVersion: 1,
      walletChatAccount: 'bob-wallet',
      routeDisplayName: 'Bob',
      imDeviceId: 'bob-phone',
      imDevicePubkeyHex: 'aabbcc',
      safetyNumber: '12 34',
      nodeEndpoints: [
        ImNodeEndpoint(
          peerId: 'peer-bob',
          multiaddr: '/ip6/::1/tcp/30334/p2p/peer-bob',
          kind: 'ip6',
        ),
      ],
      createdAtMillis: Int64(1),
      expiresAtMillis: Int64(2),
    );

    final restored = ImRouteRecord.fromBuffer(route.writeToBuffer());
    expect(restored.walletChatAccount, 'bob-wallet');
    expect(restored.nodeEndpoints.single.kind, 'ip6');
  });
}
