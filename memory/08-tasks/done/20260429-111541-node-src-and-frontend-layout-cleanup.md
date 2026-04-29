# 任务卡：继续收敛 citizenchain/node 前后端功能目录：mining 与 network 前后端合并到 mining；transaction 前后端移动到 home/transaction；gpu_miner.rs 移入 mining；src 根层节点核心文件收口到 core，桌面入口收口到 desktop，并同步修正引用、文档、注释与残留路径。

- 任务编号：20260429-111541
- 状态：done
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-04-29 11:15:41

## 任务需求

继续收敛 citizenchain/node 前后端功能目录：mining 与 network 前后端合并到 mining；transaction 前后端移动到 home/transaction；gpu_miner.rs 移入 mining；src 根层节点核心文件收口到 core，桌面入口收口到 desktop，并同步修正引用、文档、注释与残留路径。

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
- 后端目录已收敛：
  - `src/core/`：CLI、command、service、RPC、chain spec、benchmarking、TLS 证书。
  - `src/desktop/`：Tauri 桌面入口与进程内节点运行器。
  - `src/mining/`：收益看板、资源监控、网络概览、出块记录与 GPU 挖矿。
  - `src/home/transaction/`：冷钱包、本地钱包、转账签名与提交。
  - `src/shared/sfid_config.rs`：SFID 服务地址配置进入共享层。
- 前端目录已收敛：
  - `node/frontend/mining/dashboard/`：挖矿 tab 下的收益、资源、网络、出块记录。
  - `node/frontend/home/transaction/`：首页交易面板。
- 已修正 Rust 与 TypeScript 引用，避免继续依赖旧 `network`、`transaction`、`mining-dashboard` 和根层核心文件路径。
- 已同步模块文档路径与总览文件索引。
- 验证记录：
  - `cargo fmt -p node`：通过。
  - `cargo metadata --no-deps --format-version 1`：通过。
  - `npm run build`（`citizenchain/node/frontend`）：通过。
  - 目录残留断言：`src/ui`、`src/network`、`src/transaction`、`frontend/transaction`、`frontend/mining/mining-dashboard` 均已不存在。
  - `cargo check -p node`：被仓库既有 `runtime/build.rs` 门槛阻断，原因是当前环境未设置 `WASM_FILE`，该策略要求使用 CI 编译的统一 WASM。

## 完成信息

- 完成时间：2026-04-29 11:25:45
- 完成摘要：完成 citizenchain/node 前后端功能目录二次收敛：core、desktop、mining、home/transaction、shared 边界已同步，文档和残留路径已清理。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
