任务需求：
- 在 wuminapp-公民-治理-治理机构页面中，省储会（43）和省储行（43）默认折叠，标题右侧显示向右三角；点击后三角向下并展开本分组全部机构卡片。
- 国储会保持现状，不增加折叠交互。
- 删除治理机构列表的 `_sorted()` 自动排序，不再按管理员机构优先顶到前面。
- 省储会、省储行卡片支持长按拖拽，只允许在所属分组内调整位置。
- 拖动排序只在本机生效，保存为本机 UI 偏好，不写链、不跨设备同步。

所属模块：
- wuminapp / governance
- wuminapp / citizen

输入文档：
- memory/07-ai/unified-required-reading.md
- memory/07-ai/workflow.md
- memory/07-ai/context-loading-order.md
- memory/07-ai/document-boundaries.md
- memory/07-ai/definition-of-done.md
- memory/07-ai/pre-submit-checklist.md
- memory/07-ai/unified-naming.md
- memory/07-ai/module-definition-of-done/wuminapp.md
- memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md
- memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md

必须遵守：
- 不修改国储会交互。
- 不修改链上治理机构静态注册表和 runtime 数据。
- 不把本机拖拽排序同步到链上或其他设备。
- 不恢复管理员机构优先自动排序。
- 不突破 governance 模块边界。

输出物：
- 治理机构列表 UI 代码调整
- 本机排序保存逻辑
- 测试
- 文档更新
- 残留清理

验收标准：
- 省储会、省储行默认折叠。
- 点击三角可展开/折叠对应分组。
- 国储会仍直接展示。
- `_sorted()` 自动排序已删除。
- 省储会、省储行可在本分组内长按拖拽排序。
- 拖拽排序持久化到本机，下次进入仍按本机顺序展示。
- 文档已更新，残留已清理，wuminapp 完成标准已对照。

## 执行记录

- 状态：done
- 代码：`GovernanceListPage` 已删除 `_sorted()` 自动排序；国储会保持直接展示，省储会、省储行默认折叠并通过标题三角展开。
- 排序：省储会、省储行卡片支持同分组内长按拖拽；顺序保存到本机 `SharedPreferences`，只记录 `sfidNumber` 列表。
- 视觉修正：省储会、省储行折叠控件已改为标题行最右侧的线性右箭头/下箭头；国储会卡片已改为整行宽度显示到右侧边缘，并加高到 76px。
- 文档：已同步 `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md` 和 `memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md`。
- 验证：`dart analyze lib test`、`flutter test test/governance/governance_list_page_test.dart`、`git diff --check` 已通过。
