# 任务卡：治理模块改名 admins-change 与 grandpakey-change

- 任务编号：20260429-082049
- 状态：done
- 所属模块：citizenchain/governance
- 当前负责人：Codex
- 创建时间：2026-04-29 08:20:49

## 任务需求

在 `citizenchain/runtime/governance` 下保留模块位置，将管理员治理模块统一命名为 `admins-change`，将 GRANDPA 密钥治理模块统一命名为 `grandpakey-change`，并同步 runtime、依赖模块、客户端、冷钱包和文档命名口径。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md
- citizenchain/runtime/README.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-runtime.md

### 默认改动范围

- `citizenchain/runtime`
- `citizenchain/governance`
- `citizenchain/issuance`
- `citizenchain/otherpallet`
- `citizenchain/transaction`
- 必要时联动 `primitives`

### 先沟通条件

- 修改 runtime 存储结构
- 修改资格模型
- 修改提案、投票、发行核心规则


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
- 已完成治理模块目录与 memory 模块文档目录改名，两个模块仍位于 `governance` 目录下。
- 已同步 Cargo workspace、runtime 依赖、pallet 名称、benchmark 脚本、node UI 查询、wuminapp 在线端查询和 wumin 冷钱包注册表。
- 已确认当前 runtime metadata/storage prefix 会变化，pallet index 与 call index 保持不变；结合并行治理模块改名后的当前开发期 runtime 为 `spec_version = 6`，冷钱包同步适配 6。
- 已清理当前代码与当前模块文档中的旧标识残留；历史任务卡属于任务记录，不作为运行时代码入口。
- 已执行验证：
  - `cargo test -p admins-change --lib`
  - `cargo test -p grandpakey-change --lib`
  - `WASM_FILE=/private/tmp/dummy_wasm.wasm cargo test -p node governance`
  - `cargo test -p duoqian-manage --lib`
  - `cargo test -p duoqian-transfer --lib`
  - `dart format ...`
  - `flutter test`（wumin）
  - `flutter test`（wuminapp）
  - `cargo fmt --manifest-path citizenchain/Cargo.toml --package admins-change --package grandpakey-change --package node --package citizenchain --package duoqian-manage --package duoqian-transfer`

## 完成信息

- 完成时间：2026-04-29
- 完成摘要：完成管理员治理与 GRANDPA 密钥治理两个 governance 模块的彻底改名，目录、crate、runtime pallet、依赖方、客户端查询、冷钱包注册表和当前技术文档已同步到 `admins-change` / `grandpakey-change`。
