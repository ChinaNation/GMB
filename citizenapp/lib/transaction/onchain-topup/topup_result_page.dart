import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/ui/app_theme.dart';
import 'topup_api.dart';
import 'topup_models.dart';

/// 支付结果页(第 3 屏):上报付款交易 → 轮询到账。
///
/// 只有成功/失败两种终态(契合三态台账 paid/exception);出结果前显示「处理中」,
/// 超时不判失败,只提示仍在确认。
enum _Phase { processing, success, failure, unresolved }

class TopupResultPage extends StatefulWidget {
  const TopupResultPage({
    super.key,
    required this.api,
    required this.rail,
    required this.package,
    required this.gmbAddress,
    required this.evmTxHash,
    this.payerAddress,
  });

  final TopupApi api;
  final TopupRail rail;
  final TopupPackage package;
  final String gmbAddress;
  final String evmTxHash;
  final String? payerAddress;

  @override
  State<TopupResultPage> createState() => _TopupResultPageState();
}

class _TopupResultPageState extends State<TopupResultPage> {
  static const _pollInterval = Duration(seconds: 3);
  static const _maxAttempts = 40; // ≈2 分钟

  _Phase _phase = _Phase.processing;
  String _message = '支付已提交，正在确认到账…';
  bool _cancelled = false;

  @override
  void initState() {
    super.initState();
    _run();
  }

  @override
  void dispose() {
    _cancelled = true;
    super.dispose();
  }

  Future<void> _run() async {
    var submitted = false;
    for (var attempt = 0; attempt < _maxAttempts; attempt++) {
      if (_cancelled) return;
      try {
        if (!submitted) {
          final result = await widget.api.submit(
            token: widget.rail.token,
            packageId: widget.package.packageId,
            gmbAddress: widget.gmbAddress,
            evmTxHash: widget.evmTxHash,
            payerAddress: widget.payerAddress,
          );
          if (_settleFromStatus(result.status)) return;
          if (result.status == TopupOrderStatus.pending) submitted = true;
        } else {
          final status = await widget.api
              .status(chainId: widget.rail.chainId, evmTxHash: widget.evmTxHash);
          if (_settleFromStatus(status)) return;
        }
      } on TopupApiException catch (e) {
        // 到账校验明确不通过 → 失败;其余(网络抖动等)继续轮询。
        if (e.errorCode == 'topup_payment_invalid') {
          _finish(_Phase.failure, '未收到有效到账，本次失败');
          return;
        }
      }
      await Future<void>.delayed(_pollInterval);
    }
    if (!_cancelled) {
      _finish(_Phase.unresolved, '仍在确认中，可稍后在钱包查看余额');
    }
  }

  /// paid→成功;exception→失败;其余(confirming/pending/notFound)返回 false 继续轮询。
  bool _settleFromStatus(TopupOrderStatus status) {
    switch (status) {
      case TopupOrderStatus.paid:
        _finish(_Phase.success, '${widget.package.coinDisplay} 公民币已转入钱包');
        return true;
      case TopupOrderStatus.exception:
        _finish(_Phase.failure, '未收到到账，本次失败');
        return true;
      default:
        return false;
    }
  }

  void _finish(_Phase phase, String message) {
    if (!mounted) return;
    setState(() {
      _phase = phase;
      _message = message;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('支付结果'), centerTitle: true),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              _icon(),
              const SizedBox(height: 18),
              Text(_title(),
                  style: const TextStyle(
                      fontSize: 18,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.textPrimary)),
              const SizedBox(height: 6),
              Text(_message,
                  textAlign: TextAlign.center,
                  style: const TextStyle(
                      fontSize: 14, color: AppTheme.textSecondary)),
              const SizedBox(height: 6),
              Text('tx ${_shortHash(widget.evmTxHash)}',
                  style: const TextStyle(
                      fontSize: 12,
                      color: AppTheme.textTertiary,
                      fontFamily: 'monospace')),
              const SizedBox(height: 28),
              if (_phase != _Phase.processing)
                SizedBox(
                  width: double.infinity,
                  child: FilledButton(
                    style: FilledButton.styleFrom(
                      backgroundColor: AppTheme.primary,
                      padding: const EdgeInsets.symmetric(vertical: 13),
                      shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(14)),
                    ),
                    onPressed: () => Navigator.of(context)
                        .popUntil((route) => route.isFirst),
                    child: Text(
                        _phase == _Phase.success ? '查看钱包' : '返回',
                        style: const TextStyle(fontSize: 15)),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _icon() {
    switch (_phase) {
      case _Phase.processing:
        return const SizedBox(
          width: 64,
          height: 64,
          child: CircularProgressIndicator(color: AppTheme.primary),
        );
      case _Phase.success:
        return _circleIcon(Icons.check, AppTheme.success);
      case _Phase.failure:
        return _circleIcon(Icons.priority_high, AppTheme.danger);
      case _Phase.unresolved:
        return _circleIcon(Icons.hourglass_empty, AppTheme.warning);
    }
  }

  Widget _circleIcon(IconData icon, Color color) {
    return Container(
      width: 72,
      height: 72,
      decoration: BoxDecoration(color: color.withAlpha(31), shape: BoxShape.circle),
      child: Icon(icon, size: 38, color: color),
    );
  }

  String _title() {
    switch (_phase) {
      case _Phase.processing:
        return '处理中';
      case _Phase.success:
        return '已到账';
      case _Phase.failure:
        return '本次失败';
      case _Phase.unresolved:
        return '仍在确认';
    }
  }

  static String _shortHash(String hash) {
    if (hash.length <= 14) return hash;
    return '${hash.substring(0, 8)}…${hash.substring(hash.length - 4)}';
  }
}
