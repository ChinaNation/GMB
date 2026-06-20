# 任务卡：wuminapp 机构注册表重生成(地址/余额/管理员三断修复)

## 背景(根因已查实,2026-06-10 诊断)

wuminapp 所有机构账户地址不准确、余额不显示、管理员不显示。三个症状同一根因:

- `wuminapp/lib/governance/organization-manage/governance_institution_registry.generated.dart` 是 `tools/generate_wuminapp_governance_registry.mjs` 从链端 `citizenchain/runtime/primitives/china/china_{cb,ch}.rs` 生成的硬编码表。
- `84080b6a`(重构多签账户体系,derive_duoqian_account 加入 ss58 字节重派生全部地址)+ `c13e0f82`(身份ID协议重构,机构改名「公民储备委员会」)+ 重新创世烘焙新地址后,**生成器没有重跑**,注册表停留在旧地址/旧名(国储会:表内 `c17a81b8…` vs 链端真源 `39936ebd…`)。
- wuminapp 客户端无机构派生逻辑,地址展示/余额查询(`fetchFinalizedBalance(mainAccount)`)/管理员查询(`AdminAccounts` 以 mainAccount 为 key)全部依赖此表 → 全部机构同时断。

## 方案

1. 重跑 `node tools/generate_wuminapp_governance_registry.mjs` 重生成注册表(44 国储/省储会 + 43 省储行)。
2. 全仓扫描旧地址残留(`c17a81b8…` 等)与旧机构名「国家储备委员会」,确认无第二处硬编码。
3. wuminapp `flutter analyze` + `flutter test` 回归。

## 验收

- [x] 注册表国储会 mainAccount = `39936ebd…1d14db3d`,名称 = 国家公民储备委员会;87 机构(44 CB + 43 CH)与 china_{cb,ch}.rs 逐字段比对 0 mismatch(name/sfid/main/fee/stake/orgType 全字段脚本核对)
- [x] 全仓旧地址(`c17a81b8…`/`1742656a…`/`b8a5c135…`)零残留;旧名「国家储备委员会」仅剩 wuminapp/test/governance/governance_list_page_test.dart 本地自造夹具(不读注册表,不影响行为,未动)
- [x] `flutter analyze` 0 issue + `flutter test` 192/192 全过
- [ ] 真机:机构详情页地址正确、余额显示、管理员列表显示(user 验证)

## 完工记录(2026-06-11,代码完工,待真机验证)

- 重跑 `node tools/generate_wuminapp_governance_registry.mjs`,重生成 `governance_institution_registry.generated.dart`(87 机构,742 行级 diff);地址展示/余额查询/管理员查询三条链路共用此表,一次重生成三个症状同时修复。
- 防再犯提示:**每次重新创世或 china_*.rs 重派生后必须重跑此生成器**,建议挂进 bake-chainspec 流程清单。
- ~~待办:wumin 公民钱包 `lib/chain/institutions.dart` 机构名陈旧~~ → 已完成,见 `done/20260611-wumin-institutions-name-sync.md`。
