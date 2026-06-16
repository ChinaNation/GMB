// 行政区字典层单测(ADR-021):divisionName 命中/回退、formatAreaPath 三态、
// 字典 loader 灌库、listCities 按 code 去重。

import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/citizen/public/data/admin_division_bundle_loader.dart';
import 'package:wuminapp_mobile/citizen/public/data/admin_division_dto.dart';
import 'package:wuminapp_mobile/citizen/public/data/area_path_formatter.dart';

import 'fake_admin_division_store.dart';
import 'fake_public_institution_store.dart';
import 'public_nav_harness.dart';

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
  group('divisionKeyOf / scopeKeyOf', () {
    test('省/市/镇键带完整层级前缀(镇 code 全国不唯一)', () {
      expect(
        divisionKeyOf(level: AdminDivisionLevel.province, provinceCode: 'LN'),
        'province|LN||',
      );
      expect(
        divisionKeyOf(
            level: AdminDivisionLevel.city,
            provinceCode: 'LN',
            cityCode: '001'),
        'city|LN|001|',
      );
      expect(
        divisionKeyOf(
          level: AdminDivisionLevel.town,
          provinceCode: 'LN',
          cityCode: '001',
          townCode: '005',
        ),
        'town|LN|001|005',
      );
    });

    test('scopeKey 父定位:province 空、city=pcode、town=pc|cc', () {
      expect(
        scopeKeyOf(level: AdminDivisionLevel.province, provinceCode: 'LN'),
        '',
      );
      expect(
        scopeKeyOf(level: AdminDivisionLevel.city, provinceCode: 'LN'),
        'LN',
      );
      expect(
        scopeKeyOf(
            level: AdminDivisionLevel.town,
            provinceCode: 'LN',
            cityCode: '001'),
        'LN|001',
      );
    });
  });

  group('FakeAdminDivisionStore.divisionName', () {
    test('命中返回名字;未命中回退 code 本身(绝不空)', () async {
      final store = FakeAdminDivisionStore();
      store.seed(const AdminDivisionDto(
        level: AdminDivisionLevel.city,
        provinceCode: 'LN',
        cityCode: '001',
        code: '001',
        name: '广州市',
      ));
      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '001'),
          '广州市');
      // 未命中:回退 code。
      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '999'),
          '999');
    });
  });

  group('formatAreaPath', () {
    final store = FakeAdminDivisionStore()
      ..seed(const AdminDivisionDto(
        level: AdminDivisionLevel.city,
        provinceCode: 'LN',
        cityCode: '001',
        code: '001',
        name: '广州市',
      ))
      ..seed(const AdminDivisionDto(
        level: AdminDivisionLevel.town,
        provinceCode: 'LN',
        cityCode: '001',
        townCode: '005',
        code: '005',
        name: '越秀镇',
      ));

    test('有 town:省名·市名·镇名', () async {
      final path = await formatAreaPath(
        store,
        provinceName: '岭南省',
        provinceCode: 'LN',
        cityCode: '001',
        townCode: '005',
      );
      expect(path, '岭南省 · 广州市 · 越秀镇');
    });

    test('空 town:只到市,不拼空段不显 null', () async {
      final path = await formatAreaPath(
        store,
        provinceName: '岭南省',
        provinceCode: 'LN',
        cityCode: '001',
      );
      expect(path, '岭南省 · 广州市');
    });

    test('字典缺失:市名回退 code 本身', () async {
      final path = await formatAreaPath(
        store,
        provinceName: '岭南省',
        provinceCode: 'LN',
        cityCode: '777',
      );
      expect(path, '岭南省 · 777');
    });
  });

  group('AdminDivisionBundleLoader', () {
    test('灌字典:省/市/镇全量写库 + 名字可 join', () async {
      final bundle = _MapBundle({
        'assets/admin_divisions/manifest.json':
            jsonEncode({'version': 'dict-1'}),
        'assets/admin_divisions/provinces.json':
            jsonEncode([
          {'code': 'LN', 'name': '岭南省'},
        ]),
        'assets/admin_divisions/cities/LN.json': jsonEncode([
          {'code': '001', 'name': '广州市'},
        ]),
        'assets/admin_divisions/towns/LN.json': jsonEncode([
          {'city_code': '001', 'code': '005', 'name': '越秀镇'},
        ]),
      });
      final store = FakeAdminDivisionStore();
      final loader = AdminDivisionBundleLoader(store: store, bundle: bundle);

      final loaded = await loader.ensureDictionaryLoaded();

      expect(loaded, isTrue);
      // 省3键? 省1+市1+镇1=3 条。
      expect(await store.divisionCount(), 3);
      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '001'),
          '广州市');
      expect(
          await store.divisionName(AdminDivisionLevel.town, 'LN|001', '005'),
          '越秀镇');
    });

    test('库非空 → ensureDictionaryLoaded 跳过(幂等)', () async {
      final store = FakeAdminDivisionStore()
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.province,
          provinceCode: 'LN',
          code: 'LN',
          name: '岭南省',
        ));
      final loader = AdminDivisionBundleLoader(
        store: store,
        bundle: _MapBundle(const {}),
      );
      expect(await loader.ensureDictionaryLoaded(), isFalse);
    });

    test('无数据包 → loadFromBundle 返回 false 不崩', () async {
      final store = FakeAdminDivisionStore();
      final loader = AdminDivisionBundleLoader(
        store: store,
        bundle: _MapBundle(const {}),
      );
      expect(await loader.loadFromBundle(), isFalse);
    });
  });

  group('listCities 按 cityCode 去重', () {
    test('同省多机构同市 code 只出一条;LN 三市 001 是不同省内 code', () async {
      final store = FakePublicInstitutionStore();
      await store.upsertInstitutions(
        [
          seedDto('A', provinceCode: 'LN', cityCode: '001'),
          seedDto('B', provinceCode: 'LN', cityCode: '001'), // 同市 code
          seedDto('C', provinceCode: 'LN', cityCode: '002'),
        ],
        catalogVersion: 'v',
      );
      final cities = await store.listCities('LN');
      expect(cities.toSet(), {'001', '002'}); // 去重后两个市 code
    });
  });
}
