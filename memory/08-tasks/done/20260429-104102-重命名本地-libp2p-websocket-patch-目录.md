# 任务卡：重命名本地 libp2p-websocket patch 目录

- 任务编号：20260429-104102
- 状态：done
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-04-29 10:41:02

## 任务需求

将 citizenchain/node/libp2p-websocket-patch 重命名为 citizenchain/node/libp2p-websocket，并同步 Cargo patch 路径，保持覆盖 crates.io libp2p-websocket 的语义不变。

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
- 必要时联动 `citizenchain/runtime`

### 先沟通条件

- 修改节点启动方式
- 修改节点数据库或同步行为
- 修改安装包或发布形态


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`、`node` 或 `primitives`
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
- 已将 `citizenchain/node/libp2p-websocket-patch/` 重命名为 `citizenchain/node/libp2p-websocket/`。
- 已更新 `citizenchain/Cargo.toml` 的 `[patch.crates-io]` 路径，继续覆盖 crates.io 的 `libp2p-websocket`。
- 已保持本地 crate 包名 `libp2p-websocket` 不变，避免 Cargo patch 失效。
- 已同步更新 `memory/05-modules/citizenchain/node/NODE_TECHNICAL.md`，记录本地 `libp2p-websocket/` 覆盖目录与包名约束。
- 已搜索源码、正式文档与 CI 配置中的旧路径 `libp2p-websocket-patch`，未发现残留；旧路径仅保留在本任务卡需求描述与任务索引中。
- 已执行 `cargo check -p libp2p-websocket`，验证通过。
- 已尝试执行 `cargo check -p node`，Cargo 已解析到新路径 `/Users/rhett/GMB/citizenchain/node/libp2p-websocket`；后续被现有 `runtime/build.rs` 策略阻断，原因是未设置 `WASM_FILE`，该策略要求节点使用 CI 编译的统一 WASM。

## 完成信息

- 完成时间：2026-04-29 10:43:17
- 完成摘要：完成本地 libp2p-websocket 目录重命名，更新 Cargo patch 路径与节点模块文档，并完成旧路径残留检查和本地 patch crate 验证。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
