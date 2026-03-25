/// 本地 nonce 管理器。
///
/// 轻节点（smoldot）查询 nonce 只能获取链上已确认的值，无法感知全节点交易池。
/// 连续提交多笔交易时，第二笔会拿到和第一笔相同的 nonce，被链拒绝。
///
/// NonceManager 在客户端记住"已分配但尚未上链"的 nonce：
/// - 每次分配取 max(链上 nonce, 本地记录)
/// - 分配后本地记录 +1
/// - 提交失败时回退（rollback）
/// - 交易上链确认后清除本地记录（reset）
class NonceManager {
  static final instance = NonceManager._();
  NonceManager._();

  /// 每个地址的下一个可用 nonce（本地记录）。
  final _nextNonce = <String, int>{};

  /// 获取下一个可用 nonce。
  ///
  /// 先从链上查询已确认的 nonce，再和本地记录取较大值，
  /// 确保不会分配重复的 nonce。
  Future<int> getNextNonce({
    required String address,
    required Future<int> Function(String) fetchChainNonce,
  }) async {
    final chainNonce = await fetchChainNonce(address);
    final localNext = _nextNonce[address];

    int nonce;
    if (localNext != null && localNext > chainNonce) {
      nonce = localNext;
    } else {
      nonce = chainNonce;
    }

    _nextNonce[address] = nonce + 1;
    return nonce;
  }

  /// 提交失败时回退，归还已分配的 nonce。
  void rollback(String address) {
    final current = _nextNonce[address];
    if (current != null && current > 0) {
      _nextNonce[address] = current - 1;
    }
  }

  /// 交易上链确认后清除本地记录，下次从链上重新获取。
  void reset(String address) {
    _nextNonce.remove(address);
  }
}
