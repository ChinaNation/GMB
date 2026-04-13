import 'dart:async';

import 'package:flutter/material.dart';
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

/// 轻节点链路进度提示条。
///
/// 用于在页面顶部展示当前 peer / best / finalized / syncing 状态，
/// 让用户在同步过程中也能直接看到链路进度，而不是只看到“请稍后再试”。
class ChainProgressBanner extends StatefulWidget {
  const ChainProgressBanner({
    super.key,
    this.margin = const EdgeInsets.only(bottom: 12),
    this.busy = false,
    this.pollInterval = const Duration(seconds: 6),
    this.onProgressChanged,
    this.onErrorChanged,
  });

  final EdgeInsetsGeometry margin;
  final bool busy;
  final Duration pollInterval;
  final ValueChanged<LightClientStatusSnapshot?>? onProgressChanged;
  final ValueChanged<String?>? onErrorChanged;

  @override
  State<ChainProgressBanner> createState() => _ChainProgressBannerState();
}

class _ChainProgressBannerState extends State<ChainProgressBanner> {
  final ChainRpc _chainRpc = ChainRpc();

  LightClientStatusSnapshot? _progress;
  String? _error;
  bool _loading = false;
  Timer? _pollTimer;

  @override
  void initState() {
    super.initState();
    unawaited(_loadProgress());
  }

  @override
  void didUpdateWidget(covariant ChainProgressBanner oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.busy && !oldWidget.busy) {
      unawaited(_loadProgress());
    }
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    super.dispose();
  }

  Future<void> _loadProgress() async {
    _pollTimer?.cancel();
    if (mounted) {
      setState(() {
        _loading = true;
      });
    }

    try {
      final progress = await _chainRpc.fetchChainProgress();
      if (!mounted) return;
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
    final current = progress ?? _progress;
    final shouldPoll =
        current == null || !current.hasPeers || current.isSyncing || _error != null;
    if (!shouldPoll) return;
    _pollTimer = Timer(widget.pollInterval, () {
      if (!mounted) return;
      unawaited(_loadProgress());
    });
  }

  @override
  Widget build(BuildContext context) {
    final progress = _progress;
    final error = _error;

    final Color color;
    final IconData icon;
    final String title;
    final String subtitle;

    if (progress == null && error == null) {
      color = AppTheme.info;
      icon = Icons.sync;
      title = '正在读取区块链状态';
      subtitle = '正在获取 peer、best、finalized 等链路信息';
    } else if (error != null && progress == null) {
      color = AppTheme.danger;
      icon = Icons.error_outline;
      title = '区块链状态读取失败';
      subtitle = error;
    } else if (progress != null) {
      if (!progress.hasPeers) {
        color = AppTheme.warning;
        icon = Icons.portable_wifi_off_outlined;
        title = '轻节点正在连接网络';
      } else if (progress.isSyncing) {
        color = AppTheme.info;
        icon = Icons.sync;
        title = '轻节点正在同步区块头';
      } else {
        color = AppTheme.success;
        icon = Icons.check_circle_outline;
        title = '区块链已就绪';
      }
      final best = progress.bestBlockNumber != null
          ? '#${progress.bestBlockNumber}'
          : '-';
      final finalized = progress.finalizedBlockNumber != null
          ? '#${progress.finalizedBlockNumber}'
          : '-';
      subtitle = 'peer ${progress.peerCount}  best $best  finalized $finalized';
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
}
