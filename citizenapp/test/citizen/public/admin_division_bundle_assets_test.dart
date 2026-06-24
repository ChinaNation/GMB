// 回归:行政区字典数据包(assets/admin_divisions)能从真实打包资源灌入,并按
// (city, 省code, 市code) join 出市名。守护 ADR-021 字典 join + 防 assets 数据/格式
// 漂移——如 china 重烤后某省市名再次为空(回到 001 bug),本测试会变红。
//
// 用内存 fake store(不依赖 Isar native);Isar 实现侧已由真机 QA + 类型注解覆盖。

import 'package:flutter/services.dart' show rootBundle;
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/citizen/public/data/admin_division_bundle_loader.dart';
import 'package:citizenapp/citizen/public/data/admin_division_dto.dart';

import 'fake_admin_division_store.dart';
import 'fake_data_version_kv.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('真实 assets 数据包灌入后 cityNameMap(JL)[001]==南关市', () async {
    final store = FakeAdminDivisionStore();
    final loader = AdminDivisionBundleLoader(
      store: store,
      bundle: rootBundle,
      versionKv: FakeDataVersionKv(),
    );

    final changed = await loader.ensureSynced();
    expect(changed, isTrue, reason: '首装应从 assets 全量灌入字典');
    expect(await store.divisionCount(), greaterThan(40000),
        reason: '全国省/市/镇字典应≈4.2 万条');

    final jl = await store.divisionsByLevel(AdminDivisionLevel.city, 'JL');
    final nameMap = {for (final d in jl) d.code: d.divisionName};
    expect(nameMap['001'], '南关市', reason: '市名须 join 到字典,不回退 code');
    expect(nameMap['002'], '春阳市');
  });
}
