# CitizenWallet/OnChina 四项修复(钱包详情红屏/启动闪屏/管理员列表慢/治理文案)

任务需求(用户报障,1/2/4/5 直接修,3 仅再分析):
1. 钱包详情页整屏红屏崩溃 → 修
2. 打开钱包先闪蓝盾牌再进列表 → 修(去掉突兀闪屏)
4. 每次进联邦注册局→管理员列表都要等很久 → 修(加缓存)
5. 机构治理页删除"管理员是人,岗位是职位…"整段 Alert 描述 → 修

所属模块:citizenwallet(1、2)+ citizenchain/onchina(4、5)

根因:
1. wallet_detail_page.dart:315 对 ss58Address 裸 substring 无长度保护,空/短地址(残缺 Isar 实体)→ RangeError → 红屏 ErrorWidget;列表用带保护的 _truncateSs58Address 故不崩。
2. main.dart:_checking 启动态显示盾牌+转圈,期间异步读 AppLockService.isPinSet()+SecureStorage;无锁→进 HomePage。属正常门禁加载态,但闪屏突兀。
4. list_federal_registry_admins 每次开 tab 无缓存全量链读:fetch_all_federal_registry_assignments(全省扫描)+fetch_account_balances_onchain(批量余额),无 TTL 缓存。
5. PrivateDetailLayout.tsx:926-927 + GovDetailPage.tsx:253-254 两处相同 Alert message+description。

修复:
1. 详情页加 _shortAddress 守卫(length<=16 返回原串),替换裸 substring。
2. _checking 态改为中性加载(去掉盾牌图标),避免蓝盾牌闪一下。
3. (仅分析,见任务外结论)
4. AppState 增 federal_admins_cache(Arc<Mutex<HashMap<省, (Instant, Vec<Row>)>>>),TTL 30s;FederalRegistryAdminRow 加 Clone;缓存路径不持锁跨 await。
5. 删两处 Alert 整段。

验收:
- citizenwallet flutter analyze/build 通过;详情页空地址不崩
- onchina cargo check 通过;管理员列表二次开 tab 命中缓存快速返回
- 前端两处文案删除、无残留

状态:已完成 1/2/4/5(2026-07-24);3 仅分析(见 memory)

落地:
- 1 wallet_detail_page.dart:加 _shortAddress 守卫替换裸 substring。flutter analyze 通过。
- 2 main.dart:_checking 态去盾牌改中性 spinner。flutter analyze 通过。
- 4 onchina:AppState 增 federal_admins_cache(TTL 30s)+ FederalRegistryAdminRow derive Clone + catalog.rs 命中/回填(锁不跨 await)。cargo check 通过。
- 5 删两处治理 Alert;GovDetailPage 顺带删未用 Alert import(PrivateDetailLayout 的 Alert 607 行还在用,保留)。tsc -b 通过。
- 3 根因:onchina 只对公权机构(PublicManage)做链→本地投影;公民与私权机构/基金会本地库只由注册局创建流程写入,无链投影,故创世/纯链上实体在本地列表不可见。见 [[onchina-chain-projection-asymmetry-citizen-gap]]。
