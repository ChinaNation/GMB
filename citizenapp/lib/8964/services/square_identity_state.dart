import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 广场身份状态。
///
/// `owner_account` 固定使用当前钱包账户；`cid_number` 只能从链上
/// `CitizenIdentity::VotingIdentityByAccount` 读取，App 不允许自行传入链上交易。
class SquareIdentityState {
  const SquareIdentityState({
    required this.ownerAccount,
    this.walletName,
    this.cidNumber,
    this.walletIndex,
    this.pubkeyHex,
    this.isHotWallet = false,
  });

  final String ownerAccount;
  final String? walletName;
  final String? cidNumber;
  final int? walletIndex;
  final String? pubkeyHex;
  final bool isHotWallet;

  bool get hasWallet => ownerAccount.isNotEmpty;
  bool get isCertified => cidNumber != null && cidNumber!.isNotEmpty;

  String get accountLabel {
    if (!hasWallet) return '未选择钱包';
    if (ownerAccount.length <= 14) return ownerAccount;
    return '${ownerAccount.substring(0, 7)}...${ownerAccount.substring(ownerAccount.length - 7)}';
  }
}

class SquareIdentityService {
  const SquareIdentityService({
    this.walletManager,
    this.chainService,
  });

  final WalletManager? walletManager;
  final SquareChainService? chainService;

  Future<SquareIdentityState> loadCurrent() async {
    // 发动态身份统一取默认用户钱包（列表中最靠前的热钱包），与聊天同源。
    final wallet = await (walletManager ?? WalletManager()).getDefaultWallet();
    if (wallet == null) {
      return const SquareIdentityState(ownerAccount: '');
    }
    String? cidNumber;
    try {
      cidNumber = await (chainService ?? SquareChainService())
          .fetchNormalCitizenCidNumber(wallet.address);
    } catch (_) {
      cidNumber = null;
    }

    return SquareIdentityState(
      ownerAccount: wallet.address,
      walletName: wallet.walletName,
      cidNumber: cidNumber,
      walletIndex: wallet.walletIndex,
      pubkeyHex: wallet.pubkeyHex,
      isHotWallet: wallet.isHotWallet,
    );
  }
}
