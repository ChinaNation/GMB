// 卡B widget 测试脚手架:构造无网络/无 Isar 的 repo(seeded fake store)。
//
// ADR-021:机构只存 code,行政区名字来自字典。harness 同时 seed 机构 fake store +
// 行政区字典 fake store(市名 join 用),并按 provinceCode 落库/查询。

import 'package:flutter/services.dart';
import 'package:citizenapp/citizen/public/data/admin_division_bundle_loader.dart';
import 'package:citizenapp/citizen/public/data/admin_division_dto.dart';
import 'package:citizenapp/citizen/public/data/public_institution_api.dart';
import 'package:citizenapp/citizen/public/data/public_institution_bundle_loader.dart';
import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';
import 'package:citizenapp/citizen/public/data/public_institution_repository.dart';
import 'package:citizenapp/citizen/public/data/public_institution_sync_service.dart';

import 'fake_admin_division_store.dart';
import 'fake_data_version_kv.dart';
import 'fake_public_institution_store.dart';

class _NoopApi extends PublicInstitutionApi {
  _NoopApi() : super(baseUrl: 'http://test');

  @override
  Future<PublicInstitutionVersion> fetchVersion({
    required String provinceName,
    String? cityName,
  }) async =>
      // 版本恒为 'seed',与 seeded store 版本一致 → syncProvince 跳过,不发网络。
      PublicInstitutionVersion(
          provinceName: provinceName, manifestVersion: 'seed');

  @override
  Future<PublicInstitutionPage> fetchPage({
    required String provinceName,
    String? cityName,
    String? sinceVersion,
    String? afterSfid,
    int pageSize = 500,
  }) async =>
      const PublicInstitutionPage(items: [], hasMore: false);
}

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
  String sfid, {
  required String provinceCode,
  required String cityCode,
  String? name,
  String code = 'ZF',
  String townCode = '',
}) =>
    PublicInstitutionDto.fromJson(<String, dynamic>{
      'sfid_number': sfid,
      'sfid_full_name': name ?? '$cityCode$code机构',
      'province_code': provinceCode,
      'city_code': cityCode,
      'town_code': townCode,
      'institution_code': code,
      'account_count': 2,
    });

/// 构造 seeded 仓库:省顺序(省 code)+ 机构 + 各省版本戳(=seed,使同步跳过)+
/// 行政区字典(市/镇名 join)。
///
/// [provinceOrder] 传省 code(如 ['ZS']);[cityNames] = `cityCode -> 市名`,
/// [townNames] = `"<cityCode>|<townCode>" -> 镇名`,按需 seed 字典。
Future<PublicInstitutionRepository> buildSeededRepo({
  required List<String> provinceOrder,
  required List<PublicInstitutionDto> institutions,
  Map<String, String>? subscriptions, // pubkey -> sfid
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
      name: name,
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
      name: name,
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
    sync: PublicInstitutionSyncService(store: store, api: _NoopApi()),
    loader: PublicInstitutionBundleLoader(
      store: store,
      bundle: _EmptyBundle(),
      divisionLoader: divisionLoader,
      versionKv: FakeDataVersionKv(),
    ),
  );
}
