// 统一机构仓库门面(ADR-028 决策 2)——目录(CID-BFF + Isar)产出统一 [Institution];
// 为固定治理档(NRC/PRC/PRB)附 china 固定账户(行为保持)。
//
// 中文注释:
// - 包装现有 [PublicInstitutionRepository](已是本地优先秒开的目录仓库),逐步替代
//   公权/治理两套并行数据源。目录已含 NRC/PRC/PRB(NRC×1/PRC×43/PRB×43 已 seed),
//   故治理身份不再依赖静态注册表。
// - 静态注册表(kNationalCouncil/…)P1 起只保留「固定治理档账户来源」一职——其
//   china 固定账户 hex 不可派生,附到对应机构上;注册表的「列表/详情」角色后续删除。
// - 订阅、行政区所属地 join 等读路径直接复用底层 [directory],不另造。

import 'package:citizenapp/citizen/institution/institution.dart';
import 'package:citizenapp/citizen/public/data/area_path_formatter.dart';
import 'package:citizenapp/citizen/public/data/public_institution_repository.dart';
import 'package:citizenapp/citizen/public/data/public_provinces.dart';
import 'package:citizenapp/citizen/institution/governance_registry.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/isar/wallet_isar.dart';

class InstitutionRepository {
  InstitutionRepository({PublicInstitutionRepository? directory})
      : directory = directory ?? PublicInstitutionRepository();

  /// 底层目录仓库(订阅 / 行政区所属地 join / 后台同步 直接复用)。
  final PublicInstitutionRepository directory;

  /// 固定治理档(NRC/PRC/PRB)按 cidNumber 索引的静态注册表项(构建一次)。
  /// 用途仅两处:① 取 china 固定账户附到统一机构上;② 治理机构详情页 dispatch
  /// 到现成治理发起/投票/管理员页时需要 `InstitutionInfo` 入参(P1 复用,不重写)。
  static final Map<String, InstitutionInfo> _governanceInfo = {
    for (final i in <InstitutionInfo>[
      ...kNationalCouncil,
      ...kProvincialCouncils,
      ...kProvincialBanks,
    ])
      i.cidNumber: i,
  };

  /// 治理机构(NRC/PRC/PRB)的静态注册表项;非治理机构返回 null。
  InstitutionInfo? governanceInfo(String cidNumber) =>
      _governanceInfo[cidNumber];

  /// 按 CID 号取统一机构(目录身份 + 固定治理档附 china 账户)。
  /// 目录未命中时,治理机构回退静态注册表,保证治理 tab 不丢机构(行为保持)。
  Future<Institution?> getByCid(String cidNumber) async {
    final entity = await directory.getByCid(cidNumber);
    if (entity != null) return _toInstitution(entity);
    final gov = _governanceInfo[cidNumber];
    if (gov != null) return Institution.fromGovernanceInfo(gov);
    return null;
  }

  /// 某市机构列表(统一 Institution)。
  Future<List<Institution>> listInstitutionsByCity(
    String provinceCode,
    String cityCode,
  ) async {
    final rows = await directory.listInstitutionsByCity(provinceCode, cityCode);
    return rows.map(_toInstitution).toList(growable: false);
  }

  /// 按机构码集合取统一机构列表(治理/立法 tab 视图过滤入口,ADR-028 P2)。
  Future<List<Institution>> listByCodes(Set<String> institutionCodes) async {
    final rows = await directory.listByInstitutionCodes(institutionCodes);
    return rows.map(_toInstitution).toList(growable: false);
  }

  /// 某省内按机构码集合取统一机构列表(立法 tab 省导航,ADR-028 P3)。
  Future<List<Institution>> listByProvinceAndCodes(
    String provinceCode,
    Set<String> institutionCodes,
  ) async {
    final rows =
        await directory.listByProvinceAndCodes(provinceCode, institutionCodes);
    return rows.map(_toInstitution).toList(growable: false);
  }

  /// 某钱包关注的机构列表(统一 Institution)。
  Future<List<Institution>> listSubscribed(String walletPubkeyHex) async {
    final rows = await directory.listSubscribed(walletPubkeyHex);
    return rows.map(_toInstitution).toList(growable: false);
  }

  // ── 订阅(关注)passthrough ──
  Future<bool> isSubscribed(String walletPubkeyHex, String cidNumber) =>
      directory.isSubscribed(walletPubkeyHex, cidNumber);
  Future<void> subscribe(String walletPubkeyHex, String cidNumber) =>
      directory.subscribe(walletPubkeyHex, cidNumber);
  Future<void> unsubscribe(String walletPubkeyHex, String cidNumber) =>
      directory.unsubscribe(walletPubkeyHex, cidNumber);

  /// 机构所属地显示路径(详情页 所属地行;省名带"省"全名 + 字典市/镇名,ADR-021)。
  Future<String> institutionAreaPath(Institution inst) {
    return formatAreaPath(
      directory.divisionStore,
      provinceName: provinceFullNameByCode(inst.provinceCode),
      provinceCode: inst.provinceCode,
      cityCode: inst.cityCode,
      townCode: inst.townCode,
    );
  }

  Institution _toInstitution(PublicInstitutionEntity e) {
    final inst = Institution.fromPublicEntity(e);
    if (inst.isFixedGovernance) {
      final baked = _governanceInfo[inst.cidNumber]?.accounts;
      if (baked != null) return inst.withBuiltinAccounts(baked);
    }
    return inst;
  }
}
