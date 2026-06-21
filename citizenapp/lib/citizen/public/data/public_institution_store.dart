// 公权机构目录本地存储抽象(ADR-018 §九)。
//
// 中文注释:抽象出存储接口,使同步/载入服务的逻辑可用内存 fake 单测,
// 不依赖 Isar 真库;生产实现见 isar_public_institution_store.dart。
// 全部为本地读写,UI 导航零链读零现查。

import 'package:citizenapp/isar/wallet_isar.dart';

import 'public_institution_dto.dart';

abstract interface class PublicInstitutionStore {
  /// 幂等 upsert 一批机构(按 sfid_number 唯一)。
  Future<void> upsertInstitutions(
    List<PublicInstitutionDto> items, {
    required String catalogVersion,
  });

  /// 设置省份规范顺序(中枢 + 43 省 code,来自数据包 manifest)。
  Future<void> setProvinceOrder(List<String> provinceCodes);

  /// 省份规范顺序(省 code);无 manifest 时回退已落库机构的去重省 code。
  Future<List<String>> listProvinces();

  /// 某省全部市 code(按 cityCode 去重,顺序稳定)。
  ///
  /// 中文注释(ADR-021):镇 code 全国不唯一,但市 code 在省内唯一;按 code 去重,
  /// 名字由调用方查字典 join。
  Future<List<String>> listCities(String provinceCode);

  /// 某省某市全部公权机构(按 provinceCode + cityCode)。
  Future<List<PublicInstitutionEntity>> listInstitutionsByCity(
    String provinceCode,
    String cityCode,
  );

  /// 按 sfid_number 取单个机构。
  Future<PublicInstitutionEntity?> getBySfid(String sfidNumber);

  /// 某省(省 code)全部机构实体(按 provinceCode 索引查)。
  ///
  /// 中文注释(增量 reconcile 用):供 loader 逐条比对同 sfid 内容,只 upsert 真正
  /// 改名/新增的行,再删除包里已没有的废 sfid,零旧数据残留。
  Future<List<PublicInstitutionEntity>> institutionsOfProvince(
    String provinceCode,
  );

  /// 某省(省 code)全部机构 sfid_number。
  Future<List<String>> sfidsOfProvince(String provinceCode);

  /// 按 sfid_number 批量删(分块,事务内)。
  Future<void> deleteBySfids(List<String> sfids);

  /// 已落库机构总数(判断是否需要首次载入数据包)。
  Future<int> institutionCount();

  /// 取某省(省 code)已同步版本戳。
  Future<String?> provinceVersion(String provinceCode);

  /// 写某省(省 code)已同步版本戳。
  Future<void> setProvinceVersion(String provinceCode, String version);

  // ── 订阅("关注")——按钱包公钥隔离 ──

  Future<void> subscribe(String walletPubkeyHex, String sfidNumber);

  Future<void> unsubscribe(String walletPubkeyHex, String sfidNumber);

  Future<bool> isSubscribed(String walletPubkeyHex, String sfidNumber);

  /// 我订阅的机构(关注分组),跨省扁平。
  Future<List<PublicInstitutionEntity>> listSubscribed(String walletPubkeyHex);
}

/// 订阅复合唯一键:`pubkeyHex|sfidNumber`。
String subscriptionKeyOf(String walletPubkeyHex, String sfidNumber) =>
    '$walletPubkeyHex|$sfidNumber';
