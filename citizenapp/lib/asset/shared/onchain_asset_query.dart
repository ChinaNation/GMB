// 链上发行代币 storage 查询封装(框架阶段占位)。
//
// 后续任务卡 C 实装时承载:
// - 通过 substrate_api / smoldot 读 OnchainIssuance.Assets storage(asset_id → AssetMeta)
// - 读 pallet_assets::Asset(asset_id)获得 supply / accounts 数量
// - 读 pallet_assets::Account(asset_id, who)获得持币余额
// - 订阅 OnchainIssuance::Event 推送 AssetIssued / Minted / MonitorFrozen 等
//
// 当前框架阶段只声明 service 类骨架 + 查询方法签名,实际实现等业务接入。

import 'dart:typed_data';

class OnchainAssetMeta {
  const OnchainAssetMeta({
    required this.assetId,
    required this.issuerAccountId,
    required this.issuerAccount,
    required this.classKind,
    required this.decimals,
    required this.state,
    required this.name,
    required this.symbol,
  });

  /// pallet_assets AssetId。
  final int assetId;
  final Uint8List issuerAccountId;

  /// 发行机构多签账户 SS58(prefix=2027)。
  final String issuerAccount;

  /// 资产种类:'Plain' / 'Pegged'(第一期只 Plain)。
  final String classKind;

  /// 小数位 0..=18。
  final int decimals;

  /// 'Active' / 'Closed' / 'ForceClosed'。
  final String state;

  final String name;
  final String symbol;
}

// 中文注释：监管账户由 NRC 全局治理账户决定，AssetMeta 不再逐资产保存监管主体字段。

abstract class OnchainAssetQuery {
  /// 列出指定持币人(SS58)持有的所有资产 + 当前余额。
  Future<List<OnchainAssetMeta>> listAssetsHeldBy(String ss58);

  /// 按 asset_id 读资产元数据。
  Future<OnchainAssetMeta?> readAssetById(int assetId);

  /// 读特定持币人在某资产下的当前余额(已含 decimals 的 raw 值)。
  Future<BigInt> readBalance({required int assetId, required String ss58});
}
