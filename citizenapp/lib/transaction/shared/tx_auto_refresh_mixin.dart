import 'dart:async';

import 'package:flutter/widgets.dart';

import 'package:citizenapp/transaction/shared/local_tx_store.dart';

/// 交易记录展示页共用:订阅某账户在 Isar 里的交易记录变更,后台
/// [ChainTxMonitor] 一把记录写成 finalized(已确认),列表就自动重刷 ——
/// 取代"提交后延时 N 秒盲刷"。子类给出重载动作,mixin 负责订阅/去抖/取消。
///
/// 用法:
/// ```dart
/// class _FooState extends State<Foo> with TxAutoRefreshMixin<Foo> {
///   @override
///   void initState() {
///     super.initState();
///     startTxAutoRefresh(accountId);
///   }
///   @override
///   Future<void> onTxRecordsChanged() => _loadRecords();
///   @override
///   void dispose() { stopTxAutoRefresh(); super.dispose(); }
/// }
/// ```
mixin TxAutoRefreshMixin<T extends StatefulWidget> on State<T> {
  static const Duration _debounce = Duration(milliseconds: 200);

  StreamSubscription<void>? _txAutoSub;
  Timer? _txAutoDebounce;
  String? _txAutoAccountId;

  /// 库变更时执行的重载(通常是页面现成的 `_loadXxx`)。
  Future<void> onTxRecordsChanged();

  /// 开始/切换监听指定账户的交易记录变更;账户为空则停止监听。
  /// 重复传同一账户是幂等空操作(不重复订阅)。
  void startTxAutoRefresh(String? accountId) {
    String? normalized;
    try {
      normalized = (accountId == null || accountId.isEmpty)
          ? null
          : LocalTxStore.requireAccountId(accountId);
    } on Object {
      normalized = null;
    }
    if (normalized == _txAutoAccountId && _txAutoSub != null) return;
    _txAutoAccountId = normalized;
    _txAutoSub?.cancel();
    _txAutoSub = null;
    if (normalized == null) return;
    _txAutoSub = LocalTxStore.watchAccountChanges(normalized).listen((_) {
      _txAutoDebounce?.cancel();
      _txAutoDebounce = Timer(_debounce, () {
        if (!mounted) return;
        unawaited(onTxRecordsChanged());
      });
    });
  }

  /// 停止监听并释放定时器(在 `dispose` 里调)。
  void stopTxAutoRefresh() {
    _txAutoDebounce?.cancel();
    _txAutoDebounce = null;
    _txAutoSub?.cancel();
    _txAutoSub = null;
    _txAutoAccountId = null;
  }
}
