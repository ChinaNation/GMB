import 'dart:async';
import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'package:citizenapp/log/app_log.dart';
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 公民链状态读取与交易顶栏唯一可见状态。
///
/// 所有调用方共用真实轻节点轮询和门禁回调；只有交易 Tab 设置
/// [showInlineStatus] 后可见，其他页面只读取状态，不渲染连接状态 UI。
class ChainProgressBanner extends StatefulWidget {
  const ChainProgressBanner({
    super.key,
    this.margin = EdgeInsets.zero,
    this.busy = false,
    this.showInlineStatus = false,
    this.pollInterval = const Duration(seconds: 6),
    this.onProgressChanged,
    this.onErrorChanged,
    this.progressLoader,
  });

  final EdgeInsetsGeometry margin;
  final bool busy;

  /// 是否在交易 Tab 顶栏显示唯一的全局链状态。
  ///
  /// 默认关闭；不可见调用方仍会读取链状态并回传给业务门禁。
  final bool showInlineStatus;
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
    if (widget.showInlineStatus && !_isTestProcess) {
      _breathingController.repeat(reverse: true);
    }
    if (_isFlutterTest) return;
    unawaited(_loadProgress());
  }

  @override
  void didUpdateWidget(covariant ChainProgressBanner oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.showInlineStatus != oldWidget.showInlineStatus &&
        !_isTestProcess) {
      if (widget.showInlineStatus) {
        _breathingController.repeat(reverse: true);
      } else {
        _breathingController.stop();
      }
    }
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

    try {
      final progress = await (widget.progressLoader?.call() ??
          _chainRpc.fetchChainProgress());
      if (!mounted) return;
      _logProgressTransition(progress);
      setState(() {
        _progress = progress;
        _error = null;
      });
      widget.onProgressChanged?.call(progress);
      widget.onErrorChanged?.call(null);
      _scheduleNextPoll(progress: progress);
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = SmoldotClientManager.instance.buildUserFacingError(e);
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
    if (!widget.showInlineStatus) return const SizedBox.shrink();
    return _buildInlineStatus(progress: _progress, error: _error);
  }

  bool get _isFlutterTest =>
      widget.progressLoader == null &&
      Platform.environment.containsKey('FLUTTER_TEST');

  bool get _isTestProcess => Platform.environment.containsKey('FLUTTER_TEST');

  ({Color color, String semanticsStatus}) _resolveState({
    required LightClientStatusSnapshot? progress,
    required String? error,
  }) {
    if (error != null) {
      return (color: AppTheme.danger, semanticsStatus: '连接失败');
    }
    if (progress?.isUsable == true) {
      return (color: AppTheme.success, semanticsStatus: '连接正常');
    }
    return (color: AppTheme.info, semanticsStatus: '正在连接');
  }

  Widget _buildInlineStatus({
    required LightClientStatusSnapshot? progress,
    required String? error,
  }) {
    final state = _resolveState(progress: progress, error: error);
    final finalizedBlockNumber = progress?.currentVerifiedFinalizedBlockNumber;
    final detail = '最终区块 ${finalizedBlockNumber ?? '—'}';

    return Semantics(
      key: const ValueKey<String>('transaction-chain-status-inline'),
      excludeSemantics: true,
      label: '公民链${state.semanticsStatus}，$detail',
      child: Padding(
        padding: widget.margin,
        child: Row(
          children: [
            Expanded(
              child: Align(
                alignment: Alignment.centerRight,
                child: FittedBox(
                  fit: BoxFit.scaleDown,
                  alignment: Alignment.centerRight,
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      AnimatedBuilder(
                        animation: _breathingController,
                        builder: (context, _) {
                          final opacity =
                              _isTestProcess ? 1.0 : _breathingController.value;
                          return SizedBox(
                            width: 18,
                            height: 18,
                            child: Stack(
                              alignment: Alignment.center,
                              children: [
                                Opacity(
                                  opacity: opacity * 0.22,
                                  child: Container(
                                    width: 16,
                                    height: 16,
                                    decoration: BoxDecoration(
                                      color: state.color,
                                      shape: BoxShape.circle,
                                    ),
                                  ),
                                ),
                                Container(
                                  width: 8,
                                  height: 8,
                                  decoration: BoxDecoration(
                                    color: state.color,
                                    shape: BoxShape.circle,
                                  ),
                                ),
                              ],
                            ),
                          );
                        },
                      ),
                      const SizedBox(width: 6),
                      Text(
                        '公民链',
                        maxLines: 1,
                        style: TextStyle(
                          color: state.color,
                          fontSize: 13,
                          fontWeight: FontWeight.w700,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
            // 两段之间的固定空隙以顶栏几何中心为中点，确保视觉轴线落在屏幕中央。
            const SizedBox(
              key: ValueKey<String>('transaction-chain-status-center-gap'),
              width: 10,
            ),
            Expanded(
              child: Align(
                alignment: Alignment.centerLeft,
                child: FittedBox(
                  fit: BoxFit.scaleDown,
                  alignment: Alignment.centerLeft,
                  child: Text(
                    detail,
                    maxLines: 1,
                    style: TextStyle(
                      color: state.color,
                      fontSize: 12,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
