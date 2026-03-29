import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../ui/app_theme.dart';
import 'app_lock_service.dart';

enum PinInputMode { setup, verify, remove }

/// 6 位 PIN 输入页面。
class PinInputPage extends StatefulWidget {
  const PinInputPage({super.key, required this.mode});

  final PinInputMode mode;

  @override
  State<PinInputPage> createState() => _PinInputPageState();
}

class _PinInputPageState extends State<PinInputPage> {
  static const int pinLength = 6;

  String _pin = '';
  String? _firstPin;
  String _title = '';
  String _subtitle = '';
  String? _error;
  bool _locked = false;
  int _remainingSeconds = 0;
  Timer? _lockTimer;

  @override
  void initState() {
    super.initState();
    _initState();
  }

  @override
  void dispose() {
    _lockTimer?.cancel();
    super.dispose();
  }

  Future<void> _initState() async {
    if (widget.mode == PinInputMode.verify) {
      final locked = await AppLockService.isLocked();
      if (locked) {
        _startLockCountdown();
        return;
      }
    }
    _updateTitle();
  }

  void _updateTitle() {
    switch (widget.mode) {
      case PinInputMode.setup:
        setState(() {
          _title = _firstPin == null ? '设置应用密码' : '请再次输入';
          _subtitle = _firstPin == null ? '请输入 6 位数字密码' : '确认您的密码';
        });
      case PinInputMode.verify:
        setState(() {
          _title = '输入应用密码';
          _subtitle = '请输入 6 位数字密码';
        });
      case PinInputMode.remove:
        setState(() {
          _title = '关闭应用锁';
          _subtitle = '请输入当前密码以关闭';
        });
    }
  }

  Future<void> _startLockCountdown() async {
    final remaining = await AppLockService.getRemainingLockSeconds();
    if (remaining <= 0) {
      _updateTitle();
      return;
    }
    setState(() {
      _locked = true;
      _remainingSeconds = remaining;
    });
    _lockTimer?.cancel();
    _lockTimer = Timer.periodic(const Duration(seconds: 1), (_) async {
      final r = await AppLockService.getRemainingLockSeconds();
      if (!mounted) return;
      if (r <= 0) {
        _lockTimer?.cancel();
        setState(() {
          _locked = false;
          _remainingSeconds = 0;
        });
        _updateTitle();
      } else {
        setState(() => _remainingSeconds = r);
      }
    });
  }

  void _onDigit(int digit) {
    if (_pin.length >= pinLength || _locked) return;
    HapticFeedback.lightImpact();
    setState(() {
      _pin += digit.toString();
      _error = null;
    });
    if (_pin.length == pinLength) {
      _onPinComplete();
    }
  }

  void _onDelete() {
    if (_pin.isEmpty || _locked) return;
    HapticFeedback.lightImpact();
    setState(() {
      _pin = _pin.substring(0, _pin.length - 1);
      _error = null;
    });
  }

  Future<void> _onPinComplete() async {
    switch (widget.mode) {
      case PinInputMode.setup:
        await _handleSetup();
      case PinInputMode.verify:
        await _handleVerify();
      case PinInputMode.remove:
        await _handleRemove();
    }
  }

  Future<void> _handleSetup() async {
    if (_firstPin == null) {
      _firstPin = _pin;
      setState(() => _pin = '');
      _updateTitle();
    } else {
      if (_pin == _firstPin) {
        await AppLockService.setPin(_pin);
        if (!mounted) return;
        Navigator.of(context).pop(true);
      } else {
        _firstPin = null;
        setState(() {
          _pin = '';
          _error = '两次输入不一致，请重新设置';
        });
        _updateTitle();
      }
    }
  }

  Future<void> _handleVerify() async {
    final ok = await AppLockService.verifyPin(_pin);
    if (!mounted) return;

    if (ok) {
      Navigator.of(context).pop(true);
      return;
    }

    final locked = await AppLockService.isLocked();
    if (locked) {
      final pinSet = await AppLockService.isPinSet();
      if (!pinSet) {
        if (!mounted) return;
        await showDialog<void>(
          context: context,
          barrierDismissible: false,
          builder: (_) => AlertDialog(
            title: const Text('数据已清空'),
            content:
                const Text('连续多次验证错误，应用数据已全部清空。请重新启动应用。'),
            actions: [
              TextButton(
                onPressed: () => SystemNavigator.pop(),
                child: const Text('退出'),
              ),
            ],
          ),
        );
        return;
      }

      _startLockCountdown();
      setState(() => _pin = '');
      return;
    }

    final failCount = await AppLockService.getFailCount();
    final remaining = AppLockService.maxFailAttempts - failCount;
    setState(() {
      _pin = '';
      _error = '密码错误，还可尝试 $remaining 次';
    });
  }

  Future<void> _handleRemove() async {
    final ok = await AppLockService.verifyPin(_pin);
    if (!mounted) return;

    if (ok) {
      await AppLockService.removePin();
      if (!mounted) return;
      Navigator.of(context).pop(true);
    } else {
      final failCount = await AppLockService.getFailCount();
      final remaining = AppLockService.maxFailAttempts - failCount;
      setState(() {
        _pin = '';
        _error = '密码错误，还可尝试 $remaining 次';
      });
    }
  }

