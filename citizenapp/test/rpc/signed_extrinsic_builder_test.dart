import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/signed_extrinsic_builder.dart';

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
        signerPublicKey: Uint8List(32),
        signature: Uint8List(64),
        nonce: 3,
      );
      final encodedMap = extrinsic.toEncodedMap(null);

      expect(encodedMap['era'], '00');
      expect(extrinsic.blockNumber, SignedExtrinsicBuilder.immortalBlockNumber);
      expect(extrinsic.eraPeriod, SignedExtrinsicBuilder.immortalEraPeriod);
    });
  });

  group('finalized 交易结果定位', () {
    test('按 txHash 定位 finalized 区块中的 extrinsic index', () {
      final first = Uint8List.fromList([1, 2, 3]);
      final target = Uint8List.fromList([4, 5, 6]);
      final targetHash = Hasher.blake2b256.hash(target);

      final index = ChainRpc.findExtrinsicIndexInHexListSync(
        ['0x${hexOf(first)}', '0x${hexOf(target)}'],
        txHashHex: '0x${hexOf(targetHash)}',
      );

      expect(index, 1);
    });

    test('只读取目标 extrinsic index 的失败事件', () {
      // 两条 ApplyExtrinsic 失败事件：index=0 与 index=1。
      final events = Uint8List.fromList([
        0x08, // Compact<Vec> 长度 2
        0x00, 0, 0, 0, 0, // ApplyExtrinsic(0)
        0x00, 0x01, // System.ExtrinsicFailed
        0x03, 0x02, 0x05, 0, 0, 0, // Module(pallet=2,error=5)
        0x00, // topics=[]
        0x00, 1, 0, 0, 0, // ApplyExtrinsic(1)
        0x00, 0x01, // System.ExtrinsicFailed
        0x03, 0x04, 0x07, 0, 0, 0, // Module(pallet=4,error=7)
        0x00, // topics=[]
      ]);

      final failure = ChainRpc().findExtrinsicFailureInEvents(
        events,
        extrinsicIndex: 1,
      );

      expect(failure, isNotNull);
      expect(failure!.moduleIndex, 4);
      expect(failure.errorIndex, 7);
    });
  });
}
