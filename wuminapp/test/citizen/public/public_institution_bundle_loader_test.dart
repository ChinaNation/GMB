// 公权机构数据包载入单测(fake AssetBundle + fake store)。

import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_bundle_loader.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_dto.dart';

import 'fake_admin_division_store.dart';
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

void main() {
  test('载入 manifest + 省分片(行政区 code) → 写库 + 省顺序 + 版本戳', () async {
    final bundle = _MapBundle({
      'assets/public_institutions/manifest.json': jsonEncode({
        'version': '1',
        'provinces': ['ZS'],
      }),
      'assets/public_institutions/ZS.json': jsonEncode({
        'province': 'ZS',
        'manifest_version': 'cz-1',
        'institutions': [
          {
            'sfid_number': 'ZS001-ZF000-1-2026',
            'institution_name': '中枢省人民政府',
            'province_code': 'ZS',
            'city_code': '001',
            'institution_code': 'ZF',
            'account_count': 2,
          }
        ],
      }),
    });
    final store = FakePublicInstitutionStore();
    final loader = PublicInstitutionBundleLoader(
      store: store,
      bundle: bundle,
      divisionStore: FakeAdminDivisionStore(),
    );

    final loaded = await loader.ensureBundleLoaded();

    expect(loaded, isTrue);
    expect(store.byId.containsKey('ZS001-ZF000-1-2026'), isTrue);
    expect(store.byId['ZS001-ZF000-1-2026']!.provinceCode, 'ZS');
    expect(store.byId['ZS001-ZF000-1-2026']!.cityCode, '001');
    expect(await store.listProvinces(), ['ZS']);
    expect(await store.provinceVersion('ZS'), 'cz-1');
  });

  test('库非空 → ensureBundleLoaded 跳过(机构)', () async {
    final store = FakePublicInstitutionStore();
    await store.upsertInstitutions(
      [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'sfid_number': 'seed',
          'province_code': 'ZS',
          'city_code': '001',
          'institution_code': 'ZF',
          'account_count': 2,
        }),
      ],
      catalogVersion: 'x',
    );
    final loader = PublicInstitutionBundleLoader(
      store: store,
      bundle: _MapBundle(const {}),
      divisionStore: FakeAdminDivisionStore(),
    );

    expect(await loader.ensureBundleLoaded(), isFalse);
  });

  test('无数据包 → loadFromBundle 返回 false 不崩', () async {
    final store = FakePublicInstitutionStore();
    final loader = PublicInstitutionBundleLoader(
      store: store,
      bundle: _MapBundle(const {}),
      divisionStore: FakeAdminDivisionStore(),
    );
    expect(await loader.loadFromBundle(), isFalse);
  });
}
