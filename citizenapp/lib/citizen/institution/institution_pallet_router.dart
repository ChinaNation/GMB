import 'package:citizenapp/citizen/shared/institution_code_label.dart';

/// 机构生命周期 storage 的 pallet 路由(镜像链端 `PublicManage`(idx30)/`PrivateManage`(idx31) 拆分)。
///
/// 链端机构身份/账户已拆两 pallet,storage 名(`Institutions` /
/// `InstitutionAccounts` / `AccountRegisteredCid` / `CidRegisteredAccount`)不变,
/// 但 storage key 前缀 `twox_128(pallet 名)` 随 pallet 名变。端侧读必须按归属选对前缀:
///
/// - **已知机构码**(读到机构后、或 cid 派生出码):按 [InstitutionCodeLabel.isPrivateLegal]
///   单源路由(与链端 `is_private_legal_code` 逐字对齐;公权/创世/非法人默认 PublicManage,
///   与链端 `institution_manage_pallet` 一致)。
/// - **仅有账户**(反查 `AccountRegisteredCid` account→cid):无法先验公私,链端两 pallet
///   各有独立 `AccountRegisteredCid`,必须对 [managePallets] 双查取命中者,再把命中 pallet
///   贯穿后续 cid 键读(institution / institutionAccount / cidRegisteredAccount)。
///
/// 个人多签不在此路由内:它在 `personal-manage` 独立线(`PersonalAccounts`,链端 personal-manage)。
class InstitutionPalletRouter {
  InstitutionPalletRouter._();

  /// 公权机构生命周期 pallet 名(construct_runtime 别名,= storage 前缀来源)。
  static const String publicManage = 'PublicManage';

  /// 私权机构生命周期 pallet 名。
  static const String privateManage = 'PrivateManage';

  /// 机构身份/账户 storage 的两 pallet;反查 account→cid 时按此顺序双查取首个命中。
  static const List<String> managePallets = [publicManage, privateManage];

  /// 已知机构码时路由:私权法人码→PrivateManage,否则(公权/创世/非法人)→PublicManage。
  ///
  /// 与链端 `node institution_manage_pallet` 同口径:`is_private_legal_code ? Private : Public`。
  static String forInstitutionCode(String institutionCode) {
    return InstitutionCodeLabel.isPrivateLegal(institutionCode)
        ? privateManage
        : publicManage;
  }
}
