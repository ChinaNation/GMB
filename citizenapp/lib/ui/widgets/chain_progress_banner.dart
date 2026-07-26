import 'dart:async';
import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'package:citizenapp/log/app_log.dart';
import 'package:smoldot/smoldot.dart'
    show LightClientStatusSnapshot, LightClientSyncPhase;
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 轻节点链路进度提示条。
///
/// 用于在页面顶部展示当前 peer / best / finalized / syncing 状态，
/// 让用户在同步过程中也能直接看到链路进度，而不是只看到“请稍后再试”。
class ChainProgressBanner extends StatefulWidget {
  const ChainProgressBanner({
    super.key,
    this.margin = const EdgeInsets.only(bottom: 12),
    this.busy = false,
    this.compactThreeState = false,
    this.pollInterval = const Duration(seconds: 6),
    this.onProgressChanged,
    this.onErrorChanged,
    this.progressLoader,
  });

  final EdgeInsetsGeometry margin;
  final bool busy;

  /// 交易首页专用的三态紧凑展示。
  ///
  /// 默认关闭，避免改变提案、多签等页面依赖的详细轻节点进度文案。
  final bool compactThreeState;
  final Duration pollInterval;
  final ValueChanged<LightClientStatusSnapshot?>? onProgressChanged;
  final ValueChanged<String?>? onErrorChanged;

  /// 专项测试注入；生产固定走 [ChainRpc.fetchChainProgress]。
  final Future<LightClientStatusSnapshot> Function()? progressLoader;

  @override
  State<ChainProgressBanner> createState() => _ChainProgressBannerState();
}

