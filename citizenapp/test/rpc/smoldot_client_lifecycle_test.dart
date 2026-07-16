import 'dart:async';
import 'dart:collection';
import 'dart:convert';

import 'package:citizenapp/rpc/chain_event_subscription.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:smoldot/smoldot.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('并发启动复用同一个 Future 且只执行一次初始化', () async {
    final releaseStart = Completer<void>();
    var startCount = 0;
    final manager = SmoldotClientManager.forTesting(
      initialize: () {
        startCount += 1;
        return releaseStart.future;
      },
    );

    final first = manager.ensureStarted();
    final second = manager.initialize();

    expect(identical(first, second), isTrue);
    expect(startCount, 1);
    releaseStart.complete();
    await Future.wait([first, second]);

    expect(manager.initializedForTesting, isTrue);
    await manager.dispose();
  });

  test('初始化失败会清空在途状态并允许下一次重试', () async {
    var startCount = 0;
    final manager = SmoldotClientManager.forTesting(
      initialize: () async {
        startCount += 1;
        if (startCount == 1) {
          throw StateError('first start failed');
        }
      },
    );

    await expectLater(manager.ensureStarted(), throwsStateError);
    expect(manager.initializedForTesting, isFalse);

    await manager.ensureStarted();
    expect(startCount, 2);
    expect(manager.initializedForTesting, isTrue);
    await manager.dispose();
  });

  test('初始化成功后 dispose 可以释放并再次启动', () async {
    var startCount = 0;
    var disposeCount = 0;
    final manager = SmoldotClientManager.forTesting(
      initialize: () async => startCount += 1,
      dispose: () async => disposeCount += 1,
    );

    await manager.ensureStarted();
    await manager.dispose();
    expect(manager.initializedForTesting, isFalse);

    await manager.ensureStarted();
    expect(startCount, 2);
    expect(disposeCount, 1);
    expect(manager.initializedForTesting, isTrue);
    await manager.dispose();
  });

  test('dispose 会让旧的在途初始化失效且不会覆盖新生命周期', () async {
    final firstStart = Completer<void>();
    var startCount = 0;
    var disposeCount = 0;
    final manager = SmoldotClientManager.forTesting(
      initialize: () {
        startCount += 1;
        return startCount == 1 ? firstStart.future : Future<void>.value();
      },
      dispose: () async => disposeCount += 1,
    );

    final staleStart = manager.ensureStarted();
    final staleStartExpectation = expectLater(
      staleStart,
      throwsA(isA<Exception>()),
    );
    final disposing = manager.dispose();
    firstStart.complete();

    await staleStartExpectation;
    await disposing;
    expect(manager.initializedForTesting, isFalse);

    await manager.ensureStarted();
    expect(startCount, 2);
    expect(disposeCount, 1);
    expect(manager.initializedForTesting, isTrue);
    await manager.dispose();
  });

  test('dispose 进行中发起的启动会等待释放完成并进入新生命周期', () async {
    final releaseDispose = Completer<void>();
    var startCount = 0;
    var disposeFinished = false;
    final manager = SmoldotClientManager.forTesting(
      initialize: () async {
        expect(disposeFinished || startCount == 0, isTrue);
        startCount += 1;
      },
      dispose: () async {
        await releaseDispose.future;
        disposeFinished = true;
      },
    );

    await manager.ensureStarted();
    final disposing = manager.dispose();
    final restarting = manager.ensureStarted();

    expect(manager.initializedForTesting, isTrue);
    expect(startCount, 1);
    releaseDispose.complete();
    await Future.wait([disposing, restarting]);

    expect(startCount, 2);
    expect(manager.initializedForTesting, isTrue);
    await manager.dispose();
  });

  test('链订阅会等待启动结果且初始化失败时返回 false', () async {
    var startCount = 0;
    final manager = SmoldotClientManager.forTesting(
      initialize: () async {
        startCount += 1;
        throw StateError('start failed');
      },
    );
    final subscription = ChainEventSubscription(
      smoldotClientManager: manager,
    );

    expect(await subscription.connect(), isFalse);
    expect(startCount, 1);

    subscription.disconnect();
    await manager.dispose();
  });

  group('smoldot finalized database 缓存', () {
    // 缓存行为只需要稳定的链身份夹具，禁止复制任何真实创世哈希形成第二真源。
    const genesisHash =
        '0x1111111111111111111111111111111111111111111111111111111111111111';

    setUp(() {
      SharedPreferences.setMockInitialValues({});
    });

    test('从内置 #0 checkpoint 推导同次公权清单的 genesis hash', () async {
      final checkpointRaw =
          await rootBundle.loadString('assets/light_sync_state.json');
      final checkpoint = jsonDecode(checkpointRaw) as Map<String, dynamic>;
      final manifestRaw = await rootBundle
          .loadString('assets/public_institutions/manifest.json');
      final manifest = jsonDecode(manifestRaw) as Map<String, dynamic>;

      expect(
        SmoldotClientManager.genesisHashFromCheckpointForTesting(
          checkpoint['finalizedBlockHeader'] as String,
        ),
        manifest['genesis_hash'],
      );
      expect(
        () => SmoldotClientManager.genesisHashFromCheckpointForTesting(
          '0x${'00' * 32}80',
        ),
        throwsFormatException,
      );
    });

    test('无有效 database 时真实启动锚点只能是安装包固定 #0', () {
      final bundledStart = _snapshot(
        0,
        startupFinalizedBlockHash: genesisHash,
      );
      expect(
        SmoldotClientManager.bundledCheckpointStartMatchesForTesting(
          expectedGenesisHash: genesisHash,
          snapshot: bundledStart,
        ),
        isTrue,
      );
      expect(
        SmoldotClientManager.bundledCheckpointStartMatchesForTesting(
          expectedGenesisHash: genesisHash,
          snapshot: _snapshot(
            0,
            startupFinalizedSource:
                LightClientStartupFinalizedSource.localDatabase,
            startupFinalizedBlockHash: genesisHash,
          ),
        ),
        isFalse,
      );
      expect(
        SmoldotClientManager.bundledCheckpointStartMatchesForTesting(
          expectedGenesisHash: genesisHash,
          snapshot: _snapshot(
            1,
            startupFinalizedBlockNumber: 1,
            startupFinalizedBlockHash: _hashForHeight(1),
          ),
        ),
        isFalse,
      );
      expect(
        SmoldotClientManager.bundledCheckpointStartMatchesForTesting(
          expectedGenesisHash: genesisHash,
          snapshot: _snapshot(0),
        ),
        isFalse,
      );
    });

    test('旧裸格式、未知字段和跨链信封会被删除', () async {
      final manager = SmoldotClientManager.forTesting(
        initialize: () async {},
      );
      final prefs = await SharedPreferences.getInstance();

      await prefs.setString('smoldot_db_cache', 'legacy-database');
      expect(await manager.loadCachedDatabaseForTesting(genesisHash), isNull);
      expect(prefs.containsKey('smoldot_db_cache'), isFalse);

      await prefs.setString(
        'smoldot_db_cache',
        _cacheEnvelopeRaw(
          genesisHash: genesisHash,
          finalizedBlockNumber: 10,
          databaseContent: 'db-10',
          extra: const {'legacy': true},
        ),
      );
      expect(await manager.loadCachedDatabaseForTesting(genesisHash), isNull);
      expect(prefs.containsKey('smoldot_db_cache'), isFalse);

      await prefs.setString(
        'smoldot_db_cache',
        _cacheEnvelopeRaw(
          genesisHash: _hashForHeight(999),
          finalizedBlockNumber: 10,
          databaseContent: 'db-10',
        ),
      );
      expect(await manager.loadCachedDatabaseForTesting(genesisHash), isNull);
      expect(prefs.containsKey('smoldot_db_cache'), isFalse);

      await prefs.setString(
        'smoldot_db_cache',
        _cacheEnvelopeRaw(
          genesisHash: genesisHash,
          finalizedBlockNumber: 10,
          databaseContent: 'db-10',
        ),
      );
      expect(
        await manager.loadCachedDatabaseForTesting(genesisHash),
        'db-10',
      );
    });

    test('缓存恢复必须真实采用信封声明的 database anchor', () {
      final raw = _cacheEnvelopeRaw(
        genesisHash: genesisHash,
        finalizedBlockNumber: 10,
        databaseContent: 'db-10',
      );

      expect(
        SmoldotClientManager.restoredDatabaseCacheReachedForTesting(
          rawEnvelope: raw,
          expectedGenesisHash: genesisHash,
          snapshot: _snapshot(10),
        ),
        isFalse,
      );
      expect(
        SmoldotClientManager.restoredDatabaseCacheReachedForTesting(
          rawEnvelope: raw,
          expectedGenesisHash: genesisHash,
          snapshot: _snapshot(
            10,
            startupFinalizedSource:
                LightClientStartupFinalizedSource.localDatabase,
            startupFinalizedBlockNumber: 9,
            startupFinalizedBlockHash: _hashForHeight(9),
          ),
        ),
        isFalse,
      );
      expect(
        SmoldotClientManager.restoredDatabaseCacheReachedForTesting(
          rawEnvelope: raw,
          expectedGenesisHash: genesisHash,
          snapshot: _snapshot(
            10,
            startupFinalizedSource:
                LightClientStartupFinalizedSource.localDatabase,
            startupFinalizedBlockNumber: 10,
            startupFinalizedBlockHash: _hashForHeight(10),
          ),
        ),
        isTrue,
      );
      expect(
        SmoldotClientManager.restoredDatabaseCacheReachedForTesting(
          rawEnvelope: raw,
          expectedGenesisHash: genesisHash,
          snapshot: _snapshot(
            11,
            startupFinalizedSource:
                LightClientStartupFinalizedSource.localDatabase,
            startupFinalizedBlockNumber: 10,
            startupFinalizedBlockHash: _hashForHeight(10),
          ),
        ),
        isTrue,
      );
      expect(
        SmoldotClientManager.restoredDatabaseCacheReachedForTesting(
          rawEnvelope: raw,
          expectedGenesisHash: genesisHash,
          snapshot: _snapshot(
            11,
            startupFinalizedSource:
                LightClientStartupFinalizedSource.localDatabase,
            startupFinalizedBlockNumber: 10,
            startupFinalizedBlockHash: _hashForHeight(11),
          ),
        ),
        isFalse,
      );
    });

    test('导出严格串行且低 finalized 不得覆盖高缓存', () async {
      final statusQueue = Queue<LightClientStatusSnapshot>.from([
        _snapshot(10),
        _snapshot(10),
        _snapshot(20),
        _snapshot(20),
        _snapshot(19),
        _snapshot(19),
      ]);
      final releaseFirstExport = Completer<void>();
      final firstExportStarted = Completer<void>();
      var exportCount = 0;
      final manager = SmoldotClientManager.forTesting(
        initialize: () async {},
        cacheStatus: () async => statusQueue.removeFirst(),
        exportDatabase: () async {
          exportCount += 1;
          if (exportCount == 1) {
            firstExportStarted.complete();
            await releaseFirstExport.future;
          }
          return switch (exportCount) {
            1 => 'db-10',
            2 => 'db-20',
            _ => 'db-19',
          };
        },
        expectedGenesisHash: genesisHash,
      );
      await manager.ensureStarted();

      final first = manager.saveDatabaseCacheForTesting();
      await firstExportStarted.future;
      final second = manager.saveDatabaseCacheForTesting();
      await Future<void>.delayed(Duration.zero);
      expect(exportCount, 1, reason: '第二次导出必须等待第一次完成');

      releaseFirstExport.complete();
      await Future.wait([first, second]);
      await manager.saveDatabaseCacheForTesting();

      final saved = await _savedEnvelope();
      expect(saved['finalized_block_number'], 20);
      expect(saved['database_content'], 'db-20');
      await manager.dispose();
    });

    test('finalized 在导出期间推进时丢弃不稳定正文并重试', () async {
      final statusQueue = Queue<LightClientStatusSnapshot>.from([
        _snapshot(10),
        _snapshot(11),
        _snapshot(11),
        _snapshot(11),
      ]);
      final databaseQueue = Queue<String>.from(['moving-db', 'stable-db']);
      final manager = SmoldotClientManager.forTesting(
        initialize: () async {},
        cacheStatus: () async => statusQueue.removeFirst(),
        exportDatabase: () async => databaseQueue.removeFirst(),
        expectedGenesisHash: genesisHash,
      );
      await manager.ensureStarted();

      await manager.saveDatabaseCacheForTesting();

      final saved = await _savedEnvelope();
      expect(saved['finalized_block_number'], 11);
      expect(saved['database_content'], 'stable-db');
      await manager.dispose();
    });

    test('表面 finalized 已到 F 但完整验证仍为 H 时禁止落缓存', () async {
      final statusQueue = Queue<LightClientStatusSnapshot>.from([
        _snapshot(
          33,
          isSyncing: false,
          syncPhase: LightClientSyncPhase.warpDownloadingFragments,
        ),
        _snapshot(31),
        _snapshot(31),
        _snapshot(31),
        _snapshot(33),
        _snapshot(33),
        _snapshot(33),
      ]);
      var exportCount = 0;
      final manager = SmoldotClientManager.forTesting(
        initialize: () async {},
        cacheStatus: () async => statusQueue.removeFirst(),
        exportDatabase: () async => 'db-${++exportCount}',
        expectedGenesisHash: genesisHash,
      );
      await manager.ensureStarted();

      await manager.saveDatabaseCacheForTesting();
      expect(exportCount, 0, reason: 'warp 尚未 regular 时禁止导出');

      await manager.saveDatabaseCacheForTesting();
      expect((await _savedEnvelope())['finalized_block_number'], 31);
      expect(exportCount, 1);

      await manager.refreshDatabaseCacheIfAdvancedForTesting();
      expect(exportCount, 1, reason: 'finalized 未推进时不得重复导出');

      await manager.refreshDatabaseCacheIfAdvancedForTesting();
      final saved = await _savedEnvelope();
      expect(saved['finalized_block_number'], 33);
      expect(saved['database_content'], 'db-2');
      expect(exportCount, 2);
      await manager.dispose();
    });

    test('完整验证 F 落盘后下一次启动必须把同一个 F 作为 H', () async {
      final statusQueue = Queue<LightClientStatusSnapshot>.from([
        _snapshot(100),
        _snapshot(100),
      ]);
      final manager = SmoldotClientManager.forTesting(
        initialize: () async {},
        cacheStatus: () async => statusQueue.removeFirst(),
        exportDatabase: () async => 'db-f',
        expectedGenesisHash: genesisHash,
      );
      await manager.ensureStarted();
      await manager.saveDatabaseCacheForTesting();

      final prefs = await SharedPreferences.getInstance();
      final rawEnvelope = prefs.getString('smoldot_db_cache')!;
      expect(
        SmoldotClientManager.restoredDatabaseCacheReachedForTesting(
          rawEnvelope: rawEnvelope,
          expectedGenesisHash: genesisHash,
          snapshot: _snapshot(
            100,
            startupFinalizedSource:
                LightClientStartupFinalizedSource.localDatabase,
            startupFinalizedBlockNumber: 100,
            startupFinalizedBlockHash: _hashForHeight(100),
          ),
        ),
        isTrue,
      );
      await manager.dispose();
    });

    test('同高度同 hash 不重写，同高度不同 hash 清理后写入当前候选', () async {
      final hashA = _hashForHeight(20);
      final hashB = _hashForHeight(21);
      final statusQueue = Queue<LightClientStatusSnapshot>.from([
        _snapshot(20, hash: hashA),
        _snapshot(20, hash: hashA),
        _snapshot(20, hash: hashA),
        _snapshot(20, hash: hashA),
        _snapshot(20, hash: hashB),
        _snapshot(20, hash: hashB),
      ]);
      final databaseQueue = Queue<String>.from(['db-a', 'db-b', 'db-c']);
      final manager = SmoldotClientManager.forTesting(
        initialize: () async {},
        cacheStatus: () async => statusQueue.removeFirst(),
        exportDatabase: () async => databaseQueue.removeFirst(),
        expectedGenesisHash: genesisHash,
      );
      await manager.ensureStarted();

      await manager.saveDatabaseCacheForTesting();
      await manager.saveDatabaseCacheForTesting();
      expect((await _savedEnvelope())['database_content'], 'db-a');

      await manager.saveDatabaseCacheForTesting();
      final saved = await _savedEnvelope();
      expect(saved['finalized_block_hash'], hashB);
      expect(saved['database_content'], 'db-c');
      await manager.dispose();
    });

    test('dispose 使旧导出失效且新生命周期可以保存更高缓存', () async {
      final statusQueue = Queue<LightClientStatusSnapshot>.from([
        _snapshot(10),
        _snapshot(20),
        _snapshot(20),
      ]);
      final oldExportStarted = Completer<void>();
      final releaseOldExport = Completer<void>();
      var exportCount = 0;
      final manager = SmoldotClientManager.forTesting(
        initialize: () async {},
        cacheStatus: () async => statusQueue.removeFirst(),
        exportDatabase: () async {
          exportCount += 1;
          if (exportCount == 1) {
            oldExportStarted.complete();
            await releaseOldExport.future;
            return 'stale-db-10';
          }
          return 'db-20';
        },
        expectedGenesisHash: genesisHash,
      );
      await manager.ensureStarted();

      final staleSave = manager.saveDatabaseCacheForTesting();
      await oldExportStarted.future;
      final disposing = manager.dispose();
      releaseOldExport.complete();
      await Future.wait([staleSave, disposing]);
      expect(
        (await SharedPreferences.getInstance()).containsKey('smoldot_db_cache'),
        isFalse,
      );

      await manager.ensureStarted();
      await manager.saveDatabaseCacheForTesting();
      final saved = await _savedEnvelope();
      expect(saved['finalized_block_number'], 20);
      expect(saved['database_content'], 'db-20');
      await manager.dispose();
    });
  });
}

