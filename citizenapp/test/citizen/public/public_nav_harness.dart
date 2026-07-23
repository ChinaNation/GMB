// 卡B widget 测试脚手架:构造无网络/无 Isar 的 repo(seeded fake store)。
//
// ADR-021:机构只存 code,行政区名字来自字典。harness 同时 seed 机构 fake store +
// 行政区字典 fake store(市名 join 用),并按 provinceCode 落库/查询。

import 'package:flutter/services.dart';
import 'package:citizenapp/citizen/public/data/admin_division_bundle_loader.dart';
import 'package:citizenapp/citizen/public/data/admin_division_dto.dart';
import 'package:citizenapp/citizen/public/data/public_institution_bundle_loader.dart';
import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';
import 'package:citizenapp/citizen/public/data/public_institution_repository.dart';

import 'fake_admin_division_store.dart';
import 'fake_data_version_kv.dart';
import 'fake_public_institution_store.dart';

class _EmptyBundle extends AssetBundle {
  @override
  Future<ByteData> load(String key) async => throw UnimplementedError();
  @override
  Future<String> loadString(String key, {bool cache = true}) async =>
      throw Exception('no bundle');
}

/// 构造机构 seed DTO。行政区只带 code(ADR-021);[cityName] 仅供 harness 同步
/// seed 字典(市名 join),不进机构记录。
PublicInstitutionDto seedDto(
  String cid, {
  required String provinceCode,
  required String cityCode,
  String? name,
  String code = 'ZF',
  String townCode = '',
}) =>
    PublicInstitutionDto.fromJson(<String, dynamic>{
      'cid_number': cid,
      'cid_full_name': name ?? '$cityCode$code机构',
      'province_code': provinceCode,
      'city_code': cityCode,
      'town_code': townCode,
      'institution_code': code,
      'account_count': 2,
    });

/// 构造 seeded 仓库:省顺序(省 code)+ 机构 + 各省链快照版本戳+
/// 行政区字典(市/镇名 join)。
///
/// [provinceOrder] 传省 code(如 ['ZS']);[cityNames] = `cityCode -> 市名`,
/// [townNames] = `"<cityCode>|<townCode>" -> 镇名`,按需 seed 字典。
Future<PublicInstitutionRepository> buildSeededRepo({
  required List<String> provinceOrder,
  required List<PublicInstitutionDto> institutions,
  Map<String, String>? subscriptions, // publicKey -> cid
  Map<String, String>? cityNames, // "<pcode>|<ccode>" -> name
  Map<String, String>? townNames, // "<pcode>|<ccode>|<tcode>" -> name
}) async {
  final store = FakePublicInstitutionStore();
  final divisionStore = FakeAdminDivisionStore();
  await store.setProvinceOrder(provinceOrder);
  await store.upsertInstitutions(institutions, catalogVersion: 'seed');
  for (final p in provinceOrder) {
    await store.setProvinceVersion(p, 'seed');
  }
  // seed 市名字典。
  cityNames?.forEach((key, name) {
    final parts = key.split('|');
    divisionStore.seed(AdminDivisionDto(
      level: AdminDivisionLevel.city,
      provinceCode: parts[0],
      cityCode: parts.length > 1 ? parts[1] : '',
      code: parts.length > 1 ? parts[1] : '',
      divisionName: name,
    ));
  });
  // seed 镇名字典。
  townNames?.forEach((key, name) {
    final parts = key.split('|');
    divisionStore.seed(AdminDivisionDto(
      level: AdminDivisionLevel.town,
      provinceCode: parts[0],
      cityCode: parts.length > 1 ? parts[1] : '',
      townCode: parts.length > 2 ? parts[2] : '',
      code: parts.length > 2 ? parts[2] : '',
      divisionName: name,
    ));
  });
  if (subscriptions != null) {
    for (final entry in subscriptions.entries) {
      await store.subscribe(entry.key, entry.value);
    }
  }
  // 全部走内存 fake(含版本游标),ensureSynced 不触碰全局 Isar。
  final divisionLoader = AdminDivisionBundleLoader(
    store: divisionStore,
    bundle: _EmptyBundle(),
    versionKv: FakeDataVersionKv(),
  );
  return PublicInstitutionRepository(
    store: store,
    divisionStore: divisionStore,
    loader: PublicInstitutionBundleLoader(
      store: store,
      bundle: _EmptyBundle(),
      divisionLoader: divisionLoader,
      versionKv: FakeDataVersionKv(),
    ),
  );
}

/// 测试用 repo:行政区字典「延迟就绪」——`ensureSynced` 等 [gate] 打开后才把市名
/// seed 进字典 store(模拟首装时 4.2 万条字典还在灌库)。用于回归「灌库未完成→市名
/// 回退 code(001),同步完成后必须清脏缓存回刷成市名」的时序(见任务卡
/// 20260623-citizenapp-public-city-001-timing-fix)。
class LateDictRepo extends PublicInstitutionRepository {
  LateDictRepo({
    required super.store,
    required super.divisionStore,
    required super.loader,
    required this.gate,
    required this.lateCityNames,
    required FakeAdminDivisionStore divisionStoreRef,
  }) : _ds = divisionStoreRef;

  /// 字典灌库完成信号:打开前 `cityNameMap` 查空 → 市名回退 code。
  final Future<void> gate;

  /// `"<pcode>|<ccode>" -> 市名`,在 [gate] 打开后才 seed(模拟字典刚灌完)。
  final Map<String, String> lateCityNames;
  final FakeAdminDivisionStore _ds;

  @override
  Future<bool> ensureSynced() async {
    await gate;
    lateCityNames.forEach((key, name) {
      final parts = key.split('|');
      _ds.seed(AdminDivisionDto(
        level: AdminDivisionLevel.city,
        provinceCode: parts[0],
        cityCode: parts.length > 1 ? parts[1] : '',
        code: parts.length > 1 ? parts[1] : '',
        divisionName: name,
      ));
    });
    return true;
  }
}

/// 构造「字典延迟就绪」仓库:机构先就绪、字典等 [gate] 打开才 seed。
Future<LateDictRepo> buildLateDictRepo({
  required List<String> provinceOrder,
  required List<PublicInstitutionDto> institutions,
  required Map<String, String> lateCityNames,
  required Future<void> gate,
}) async {
  final store = FakePublicInstitutionStore();
  final divisionStore = FakeAdminDivisionStore();
  await store.setProvinceOrder(provinceOrder);
  await store.upsertInstitutions(institutions, catalogVersion: 'seed');
  for (final p in provinceOrder) {
    await store.setProvinceVersion(p, 'seed');
  }
  final divisionLoader = AdminDivisionBundleLoader(
    store: divisionStore,
    bundle: _EmptyBundle(),
    versionKv: FakeDataVersionKv(),
  );
  return LateDictRepo(
    store: store,
    divisionStore: divisionStore,
    loader: PublicInstitutionBundleLoader(
      store: store,
      bundle: _EmptyBundle(),
      divisionLoader: divisionLoader,
      versionKv: FakeDataVersionKv(),
    ),
    gate: gate,
    lateCityNames: lateCityNames,
    divisionStoreRef: divisionStore,
  );
}
