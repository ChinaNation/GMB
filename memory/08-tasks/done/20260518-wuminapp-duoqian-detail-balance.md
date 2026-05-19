# 任务卡：wuminapp 多签详情页余额本地优先修复

任务需求：

- 修复多签详情页进入后账户余额直接不显示的问题。
- 保持详情页首屏本地持久化优先，不恢复全屏转圈。
- 状态刷新时间和余额刷新时间必须分开判断；状态 TTL 未过不能阻止 Active 余额首次读取。
- 链上余额读取失败时保留本地余额，不把页面打成加载失败。
- 完成后更新文档、完善中文注释、清理残留。

所属模块：

- wuminapp / governance / personal-manage / organization-manage / isar

预计修改目录：

- `wuminapp/lib/governance/personal-manage/`：修复个人多签详情页 Active 余额静默刷新逻辑；涉及 Flutter 代码。
- `wuminapp/lib/governance/organization-manage/`：修复机构多签详情页 Active 余额静默刷新逻辑；涉及 Flutter 代码。
- `memory/05-modules/wuminapp/governance/`：补充多签详情页状态 TTL 与余额 TTL 分离规则；涉及技术文档。
- `memory/05-modules/wuminapp/personal-manage/`：补充个人多签详情页余额刷新规则；涉及技术文档。
- `memory/08-tasks/`：记录任务进度、验收标准和完成摘要；涉及任务文档。

验收标准：

- Active 多签详情页本地没有余额快照时，进入页面后能静默读取余额并显示。
- 状态 TTL 未过时，不重复读取管理员/阈值等链上详情。
- 余额刷新失败不清空已有本地余额。
- 详情页不恢复全屏转圈和“同步中”提示。
- `flutter analyze` 通过。
- 相关治理测试通过。

执行进度：

- [x] 创建任务卡
- [x] 拆分余额刷新 TTL
- [x] 更新文档
- [x] 残留检查与测试

完成摘要：

- 个人/机构多签详情页新增独立余额刷新判断：Active 账户本地余额为空或余额刷新时间超过 10 分钟时，只静默读取余额。
- 状态 TTL 未过时，不再阻止 Active 余额首次读取。
- 详情页完整链上刷新时，如果余额读取失败，会保留旧余额，不把 `balanceYuan` 覆盖为 null。
- 多签列表页批量刷新状态写详情快照时，会保留已有 `balanceYuan` 和 `lastBalanceRefreshAtMillis`，避免状态刷新覆盖余额。
- 已同步 wuminapp 总文档、governance 技术文档和 personal-manage 技术文档。

验收结果：

- Active 多签详情页本地没有余额快照时，会触发单独余额刷新。
- 状态 TTL 未过时，不重复读取管理员/阈值。
- 余额读取失败保留本地旧余额。
