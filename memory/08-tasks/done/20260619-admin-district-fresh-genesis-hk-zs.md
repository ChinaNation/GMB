# 任务卡：行政区重新创世修正香港九龙香洲中山

- 任务编号：20260619-admin-district-fresh-genesis-hk-zs
- 状态：done
- 所属模块：sfid,wuminapp
- 当前负责人：Codex
- 创建时间：2026-06-19

## 任务需求

按重新创世口径修正 `sfid/backend/china/china.sqlite` 行政区真源:岭南省拆分香港市和九龙市,新增香洲市;重建广东省中山市,清理中山镇级伪市壳;辽宁 `LI/012 中山市` 改名为 `青泥市`;版本回到 1,墓碑清空,市镇 code 按新基线重排。

## 执行顺序

1. 先只修改 `china.sqlite`。
2. 修改后分析原东莞同类镇级伪市壳问题。
3. 行政区数据确认没问题后,再生成行政区包、reconcile 公权机构、生成公权机构包。

## 边界

- 本任务未修改 `citizenchain/runtime/`。
- 行政区校验通过后已生成行政区包。
- 行政区校验通过后已执行公权机构 reconcile、严格校验和公权机构包生成。
- 重新创世基线下清空 `city_tombstones`、`town_tombstones` 和变更历史。

## 预计修改目录

- `sfid/backend/china/`:修改 `china.sqlite`,重建行政区真源并进行本地校验。
- `sfid/backend/`:用当前 `china.sqlite` 对账本地 SFID 运行库公权机构目录。
- `wuminapp/assets/admin_divisions/`:重新生成行政区随包只读快照。
- `wuminapp/assets/public_institutions/`:重新生成公民端公权机构随包只读快照。
- `memory/`:记录任务卡、ADR 和技术文档验收结果。

## 当前进展

- 已修改 `sfid/backend/china/china.sqlite`。
- 已按重新创世口径将行政区版本重置为 `1`。
- 已清空 `city_tombstones`、`town_tombstones`、`admin_division_change_log`、`admin_division_versions` 旧变更记录,并写入版本 `1` 基线记录。
- 已将辽宁 `LI/012 中山市` 改名为 `青泥市`,未新增 `中山区` 或 `大连中山市`。
- 已将岭南省城市顺序重建为:香港市、九龙市、新界市、澳门市、珠海市、香洲市、金湾市、斗门市、盐田市、坪山市。
- 已新增岭南省 `九龙市`,并按镇级承载九龙现实区域。
- 已新增岭南省 `香洲市`,包含 `坦洲镇`、`神湾镇`、`三乡镇`、`唐家湾镇`。
- 已重建广东省 `中山市`,合并原中山镇级伪市壳,并将原 `东升镇` 并入 `小榄镇`。
- 为避免全国市名重复,已将原 `XK/021 九龙市` 改名为 `呷尔市`。
- 已删除广东省重复的 `GD/132 三市` 残留;该残留与现有 `GD/038 三水市` 的镇村数据完全重复,删除后已将广东后续市代码整体前移。
- 已按确认方案拆分原东莞区域,不恢复单一 `东莞市`,改为 `中堂市`、`莞城市`、`石龙市`、`万江市`、`虎门市`、`常平市`、`塘厦市` 七个市。
- `企石镇`、`横沥镇` 已归入 `常平市`;`沙田镇`、`厚街镇` 已归入 `虎门市`。
- `松山湖市`、`东莞港市`、`东莞生态园市` 功能区壳已删除。
- `石龙市` 已重建为 `石龙镇`、`石排镇`、`茶山镇`、`东坑镇`、`寮步镇`、`大岭山镇`、`大朗镇`,原 `高庄镇`、`龙兴镇`、`人民路镇`、`龙河镇` 假镇已删除。

## 已执行校验

- `PRAGMA integrity_check`:通过。
- `python3 sfid/backend/china/check_code_immutable.py`:通过。
- 全国市名重复检查:0。
- 同市镇名重复检查:0。
- 同镇村/路名重复检查:0。
- 孤儿镇检查:0。
- 孤儿村/路检查:0。
- 市 code 连续性检查:0 断档。
- 镇 code 连续性检查:0 断档。
- 省数量:43。
- 市数量:2898。
- 镇数量:39724。
- 村/路数量:603901。

## 东莞复核结论

广东省原东莞区域已按确认口径处理:以 `中堂市`、`莞城市`、`石龙市`、`万江市`、`虎门市`、`常平市`、`塘厦市` 七个市承载原东莞 4 个街道和 28 个镇。功能区壳 `松山湖市`、`东莞港市`、`东莞生态园市` 已删除。

当前 `china.sqlite` 行政区校验已通过。下一步可以生成行政区包,再执行公权机构 reconcile、严格校验和公权机构包生成。

## 数据包与公权机构结果

- 已运行 `node wuminapp/tools/generate_admin_division_bundle.mjs`。
- 行政区包:version=1,province_count=43,city_count=2898,town_count=39724,china_sqlite_sha256=`0db5080d05c1dcb184c6c8f9b4ad0d6fc4fbef17f88c645ba256e86ab450161d`。
- 已运行 `sfid-backend reconcile-gov --changed-only`:scopes=43,inserted=3543,updated=245100,account_inserted=497329,removed=4243。
- 已运行 `sfid-backend check-gov --strict`:ok=true,manifest_current=true,target_count=248643,active_count=248643,missing=0,mismatched=0,missing_accounts=0,obsolete=0,catalog_hash=`856b48488086cabda027d16d47df352642377c31c41aeabcf927caa8187758ac`。
- 已用当前代码临时启动 SFID 后端 `127.0.0.1:8898`,确认旧 `127.0.0.1:8899` 服务仍为旧接口后,从新接口重新生成 `wuminapp/assets/public_institutions/`。
- 公权机构包:version=1,provinces=43,total=248643;与 strict target_count 差额 0。
- 已确认公民端包包含 `CITY_POLICE=2898`、`CITY_EDU=2898`、`NATIONAL_EDU=1` 等公权机构,未沿用旧接口排除结果。
