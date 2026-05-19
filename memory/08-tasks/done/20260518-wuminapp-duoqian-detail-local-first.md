# 任务卡：wuminapp 多签详情页本地优先加载

任务需求：

- 优化 wuminapp 多签账户详情页，避免每次进入个人/机构多签详情都全屏转圈等待链上。
- 明确区分本机持久化数据与链上权威数据；本机已储存的数据必须直接显示。
- 链上读取只按 TTL、下拉刷新和交易/投票/关闭返回等精准场景触发。
- 不显示“同步中”等多余提示；链上刷新失败不得覆盖或阻断本地页面。
- 完成后更新技术文档、补充中文注释并清理残留。

所属模块：

- wuminapp / governance / personal-manage / organization-manage / isar

必须遵守：

- 不改变链上 storage 契约、交易载荷和投票职责边界。
- 不把网络失败误判为链上 Closed。
- 详情页首屏只能等待本地 Isar 读取，不得等待链上 RPC。
- 所有 Isar 读写必须走 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()`。
- 关键 Flutter 交互和本地状态逻辑必须补中文注释。

预计修改目录：

- `wuminapp/lib/isar/`：增加多签详情本地持久化快照读写；涉及本地存储代码，不新增 Isar schema。
- `wuminapp/lib/governance/personal-manage/`：改造个人多签详情页首屏本地展示、TTL 静默刷新和强制刷新；涉及 Flutter 业务代码。
- `wuminapp/lib/governance/organization-manage/`：改造机构多签详情页首屏本地展示、TTL 静默刷新和强制刷新；涉及 Flutter 业务代码。
- `memory/01-architecture/wuminapp/`：记录 wuminapp 多签详情页 local-first 规则；涉及技术文档。
- `memory/05-modules/wuminapp/governance/`：补充治理多签详情页数据分层与刷新边界；涉及技术文档。
- `memory/05-modules/wuminapp/personal-manage/`：补充个人多签详情页本地持久化状态规则；涉及技术文档。
- `memory/08-tasks/`：记录任务进度、验收标准和完成摘要；涉及任务文档。

验收标准：

- 从多签列表进入个人详情页无全屏转圈。
- 从多签列表进入机构详情页无全屏转圈。
- 断网情况下，只要本地有记录，详情页仍可显示。
- TTL 内重复进入不触发详情链上查询。
- 下拉刷新、投票/关闭/转账返回可强制刷新当前账户。
- `flutter analyze` 通过。
- 相关治理测试通过。
- 文档、注释和残留检查完成。

执行进度：

- [x] 创建任务卡
- [x] 增加本地详情快照读写
- [x] 改造个人多签详情页
- [x] 改造机构多签详情页
- [x] 更新文档
- [x] 残留检查与测试

完成摘要：

- 新增 `DuoqianLocalDetailSnapshot`，通过 `AppKvEntity` 持久化个人/机构多签详情快照，不新增 Isar schema。
- 多签列表链上批量刷新成功时，同步写入详情快照，详情页后续可直接读取本机已储存数据。
- 个人多签详情页首屏改为读取 `PersonalDuoqianEntity`、`PersonalDuoqianLocalState`、`personal_duoqian_detail:*` 和本地提案金额快照，不再等待链上 RPC。
- 机构多签详情页首屏改为读取 `DuoqianInstitutionEntity`、`InstitutionDuoqianLocalState` 和 `institution_duoqian_detail:*`，不再等待链上 RPC。
- 详情页自动链上刷新受 TTL 控制；Active 为 60 分钟，Pending/Closed 为 10 分钟。
- 下拉刷新、转账提案返回、管理员页返回、关闭/撤销操作前后才强制刷新当前账户。
- 详情页已移除全屏转圈和“加载失败”首屏路径；链上刷新失败保留本机持久化数据。
- 已同步 wuminapp 总文档、治理技术文档和 personal-manage 技术文档。

验收结果：

- `flutter analyze`：通过。
- `flutter test test/governance/personal-manage test/governance/organization-manage`：通过。
- 残留扫描：个人/机构多签详情页不再包含全屏 `CircularProgressIndicator`、`fetchPersonalAccount()`、`fetchDuoqianAccount()`、`fetchAdmins()` 或“同步中”文案。
