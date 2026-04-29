# 任务卡：结合全仓库实现复查 pow-difficulty 与 sfid-system 模块，检查是否存在安全漏洞、改进点、实现与文档偏差、中文注释缺失、文档更新需求和需要清理的残留

- 任务编号：20260405-124627
- 状态：open
- 所属模块：citizenchain/runtime/otherpallet/pow-difficulty、citizenchain/runtime/otherpallet/sfid-system
- 当前负责人：Codex
- 创建时间：2026-04-05 12:46:27

## 任务需求

结合全仓库实现复查 pow-difficulty 与 sfid-system 模块，检查是否存在安全漏洞、改进点、实现与文档偏差、中文注释缺失、文档更新需求和需要清理的残留

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
- citizenchain/runtime/otherpallet/pow-difficulty/src/lib.rs
- citizenchain/runtime/otherpallet/sfid-system/src/lib.rs
- citizenchain/runtime/src/configs/mod.rs
- memory/05-modules/citizenchain/runtime/otherpallet/pow-difficulty/POW_DIFFICULTY_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/otherpallet/sfid-system/SFID_SYSTEM_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-runtime.md

### 默认改动范围

- `citizenchain/runtime`
- `citizenchain/node`
- 必要时联动 `memory/05-modules/wuminapp`

### 先沟通条件

- 修改 runtime 存储结构
- 修改难度调节核心规则
- 修改 SFID 验签消息、权限边界或兼容契约

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
- 已复查 `pow-difficulty`、`sfid-system`、runtime verifier / runtime api 接线、node PoW 消费链路、`wuminapp` SFID 主账户读取口径，以及对应技术文档/旧任务卡。
- `pow-difficulty` 旧审查中的高优先级问题仍然存在：`on_finalize` 继续对空块执行 panic 型 `assert!`，链规则仍与节点层“空交易池不挖矿”策略耦合。
- `sfid-system` 旧审查里的测试覆盖缺口已基本补齐：本体 17 条单测与 `citizen-issuance` 的 7 条 `integration_bind_sfid` 都通过。
- 已清理可直接落地的残留：
  - 为 `pow-difficulty` 补上 `memory/scripts/load-context.sh` 上下文装载入口；
  - 更新 `NODE_TECHNICAL.md`，移除已过期的“PoW 作者身份无密码学绑定”旧口径，改成当前真实风险“空块策略仍与 runtime panic 耦合”；
  - 更新 `sfid-system` 技术文档，修正权重口径和中文乱码；
  - 在两个模块的 `weights.rs` 顶部补充“当前 benchmark 可信度/漂移状态”说明。
- 已尝试通过 benchmark-runtime 重生权重文件，但当前本地 `WASM_BUILD_FROM_SOURCE=1` 构建会在 `wasm32v1-none` 依赖上失败，正式 `weights.rs` 本轮无法直接重生成。

## 当前结论

- `pow-difficulty`：存在 1 个仍未关闭的高优先级风险，空块会在 runtime `on_finalize` 中触发 panic；这不是文档问题，而是当前链规则本身。
- `sfid-system`：未发现新的高危逻辑漏洞；当前主要残留是 `weights.rs` 仍引用旧存储模型，技术文档与测试口径本轮已基本对齐。
- 两个模块都未发现新的节点/UI/轻客户端契约错位；`wuminapp` 读取 `SfidMainAccount` 的格式与现状一致。

## 已执行验证

- `cargo test --offline --manifest-path /Users/rhett/GMB/citizenchain/runtime/otherpallet/pow-difficulty/Cargo.toml -- --nocapture`
- `cargo test --offline --manifest-path /Users/rhett/GMB/citizenchain/runtime/otherpallet/sfid-system/Cargo.toml -- --nocapture`
- `cargo check --offline --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml -p pow-difficulty --features runtime-benchmarks`
- `cargo check --offline --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml -p sfid-system --features runtime-benchmarks`
- `cargo test --offline --manifest-path /Users/rhett/GMB/citizenchain/runtime/issuance/citizen-issuance/Cargo.toml --test integration_bind_sfid -- --nocapture`
- `WASM_BUILD_FROM_SOURCE=1 cargo build --offline --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml -p node --features runtime-benchmarks`

## 验证阻塞

- 使用 `WASM_BUILD_FROM_SOURCE=1` 构建 benchmark-runtime 节点时，`citizenchain/runtime` 的 wasm 构建在 `byte-slice-cast` 依赖上失败（`wasm32v1-none` 下找不到 `std`），因此本轮无法重生成 `pow-difficulty` / `sfid-system` 的正式 `weights.rs`。
