# 任务卡：使用 fuwuqi.sh 脚本以清链模式部署国储会节点到 147.224.14.117；部署成功后输出节点矿工地址，等待转账完成后再绑定手续费钱包地址。

- 任务编号：20260429-142147
- 状态：open
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-04-29 14:21:47

## 任务需求

使用 fuwuqi.sh 脚本以清链模式部署国储会节点到 147.224.14.117；部署成功后输出节点矿工地址，等待转账完成后再绑定手续费钱包地址。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-node.md

### 默认改动范围

- `citizenchain/node`
- `memory/05-modules/citizenchain/node`

### 先沟通条件

- 修改桌面前端与节点后端的交互边界
- 修改桌面打包、sidecar 或安装包行为

## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`（含桌面端）或 `primitives`
- 关键 Rust 或前端逻辑必须补中文注释
- 改动链规则、存储或发布行为前必须先沟通
- 如果改动 `runtime` 且会影响 `wuminapp` 在线端或 `wumin` 冷钱包二维码签名/验签兼容性，必须先暂停单边修改，转为跨模块任务
- 触发项至少检查：`spec_version` / `transaction_version`、pallet index、call index、metadata 编码依赖、冷钱包 `pallet_registry` 与 `payload_decoder`
- 未把 `wuminapp` 在线端和 `wumin` 冷钱包的对应更新纳入本次执行范围前，不允许继续 runtime 改动
- 文档与残留必须一起收口

## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/citizenchain.md

# CitizenChain 完成标准

- 改动范围和所属模块清晰
- 关键逻辑已补中文注释
- 文档已同步更新
- 影响链规则、存储或发布行为的点都已先沟通
- 残留已清理


## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 2026-04-29：已按清链模式启动部署流程，本地成功下载 Linux deb 安装包；远程首次初始化前，`ubuntu@147.224.14.117` 使用脚本内默认 SSH key 被服务器拒绝（publickey），尚未执行远程清链、安装或密钥写入。
- 2026-04-29：确认脚本默认 key 为 `/Users/rhett/.ssh/ed25519`，本机 SSH config 也配置 `oracle -> ubuntu@147.224.14.117` 使用同一 key；随后探测 `ubuntu/opc/root/admin/debian` 均在 22 端口连接超时，`curl telnet://147.224.14.117:22` 也超时。当前阻塞点变为服务器 SSH 端口不可达，仍未进入远程清链或安装。
