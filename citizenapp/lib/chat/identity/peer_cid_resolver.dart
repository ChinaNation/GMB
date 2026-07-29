import '../../my/myid/citizen_identity_chain_reader.dart';

/// 把对端钱包账户 account_id 解析成其身份主键 CID 号（Chat 路由键）。
///
/// 二维码等账户边界输入只有 account_id；进入 CID 关系或传输层前必须解析成
/// cid_number。解析顺序：
///   1. 进程内缓存（一次会话内同一对端只链读一次）；
///   2. 链读 [CitizenIdentityChainReader.readByAccountId]，取其 `cidNumber`；
///   3. 成功后只回写进程缓存。
///
/// 对端未绑定 CID（`readByAccountId` 返回 `null` 或 CID 空）时**显式抛错**，
/// 绝不静默错投。见 memory `citizenapp-cid-identity-master-key`。
class PeerCidResolver {
  PeerCidResolver({
    CitizenIdentityChainReader? chainReader,
  }) : _chainReader = chainReader ?? CitizenIdentityChainReader();

  final CitizenIdentityChainReader _chainReader;

  /// 进程内 account_id → cid_number 缓存（会话级；换账户/重启后自然失效）。
  final Map<String, String> _cache = <String, String>{};

  /// 解析对端身份主键 CID 号；未绑定 CID 时抛错。
  Future<String> resolve(String accountId) async {
    final cached = _cache[accountId];
    if (cached != null && cached.isNotEmpty) {
      return cached;
    }
    final snapshot = await _chainReader.readByAccountId(accountId);
    final cidNumber = snapshot?.cidNumber ?? '';
    if (cidNumber.isEmpty) {
      throw StateError('对方尚未绑定身份（CID）');
    }
    _cache[accountId] = cidNumber;
    return cidNumber;
  }
}
