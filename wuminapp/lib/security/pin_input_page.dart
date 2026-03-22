import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'app_lock_service.dart';

/// PIN 输入模式。
enum PinInputMode {
  /// 设置新 PIN：输入两次。
  setup,

  /// 验证 PIN：输入一次。
  verify,

  /// 关闭 PIN：输入一次验证后删除。
  remove,
}

/// 6 位 PIN 输入页面。
///
/// [mode] 决定行为：
/// - [PinInputMode.setup]：输入两次设置 PIN，成功 pop(true)
/// - [PinInputMode.verify]：输入一次验证，成功 pop(true)
/// - [PinInputMode.remove]：输入一次验证后删除 PIN，成功 pop(true)
class PinInputPage extends StatefulWidget {
  const PinInputPage({super.key, required this.mode});

  final PinInputMode mode;

  @override
  State<PinInputPage> createState() => _PinInputPageState();
}

class _PinInputPageState extends State<PinInputPage> {
  static const int pinLength = 6;

  String _pin = '';
  String? _firstPin; // setup 模式下第一次输入的 PIN
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
    // 先检查是否被锁定
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
      // 第一次输入
      _firstPin = _pin;
      setState(() => _pin = '');
      _updateTitle();
    } else {
      // 第二次输入
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

    // 检查是否被锁定
    final locked = await AppLockService.isLocked();
    if (locked) {
      // 检查是否已清空数据（lockCount >= 3 时 verifyPin 内部已调用 wipeAllData）
      final pinSet = await AppLockService.isPinSet();
      if (!pinSet) {
        // 数据已清空，需要重启应用
        if (!mounted) return;
        await showDialog<void>(
          context: context,
          barrierDismissible: false,
          builder: (_) => AlertDialog(
            title: const Text('数据已清空'),
            content: const Text('连续多次验证错误，应用数据已全部清空。请重新启动应用。'),
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
    if (hours > 0) {
      return '$hours 小时 $minutes 分 $seconds 秒';
    }
    if (minutes > 0) {
      return '$minutes 分 $seconds 秒';
    }
    return '$seconds 秒';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: widget.mode != PinInputMode.verify
          ? AppBar(
              title: Text(_title),
              centerTitle: true,
            )
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
            const Icon(Icons.lock_clock, size: 64, color: Colors.red),
            const SizedBox(height: 24),
            const Text(
              '应用已锁定',
              style: TextStyle(fontSize: 20, fontWeight: FontWeight.w700),
            ),
            const SizedBox(height: 12),
            Text(
              '连续验证错误次数过多\n请在 ${_formatDuration(_remainingSeconds)} 后重试',
              textAlign: TextAlign.center,
              style: const TextStyle(fontSize: 14, color: Colors.grey),
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
          const Icon(Icons.lock_outline, size: 48, color: Color(0xFF007A74)),
          const SizedBox(height: 16),
        ],
        Text(
          widget.mode == PinInputMode.verify ? _title : _subtitle,
          style: const TextStyle(fontSize: 16, color: Colors.black54),
        ),
        if (widget.mode == PinInputMode.setup && _firstPin == null) ...[
          const SizedBox(height: 8),
          Container(
            margin: const EdgeInsets.symmetric(horizontal: 48),
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: Colors.orange.shade50,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: Colors.orange.shade200),
            ),
            child: Text(
              '请牢记密码。忘记密码将清空所有数据。',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 12,
                color: Colors.orange.shade800,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
        ],
        const SizedBox(height: 32),
        // PIN 圆点指示器
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: List.generate(pinLength, (i) {
            final filled = i < _pin.length;
            return Container(
              margin: const EdgeInsets.symmetric(horizontal: 8),
              width: 16,
              height: 16,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: filled ? const Color(0xFF007A74) : Colors.transparent,
                border: Border.all(
                  color: const Color(0xFF007A74),
                  width: 2,
                ),
              ),
            );
          }),
        ),
        if (_error != null) ...[
          const SizedBox(height: 12),
          Text(
            _error!,
            style: const TextStyle(color: Colors.red, fontSize: 13),
          ),
        ],
        const Spacer(flex: 1),
        // 数字键盘
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
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [_key(1), _key(2), _key(3)],
          ),
          const SizedBox(height: 16),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [_key(4), _key(5), _key(6)],
          ),
          const SizedBox(height: 16),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [_key(7), _key(8), _key(9)],
          ),
          const SizedBox(height: 16),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              // 空白占位
              const SizedBox(width: 72, height: 72),
              _key(0),
              // 删除键
              SizedBox(
                width: 72,
                height: 72,
                child: IconButton(
                  onPressed: _onDelete,
                  icon: const Icon(Icons.backspace_outlined, size: 24),
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
          child: Center(
            child: Text(
              '$digit',
              style: const TextStyle(
                fontSize: 28,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
