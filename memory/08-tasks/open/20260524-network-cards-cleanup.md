任务需求：
删除区块链软件挖矿页网络区的“总节点数”和“清算节点”两个卡片及其对应返回字段，保留治理节点、在线节点、全节点、轻节点四张卡片。

状态：
已执行

所属模块：
CitizenChain node 挖矿页网络总览

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/unified-required-reading.md
- memory/07-ai/workflow.md
- memory/07-ai/definition-of-done.md
- memory/07-ai/pre-submit-checklist.md
- memory/05-modules/citizenchain/node/mining/network-overview/NETWORK_OVERVIEW_TECHNICAL.md
- memory/05-modules/citizenchain/node/mining/dashboard/MINING_DASHBOARD_TECHNICAL.md
- memory/07-ai/module-definition-of-done/citizenchain.md

必须遵守：
- 不修改 runtime。
- 不新增链上 storage。
- 不改变全节点/轻节点后续新口径，本任务只删除两个卡片及其功能字段。
- 不突破 CitizenChain node 挖矿页网络总览模块边界。
- 改代码后同步更新文档、完善必要中文注释、清理残留。

输出物：
- 前端卡片布局调整。
- 后端 NetworkOverview 返回字段清理。
- 类型与测试同步更新。
- 模块技术文档更新。
- 残留引用清理。

验收标准：
- 挖矿页不再显示“总节点数”和“清算节点”。
- `NetworkOverview` 不再返回 `totalNodes/total_nodes` 和 `clearingNodes/clearing_nodes`。
- 治理节点、在线节点、全节点、轻节点仍正常显示。
- 相关测试或类型检查通过，若无法运行需说明原因。
- 文档已同步更新。

执行记录：
- 已删除前端“总节点数”“清算节点”卡片，网络区改为治理节点、在线节点、全节点、轻节点 2×2 展示。
- 已删除 `NetworkOverview` 的 `total_nodes` 与 `clearing_nodes` 后端返回字段，并清理对应前端类型。
- 已清理总节点数依赖的 `known-peers` 维护逻辑，以及清算节点卡片专用的链上计数函数。
- 已同步更新挖矿网络总览与挖矿看板技术文档。
- 已通过 Rust 单测与 TypeScript 类型检查。