LightClientStatusSnapshot _snapshot(
  int height, {
  String? hash,
  bool isSyncing = false,
  LightClientSyncPhase syncPhase = LightClientSyncPhase.regular,
  LightClientStartupFinalizedSource startupFinalizedSource =
      LightClientStartupFinalizedSource.bundledCheckpoint,
  int startupFinalizedBlockNumber = 0,
  String? startupFinalizedBlockHash,
}) {
  final isUsable = !isSyncing && syncPhase == LightClientSyncPhase.regular;
  final currentVerifiedHeight = isUsable ? height : startupFinalizedBlockNumber;
  final currentVerifiedHash = isUsable
      ? (hash ?? _hashForHeight(height))
      : (startupFinalizedBlockHash ??
          _hashForHeight(startupFinalizedBlockNumber));
  return LightClientStatusSnapshot(
    peerCount: 1,
    isSyncing: isSyncing,
    isUsable: isUsable,
    syncPhase: syncPhase,
    bestBlockNumber: height,
    bestBlockHash: hash ?? _hashForHeight(height),
    finalizedBlockNumber: height,
    finalizedBlockHash: hash ?? _hashForHeight(height),
    startupFinalizedSource: startupFinalizedSource,
    startupFinalizedBlockNumber: startupFinalizedBlockNumber,
    startupFinalizedBlockHash: startupFinalizedBlockHash ??
        _hashForHeight(startupFinalizedBlockNumber),
    highestPeerFinalizedBlockNumber: height,
    currentVerifiedFinalizedBlockNumber: currentVerifiedHeight,
    currentVerifiedFinalizedBlockHash: currentVerifiedHash,
    warpTargetFinalizedBlockNumber: isUsable ? null : height,
    warpTargetFinalizedBlockHash:
        isUsable ? null : (hash ?? _hashForHeight(height)),
    warpRequestCount: 0,
    activeWarpFragmentRequestCount: 0,
    activeWarpStorageRequestCount: 0,
    activeWarpCallProofRequestCount: 0,
    warpReceivedFragmentCount: 0,
    warpVerifiedFragmentCount: 0,
    warpRejectedFragmentCount: 0,
  );
}

String _hashForHeight(int height) =>
    '0x${height.toRadixString(16).padLeft(64, '0')}';

String _cacheEnvelopeRaw({
  required String genesisHash,
  required int finalizedBlockNumber,
  required String databaseContent,
  Map<String, dynamic> extra = const {},
}) {
  return jsonEncode({
    'schema': 'citizenapp.smoldot.database.v1',
    'genesis_hash': genesisHash,
    'finalized_block_number': finalizedBlockNumber,
    'finalized_block_hash': _hashForHeight(finalizedBlockNumber),
    'database_content': databaseContent,
    ...extra,
  });
}

Future<Map<String, dynamic>> _savedEnvelope() async {
  final prefs = await SharedPreferences.getInstance();
  return jsonDecode(prefs.getString('smoldot_db_cache')!)
      as Map<String, dynamic>;
}
