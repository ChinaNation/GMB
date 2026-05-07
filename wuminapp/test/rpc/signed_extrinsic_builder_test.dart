import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/rpc/signed_extrinsic_builder.dart';

void main() {
  String hexOf(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  group('SignedExtrinsicBuilder', () {
    test('builds signing payload with immortal era and genesis blockHash', () {
      final genesisHash = Uint8List.fromList(List<int>.generate(32, (i) => i));
      final payload = SignedExtrinsicBuilder.buildImmortalSigningPayload(
        callData: Uint8List.fromList([0x02, 0x03]),
        specVersion: 42,
        transactionVersion: 7,
        genesisHash: genesisHash,
        nonce: 9,
      );
      final encodedMap = payload.toEncodedMap(null);

      expect(encodedMap['era'], '00');
      expect(encodedMap['blockHash'], hexOf(genesisHash));
      expect(encodedMap['genesisHash'], hexOf(genesisHash));
      expect(payload.blockNumber, SignedExtrinsicBuilder.immortalBlockNumber);
      expect(payload.eraPeriod, SignedExtrinsicBuilder.immortalEraPeriod);
    });

    test('builds extrinsic payload with immortal era', () {
      final extrinsic = SignedExtrinsicBuilder.buildImmortalExtrinsicPayload(
        callData: Uint8List.fromList([0x16, 0x00]),
        signerPubkey: Uint8List(32),
        signature: Uint8List(64),
        nonce: 3,
      );
      final encodedMap = extrinsic.toEncodedMap(null);

      expect(encodedMap['era'], '00');
      expect(extrinsic.blockNumber, SignedExtrinsicBuilder.immortalBlockNumber);
      expect(extrinsic.eraPeriod, SignedExtrinsicBuilder.immortalEraPeriod);
    });
  });
}
