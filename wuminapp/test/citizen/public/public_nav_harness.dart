// 卡B widget 测试脚手架:构造无网络/无 Isar 的 repo(seeded fake store)。

import 'package:flutter/services.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_api.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_bundle_loader.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_dto.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_repository.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_sync_service.dart';

import 'fake_public_institution_store.dart';

class _NoopApi extends PublicInstitutionApi {
  _NoopApi() : super(baseUrl: 'http://test');

  @override
  Future<PublicInstitutionVersion> fetchVersion({
    required String province,
    String? city,
  }) async =>
      // 版本恒为 'seed',与 seeded store 版本一致 → syncProvince 跳过,不发网络。
      PublicInstitutionVersion(province: province, manifestVersion: 'seed');

  @override
  Future<PublicInstitutionPage> fetchPage({
    required String province,
    String? city,
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

PublicInstitutionDto seedDto(
  String sfid, {
  required String province,
  required String city,
  String? name,
  String code = 'ZF',
}) =>
    PublicInstitutionDto.fromJson(<String, dynamic>{
      'sfid_number': sfid,
      'institution_name': name ?? '$city$code机构',
      'province': province,
      'city': city,
      'institution_code': code,
      'account_count': 2,
    });

/// 构造 seeded 仓库:省顺序 + 机构 + 各省版本戳(=seed,使同步跳过)。
Future<PublicInstitutionRepository> buildSeededRepo({
  required List<String> provinceOrder,
  required List<PublicInstitutionDto> institutions,
  Map<String, String>? subscriptions, // pubkey -> sfid
}) async {
  final store = FakePublicInstitutionStore();
  await store.setProvinceOrder(provinceOrder);
  await store.upsertInstitutions(institutions, catalogVersion: 'seed');
  for (final p in provinceOrder) {
    await store.setProvinceVersion(p, 'seed');
  }
  if (subscriptions != null) {
    for (final entry in subscriptions.entries) {
      await store.subscribe(entry.key, entry.value);
    }
  }
  return PublicInstitutionRepository(
    store: store,
    sync: PublicInstitutionSyncService(store: store, api: _NoopApi()),
    loader: PublicInstitutionBundleLoader(store: store, bundle: _EmptyBundle()),
  );
}
