任务需求：
- 优化 wuminapp-公民-治理-治理机构详情页加载方式。
- 区分治理机构固定本地数据、后台更新数据和实时刷新数据。
- 固定本地数据进入页面立即展示，不再被链上管理员、余额、提案读取阻塞。
- 动态链上数据按区域后台读取并局部显示加载/错误/重试状态。
- 完成后同步文档、补充必要中文注释并清理残留。

所属模块：
- wuminapp / governance
- wuminapp / transaction / duoqian-transfer

输入文档：
- memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md
- memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md

必须遵守：
- 不修改治理机构静态注册表和链上账户定义。
- 不把固定本地数据显示依赖链上读取结果。
- 不把治理机构详情页的动态数据改成本地伪造结果。
- 不突破 governance 与 transaction 模块边界。

输出物：
- 治理机构详情页分区加载代码。
- 管理员/提案/余额缓存和刷新边界优化。
- 技术文档更新。
- 残留检查与验证记录。

验收标准：
- 进入治理机构详情页后，机构名称、身份 ID、主账户、阈值等固定数据立即显示。
- 管理员、余额、提案读取失败时只影响对应区域，不出现整页转圈。
- 下拉刷新会强制刷新管理员、余额、提案和已展开的更多账户余额。
- 文档已说明固定本地数据、后台更新数据和实时刷新数据的边界。
- `dart analyze lib test` 和相关治理测试通过。

## 执行记录

- 状态：done
- 代码：`InstitutionDetailPage` 已删除整页 `_loading` 阻塞，改为管理员、主账户余额、提案列表三个分区后台加载状态。
- 首屏：机构名称、身份 ID、主账户地址、制度账户类型和固定阈值现在直接从本地数据展示，不等待链上 RPC。
- 缓存：`AdminAccountService` 增加 30 秒共享短缓存与 in-flight 去重；`DuoqianTransferProposalFeed` 增加机构主余额和机构可见提案短缓存，下拉刷新/返回刷新会强制清理。
- 文档：已同步 `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md` 与 `memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md`。
- 验证：`dart analyze lib test`、`flutter test test/governance/governance_list_page_test.dart`、`flutter test test/governance/admins-change/institution_admin_service_test.dart`、`flutter test test/governance/admins-change/admins_change_codec_test.dart`、`git diff --check` 已通过。
