# 任务卡：清算行功能完成度补齐：区块链侧技术闭环，包括 runtime、node、NodeUI、验收与文档

- 任务编号：20260428-124931
- 状态：open
- 所属模块：citizenchain
- 当前负责人：Codex
- 创建时间：2026-04-28 12:49:31

## 任务需求

清算行功能完成度补齐：区块链侧技术闭环，包括 runtime、node、NodeUI、验收与文档

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/04-decisions/ADR-007-clearing-bank-three-phase.md
- memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md
- memory/05-modules/citizenchain/runtime/transaction/offchain-transaction-pos/STEP1_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/transaction/offchain-transaction-pos/STEP2A_RUNTIME.md
- memory/05-modules/citizenchain/node/offchain/STEP2B_I_NODE.md
- memory/05-modules/citizenchain/node/offchain/STEP2B_II_A_PACKER.md
- memory/05-modules/citizenchain/node/offchain/STEP2B_II_B_1_SIGNER.md
- memory/05-modules/citizenchain/node/offchain/STEP2B_II_B_2_B_INTEGRATION.md
- memory/05-modules/citizenchain/node/offchain/STEP2B_III_A_EVENT_LISTENER.md
- memory/05-modules/citizenchain/node/offchain/STEP2B_III_B_RESERVE_MONITOR.md
- memory/05-modules/citizenchain/node/offchain/STEP2_E_CROSS_BANK_GHOST_FIX.md

## 本卡目标

- 把区块链侧完成度从约 82% 补到可验收状态。
- 范围包括 `citizenchain/runtime/transaction/offchain-transaction-pos`、`citizenchain/runtime/src/configs/mod.rs`、`citizenchain/node/src/offchain`、`citizenchain/node/frontend/offchain` 与相关文档。
- 不直接修改 `sfid`、`wuminapp`、`wumin`；跨端契约变化必须先同步对应任务卡。

## 待补齐清单

- `submit_offchain_batch_v2` 严格校验 `batch_seq` 与 `batch_signature`，不再只作为审计冗余。
- `offchain_submitPayment` 返回真实清算行 ACK 签名，去掉 `[0u8; 64]` 占位。
- 节点侧 `accept_payment` 与 runtime 收款方主导、跨行规则保持一致，避免移动端绕过 UI 后产生延迟失败。
- 补齐清算行 pallet 的权重 / benchmark，替换当前占位权重。
- 整理 NodeUI 清算行 tab 的剩余禁用项，明确哪些归本卡、哪些归 SFID / wumin / wuminapp。
- 清理或标记旧省储行 L2 清算文档残留，避免新旧模型混淆。
- 补一轮 dev 链区块链侧验收记录：注册清算行、更新端点、注销、绑定、充值、同行批次、跨行批次、对账。

## 验收标准

- `cargo test --manifest-path citizenchain/Cargo.toml -p offchain-transaction-pos --lib` 通过。
- 设置 `WASM_FILE` 后，`cargo test --manifest-path citizenchain/Cargo.toml -p node clearing_bank` 通过。
- `citizenchain/node/frontend` 执行 `npm run build` 通过。
- dev 链手工或脚本验收至少覆盖：清算行节点声明、wuminapp 提交后的节点 pending、packer 上链、runtime settlement、event_listener 回写、reserve_monitor 无差异。
- 文档更新并清理旧省储行清算残留。

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
- 2026-04-28 Codex 执行：
  - runtime `submit_offchain_batch_v2` 已启用 batch 级 sr25519 签名校验。
  - runtime 新增 `LastClearingBatchSeq[bank]`，要求 `batch_seq == last + 1`，仅 settlement 成功后推进。
  - runtime settlement 新增 `UserBank[payer]` / `UserBank[recipient]` 与 item 内 `payer_bank` / `recipient_bank` 一致性校验。
  - node `OffchainPacker` 启动时读取链上 `LastClearingBatchSeq`，重启后从链上序号续跑。
  - node `offchain_submitPayment` 去掉 `[0u8;64]` ACK，占用 `KeystoreBatchSigner` 生成真实 L2 ACK；入 pending 前早拒错路由、绑定漂移、未配置费率和手续费不一致。
  - runtime `spec_version` 从 3 bump 到 4；call 编码未变，`transaction_version` 保持 2。
  - 已标记旧省储行 L2 清算协议文档为 legacy，避免新旧模型混淆。
  - 已补中文注释与技术文档：ADR-007、Step2A runtime、Step2B node/RPC/packer 文档。

## 本轮验证

- 通过：`cargo test --manifest-path citizenchain/Cargo.toml -p offchain-transaction-pos --lib`（23 passed）
- 通过：`WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test --manifest-path citizenchain/Cargo.toml -p node clearing_bank`（18 passed）
- 通过：`WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test --manifest-path citizenchain/Cargo.toml -p node offchain`（43 passed）
- 通过：`npm run build`（`citizenchain/node/frontend`）

## 本轮剩余项

- 权重 / benchmark 仍是占位，未生成正式 benchmark。
- NodeUI 清算行前后端目录已在 2026-04-29 收口到 `node/src/offchain` 与 `node/frontend/offchain`；`dist/*` 属构建产物,不作为业务真源。
- dev 链全流程手工验收尚未执行：注册清算行、wuminapp 提交、packer 上链、event_listener 回写、reserve_monitor 对账。
