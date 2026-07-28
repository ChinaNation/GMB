import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import 'citizen_identity_chain_reader.dart';

/// 身份账户解析结果:CID 绑定的钱包账户 + 其链上身份闭环快照。
class ResolvedIdentity {
  const ResolvedIdentity({
    required this.accountId,
    required this.ss58Address,
    required this.accountIndex,
    required this.snapshot,
  });

  /// CID 绑定的钱包账户(命中者);全未命中(未注册)时回退账户0。
  final String accountId;
  final String ss58Address;
  final int accountIndex;

  /// 链上身份闭环快照;`null` = 未注册(纯访客,回退账户0)。
  final CitizenIdentityChainSnapshot? snapshot;

  /// 是否已占到一个 CID(匿名 / 投票 / 竞选任一皆为 true)。
  bool get isRegistered => snapshot != null;
}

/// 身份账户单源 —— 把「身份主键」从"钱包列表最靠前的热钱包(账户0)"改成
/// **"CID 绑定的那个钱包账户"**(可为任意 `//n`)。
///
/// 见 memory `citizenapp-cid-identity-master-key`:非链功能唯一身份主键 = CID 号,
/// 钱包账户只是与该 CID 绑定的鉴权凭证;鉴权授权取决于 CID 当前绑定了哪个账户。
/// 私钥泄漏可换绑到新账户而 CID(及其通讯录/动态/文章/粉丝)永不丢失。
class IdentityAccountResolver {
  IdentityAccountResolver({
    WalletManager? walletManager,
    CitizenIdentityChainReader? chainReader,
    ChainRpc? chainRpc,
  })  : _walletManager = walletManager ?? WalletManager(),
        _chainReader =
            chainReader ?? CitizenIdentityChainReader(chainRpc: chainRpc);

  final WalletManager _walletManager;
  final CitizenIdentityChainReader _chainReader;

  /// 解析当前热钱包下的身份账户。
  ///
  /// **优先账户0**(常态身份即绑账户0,命中即返回,只 1 次链读);账户0 无 CID 才
  /// 遍历子账户。首个 `readByAccountId` 闭环命中者即身份账户。全未命中 → 回退账户0
  /// (未注册,供门控判"无 CID")。无热钱包 → `null`。
  ///
  /// 链读异常**不吞**(上抛给调用方 fail-closed,绝不静默降级成访客/未注册)。
  Future<ResolvedIdentity?> resolve() async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) return null;

    // 优先账户0(单钱包多账户下身份绝大多数就绑账户0)。
    final snap0 = await _chainReader.readByAccountId(wallet.accountId);
    if (snap0 != null) {
      return ResolvedIdentity(
        accountId: wallet.accountId,
        ss58Address: wallet.ss58Address,
        accountIndex: 0,
        snapshot: snap0,
      );
    }

    // 账户0 未绑 CID → 查其余本地子账户(换绑到 //n 的情形)。
    final accounts = await _walletManager.getAccounts(wallet.accountId);
    for (final account
        in accounts.where((a) => a.accountId != wallet.accountId)) {
      final snap = await _chainReader.readByAccountId(account.accountId);
      if (snap != null) {
        return ResolvedIdentity(
          accountId: account.accountId,
          ss58Address: account.ss58Address,
          accountIndex: account.accountIndex,
          snapshot: snap,
        );
      }
    }

    // 全未命中 → 回退账户0(未注册)。门控据 isRegistered=false 判"无 CID"。
    return ResolvedIdentity(
      accountId: wallet.accountId,
      ss58Address: wallet.ss58Address,
      accountIndex: 0,
      snapshot: null,
    );
  }
}