class _ChainProgressBannerState extends State<ChainProgressBanner>
    with SingleTickerProviderStateMixin {
  final ChainRpc _chainRpc = ChainRpc();

  late final AnimationController _breathingController;
  LightClientStatusSnapshot? _progress;
  String? _error;
  bool _loading = false;
  Timer? _pollTimer;
  String? _lastLoggedProgress;

  @override
  void initState() {
    super.initState();
    _breathingController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
      lowerBound: 0.35,
      upperBound: 1,
    );
    // Widget test 中不启动无限动画，避免 pumpAndSettle 无法稳定；真机保持呼吸效果。
    if (!_isTestProcess) {
      _breathingController.repeat(reverse: true);
    }
    if (_isFlutterTest) return;
    unawaited(_loadProgress());
  }

  @override
  void didUpdateWidget(covariant ChainProgressBanner oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (_isFlutterTest) return;
    if (widget.busy && !oldWidget.busy) {
      unawaited(_loadProgress());
    }
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    _breathingController.dispose();
    super.dispose();
  }

  Future<void> _loadProgress() async {
    if (_isFlutterTest) return;
    _pollTimer?.cancel();
    if (mounted) {
      setState(() {
        _loading = true;
      });
    }

    try {
      final progress = await (widget.progressLoader?.call() ??
          _chainRpc.fetchChainProgress());
      if (!mounted) return;
      _logProgressTransition(progress);
      setState(() {
        _progress = progress;
        _error = null;
        _loading = false;
      });
      widget.onProgressChanged?.call(progress);
      widget.onErrorChanged?.call(null);
      _scheduleNextPoll(progress: progress);
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = SmoldotClientManager.instance.buildUserFacingError(e);
        _loading = false;
      });
      widget.onProgressChanged?.call(_progress);
      widget.onErrorChanged?.call(_error);
      _scheduleNextPoll();
    }
  }

  void _scheduleNextPoll({LightClientStatusSnapshot? progress}) {
    if (_isFlutterTest) return;
    final current = progress ?? _progress;
    // runtime near-head 可能先于 warp 状态机收口；完整可用前必须持续轮询。
    final shouldPoll = current == null || !current.isUsable || _error != null;
    if (!shouldPoll) return;
    _pollTimer = Timer(widget.pollInterval, () {
      if (!mounted) return;
      unawaited(_loadProgress());
    });
  }

  void _logProgressTransition(LightClientStatusSnapshot progress) {
    final signature = '${progress.syncPhase.wireValue}/'
        '${progress.isSyncing}/'
        '${progress.isUsable}/'
        '${progress.warpRequestCount}/'
        '${progress.warpReceivedFragmentCount}/'
        '${progress.warpVerifiedFragmentCount}/'
        '${progress.warpRejectedFragmentCount}/'
        '${progress.warpLastFailure?.wireValue}/'
        '${progress.finalizedBlockNumber}';
    if (_lastLoggedProgress == signature) return;
    _lastLoggedProgress = signature;
    AppLog.d(
      '[SmoldotStatus] phase=${progress.syncPhase.wireValue}, '
      'syncing=${progress.isSyncing}, '
      'usable=${progress.isUsable}, '
      'source=${progress.startupFinalizedSource?.wireValue}, '
      'startup=#${progress.startupFinalizedBlockNumber}, '
      'peer_finalized=#${progress.highestPeerFinalizedBlockNumber}, '
      'verified=#${progress.currentVerifiedFinalizedBlockNumber}, '
      'warp_target=#${progress.warpTargetFinalizedBlockNumber}, '
      'requests=${progress.warpRequestCount}, '
      'active_fragments=${progress.activeWarpFragmentRequestCount}, '
      'active_storage=${progress.activeWarpStorageRequestCount}, '
      'active_call_proof=${progress.activeWarpCallProofRequestCount}, '
      'received=${progress.warpReceivedFragmentCount}, '
      'verified=${progress.warpVerifiedFragmentCount}, '
      'rejected=${progress.warpRejectedFragmentCount}, '
      'last_failure=${progress.warpLastFailure?.wireValue}, '
      'best=#${progress.bestBlockNumber}, '
      'surface_finalized=#${progress.finalizedBlockNumber}',
    );
  }

  @override
  Widget build(BuildContext context) {
    final progress = _progress;
    final error = _error;

    if (widget.compactThreeState) {
      return _buildCompactThreeState(progress: progress, error: error);
    }

    final Color color;
    final IconData icon;
    final String title;
    final String subtitle;

    if (_isFlutterTest) {
      // widget test 保留提示条结构，但不读取链状态、不创建轮询定时器。
      // 测试环境没有真实轻节点，继续创建链状态轮询会让 pumpAndSettle 等不到稳定帧。
      color = AppTheme.info;
      icon = Icons.sync_disabled;
      title = '测试环境已跳过轻节点状态读取';
      subtitle = '真机运行时会正常读取已连接节点、最新区块、已验证区块等链路信息';
    } else if (progress == null && error == null) {
      color = AppTheme.info;
      icon = Icons.sync;
      title = '正在读取轻节点状态';
      subtitle = '正在获取已连接节点、最新区块、已验证区块等链路信息';
    } else if (error != null && progress == null) {
      color = AppTheme.danger;
      icon = Icons.error_outline;
      title = '轻节点状态读取失败';
      subtitle = error;
    } else if (progress != null) {
      if (progress.isUsable) {
        color = AppTheme.success;
        icon = Icons.check_circle_outline;
        title = '轻节点已就绪';
      } else if (!progress.hasPeers) {
        color = AppTheme.warning;
        icon = Icons.portable_wifi_off_outlined;
        title = '轻节点正在连接网络';
      } else if (progress.syncPhase ==
          LightClientSyncPhase.warpDownloadingFragments) {
        color = AppTheme.info;
        icon = Icons.downloading_outlined;
        title = '轻节点正在下载最终性证明';
      } else if (progress.syncPhase ==
          LightClientSyncPhase.warpVerifyingFragments) {
        color = AppTheme.info;
        icon = Icons.verified_outlined;
        title = '轻节点正在快速验证最终性';
      } else if (progress.syncPhase ==
          LightClientSyncPhase.warpDownloadingTargetState) {
        color = AppTheme.info;
        icon = Icons.downloading_outlined;
        title = '轻节点正在下载最新链状态';
      } else if (progress.syncPhase ==
          LightClientSyncPhase.warpBuildingRuntime) {
        color = AppTheme.info;
        icon = Icons.memory_outlined;
        title = '轻节点正在构建最新运行时';
      } else if (progress.syncPhase ==
          LightClientSyncPhase.warpBuildingChainInformation) {
        color = AppTheme.info;
        icon = Icons.account_tree_outlined;
        title = '轻节点正在构建最新链信息';
      } else {
        color = AppTheme.info;
        icon = Icons.sync;
        title = '轻节点正在同步尾部区块';
      }
      final best = progress.bestBlockNumber != null
          ? '#${progress.bestBlockNumber}'
          : '-';
      final finalized = '#${progress.currentVerifiedFinalizedBlockNumber}';
      if (progress.isWarping) {
        final startup = progress.startupFinalizedBlockNumber != null
            ? '#${progress.startupFinalizedBlockNumber}'
            : '-';
        final warp = progress.warpTargetFinalizedBlockNumber != null
            ? '#${progress.warpTargetFinalizedBlockNumber}'
            : '-';
        final peerFinalized = progress.highestPeerFinalizedBlockNumber != null
            ? '#${progress.highestPeerFinalizedBlockNumber}'
            : '-';
        final failure = progress.warpLastFailure?.wireValue;
        final verified = '#${progress.currentVerifiedFinalizedBlockNumber}';
        subtitle = '已连接节点 ${progress.peerCount}  启动 $startup  '
            '已验证 $verified  目标 $warp  '
            '节点已验证区块 $peerFinalized\n'
            'proof 收到 ${progress.warpReceivedFragmentCount} / '
            '验证 ${progress.warpVerifiedFragmentCount} / '
            '拒绝 ${progress.warpRejectedFragmentCount}'
            '${failure == null ? '' : '  失败 $failure'}';
      } else {
        subtitle = '已连接节点 ${progress.peerCount}  最新区块 $best  '
            '已验证区块 $finalized';
      }
    } else {
      return const SizedBox.shrink();
    }

    return Container(
      margin: widget.margin,
      padding: const EdgeInsets.fromLTRB(12, 10, 12, 10),
      decoration: AppTheme.bannerDecoration(color),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.only(top: 1),
            child: Icon(icon, size: 18, color: color),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        title,
                        style: TextStyle(
                          fontSize: 13,
                          fontWeight: FontWeight.w700,
                          color: color,
                        ),
                      ),
                    ),
                    if (_loading || widget.busy)
                      const SizedBox(
                        width: 14,
                        height: 14,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      ),
                  ],
                ),
                const SizedBox(height: 4),
                Text(
                  subtitle,
                  style: const TextStyle(
                    fontSize: 12,
                    height: 1.4,
                    color: AppTheme.textSecondary,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  bool get _isFlutterTest =>
      widget.progressLoader == null &&
      Platform.environment.containsKey('FLUTTER_TEST');

  bool get _isTestProcess => Platform.environment.containsKey('FLUTTER_TEST');

  Widget _buildCompactThreeState({
    required LightClientStatusSnapshot? progress,
    required String? error,
  }) {
    final Color color;
    final String status;
    final String detail;

    if (error != null) {
      color = AppTheme.danger;
      status = '连接失败';
      detail = '下拉刷新后重试';
    } else if (progress?.isUsable == true) {
      color = AppTheme.success;
      status = '已更新';
      detail = '最终区块 ${progress!.currentVerifiedFinalizedBlockNumber}';
    } else {
      color = AppTheme.info;
      status = '更新中';
      if (progress == null) {
        detail = '正在读取连接状态';
      } else if (!progress.hasPeers) {
        detail = '正在连接网络';
      } else {
        detail = '已连接节点 ${progress.peerCount}';
      }
    }

    return Container(
      key: const ValueKey<String>('transaction-chain-status'),
      margin: widget.margin,
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
      child: Row(
        children: [
          AnimatedBuilder(
            animation: _breathingController,
            builder: (context, _) {
              final opacity = _isTestProcess ? 1.0 : _breathingController.value;
              return SizedBox(
                width: 20,
                height: 20,
                child: Stack(
                  alignment: Alignment.center,
                  children: [
                    Opacity(
                      opacity: opacity * 0.22,
                      child: Container(
                        width: 18,
                        height: 18,
                        decoration: BoxDecoration(
                          color: color,
                          shape: BoxShape.circle,
                        ),
                      ),
                    ),
                    Container(
                      width: 9,
                      height: 9,
                      decoration: BoxDecoration(
                        color: color,
                        shape: BoxShape.circle,
                      ),
                    ),
                  ],
                ),
              );
            },
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              '公民链 $status',
              style: TextStyle(
                color: color,
                fontSize: 14,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
          const SizedBox(width: 12),
          Flexible(
            child: Text(
              detail,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              textAlign: TextAlign.right,
              style: const TextStyle(
                color: AppTheme.textSecondary,
                fontSize: 12,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
