// 公权机构目录本地存储抽象(ADR-018 §九)。
//
// 抽象出存储接口,使同步/载入服务的逻辑可用内存 fake 单测,
// 不依赖 Isar 真库;生产实现见 isar_public_institution_store.dart。
// 全部为本地读写,UI 导航零链读零现查。

import 'package:citizenapp/isar/wallet_isar.dart';

import 'public_institution_dto.dart';

abstract interface class PublicInstitutionStore {
  /// 幂等 upsert 一批机构(按 cid_number 唯一)。
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
  /// (ADR-021):镇 code 全国不唯一,但市 code 在省内唯一;按 code 去重,
  /// 名字由调用方查字典 join。
  Future<List<String>> listCities(String provinceCode);

  /// 某省某市全部公权机构(按 provinceCode + cityCode)。
  Future<List<PublicInstitutionEntity>> listInstitutionsByCity(
    String provinceCode,
    String cityCode,
  );

  /// 按 cid_number 取单个机构。
  Future<PublicInstitutionEntity?> getByCid(String cidNumber);

  /// 按机构码集合取全部机构(跨省扁平;institutionCode 索引 anyOf 查,非全表扫)。
  ///
  /// (ADR-028 P2):五子 tab 的治理/立法等机构视图按 institution_code
  /// 过滤统一目录的入口。
  Future<List<PublicInstitutionEntity>> listByInstitutionCodes(
    Set<String> institutionCodes,
  );

  /// 某省内按机构码集合取机构(provinceCode 索引 + institutionCode anyOf)。
  ///
  /// (ADR-028 P3):立法 tab 省导航选某省后,取该省 省立法院/省议会 +
  /// 全部市的市立法会。
  Future<List<PublicInstitutionEntity>> listByProvinceAndCodes(
    String provinceCode,
    Set<String> institutionCodes,
  );

  /// 某省(省 code)全部机构实体(按 provinceCode 索引查)。
  ///
  /// (增量 reconcile 用):供 loader 逐条比对同 cid 内容,只 upsert 真正
  /// 改名/新增的行,再删除包里已没有的废 cid,零旧数据残留。
  Future<List<PublicInstitutionEntity>> institutionsOfProvince(
    String provinceCode,
  );

  /// 某省(省 code)全部机构 cid_number。
  Future<List<String>> cidsOfProvince(String provinceCode);

  /// 按 cid_number 批量删(分块,事务内)。
  Future<void> deleteByCids(List<String> cids);

  /// 已落库机构总数(判断是否需要首次载入数据包)。
  Future<int> institutionCount();

  /// 取某省(省 code)已同步版本戳。
  Future<String?> provinceVersion(String provinceCode);

  /// 写某省(省 code)已同步版本戳。
  Future<void> setProvinceVersion(String provinceCode, String version);

  // ── 订阅("关注")——按钱包公钥隔离 ──

  Future<void> subscribe(String walletPubkeyHex, String cidNumber);

  Future<void> unsubscribe(String walletPubkeyHex, String cidNumber);

  Future<bool> isSubscribed(String walletPubkeyHex, String cidNumber);

  /// 我订阅的机构(关注分组),跨省扁平。
  Future<List<PublicInstitutionEntity>> listSubscribed(String walletPubkeyHex);
}

/// 订阅复合唯一键:`pubkeyHex|cidNumber`。
String subscriptionKeyOf(String walletPubkeyHex, String cidNumber) =>
    '$walletPubkeyHex|$cidNumber';
