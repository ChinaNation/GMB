// 内存 fake store —— 测同步/载入/订阅逻辑,不依赖 Isar 真库。

import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';
import 'package:citizenapp/citizen/public/data/public_institution_store.dart';
import 'package:citizenapp/isar/app_isar.dart';

class FakePublicInstitutionStore implements PublicInstitutionStore {
  final Map<String, PublicInstitutionDto> byId = {};
  final Map<String, String> provinceVersions = {};
  final Set<String> subs = {};
  List<String> provinceOrder = [];
  int upsertCalls = 0;
  int upsertItemCount = 0;
  int deleteCalls = 0;
  String? lastCatalogVersion;
  List<String> lastUpsertCids = const [];

  PublicInstitutionEntity _entity(PublicInstitutionDto d) =>
      d.toEntity(catalogVersion: 'fake', updatedAtMillis: 0);

  @override
  Future<void> upsertInstitutions(
    List<PublicInstitutionDto> items, {
    required String catalogVersion,
  }) async {
    upsertCalls++;
    upsertItemCount += items.length;
    lastCatalogVersion = catalogVersion;
    lastUpsertCids = items.map((d) => d.cidNumber).toList(growable: false);
    for (final d in items) {
      byId[d.cidNumber] = d;
    }
  }

  @override
  Future<void> setProvinceOrder(List<String> provinces) async {
    provinceOrder = List.of(provinces);
  }

  @override
  Future<List<String>> listProvinces() async => provinceOrder.isNotEmpty
      ? List.of(provinceOrder)
      : byId.values.map((e) => e.provinceCode).toSet().toList();

  @override
  Future<List<String>> listCities(String provinceCode) async => byId.values
      .where((e) => e.provinceCode == provinceCode)
      .map((e) => e.cityCode)
      .where((c) => c.isNotEmpty)
      .toSet()
      .toList();

  @override
  Future<List<PublicInstitutionEntity>> listInstitutionsByCity(
    String provinceCode,
    String cityCode,
  ) async =>
      byId.values
          .where(
              (e) => e.provinceCode == provinceCode && e.cityCode == cityCode)
          .map(_entity)
          .toList();

  @override
  Future<PublicInstitutionEntity?> getByCid(String cidNumber) async {
    final d = byId[cidNumber];
    return d == null ? null : _entity(d);
  }

  @override
  Future<List<PublicInstitutionEntity>> listByInstitutionCodes(
    Set<String> institutionCodes,
  ) async =>
      byId.values
          .where((e) => institutionCodes.contains(e.institutionCode))
          .map(_entity)
          .toList(growable: false);

  @override
  Future<List<PublicInstitutionEntity>> listByProvinceAndCodes(
    String provinceCode,
    Set<String> institutionCodes,
  ) async =>
      byId.values
          .where((e) =>
              e.provinceCode == provinceCode &&
              institutionCodes.contains(e.institutionCode))
          .map(_entity)
          .toList(growable: false);

  @override
  Future<List<PublicInstitutionEntity>> institutionsOfProvince(
    String provinceCode,
  ) async =>
      byId.values
          .where((e) => e.provinceCode == provinceCode)
          .map(_entity)
          .toList(growable: false);

  @override
  Future<List<String>> cidsOfProvince(String provinceCode) async => byId.values
      .where((e) => e.provinceCode == provinceCode)
      .map((e) => e.cidNumber)
      .toList(growable: false);

  @override
  Future<void> deleteByCids(List<String> cids) async {
    deleteCalls++;
    for (final cid in cids) {
      byId.remove(cid);
    }
  }

  @override
  Future<int> institutionCount() async => byId.length;

  @override
  Future<String?> provinceVersion(String province) async =>
      provinceVersions[province];

  @override
  Future<void> setProvinceVersion(String province, String version) async {
    provinceVersions[province] = version;
  }

  @override
  Future<void> subscribe(String accountId, String cidNumber) async {
    subs.add(subscriptionKeyOf(accountId, cidNumber));
  }

  @override
  Future<void> unsubscribe(String accountId, String cidNumber) async {
    subs.remove(subscriptionKeyOf(accountId, cidNumber));
  }

  @override
  Future<bool> isSubscribed(String accountId, String cidNumber) async =>
      subs.contains(subscriptionKeyOf(accountId, cidNumber));

  @override
  Future<List<PublicInstitutionEntity>> listSubscribed(
    String accountId,
  ) async {
    final out = <PublicInstitutionEntity>[];
    for (final key in subs) {
      if (!key.startsWith('$accountId|')) continue;
      final cid = key.substring(accountId.length + 1);
      final d = byId[cid];
      if (d != null) out.add(_entity(d));
    }
    return out;
  }
}
