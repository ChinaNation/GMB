任务需求：
- 优化整个 wuminapp 中仍然依赖频繁链上读取的页面和服务。
- 将适合首屏展示、列表展示、历史保留、低频状态显示的数据改为本地持久化优先。
- 将仍需读取区块链的数据改为 TTL、批量读取、后台刷新、手动刷新和提交前复核组合，降低对链节点的依赖和压力。
- 完成后更新文档、完善中文注释并清理残留。

所属模块：
- wuminapp / governance / proposal
- wuminapp / governance / admins-change
- wuminapp / transaction / duoqian-transfer
- wuminapp / transaction / offchain-transaction
- wuminapp / asset

必须遵守：
- 本地持久化只作为展示读库和历史快照，不能作为投票、执行、转账、nonce、提交前余额和提交前身份判断的最终真相。
- 投票、执行、转账提交前必须重新读取链上 runtime storage 或 runtime call。
- 不新增 Isar collection schema；优先复用 `AppKvEntity`，避免迁移和生成文件扩大化。
- 不保留旧的重复链读取路径；完成后清理残留。

输出物：
- 提案详情/投票状态本地持久化快照。
- 管理员主体本地持久化短缓存。
- 余额展示本地持久化快照。
- 清算行目录链上 endpoint 本地持久化短缓存。
- 批量查票链读取能力。
- 文档更新、残留检查和验证记录。

验收标准：
- 进入提案详情页时，有本地快照时可先显示，不再必须等待所有链上读取完成。
- 管理员投票状态读取支持批量 storage 查询，避免按管理员逐条 RPC。
- 管理员主体在 App 重启后仍可先读本地快照，后台再低频刷新链上数据。
- 余额展示可先读本地快照，提交前余额校验仍实时读链。
- 清算行 endpoint 不再每次搜索结果都必然逐条读链。
- `dart analyze lib test` 和相关治理/交易测试通过。

## 执行记录

- 状态：done
- 本地读库：新增 `ProposalDetailLocalStore`，复用 Isar `AppKvEntity` 保存提案详情、管理员快照、投票记录、待确认投票和业务详情；新增 `AccountBalanceSnapshotStore` 保存账户余额展示快照。
- 批量链读取：`InternalVoteQueryService` 增加 `fetchAdminVotesBatch()`；`RuntimeUpgradeService` 增加 `fetchJointAdminVotesBatch()`；转账详情、多签管理详情、Runtime 升级详情、广场红点和个人多签管理员激活列表均改为批量查票。
- 管理员主体：`AdminSubjectService` 增加 10 分钟本地持久化短缓存，保留 30 秒内存缓存和提交前链上复核边界。
- 余额展示：转账页、安全基金页、手续费划转页、个人/机构多签关闭页、治理机构详情主余额和更多账户余额接入本地余额快照；提交前余额 guard 仍实时读链。
- 清算行目录：`ClearingBankDirectory` 对链上 `ClearingBankNodes` endpoint 增加 24 小时本地快照，对 `UserBank[user]` 增加 3 分钟短快照。
- 文档：已同步 `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md` 与 `memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md`。
- 测试：新增 `test/governance/proposal_detail_local_store_test.dart`；`dart analyze lib test`、治理/提案相关 `flutter test`、`git diff --check` 均通过。

