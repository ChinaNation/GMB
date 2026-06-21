// 公权机构增量同步逻辑单测(版本比对 → 跳过/重拉)。

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/public/data/public_institution_api.dart';
import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';
import 'package:citizenapp/citizen/public/data/public_institution_sync_service.dart';

import 'fake_public_institution_store.dart';

class _FakeApi extends PublicInstitutionApi {
  _FakeApi({required this.version, required this.pages})
      : super(baseUrl: 'http://test');

  final String? version;
  final List<PublicInstitutionPage> pages;
  int versionCalls = 0;
  int pageCalls = 0;

  @override
  Future<PublicInstitutionVersion> fetchVersion({
    required String provinceName,
    String? cityName,
  }) async {
    versionCalls++;
    return PublicInstitutionVersion(
        provinceName: provinceName, manifestVersion: version);
  }

  @override
  Future<PublicInstitutionPage> fetchPage({
    required String provinceName,
    String? cityName,
    String? sinceVersion,
    String? afterSfid,
    int pageSize = 500,
  }) async {
    final page = pages[pageCalls.clamp(0, pages.length - 1)];
    pageCalls++;
    return page;
  }
}

PublicInstitutionDto _dto(String sfid, String cityCode) =>
    PublicInstitutionDto.fromJson(<String, dynamic>{
      'sfid_number': sfid,
      'province_code': 'ZS',
      'city_code': cityCode,
      'institution_code': 'ZF',
      'account_count': 2,
    });

void main() {
  test('版本一致 → 跳过,不拉取不写库', () async {
    final store = FakePublicInstitutionStore();
    await store.setProvinceVersion('中枢', 'v1');
    final api = _FakeApi(version: 'v1', pages: const []);
    final sync = PublicInstitutionSyncService(store: store, api: api);

    final changed = await sync.syncProvince('中枢');

    expect(changed, isFalse);
    expect(api.pageCalls, 0);
    expect(store.upsertCalls, 0);
  });

  test('版本变化 → 全量重拉 + 写库 + 更新版本戳', () async {
    final store = FakePublicInstitutionStore();
    await store.setProvinceVersion('中枢', 'old');
    final api = _FakeApi(
      version: 'new',
      pages: [
        PublicInstitutionPage(
          items: [_dto('A', '中央'), _dto('B', '中央')],
          hasMore: false,
        ),
      ],
    );
    final sync = PublicInstitutionSyncService(store: store, api: api);

    final changed = await sync.syncProvince('中枢');

    expect(changed, isTrue);
    expect(store.byId.keys, containsAll(<String>['A', 'B']));
    expect(await store.provinceVersion('中枢'), 'new');
    expect(store.lastCatalogVersion, 'new');
  });

  test('多页 → 翻页直到 hasMore=false', () async {
    final store = FakePublicInstitutionStore();
    final api = _FakeApi(
      version: 'v',
      pages: [
        PublicInstitutionPage(items: [_dto('A', '甲')], hasMore: true),
        PublicInstitutionPage(items: [_dto('B', '乙')], hasMore: false),
      ],
    );
    final sync = PublicInstitutionSyncService(store: store, api: api);

    await sync.syncProvince('中枢');

    expect(api.pageCalls, 2);
    expect(store.byId.length, 2);
  });
}
