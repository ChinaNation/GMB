# CitizenApp 立法 tab 省级机构统一顺序 + 市按市代码排序

任务需求：公民tab-立法子tab-省市立法机构，各省下机构顺序统一为「省立法院 · 省参议会 · 省众议会」，
其后各市按市代码 001、002… 升序。
所属模块：citizenapp / citizen 立法

## 定稿（用户确认）

- 省级顺序：省立法院 → 省参议会(PSN) → 省众议会(PRP)（原为立法院→众议会→参议会），与顶部国家卡一致。
- 市立法会：由「名称字典序」改为「市代码 cityCode 升序（001、002…）」；用 int 解析更稳（防个别未补零）。
- 范围只改省级；国家级本就是参议会→众议会，无需动。

## 落点（`lib/citizen/legislation/legislation_tab.dart`，纯前端排序）

1. `_provinceCodeOrder`（L55）`['PLG','PRP','PSN','CLEG']` → `['PLG','PSN','PRP','CLEG']`。
2. `_selectProvince` 次级比较（L103）`cidShortNameOrFullName` → `(int.tryParse(a.cityCode)??0).compareTo(...)`。
   省三机构各单条分组，次级键仅对 CLEG 多市生效。
3. 文件头注释 L13-14「省立法院/省众议会/省参议会」→「省立法院/省参议会/省众议会 + 市按代码升序」。

## 边界 / 依据

- 数据源 `listByProvinceAndCodes(provinceCode,{PLG,PSN,PRP,CLEG})` 已返回省三机构 + 全部市 CLEG；
  store/后端/链不动。
- 市代码为每省三位补零 001/002…（admin_division_dto 例 `city|LN|001`，ADR-021）。
- `Institution.cityCode`（institution.dart:56）现成。

## 输出物 / 验收

- 改 legislation_tab.dart 两处排序 + 注释；`flutter analyze` 0 问题。
- 无既有立法 tab 排序测试；可选新增 widget 测试断言省三机构顺序 + 市 cityCode 升序（需 fake InstitutionRepository）。

## 执行结果（2026-07-21）

- `legislation_tab.dart`：
  - `_provinceCodeOrder` → `['PLG','PSN','PRP','CLEG']`（省立法院→参议会→众议会→市立法会）。
  - 排序抽成顶层 `@visibleForTesting sortProvinceLegislationRows(rows)`：主序按 `_provinceCodeOrder`，
    同码内按 `int.tryParse(cityCode)` 升序（数值序，防未补零）；`_selectProvince` 改调它（消除内联重复）。
  - 文件头注释 L13-14 同步为「省立法院/省参议会/省众议会 + 市按代码升序」。
- **测试**：新增 `test/citizen/legislation/legislation_order_test.dart` 两用例——①乱序输入→
  `[PLG,PSN,PRP,CLEG001,CLEG002,CLEG003]`；②市代码数值序（1<2<10）。`flutter analyze` 0 问题，2/2 通过。
- **边界**：store/后端/链未动（数据源已返回省三机构+全部市 CLEG）；国家级本就参议会→众议会，未改。
