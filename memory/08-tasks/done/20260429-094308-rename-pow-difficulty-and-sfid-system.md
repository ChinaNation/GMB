# 任务卡：收口模块命名为 pow-difficulty 与 sfid-system

- 任务编号：20260429-094308
- 状态：done
- 所属模块：citizenchain/otherpallet
- 当前负责人：Codex
- 创建时间：2026-04-29 09:43:08

## 任务需求

将 PoW 难度模块统一命名为 `pow-difficulty`，将链上 SFID 系统模块统一命名为 `sfid-system`，并同步代码、脚本、前后端引用和文档。

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

- 开工前先确认任务属于 `runtime`、`node`、`nodeui` 或 `primitives`
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
- 已将两个 otherpallet 目录收口为 `pow-difficulty` 与 `sfid-system`。
- 已同步 Rust crate 依赖名、runtime pallet 名、genesis 配置段、benchmark 脚本、SFID 后端动态链调用、wuminapp storage key、website 展示名与 `memory/` 技术文档路径。
- 已将 `memory/scripts/load-context.sh` 和 `memory/08-tasks/index.md` 更新到新模块名。
- 已清理旧名残留：旧 PoW 难度模块名与旧 SFID 链上模块名在工作区文件内容与文件名扫描中均无命中。
- 已执行 `rustfmt --edition 2021` 与 `dart format`，确保本次触达的 Rust/Dart 文件格式化。

## 验证记录

- `cargo metadata --offline --no-deps --manifest-path citizenchain/Cargo.toml` 通过，workspace 成员已解析到 `runtime/otherpallet/pow-difficulty` 与 `runtime/otherpallet/sfid-system`。
- `cargo test --offline --manifest-path citizenchain/runtime/otherpallet/pow-difficulty/Cargo.toml -- --nocapture` 通过，9 个测试全部成功。
- `cargo test --offline --manifest-path citizenchain/runtime/otherpallet/sfid-system/Cargo.toml -- --nocapture` 通过，26 个测试全部成功。
- `cargo check --offline --manifest-path citizenchain/runtime/issuance/citizen-issuance/Cargo.toml` 通过。
- `cargo check --offline --manifest-path citizenchain/runtime/transaction/duoqian-manage/Cargo.toml` 通过。
- `cargo check --offline --manifest-path sfid/backend/Cargo.toml` 通过，仅保留既有未使用代码警告。
- `flutter analyze` 通过，`wuminapp` 无分析问题。
- `npm run build` 通过，`website` 可构建。
- `cargo check --offline --manifest-path citizenchain/Cargo.toml -p pow-difficulty -p sfid-system -p citizen-issuance -p duoqian-manage -p citizenchain -p node` 触发 runtime build script 的统一 WASM 策略阻塞：`WASM_FILE 环境变量未设置`。该失败来自仓库构建策略，不是本次改名引用残留。

## 完成信息

- 完成时间：2026-04-29 10:02:39
- 完成摘要：完成旧 PoW 难度模块名到 `pow-difficulty`、旧 SFID 链上模块名到 `sfid-system` 的全仓命名收口；同步代码、脚本、前后端与 memory 文档，并清理旧名残留。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
