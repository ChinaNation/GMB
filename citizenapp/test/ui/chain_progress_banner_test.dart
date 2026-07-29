import 'dart:collection';

import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/widgets/chain_progress_banner.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:smoldot/smoldot.dart';

void main() {
  testWidgets('交易顶栏只用整组颜色表达连接状态', (tester) async {
    final snapshots = Queue<LightClientStatusSnapshot>.from([
      _snapshot(
        isSyncing: true,
        isUsable: false,
        syncPhase: LightClientSyncPhase.warpVerifyingFragments,
        warpRequestCount: 1,
        warpReceivedFragmentCount: 1,
        warpVerifiedFragmentCount: 0,
      ),
      _snapshot(
        isSyncing: false,
        isUsable: true,
        syncPhase: LightClientSyncPhase.regular,
        warpRequestCount: 1,
        warpReceivedFragmentCount: 1,
        warpVerifiedFragmentCount: 1,
      ),
    ]);

    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.lightTheme,
        home: Scaffold(
          body: ChainProgressBanner(
            showInlineStatus: true,
            pollInterval: const Duration(milliseconds: 10),
            progressLoader: () async => snapshots.removeFirst(),
          ),
        ),
      ),
    );
    await tester.pump();

    var chainLabel = tester.widget<Text>(find.text('公民链'));
    var finalizedLabel = tester.widget<Text>(find.text('最终区块 0'));
    expect(chainLabel.style?.color, AppTheme.info);
    expect(finalizedLabel.style?.color, AppTheme.info);
    expect(find.textContaining('更新中'), findsNothing);

    await tester.pump(const Duration(milliseconds: 10));
    await tester.pump();
    chainLabel = tester.widget<Text>(find.text('公民链'));
    finalizedLabel = tester.widget<Text>(find.text('最终区块 33'));
    expect(chainLabel.style?.color, AppTheme.success);
    expect(finalizedLabel.style?.color, AppTheme.success);
    expect(find.textContaining('已更新'), findsNothing);
  });

  testWidgets('交易顶栏读取失败时整组变红但不显示失败文字', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.lightTheme,
        home: Scaffold(
          body: ChainProgressBanner(
            showInlineStatus: true,
            progressLoader: () async => throw StateError('offline'),
          ),
        ),
      ),
    );
    await tester.pump();

    final chainLabel = tester.widget<Text>(find.text('公民链'));
    final finalizedLabel = tester.widget<Text>(find.text('最终区块 —'));
    expect(chainLabel.style?.color, AppTheme.danger);
    expect(finalizedLabel.style?.color, AppTheme.danger);
    expect(find.textContaining('连接失败'), findsNothing);
  });

  testWidgets('其他页面只读取状态且不渲染任何连接状态', (tester) async {
    LightClientStatusSnapshot? receivedProgress;

    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.lightTheme,
        home: Scaffold(
          body: ChainProgressBanner(
            progressLoader: () async => _snapshot(
              isSyncing: false,
              isUsable: true,
              syncPhase: LightClientSyncPhase.regular,
              warpRequestCount: 1,
              warpReceivedFragmentCount: 1,
              warpVerifiedFragmentCount: 1,
            ),
            onProgressChanged: (progress) => receivedProgress = progress,
          ),
        ),
      ),
    );
    await tester.pump();

    expect(receivedProgress?.currentVerifiedFinalizedBlockNumber, 33);
    expect(
      find.byKey(
        const ValueKey<String>('transaction-chain-status-inline'),
      ),
      findsNothing,
    );
    expect(find.text('公民链'), findsNothing);
    expect(find.textContaining('最终区块'), findsNothing);
  });

  testWidgets('不可见状态读取仍持续轮询直到 regular', (tester) async {
    final snapshots = Queue<LightClientStatusSnapshot>.from([
      _snapshot(
        isSyncing: false,
        isUsable: false,
        syncPhase: LightClientSyncPhase.warpVerifyingFragments,
        warpRequestCount: 1,
        warpReceivedFragmentCount: 1,
        warpVerifiedFragmentCount: 0,
      ),
      _snapshot(
        isSyncing: true,
        isUsable: false,
        syncPhase: LightClientSyncPhase.warpBuildingChainInformation,
        warpRequestCount: 1,
        warpReceivedFragmentCount: 1,
        warpVerifiedFragmentCount: 1,
      ),
      _snapshot(
        isSyncing: false,
        isUsable: true,
        syncPhase: LightClientSyncPhase.regular,
        warpRequestCount: 1,
        warpReceivedFragmentCount: 1,
        warpVerifiedFragmentCount: 1,
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
    expect(find.textContaining('轻节点'), findsNothing);

    await tester.pump(const Duration(milliseconds: 10));
    await tester.pump();
    expect(find.textContaining('轻节点'), findsNothing);

    await tester.pump(const Duration(milliseconds: 10));
    await tester.pump();
    expect(find.textContaining('轻节点'), findsNothing);
    expect(loadCount, 3);

    // ready 快照不再继续轮询，避免稳定期制造后台开销。
    await tester.pump(const Duration(milliseconds: 50));
    expect(loadCount, 3);
  });
}

LightClientStatusSnapshot _snapshot({
  required bool isSyncing,
  required bool isUsable,
  required LightClientSyncPhase syncPhase,
  required int warpRequestCount,
  required int warpReceivedFragmentCount,
  required int warpVerifiedFragmentCount,
}) {
  const finalizedHash =
      '0xe3985a35f8668d74f1552be80e1e4c5c01fcce7f7c757cc0cf254ec21a1d2d9c';
  // UI 状态夹具使用合成哈希，不绑定任何真实创世版本。
  const genesisHash =
      '0x3333333333333333333333333333333333333333333333333333333333333333';
  return LightClientStatusSnapshot(
    peerCount: 5,
    isSyncing: isSyncing,
    isUsable: isUsable,
    syncPhase: syncPhase,
    bestBlockNumber: 33,
    bestBlockHash: finalizedHash,
    finalizedBlockNumber: 33,
    finalizedBlockHash: finalizedHash,
    startupFinalizedSource: LightClientStartupFinalizedSource.bundledCheckpoint,
    startupFinalizedBlockNumber: 0,
    startupFinalizedBlockHash: genesisHash,
    highestPeerFinalizedBlockNumber: 33,
    currentVerifiedFinalizedBlockNumber:
        syncPhase == LightClientSyncPhase.regular ? 33 : 0,
    currentVerifiedFinalizedBlockHash:
        syncPhase == LightClientSyncPhase.regular ? finalizedHash : genesisHash,
    warpTargetFinalizedBlockNumber:
        syncPhase == LightClientSyncPhase.regular ? null : 33,
    warpTargetFinalizedBlockHash:
        syncPhase == LightClientSyncPhase.regular ? null : finalizedHash,
    warpRequestCount: warpRequestCount,
    activeWarpFragmentRequestCount: 0,
    activeWarpStorageRequestCount: 0,
    activeWarpCallProofRequestCount: 0,
    warpReceivedFragmentCount: warpReceivedFragmentCount,
    warpVerifiedFragmentCount: warpVerifiedFragmentCount,
    warpRejectedFragmentCount: 0,
  );
}
