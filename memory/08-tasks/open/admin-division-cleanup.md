# 行政区重新创世清理

- 状态:完成
- 模块:CID 行政区
- 创建时间:2026-06-19

## 目标

按重新创世口径清理 `citizencode/backend/china/china.sqlite` 中的行政区数据,只处理行政区名字、归属、伪行政区和 code 重排。

本任务不生成 citizenapp 行政区包、不生成 CPMS 快照、不生成 CID 公权机构,不修改 `citizenchain/runtime/**`。

## 规则

- 重新创世:不写 city/town tombstone,最终清空 tombstone 表。
- 版本保持 `admin_division_version=1`。
- 省 code 保持 43 省既有 code。
- 市 code 按每省最终顺序从 `001` 重排。
- 镇 code 按每市最终顺序从 `001` 重排。
- 镇下地址段无独立行政区 code,只重排 `sort_order`。
- 省名全国唯一、市名全国唯一、同一市下镇名唯一、同一镇下地址段名唯一。

## 已确认处理

- 河南 `HE/037 人和市` 保留,对应现实河南平顶山市石龙区;不新增河南 `石龙市`。
- 广东 `GD/101 石龙市` 保留,东莞拆分不整体重做。
- `HE/037 人和市 / 龙兴镇` 补齐 `北郎店社区村`。
- 矿区类市按“下辖镇少于 5 个删除并入周边市;5 个以上按正式县级行政区改名保留”处理。
- 海南儋州区域保留 `儋州市` 名称,在 `兰洋市 / 新州市 / 白马井市 / 那大市` 四组中选一组命名为 `儋州市`。

## 执行结果

- `china.sqlite` 当前仍为版本 `1`。
- 数据规模:43 省、2872 市、39227 镇、598655 地址段。
- SHA-256:`c477cb5a300eac9f56d53beaef235617a6fc64584a0f1cffd8c85b2537840bbb`。
- `address_units.source_code`:598655 条全部非空;其中 535084 条为官方数字来源码,63571 条为 `LOCAL-*` 本地稳定来源码。
- 福建、海南删除/合并后的市 code 空洞已按重新创世口径重排,省内市 code 连续。
- `city_tombstones=0`,`town_tombstones=0`。
- 海南儋州区域收敛为 `儋州市`、`兰洋市`、`白马井市`、`新州市`。
- 河南 `人和市` 保留为现实石龙区系统唯一名,广东 `石龙市` 保留,全国只有一个 `石龙市`。
- 少于 5 个镇的矿区壳删除并入相邻市;5 个及以上镇的正式矿区改为普通市名。
- `县市` 后缀、截断名、重复错挂壳和已确认伪镇已处理。
- 重复错挂壳和已确认伪镇已删除;最终审计发现的 42 个镇级伪行政区已删除,同步删除其下 235 条地址段。内蒙古旗类后缀已去掉“旗”,河南 `社旗市` 保留本名。
- 154 条地址段名称中的组织或管理机构词已清理;`社区` 作为合法地址段保留。
- 568 条 `xx虚拟路` 已归一为 `xx`;3 条纯 `虚拟路` 已删除,其中 2 条对应的功能区壳镇同步删除。
- 46 条原始名含 `社区` 的纯功能词地址段已恢复为 `xx社区`;26 条 `LOCAL-*` 来源的 `xx虚拟路` 合成占位地址段已删除,同步删除因此空掉的 24 个镇并重排受影响市的镇 code。
- 2026-06-20:已重新生成 citizenapp 行政区包,manifest `version=1`,43 省、2872 市、39227 镇。
- 2026-06-20:已执行 CID 公权机构 `reconcile-gov --changed-only` 和 `check-gov --strict`;本轮复跑 reconcile 为 `scopes=0 inserted=0 updated=0 account_inserted=0 removed=0`,strict 结果为 `target_count=245716 active_count=245716 missing=0 mismatched=0 missing_accounts=0 obsolete=0`。
- 2026-06-20:已通过当前 CID 真实公开接口重新生成 citizenapp 公权机构包,43 省、245716 条,并完成行政区 code 交叉检查 `bad_count=0`。

## 不做范围

- 已生成 citizenapp 行政区包和公权机构包。
- CPMS 不维护第二份行政区数据包源码;发布安装包时从同一 `china.sqlite` 拷贝随包快照。
- 已同步 CID 运行库自动公权机构目录。
- 不修改 runtime。

## 验收

- `PRAGMA integrity_check`:ok。
- `python3 citizencode/backend/china/check_code_immutable.py`:PASS。
- 省名重复:0。
- 市名全国重复:0。
- 同市镇名重复:0。
- 同镇地址段名重复:0。
- city/town tombstone:0。
- 内蒙古旗类 `旗市` 残留:0。
- `县市` 残留:0。
- 镇级伪行政区关键词命中:0。
- 本轮按要求未生成 citizenapp / CPMS 数据包,未生成 CID 公权机构。
