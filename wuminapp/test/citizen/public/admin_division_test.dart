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
import 'fake_data_version_kv.dart';
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

  group('AdminDivisionBundleLoader.ensureSynced (版本驱动增量 reconcile)', () {
    // 当前 manifest:provinces:[{code,ver}]。
    Map<String, String> bundleFiles({
      required String version,
      required Map<String, String> provinceVers, // code -> ver
      required List<Map<String, dynamic>> provinces, // [{code,name}]
      required Map<String, List<Map<String, dynamic>>> cities, // pcode -> rows
      required Map<String, List<Map<String, dynamic>>> towns, // pcode -> rows
    }) {
      final files = <String, String>{
        'assets/admin_divisions/manifest.json': jsonEncode({
          'version': version,
          'provinces': provinceVers.entries
              .map((e) => {'code': e.key, 'ver': e.value})
              .toList(),
        }),
        'assets/admin_divisions/provinces.json': jsonEncode(provinces),
      };
      cities.forEach((pcode, rows) {
        files['assets/admin_divisions/cities/$pcode.json'] = jsonEncode(rows);
      });
      towns.forEach((pcode, rows) {
        files['assets/admin_divisions/towns/$pcode.json'] = jsonEncode(rows);
      });
      return files;
    }

    test('首装(库空,无 stored 版本)→ 全量灌入', () async {
      final bundle = _MapBundle(bundleFiles(
        version: 'v1',
        provinceVers: const {'LN': 'pln1'},
        provinces: const [
          {'code': 'LN', 'name': '岭南省'},
        ],
        cities: const {
          'LN': [
            {'code': '001', 'name': '广州市'},
          ],
        },
        towns: const {
          'LN': [
            {'city_code': '001', 'code': '005', 'name': '越秀镇'},
          ],
        },
      ));
      final store = FakeAdminDivisionStore();
      final kv = FakeDataVersionKv();
      final loader = AdminDivisionBundleLoader(
          store: store, bundle: bundle, versionKv: kv);

      final changed = await loader.ensureSynced();

      expect(changed, isTrue);
      expect(await store.divisionCount(), 3); // 省1+市1+镇1
      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '001'),
          '广州市');
      expect(kv.globalVersion, 'v1');
      expect(kv.provinceVersions, {'LN': 'pln1'});
    });

    test('全局 version 相等但省级 ver 变化 → 仍按省 reconcile', () async {
      final store = FakeAdminDivisionStore()
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.province,
          provinceCode: 'LN',
          code: 'LN',
          name: '岭南省',
        ))
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.city,
          provinceCode: 'LN',
          cityCode: '001',
          code: '001',
          name: '旧广州市',
        ));
      final kv = FakeDataVersionKv()
        ..globalVersion = 'v1'
        ..provinceVersions = {'LN': 'pln1'};
      final bundle = _MapBundle(bundleFiles(
        version: 'v1',
        provinceVers: const {'LN': 'pln2'},
        provinces: const [
          {'code': 'LN', 'name': '岭南省'},
        ],
        cities: const {
          'LN': [
            {'code': '001', 'name': '广州市'},
          ],
        },
        towns: const {},
      ));
      final loader = AdminDivisionBundleLoader(
          store: store, bundle: bundle, versionKv: kv);

      final changed = await loader.ensureSynced();

      expect(changed, isTrue);
      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '001'),
          '广州市');
      expect(kv.globalVersion, 'v1');
      expect(kv.provinceVersions, {'LN': 'pln2'});
    });

    test('没变的省:ver 相同 → 该省不 reconcile(不 upsert/不 delete)', () async {
      // 本地已有 LN(ver=pln1)+ GZ(ver=pgz1);manifest 全局 version 变(v1→v2),
      // 但只有 GZ 的 ver 变了(pgz1→pgz2)。期望:LN 不动,只 reconcile GZ。
      final store = FakeAdminDivisionStore()
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.province,
          provinceCode: 'LN',
          code: 'LN',
          name: '岭南省',
        ))
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.city,
          provinceCode: 'LN',
          cityCode: '001',
          code: '001',
          name: '广州市',
        ))
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.city,
          provinceCode: 'GZ',
          cityCode: '001',
          code: '001',
          name: '南宁市',
        ));
      final kv = FakeDataVersionKv()
        ..globalVersion = 'v1'
        ..provinceVersions = {'LN': 'pln1', 'GZ': 'pgz1'};
      final bundle = _MapBundle(bundleFiles(
        version: 'v2',
        provinceVers: const {'LN': 'pln1', 'GZ': 'pgz2'},
        provinces: const [
          {'code': 'LN', 'name': '岭南省'},
          {'code': 'GZ', 'name': '广西省'},
        ],
        cities: const {
          'GZ': [
            {'code': '001', 'name': '南宁市改名'},
          ],
        },
        towns: const {},
      ));
      final loader = AdminDivisionBundleLoader(
          store: store, bundle: bundle, versionKv: kv);

      final changed = await loader.ensureSynced();

      expect(changed, isTrue);
      // GZ reconcile 了:南宁市 → 南宁市改名。
      expect(await store.divisionName(AdminDivisionLevel.city, 'GZ', '001'),
          '南宁市改名');
      // LN 没动:广州市仍在,key 未被删。
      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '001'),
          '广州市');
      expect(kv.globalVersion, 'v2');
      expect(kv.provinceVersions, {'LN': 'pln1', 'GZ': 'pgz2'});
    });

    test('reconcile 改名:code 不变 name 变 → 同条更新,同省其他条不变', () async {
      final store = FakeAdminDivisionStore()
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.province,
          provinceCode: 'LN',
          code: 'LN',
          name: '岭南省',
        ))
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.city,
          provinceCode: 'LN',
          cityCode: '001',
          code: '001',
          name: '旧广州市',
        ))
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.city,
          provinceCode: 'LN',
          cityCode: '002',
          code: '002',
          name: '深圳市',
        ));
      final kv = FakeDataVersionKv()
        ..globalVersion = 'v1'
        ..provinceVersions = {'LN': 'pln1'};
      final bundle = _MapBundle(bundleFiles(
        version: 'v2',
        provinceVers: const {'LN': 'pln2'},
        provinces: const [
          {'code': 'LN', 'name': '岭南省'},
        ],
        cities: const {
          'LN': [
            {'code': '001', 'name': '广州市'}, // 改名
            {'code': '002', 'name': '深圳市'}, // 不变
          ],
        },
        towns: const {},
      ));
      final loader = AdminDivisionBundleLoader(
          store: store, bundle: bundle, versionKv: kv);

      await loader.ensureSynced();

      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '001'),
          '广州市'); // 更新
      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '002'),
          '深圳市'); // 不变
      expect(store.lastUpsertKeys, ['city|LN|001|']); // 只写改名这一条
      // 两市都还在 + 省级记录(reconcile 从 provinces.json 一并写),无残留多余条。
      final keys = await store.divisionKeysOfProvince('LN');
      expect(keys.toSet(), {'province|LN||', 'city|LN|001|', 'city|LN|002|'});
    });

    test('reconcile 删除:包里少一个 code → 该 divisionKey 被删', () async {
      final store = FakeAdminDivisionStore()
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.city,
          provinceCode: 'LN',
          cityCode: '001',
          code: '001',
          name: '广州市',
        ))
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.city,
          provinceCode: 'LN',
          cityCode: '002',
          code: '002',
          name: '将被删的市',
        ));
      final kv = FakeDataVersionKv()
        ..globalVersion = 'v1'
        ..provinceVersions = {'LN': 'pln1'};
      final bundle = _MapBundle(bundleFiles(
        version: 'v2',
        provinceVers: const {'LN': 'pln2'},
        provinces: const [
          {'code': 'LN', 'name': '岭南省'},
        ],
        cities: const {
          'LN': [
            {'code': '001', 'name': '广州市'}, // 只剩这一个
          ],
        },
        towns: const {},
      ));
      final loader = AdminDivisionBundleLoader(
          store: store, bundle: bundle, versionKv: kv);

      await loader.ensureSynced();

      final keys = await store.divisionKeysOfProvince('LN');
      // 002 被删;省级记录由 reconcile 从 provinces.json 写入。
      expect(keys.toSet(), {'province|LN||', 'city|LN|001|'});
    });

    test('reconcile 新增:包里多一个 → 插入', () async {
      final store = FakeAdminDivisionStore()
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.city,
          provinceCode: 'LN',
          cityCode: '001',
          code: '001',
          name: '广州市',
        ));
      final kv = FakeDataVersionKv()
        ..globalVersion = 'v1'
        ..provinceVersions = {'LN': 'pln1'};
      final bundle = _MapBundle(bundleFiles(
        version: 'v2',
        provinceVers: const {'LN': 'pln2'},
        provinces: const [
          {'code': 'LN', 'name': '岭南省'},
        ],
        cities: const {
          'LN': [
            {'code': '001', 'name': '广州市'},
            {'code': '003', 'name': '新增市'}, // 新增
          ],
        },
        towns: const {},
      ));
      final loader = AdminDivisionBundleLoader(
          store: store, bundle: bundle, versionKv: kv);

      await loader.ensureSynced();

      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '003'),
          '新增市');
      final keys = await store.divisionKeysOfProvince('LN');
      // 省级记录由 reconcile 从 provinces.json 写入。
      expect(keys.toSet(), {'province|LN||', 'city|LN|001|', 'city|LN|003|'});
    });

    test('manifest 缺省级版本表 → 不写库、不删除本地数据', () async {
      final bundle = _MapBundle({
        'assets/admin_divisions/manifest.json':
            jsonEncode({'version': 'invalid'}),
        'assets/admin_divisions/provinces.json': jsonEncode(const [
          {'code': 'LN', 'name': '岭南省'},
        ]),
        'assets/admin_divisions/cities/LN.json': jsonEncode(const [
          {'code': '001', 'name': '广州市'},
        ]),
      });
      final store = FakeAdminDivisionStore()
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.city,
          provinceCode: 'LN',
          cityCode: '001',
          code: '001',
          name: '旧广州市',
        ))
        ..seed(const AdminDivisionDto(
          level: AdminDivisionLevel.city,
          provinceCode: 'LN',
          cityCode: '002',
          code: '002',
          name: '旧残留市',
        ));
      final loader = AdminDivisionBundleLoader(
        store: store,
        bundle: bundle,
        versionKv: FakeDataVersionKv(),
      );
      final upsertBefore = store.upsertCalls;
      final deleteBefore = store.deleteCalls;

      final changed = await loader.ensureSynced();
      expect(changed, isFalse);
      expect(store.upsertCalls, upsertBefore);
      expect(store.deleteCalls, deleteBefore);
      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '001'),
          '旧广州市');
      expect(await store.divisionName(AdminDivisionLevel.city, 'LN', '002'),
          '旧残留市');
    });

    test('无数据包 → loadFromBundle 返回 false 不崩', () async {
      final store = FakeAdminDivisionStore();
      final loader = AdminDivisionBundleLoader(
        store: store,
        bundle: _MapBundle(const {}),
        versionKv: FakeDataVersionKv(),
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
