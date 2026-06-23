// 公权机构数据包载入单测 —— 版本驱动增量 reconcile(fake AssetBundle + fake store)。

import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/public/data/admin_division_bundle_loader.dart';
import 'package:citizenapp/citizen/public/data/public_institution_bundle_loader.dart';
import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';

import 'fake_admin_division_store.dart';
import 'fake_data_version_kv.dart';
import 'fake_public_institution_store.dart';

class _MapBundle extends AssetBundle {
  _MapBundle(this._files);
  final Map<String, String> _files;

  @override
  Future<ByteData> load(String key) async => throw UnimplementedError();

  @override
  Future<String> loadString(String key, {bool cache = true}) async {
    final value = _files[key];
    if (value == null) throw FlutterError('asset not found: $key');
    return value;
  }
}

/// 构造一个不触碰全局 Isar 的机构 loader(字典 loader 也走 fake + fake 版本游标)。
PublicInstitutionBundleLoader buildLoader({
  required FakePublicInstitutionStore store,
  required AssetBundle bundle,
  FakeDataVersionKv? institutionKv,
  FakeDataVersionKv? divisionKv,
  FakeAdminDivisionStore? divisionStore,
}) {
  final divLoader = AdminDivisionBundleLoader(
    store: divisionStore ?? FakeAdminDivisionStore(),
    bundle: bundle,
    versionKv: divisionKv ?? FakeDataVersionKv(),
  );
  return PublicInstitutionBundleLoader(
    store: store,
    bundle: bundle,
    divisionLoader: divLoader,
    versionKv: institutionKv ?? FakeDataVersionKv(),
  );
}

/// 新格式机构 manifest:provinces:[{province_name,manifest_version}]。
String _instManifest({
  required String version,
  required List<Map<String, String>> provinces,
}) =>
    jsonEncode({'version': version, 'provinces': provinces});

