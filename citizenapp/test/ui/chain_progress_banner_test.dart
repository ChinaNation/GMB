import 'dart:collection';

import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/widgets/chain_progress_banner.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:smoldot/smoldot.dart';

void main() {
  testWidgets('runtime 已近头但 warp 未结束时继续轮询直到 regular', (tester) async {
    final snapshots = Queue<LightClientStatusSnapshot>.from([
      _snapshot(
        isSyncing: false,
        syncMode: LightClientSyncMode.warpFragments,
        warpRequestCount: 1,
        warpFragmentCount: 1,
      ),
      _snapshot(
        isSyncing: true,
        syncMode: LightClientSyncMode.warpChainInformation,
        warpRequestCount: 1,
        warpFragmentCount: 1,
      ),
      _snapshot(
        isSyncing: false,
        syncMode: LightClientSyncMode.regular,
        warpRequestCount: 1,
        warpFragmentCount: 1,
      ),
    ]);
    var loadCount = 0;

    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.lightTheme,
        home: Scaffold(
          body: ChainProgressBanner(
            pollInterval: const Duration(milliseconds: 10),
            progressLoader: () async {
              loadCount += 1;
              return snapshots.removeFirst();
            },
          ),
        ),
      ),
    );

    await tester.pump();
    expect(find.text('轻节点正在快速验证最终性'), findsOneWidget);

    await tester.pump(const Duration(milliseconds: 10));
    await tester.pump();
    expect(find.text('轻节点正在加载最新链状态'), findsOneWidget);

    await tester.pump(const Duration(milliseconds: 10));
    await tester.pump();
    expect(find.text('轻节点已就绪'), findsOneWidget);
    expect(loadCount, 3);

    // ready 快照不再继续轮询，避免稳定期制造后台开销。
    await tester.pump(const Duration(milliseconds: 50));
    expect(loadCount, 3);
  });
}

LightClientStatusSnapshot _snapshot({
  required bool isSyncing,
  required LightClientSyncMode syncMode,
  required int warpRequestCount,
  required int warpFragmentCount,
}) {
  const finalizedHash =
      '0xe3985a35f8668d74f1552be80e1e4c5c01fcce7f7c757cc0cf254ec21a1d2d9c';
  return LightClientStatusSnapshot(
    peerCount: 5,
    isSyncing: isSyncing,
    syncMode: syncMode,
    bestBlockNumber: 33,
    bestBlockHash: finalizedHash,
    finalizedBlockNumber: 33,
    finalizedBlockHash: finalizedHash,
    startupFinalizedBlockNumber: 0,
    highestPeerFinalizedBlockNumber: 33,
    warpFinalizedBlockNumber: 33,
    warpRequestCount: warpRequestCount,
    warpFragmentCount: warpFragmentCount,
  );
}
