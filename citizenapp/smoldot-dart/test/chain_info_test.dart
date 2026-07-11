import 'dart:io';
import 'package:test/test.dart';
import 'package:smoldot/smoldot.dart';

void main() {
  group('LightClientStatusSnapshot codec', () {
    Map<String, dynamic> snapshotJson(String syncMode) => {
      'peerCount': 5,
      'isSyncing': syncMode != 'regular',
      'syncMode': syncMode,
      'bestBlockNumber': 50000,
      'bestBlockHash': '0x${List.filled(32, '11').join()}',
      'finalizedBlockNumber': 49998,
      'finalizedBlockHash': '0x${List.filled(32, '22').join()}',
      'startupFinalizedBlockNumber': 0,
      'highestPeerFinalizedBlockNumber': 49998,
      'warpFinalizedBlockNumber': 49998,
      'warpRequestCount': 1,
      'warpFragmentCount': 2,
    };

    test('解析 regular 与两种 warp 阶段并保留计数', () {
      for (final entry in {
        'regular': LightClientSyncMode.regular,
        'warpFragments': LightClientSyncMode.warpFragments,
        'warpChainInformation': LightClientSyncMode.warpChainInformation,
      }.entries) {
        final snapshot = LightClientStatusSnapshot.fromJson(
          snapshotJson(entry.key),
        );
        expect(snapshot.syncMode, entry.value);
        expect(snapshot.startupFinalizedBlockNumber, 0);
        expect(snapshot.highestPeerFinalizedBlockNumber, 49998);
        expect(snapshot.warpFinalizedBlockNumber, 49998);
        expect(snapshot.warpRequestCount, 1);
        expect(snapshot.warpFragmentCount, 2);
        expect(snapshot.toJson()['syncMode'], entry.key);
      }
    });

    test('未知同步模式不得伪装成 regular 或已完成', () {
      expect(
        () => LightClientStatusSnapshot.fromJson(snapshotJson('unknown')),
        throwsFormatException,
      );
    });

    test('runtime 已近头但仍处于 warp 时不得映射为 synced', () {
      final contradictory = snapshotJson('warpFragments')
        ..['isSyncing'] = false;
      final warp = LightClientStatusSnapshot.fromJson(contradictory);
      expect(warp.isUsable, isFalse);
      expect(warp.chainStatus, ChainStatus.syncing);

      final regularJson = snapshotJson('regular')..['isSyncing'] = false;
      final regular = LightClientStatusSnapshot.fromJson(regularJson);
      expect(regular.isUsable, isTrue);
      expect(regular.chainStatus, ChainStatus.synced);
    });
  });

  group('Chain Info Tests', () {
    late SmoldotClient client;
    late Chain chain;

    setUpAll(() async {
      client = SmoldotClient(config: SmoldotConfig(maxLogLevel: 3));
      await client.initialize();

      // Load Westend chain spec
      final westendSpecFile = File('test/fixtures/westend.json');
      expect(
        westendSpecFile.existsSync(),
        isTrue,
        reason:
            'Westend chain spec not found. Run: curl -o test/fixtures/westend.json https://raw.githubusercontent.com/smol-dot/smoldot/main/demo-chain-specs/westend.json',
      );

      final westendSpec = await westendSpecFile.readAsString();
      chain = await client.addChain(AddChainConfig(chainSpec: westendSpec));
    });

    tearDownAll(() async {
      if (client.isInitialized) {
        await client.dispose();
      }
    });

    test('should get chain info', () async {
      final info = await chain.getInfo();

      expect(info, isNotNull);
      expect(info.chainId, equals(chain.chainId));
      expect(info.name, equals('Westend'));
      expect(info.status, isA<ChainStatus>());
      expect(info.peerCount, greaterThanOrEqualTo(0));
      expect(info.bestBlockNumber, isNotNull);
      expect(info.bestBlockHash, isNotNull);

      print('Chain Info:');
      print('  Name: ${info.name}');
      print('  Status: ${info.status}');
      print('  Peers: ${info.peerCount}');
      print('  Block: ${info.bestBlockNumber}');
      print('  Hash: ${info.bestBlockHash}');
    });

    test('should get best block number', () async {
      final blockNumber = await chain.getBestBlockNumber();

      expect(blockNumber, isNotNull);
      expect(blockNumber!, greaterThan(0));
      print('Best block number: $blockNumber');
    });

    test('should get best block hash', () async {
      final blockHash = await chain.getBestBlockHash();

      expect(blockHash, isNotNull);
      expect(blockHash!, startsWith('0x'));
      expect(blockHash.length, equals(66)); // 0x + 64 hex characters
      print('Best block hash: $blockHash');
    });

    test('should get peer count', () async {
      final peerCount = await chain.getPeerCount();

      expect(peerCount, greaterThanOrEqualTo(0));
      print('Peer count: $peerCount');
    });

    test('should get chain status', () async {
      final status = await chain.getStatus();

      expect(status, isA<ChainStatus>());
      print('Chain status: $status');
    });

    test('should handle multiple concurrent chain info requests', () async {
      final futures = [
        chain.getBestBlockNumber(),
        chain.getBestBlockHash(),
        chain.getPeerCount(),
        chain.getStatus(),
      ];

      final results = await Future.wait(futures);

      expect(results[0], isNotNull); // block number
      expect(results[1], isNotNull); // block hash
      expect(results[2], greaterThanOrEqualTo(0)); // peer count
      expect(results[3], isA<ChainStatus>()); // status

      print('Concurrent requests completed successfully');
    });
  });
}
