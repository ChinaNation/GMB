# 任务卡：citizenapp 公权机构市卡片显示 001 修复（字典灌库时序）

创建：2026-06-23

## 根因（已用真 assets + 真 Isar 端到端测试坐实）

citizenapp 公民-公权-公权机构页,每省下的市卡片 + 市页顶部都显示 `001/002` 而非市名。

根因 = **`_bootstrap` 里 `unawaited(_repo.ensureSynced())` 发射后不管**（public_page.dart:80）：
- 首装时字典(4.2 万条行政区)还在后台灌 Isar,`_loadCityVms` 的 `cityNameMap` 查到空字典 → 市名回退 code(001)。
- 更糟:这份 001 被写进 `_cityCache`(L129),之后秒显也是脏的 001。
- 字典灌完后无任何机制刷新 UI 重 join。
- 卸载重装也 001 = 每次全新装从零重灌 4.2 万条(最慢),一进去字典远没灌完。

排错验证(关键):用真实 assets + **真 Isar** 端到端跑 `ensureSynced → cityNameMap`,`divisionCount=42002`、`JL/001=南关市` 完美查到、测试全过。证明数据/解析/Isar 读写/查询全对,唯一差异 = 测试 `await` vs 真机 `unawaited`。

## 修改范围（仅前端 1 文件,链端/后端/数据包零改动）

- `citizenapp/lib/citizen/public/public_page.dart`：
  1. `_bootstrap`:`unawaited(ensureSynced())` → `unawaited(_syncThenRefresh())`，同步完成后清 `_cityCache` 脏缓存 + `_selectGroup(_selected)` 重 join。
  2. `_selectGroup` 缓存加固:仅当字典就绪(有市 join 到非 code 名)才写 `_cityCache`,避免缓存灌库未完成的脏 001。

## 验收

- [ ] flutter analyze 0 issue。
- [ ] 真实-assets 回归测试通过(loadFromBundle→cityNameMap 查到南关市)。
- [ ] 临时复现测试清理(删真 Isar 网络依赖那个,真实-assets 那个转正式回归)。

## 执行记录

- [x] 改 public_page.dart：`_bootstrap`→`_syncThenRefresh`(同步完成清脏缓存+重 join)、`_selectGroup`/`_refreshProvince` 缓存加固(字典就绪才写缓存)。
- [x] 清理/转正临时测试：删 2 个临时复现(`*_real_isar_repro`/`*_real_assets_repro`);转正式回归 `admin_division_bundle_assets_test.dart`(真 assets join 南关市)+ harness `LateDictRepo`/`buildLateDictRepo` + `public_page_test.dart` 时序回归。
- [x] flutter analyze 0 issue；`flutter test` 21 passed(时序回归+assets 回归+现有 public_page 4+admin_division 15)。
- [x] memory：[[feedback-unawaited-bg-sync-needs-completion-refresh]] + ADR-021 实现坑备注。

## 验证说明
- 静态：analyze 0；回归测试覆盖「字典延迟就绪→先 code→完成后回刷市名」+「真 assets 数据包 join 南关市」。
- 运行时最终确认：用当前代码 `flutter build` 重装,进公权页等首装字典灌完(秒级~十几秒)后市名应显示「xx市」可点入;市页顶部显示「xx市公权机构」。
- 根因坐实方式留档:`memory` feedback 记录了「真 assets+真 Isar 端到端复现」方法,避免再凭静态猜测误判。
