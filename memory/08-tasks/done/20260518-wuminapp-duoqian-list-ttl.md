# 任务卡：wuminapp 多签列表 TTL 与批量刷新优化

任务需求：

- 优化 wuminapp 底部“多签”页面，避免每次进入页面都阻塞查询区块链。
- Active 多签账户 60 分钟内不自动重复查询链上状态。
- Pending / Closed 多签账户 10 分钟内不自动重复查询链上状态。
- 只有用户下拉刷新才强制刷新已知账户状态并执行全量 discovery。
- 自动 discovery 只允许首次进入多签 Tab 或本机钱包列表变化时触发。
- 创建、关闭、投票、删除返回列表时只刷新相关账户，不全量扫描。
- 链上状态刷新使用分阶段批量 storage 查询，减少手机端等待和链节点压力。

所属模块：

- wuminapp / governance / personal-manage / organization-manage / isar

输入文档：

- `memory/07-ai/unified-required-reading.md`
- `memory/07-ai/workflow.md`
- `memory/07-ai/context-loading-order.md`
- `memory/07-ai/document-boundaries.md`
- `memory/07-ai/definition-of-done.md`
- `memory/07-ai/pre-submit-checklist.md`
- `memory/07-ai/unified-naming.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/module-definition-of-done/wuminapp.md`
- `memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md`
- `memory/05-modules/wuminapp/personal-manage/PERSONAL_MANAGE_WUMINAPP_TECHNICAL.md`

必须遵守：

- 不改变链上 storage 契约、交易载荷和投票职责边界。
- 不把网络失败误判为链上 Closed。
- 多签页首屏只能等待本地 Isar 读取，不得等待链上状态刷新或 discovery。
- 所有 Isar 读写必须走 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()`。
- 关键 Flutter 交互和本地状态逻辑必须补中文注释。

输出物：

- 多签列表页加载、TTL、下拉刷新和精准刷新代码。
- 个人多签批量状态查询代码。
- 机构多签批量状态查询代码。
- 本地状态读取扩展，返回状态和最近链上同步时间。
- 技术文档更新。
- 残留检查。

预计修改目录：

- `wuminapp/lib/governance/`：调整多签列表页首屏、本地缓存、后台刷新和精准刷新入口；涉及 Flutter 业务代码。
- `wuminapp/lib/governance/personal-manage/`：增加个人多签批量状态读取；涉及 Flutter 业务代码。
- `wuminapp/lib/governance/organization-manage/`：增加机构多签批量状态读取；涉及 Flutter 业务代码。
- `wuminapp/lib/isar/`：扩展本地状态读取结构，复用 `AppKvEntity.intValue` 作为同步时间；涉及本地存储代码，不新增 schema。
- `memory/05-modules/wuminapp/`：同步多签列表加载与 discovery 边界；涉及技术文档。
- `memory/08-tasks/`：记录任务进度、验收标准和完成摘要；涉及任务文档。

完成摘要：

- 多签列表首屏已改为只读本地 Isar，不再等待链上状态查询。
- 本地状态读取已扩展为 `status + lastSyncAtMillis`，复用 `AppKvEntity`，未新增 Isar schema。
- Active 状态自动刷新 TTL 为 60 分钟；Pending / Closed 状态自动刷新 TTL 为 10 分钟；下拉刷新忽略 TTL。
- 自动 discovery 已收敛为首次进入或钱包 pubkey fingerprint 变化时触发；下拉刷新才强制全量 discovery。
- 个人多签新增 `fetchPersonalAccountsBatch()`，按 `PersonalDuoqians / Subjects / ActiveDynamicThresholds / PendingDynamicThresholds` 分阶段批量读取。
- 机构多签新增 `fetchDuoqianAccountsBatch()`，按 `AccountRegisteredSfid / InstitutionAccounts / Subjects / ActiveDynamicThresholds / PendingDynamicThresholds` 分阶段批量读取。
- `ChainRpc` 新增 `fetchStorageBatchChunked()`，统一限制 storage 批量读取分块大小。
- 从详情页返回列表时只精准刷新当前多签账户；创建返回只重读本地记录，不触发全量扫描。
- 已同步 wuminapp 总文档、治理技术文档和 personal-manage 技术文档。

验收结果：

- `flutter analyze`：通过。
- `flutter test test/governance/personal-manage test/governance/organization-manage`：通过。
- `git diff --check`：通过。
- 残留扫描：未发现旧的列表页逐个账户查链、每日自动扫描或“扫描我的多签”入口残留。

执行进度：

- [x] 创建任务卡
- [x] 改造本地首屏与 TTL 刷新
- [x] 实现个人/机构批量状态查询
- [x] 收敛 discovery 触发条件
- [x] 同步文档
- [x] 残留检查与测试
