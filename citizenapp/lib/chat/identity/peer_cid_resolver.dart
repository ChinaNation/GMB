import 'dart:async';

import '../../my/myid/citizen_identity_chain_reader.dart';
import '../../my/user/contact_service.dart';

/// 把对端钱包账户 account_id 解析成其身份主键 CID 号（Chat 路由键）。
///
/// 前端联系人只有 account_id；进传输层前必须解析成 cid_number。解析顺序：
///   1. 进程内缓存（一次会话内同一对端只链读一次）；
///   2. 链读 [CitizenIdentityChainReader.readByAccountId]，取其 `cidNumber`；
///   3. 成功后回写进程缓存，并尽力持久化到通讯录 `UserContact.cidNumber`（供
///      通讯录/资料页免再链读）。
///
/// 对端未绑定 CID（`readByAccountId` 返回 `null` 或 CID 空）时**显式抛错**，
/// 绝不静默错投。见 memory `citizenapp-cid-identity-master-key`。
class PeerCidResolver {
  PeerCidResolver({
    CitizenIdentityChainReader? chainReader,
    UserContactService? contactService,
  })  : _chainReader = chainReader ?? CitizenIdentityChainReader(),
        _contactService = contactService ?? UserContactService();

  final CitizenIdentityChainReader _chainReader;
  final UserContactService _contactService;

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
      throw StateError('对方尚未绑定身份（CID），暂时无法向其发送消息');
    }
    _cache[accountId] = cidNumber;
    // 尽力把解析结果回写通讯录缓存，供后续免链读；失败不影响本次发送。
    unawaited(
      _contactService
          .cacheContactCidNumber(accountId, cidNumber)
          .catchError((Object _) {}),
    );
    return cidNumber;
  }
}
