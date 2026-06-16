// ADR-021 守卫:链上省名集合 == 字典 provinces.json 省名集合。
//
// 中文注释:省名走链上常量(认可的省名源),但必须与 china.sqlite 派生字典逐字对齐
// (否则点省查不到机构 / 显示与数据漂移)。把"逐字对齐"约定变成 CI 守卫:china.sqlite
// 省名一旦与链上常量不一致,本测试即红。

import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_provinces.dart';

void main() {
  test('链上省名集合 == 字典 provinces.json 省名集合(逐字对齐守卫)', () {
    final file = File('assets/admin_divisions/provinces.json');
    expect(file.existsSync(), isTrue,
        reason: 'assets/admin_divisions/provinces.json 必须存在(字典数据包)');

    final dictNames = (jsonDecode(file.readAsStringSync()) as List<dynamic>)
        .map((e) => (e as Map<String, dynamic>)['name'] as String)
        .toSet();
    final chainNames = publicProvinceNamesSet();

    // 集合相等:链上常量与字典一一对应,零漂移。
    expect(chainNames, equals(dictNames),
        reason: '链上省名与 china.sqlite 派生字典省名不一致——'
            '改名只改 china.sqlite,且需同步链上常量。');
  });

  test('省 code → 全名/展示名映射来自链上常量(中枢省=ZS)', () {
    expect(provinceFullNameByCode('ZS'), '中枢省');
    expect(provinceDisplayNameByCode('ZS'), '中枢');
    // 未知 code 回退 code 本身(绝不崩)。
    expect(provinceFullNameByCode('ZZ'), 'ZZ');
  });
}