void main() {
  test('首装:载入 manifest(新格式)+ 省分片 → 写库 + 省顺序 + 版本戳', () async {
    final bundle = _MapBundle({
      'assets/public_institutions/manifest.json': _instManifest(
        version: '1',
        provinces: const [
          {'province_name': '中枢省', 'manifest_version': 'cz-1'},
        ],
      ),
      'assets/public_institutions/中枢省.json': jsonEncode({
        'province_name': '中枢省',
        'manifest_version': 'cz-1',
        'institutions': [
          {
            'cid_number': 'ZS001-ZF000-1-2026',
            'cid_full_name': '中枢省人民政府',
            'province_code': 'ZS',
            'city_code': '001',
            'institution_code': 'ZF',
            'account_count': 2,
          }
        ],
      }),
    });
    final store = FakePublicInstitutionStore();
    final kv = FakeDataVersionKv();
    final loader = buildLoader(
      store: store,
      bundle: bundle,
      institutionKv: kv,
    );

    final changed = await loader.ensureSynced();

    expect(changed, isTrue);
    expect(store.byId.containsKey('ZS001-ZF000-1-2026'), isTrue);
    expect(store.byId['ZS001-ZF000-1-2026']!.provinceCode, 'ZS');
    expect(store.byId['ZS001-ZF000-1-2026']!.cityCode, '001');
    expect(await store.listProvinces(), ['中枢省']);
    expect(await store.provinceVersion('中枢省'), 'cz-1');
    expect(kv.globalVersion, '1');
    expect(kv.provinceVersions, {'中枢省': 'cz-1'});
  });

  test('全局 version 相等但省级 manifest_version 变化 → 仍 reconcile 机构分片', () async {
    final store = FakePublicInstitutionStore();
    await store.upsertInstitutions(
      [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': 'ZS001-ZF000-1-2026',
          'cid_full_name': '旧名机构',
          'province_code': 'ZS',
          'city_code': '001',
          'institution_code': 'ZF',
          'account_count': 2,
        }),
      ],
      catalogVersion: 'x',
    );
    final kv = FakeDataVersionKv()
      ..globalVersion = '1'
      ..provinceVersions = {'中枢省': 'cz-old'};
    final bundle = _MapBundle({
      'assets/public_institutions/manifest.json': _instManifest(
        version: '1',
        provinces: const [
          {'province_name': '中枢省', 'manifest_version': 'cz-new'},
        ],
      ),
      'assets/public_institutions/中枢省.json': jsonEncode({
        'province_name': '中枢省',
        'manifest_version': 'cz-new',
        'institutions': [
          {
            'cid_number': 'ZS001-ZF000-1-2026',
            'cid_full_name': '新名机构',
            'province_code': 'ZS',
            'city_code': '001',
            'institution_code': 'ZF',
            'account_count': 2,
          }
        ],
      }),
    });
    final loader = buildLoader(store: store, bundle: bundle, institutionKv: kv);

    final changed = await loader.ensureSynced();

    expect(changed, isTrue);
    expect(store.byId['ZS001-ZF000-1-2026']!.cidFullName, '新名机构');
    expect(kv.globalVersion, '1');
    expect(kv.provinceVersions, {'中枢省': 'cz-new'});
  });

  test('reconcile:改名 + 删除 + 新增,没变的省不动', () async {
    // 本地:中枢省[A 旧名, B 待删](manifest_version=cz-1)、岭南省[X](manifest_version=ln-1)。
    // manifest:中枢省 manifest_version 变(cz-1→cz-2),岭南省 manifest_version 不变。
    final store = FakePublicInstitutionStore();
    await store.upsertInstitutions(
      [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': 'A',
          'cid_full_name': '旧名机构',
          'province_code': 'ZS',
          'city_code': '001',
          'institution_code': 'ZF',
          'account_count': 1,
        }),
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': 'B',
          'cid_full_name': '待删机构',
          'province_code': 'ZS',
          'city_code': '001',
          'institution_code': 'ZF',
          'account_count': 1,
        }),
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': 'X',
          'cid_full_name': '岭南机构',
          'province_code': 'LN',
          'city_code': '001',
          'institution_code': 'ZF',
          'account_count': 1,
        }),
      ],
      catalogVersion: 'seed',
    );
    final kv = FakeDataVersionKv()
      ..globalVersion = 'v1'
      ..provinceVersions = {'中枢省': 'cz-1', '岭南省': 'ln-1'};
    final bundle = _MapBundle({
      'assets/public_institutions/manifest.json': _instManifest(
        version: 'v2',
        provinces: const [
          {'province_name': '中枢省', 'manifest_version': 'cz-2'}, // 变了
          {'province_name': '岭南省', 'manifest_version': 'ln-1'}, // 不变
        ],
      ),
      'assets/public_institutions/中枢省.json': jsonEncode({
        'province_name': '中枢省',
        'manifest_version': 'cz-2',
        'institutions': [
          // A 改名,B 不在(删),C 新增。
          {
            'cid_number': 'A',
            'cid_full_name': '新名机构',
            'province_code': 'ZS',
            'city_code': '001',
            'institution_code': 'ZF',
            'account_count': 1,
          },
          {
            'cid_number': 'C',
            'cid_full_name': '新增机构',
            'province_code': 'ZS',
            'city_code': '002',
            'institution_code': 'ZF',
            'account_count': 1,
          },
        ],
      }),
      // 岭南省分片故意不提供:若误读会 return(无分片),证明没读它。
    });
    final loader = buildLoader(store: store, bundle: bundle, institutionKv: kv);

    final changed = await loader.ensureSynced();

    expect(changed, isTrue);
    expect(store.byId['A']!.cidFullName, '新名机构'); // 改名
    expect(store.byId.containsKey('B'), isFalse); // 删除
    expect(store.byId['C']!.cidFullName, '新增机构'); // 新增
    expect(store.byId.containsKey('X'), isTrue); // 岭南省没动,X 仍在
    expect(store.lastUpsertCids, ['A', 'C']); // 只写改名/新增,不重写整省
    expect(kv.globalVersion, 'v2');
    expect(kv.provinceVersions, {'中枢省': 'cz-2', '岭南省': 'ln-1'});
  });

  test('没变的省不读分片(deleteCalls/upsertCalls 不为该省增加)', () async {
    final store = FakePublicInstitutionStore();
    await store.upsertInstitutions(
      [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': 'X',
          'province_code': 'LN',
          'city_code': '001',
          'institution_code': 'ZF',
          'account_count': 1,
        }),
      ],
      catalogVersion: 'seed',
    );
    final kv = FakeDataVersionKv()
      ..globalVersion = 'v1'
      ..provinceVersions = {'岭南省': 'ln-1'};
    // 全局 version 变了(强制进入逐省比对),但岭南省 manifest_version 没变 → 不 reconcile。
    final bundle = _MapBundle({
      'assets/public_institutions/manifest.json': _instManifest(
        version: 'v2',
        provinces: const [
          {'province_name': '岭南省', 'manifest_version': 'ln-1'},
        ],
      ),
    });
    final loader = buildLoader(store: store, bundle: bundle, institutionKv: kv);

    final upsertBefore = store.upsertCalls;
    final deleteBefore = store.deleteCalls;
    final changed = await loader.ensureSynced();

    expect(changed, isFalse);
    expect(store.upsertCalls, upsertBefore); // 没读分片、没 upsert
    expect(store.deleteCalls, deleteBefore);
    // 但全局 version 仍落到 v2(完成标记)。
    expect(kv.globalVersion, 'v2');
  });

  test('manifest 缺省级版本表 → 不写库、不删除本地数据', () async {
    final bundle = _MapBundle({
      'assets/public_institutions/manifest.json': jsonEncode({
        'version': '1',
        'provinces': ['中枢省'],
      }),
      'assets/public_institutions/中枢省.json': jsonEncode({
        'province_name': '中枢省',
        'manifest_version': 'cz-1',
        'institutions': [
          {
            'cid_number': 'ZS001',
            'cid_full_name': '新名机构',
            'province_code': 'ZS',
            'city_code': '001',
            'institution_code': 'ZF',
            'account_count': 2,
          }
        ],
      }),
    });
    final store = FakePublicInstitutionStore();
    await store.upsertInstitutions(
      [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': 'ZS001',
          'cid_full_name': '旧名机构',
          'province_code': 'ZS',
          'city_code': '001',
          'institution_code': 'ZF',
          'account_count': 2,
        }),
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': 'STALE',
          'cid_full_name': '旧残留机构',
          'province_code': 'ZS',
          'city_code': '001',
          'institution_code': 'ZF',
          'account_count': 2,
        }),
      ],
      catalogVersion: 'old',
    );
    final loader = buildLoader(store: store, bundle: bundle);
    final upsertBefore = store.upsertCalls;
    final deleteBefore = store.deleteCalls;

    final changed = await loader.ensureSynced();
    expect(changed, isFalse);
    expect(store.upsertCalls, upsertBefore);
    expect(store.deleteCalls, deleteBefore);
    expect(store.byId.containsKey('ZS001'), isTrue);
    expect(store.byId['ZS001']!.cidFullName, '旧名机构');
    expect(store.byId.containsKey('STALE'), isTrue);
    expect(store.provinceOrder, isEmpty);
    expect(await store.listProvinces(), ['ZS']);
  });

  test('无数据包 → loadFromBundle 返回 false 不崩', () async {
    final store = FakePublicInstitutionStore();
    final loader = buildLoader(store: store, bundle: _MapBundle(const {}));
    expect(await loader.loadFromBundle(), isFalse);
  });
}
