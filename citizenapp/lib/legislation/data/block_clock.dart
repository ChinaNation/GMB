// 块号 → 近似日期(ADR-028 P3)——法律 `published_at/effective_at` 是块号,
// 经 `GenesisPalletApi.target_block_time_ms` 换算成可读日期。
//
// 中文注释:以当前 finalized 块为锚(≈now),`date(b) = now + (b-current)*interval`。
// interval 与 current 每实例只取一次(页面短生命周期);只读、不增负载。

import 'dart:typed_data';

import 'package:citizenapp/rpc/runtime_api.dart';

class BlockClock {
  BlockClock({RuntimeApi? api}) : _api = api ?? RuntimeApi();

  final RuntimeApi _api;

  /// 目标出块间隔(ms);读不到回退 6000。
  static const int _fallbackIntervalMs = 6000;

  int? _intervalMs;
  int? _currentBlock;

  /// 块号 → 近似日期(过去块=已过,未来块=将至)。读链失败回退 now。
  Future<DateTime> dateOf(int block) async {
    try {
      _intervalMs ??= await _fetchInterval();
      _currentBlock ??= await _api.finalizedBlockNumber();
      final deltaMs = (block - _currentBlock!) * _intervalMs!;
      return DateTime.now().add(Duration(milliseconds: deltaMs));
    } on Object {
      return DateTime.now();
    }
  }

  Future<int> _fetchInterval() async {
    final raw =
        await _api.call('GenesisPalletApi_target_block_time_ms', Uint8List(0));
    if (raw == null || raw.isEmpty) return _fallbackIntervalMs;
    var v = 0;
    for (var k = 0; k < 8 && k < raw.length; k++) {
      v |= raw[k] << (8 * k);
    }
    return v == 0 ? _fallbackIntervalMs : v;
  }

  /// 日期格式化 `YYYY-MM-DD`。
  static String formatDate(DateTime d) {
    final m = d.month.toString().padLeft(2, '0');
    final day = d.day.toString().padLeft(2, '0');
    return '${d.year}-$m-$day';
  }
}
