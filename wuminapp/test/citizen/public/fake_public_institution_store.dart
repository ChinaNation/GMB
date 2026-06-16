// 内存 fake store —— 测同步/载入/订阅逻辑,不依赖 Isar 真库。

import 'package:wuminapp_mobile/citizen/public/data/public_institution_dto.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_store.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

class FakePublicInstitutionStore implements PublicInstitutionStore {
  final Map<String, PublicInstitutionDto> byId = {};
  final Map<String, String> provinceVersions = {};
  final Set<String> subs = {};
  List<String> provinceOrder = [];
  int upsertCalls = 0;
  String? lastCatalogVersion;

  PublicInstitutionEntity _entity(PublicInstitutionDto d) =>
      d.toEntity(catalogVersion: 'fake', updatedAtMillis: 0);

  @override
  Future<void> upsertInstitutions(
    List<PublicInstitutionDto> items, {
    required String catalogVersion,
  }) async {
    upsertCalls++;
    lastCatalogVersion = catalogVersion;
    for (final d in items) {
      byId[d.sfidNumber] = d;
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
          .where((e) =>
              e.provinceCode == provinceCode && e.cityCode == cityCode)
          .map(_entity)
          .toList();

  @override
  Future<PublicInstitutionEntity?> getBySfid(String sfidNumber) async {
    final d = byId[sfidNumber];
    return d == null ? null : _entity(d);
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
  Future<void> subscribe(String walletPubkeyHex, String sfidNumber) async {
    subs.add(subscriptionKeyOf(walletPubkeyHex, sfidNumber));
  }

  @override
  Future<void> unsubscribe(String walletPubkeyHex, String sfidNumber) async {
    subs.remove(subscriptionKeyOf(walletPubkeyHex, sfidNumber));
  }

  @override
  Future<bool> isSubscribed(String walletPubkeyHex, String sfidNumber) async =>
      subs.contains(subscriptionKeyOf(walletPubkeyHex, sfidNumber));

  @override
  Future<List<PublicInstitutionEntity>> listSubscribed(
    String walletPubkeyHex,
  ) async {
    final out = <PublicInstitutionEntity>[];
    for (final key in subs) {
      if (!key.startsWith('$walletPubkeyHex|')) continue;
      final sfid = key.substring(walletPubkeyHex.length + 1);
      final d = byId[sfid];
      if (d != null) out.add(_entity(d));
    }
    return out;
  }
}