  String _formatDuration(int totalSeconds) {
    final hours = totalSeconds ~/ 3600;
    final minutes = (totalSeconds % 3600) ~/ 60;
    final seconds = totalSeconds % 60;
    if (hours > 0) return '$hours 小时 $minutes 分 $seconds 秒';
    if (minutes > 0) return '$minutes 分 $seconds 秒';
    return '$seconds 秒';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: widget.mode != PinInputMode.verify
          ? AppBar(title: Text(_title), centerTitle: true)
          : null,
      body: SafeArea(
        child: _locked ? _buildLockedView() : _buildPinView(),
      ),
    );
  }

  Widget _buildLockedView() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 80,
              height: 80,
              decoration: BoxDecoration(
                color: AppTheme.danger.withAlpha(20),
                borderRadius: BorderRadius.circular(20),
              ),
              child: const Icon(Icons.lock_clock_rounded,
                  size: 40, color: AppTheme.danger),
            ),
            const SizedBox(height: 28),
            const Text(
              '应用已锁定',
              style: TextStyle(
                fontSize: 22,
                fontWeight: FontWeight.w700,
                color: AppTheme.textPrimary,
              ),
            ),
            const SizedBox(height: 12),
            Text(
              '连续验证错误次数过多\n请在 ${_formatDuration(_remainingSeconds)} 后重试',
              textAlign: TextAlign.center,
              style: const TextStyle(
                fontSize: 14,
                color: AppTheme.textSecondary,
                height: 1.5,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPinView() {
    return Column(
      children: [
        const Spacer(flex: 2),
        if (widget.mode == PinInputMode.verify) ...[
          Container(
            width: 64,
            height: 64,
            decoration: BoxDecoration(
              gradient: AppTheme.primaryGradient,
              borderRadius: BorderRadius.circular(16),
              boxShadow: [
                BoxShadow(
                  color: AppTheme.primary.withAlpha(50),
                  blurRadius: 20,
                  offset: const Offset(0, 8),
                ),
              ],
            ),
            child: const Icon(Icons.lock_outline_rounded,
                size: 30, color: Colors.white),
          ),
          const SizedBox(height: 20),
        ],
        Text(
          widget.mode == PinInputMode.verify ? _title : _subtitle,
          style: const TextStyle(
            fontSize: 16,
            color: AppTheme.textSecondary,
            letterSpacing: 0.5,
          ),
        ),
        if (widget.mode == PinInputMode.setup && _firstPin == null) ...[
          const SizedBox(height: 10),
          Container(
            margin: const EdgeInsets.symmetric(horizontal: 48),
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
            decoration: AppTheme.bannerDecoration(AppTheme.warning),
            child: Text(
              '请牢记密码。忘记密码将清空所有数据。',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 12,
                color: AppTheme.warning,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
        ],
        const SizedBox(height: 36),
        // PIN 圆点
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: List.generate(pinLength, (i) {
            final filled = i < _pin.length;
            return AnimatedContainer(
              duration: const Duration(milliseconds: 150),
              margin: const EdgeInsets.symmetric(horizontal: 10),
              width: filled ? 18 : 14,
              height: filled ? 18 : 14,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: filled ? AppTheme.primary : Colors.transparent,
                border: Border.all(
                  color: filled ? AppTheme.primary : AppTheme.textTertiary,
                  width: 2,
                ),
                boxShadow: filled
                    ? [
                        BoxShadow(
                          color: AppTheme.primary.withAlpha(60),
                          blurRadius: 8,
                        ),
                      ]
                    : null,
              ),
            );
          }),
        ),
        if (_error != null) ...[
          const SizedBox(height: 14),
          Text(
            _error!,
            style: const TextStyle(
                color: AppTheme.danger,
                fontSize: 13,
                fontWeight: FontWeight.w500),
          ),
        ],
        const Spacer(flex: 1),
        _buildKeypad(),
        const SizedBox(height: 32),
      ],
    );
  }

  Widget _buildKeypad() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 48),
      child: Column(
        children: [
          for (final row in [
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9],
          ]) ...[
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: row.map((d) => _key(d)).toList(),
            ),
            const SizedBox(height: 14),
          ],
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              const SizedBox(width: 72, height: 72),
              _key(0),
              SizedBox(
                width: 72,
                height: 72,
                child: IconButton(
                  onPressed: _onDelete,
                  icon: const Icon(Icons.backspace_outlined,
                      size: 22, color: AppTheme.textSecondary),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _key(int digit) {
    return SizedBox(
      width: 72,
      height: 72,
      child: Material(
        color: Colors.transparent,
        shape: const CircleBorder(),
        clipBehavior: Clip.hardEdge,
        child: InkWell(
          onTap: () => _onDigit(digit),
          customBorder: const CircleBorder(),
          splashColor: AppTheme.primary.withAlpha(30),
          child: Container(
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              border: Border.all(
                color: AppTheme.border,
                width: 1,
              ),
            ),
            child: Center(
              child: Text(
                '$digit',
                style: const TextStyle(
                  fontSize: 26,
                  fontWeight: FontWeight.w500,
                  color: AppTheme.textPrimary,
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
