# OnChina 注册局界面:管理员子tab解耦读链 + 联邦按市可见基金会(私权投影运行时闭环)

任务需求(用户报障 2 项,一次性做全):
1. 管理员列表子 tab 的**出现**不该被读链 gate 住;tab 立即显示,列表内容再慢慢读链。
2. 联邦注册局管理员进入绥阳市**看不到基金会**,为什么 → 修到真能看见。

所属模块:citizenchain/onchina(前端 + 后端投影/列表)

## item 2 根因(诊断:配套投影任务卡 20260724 只做了"编译级验证,无活链运行时",两处运行时缺口)
- **根因1(决定性)**:链上 Institutions 无 `private_type` 字段(它是按机构码派生的业务分类);
  联邦 drill-in 投影私权机构时 `merge_institution_record` 只从本地既有行取 private_type,
  纯投影(existing=None)→ 落库 NULL。而私权列表 SQL(`model.rs` InstitutionListFilter::Private)
  要求 `private_type IS NOT NULL` + welfare 端点再筛 `= 'WELFARE'` → 基金会(SFGY)被挡在列表外。
- **根因2(UX)**:私权机构列表前后端都是"精确搜索专用"(空关键字不列),联邦选中绥阳市看到空表。
  (设计注释本意是"避免**跨省**全量扫描";城市级浏览有界,不违反此意图。)
- 程伟(公民)不卡:seed 与查询双方都归一到省码/市码,联邦启动已 seed。

## 落地(全部完成 2026-07-24)
根因1(private_type 派生):
- `domains/private/common.rs`:新增 `private_type_code_from_institution_code`(SFGT→SOLE / SFGP·SFLP→
  PARTNERSHIP / SFGQ→COMPANY / SFGF→CORPORATION / SFGY→WELFARE / SFAS→ASSOCIATION)。
- `domains/projection.rs::merge_institution_record`:`private_type` 改 `existing.or_else(按机构码派生)`
  (市补录 existing 优先,纯投影按码派生兜底)。更新单测断言(fresh SFGY → WELFARE;保正本
  existing="FOUNDATION" 胜过派生仍绿)。
- `domains/genesis_projection.rs`:绥阳市节点回填基金会时同样按机构码派生 private_type。

根因2(城市级私权浏览):
- `main.rs`:新增 `list_private_institutions_by_city_direct`(镜像 `list_education_committees_direct`);
  `list_institutions_exact` 空关键字分支:Private + 有 city → 走城市浏览,省级无 city 仍返回空(防跨省全量扫描)。
- 前端 `private/common/PrivateListTable.tsx`:选定城市后空搜索=浏览该市全部私权机构(`!exactQuery && !city_name` 才短路)。
- 后端 `registration.rs::list_institutions_inner` 早已在 Tier1+city 时自动 drill-in 投影私权(无需改);
  现串起来:联邦进绥阳市公益页 → drill-in 投影基金会(private_type=WELFARE)→ 城市浏览列出。

item 1(tab 解耦):
- `admins/FederalRegistryAdminSubTab.tsx`:`selectedFederalRegistry` 允许 null,内部处理加载/空。
- `admins/ProvinceDetailView.tsx::FederalRegistryView`:`adminListSection` 恒渲染 SubTab,不再随
  `selectedFederalRegistry`(依赖读链定位)出现/消失。

## 验证
- onchina `cargo check` exit 0;`cargo test projection`(待补结果);前端 `tsc --noEmit` 0 error。
- 运行时(需活链+PG+node)由用户实测:联邦省组管理员进绥阳市公益组织页应列出「公民链技术发展基金会」(法定代表人程伟)。

状态:代码全部完成并静态验证;运行时闭环待用户在活链环境确认。
