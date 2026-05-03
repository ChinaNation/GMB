# 任务卡：修复 voting-engine benchmark internal_vote 权重和 duoqian 治理边界

- 任务编号：20260502-193144
- 状态：done
- 所属模块：citizenchain/runtime/governance
- 当前负责人：Codex
- 创建时间：2026-05-02 19:31:44

## 任务需求

修复 voting-engine runtime-benchmarks 编译失败、internal_vote 免费且权重占位风险、admins-change 对 ORG_DUOQIAN 管理员替换入口泄漏；完成后更新文档、完善中文注释并清理残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

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
- 已修复 `voting-engine/src/benchmarks.rs`：`citizen_vote` benchmark 补齐 `province` 与 `signer_admin_pubkey`；`joint_vote` benchmark setup 改走真实 `do_create_joint_proposal` 路径。
- 已修复全 runtime benchmark 聚合编译阻塞：`resolution-issuance` 与 `runtime-upgrade` 的 propose benchmark 同步补齐 ADR-008 step3 参数。
- 已将 runtime 金额提取策略中的 `VotingEngine::internal_vote` 从 `NoAmount` 改为 `Amount(100_000)`，即管理员提交内部投票 extrinsic 固定 1 元/次；自动 executor 回调不单独产生另一笔用户交易。
- 已将 `voting-engine/src/weights.rs` 中 `internal_vote` 的旧占位权重替换为偏高保守 callback fallback，并去掉“与 joint_vote 同量级”的旧说明。
- 已在 `admins-change::propose_admin_replacement` 加入 org 白名单，仅允许 `ORG_NRC / ORG_PRC / ORG_PRB`，阻断 `ORG_DUOQIAN` 从通用管理员替换入口绕出第二条治理路径。
- 已补充 `duoqian_subjects_cannot_use_admin_replacement_entry` 测试，并补齐 admins-change 测试 mock 的 ADR-008 trait 签名。
- 已更新 `VOTINGENGINE_TECHNICAL.md`、`ADMINSCHANGE_TECHNICAL.md`、`ONCHAIN_TECHNICAL.md`、`RESOLUTIONISSUANCE_TECHNICAL.md`、`RUNTIMEUPGRADE_TECHNICAL.md`。
- 已清理 `cargo fmt --all` 造成的无关格式化残留，仅保留本任务相关文件。

## 验证记录

- 通过：`cargo check -p voting-engine --features runtime-benchmarks`
- 通过：`cargo test -p voting-engine --lib --features runtime-benchmarks`
- 通过：`cargo test -p admins-change --lib --features runtime-benchmarks`
- 通过：`WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo test -p citizenchain --lib onchain_tx_amount_extractor_covers_noamount_amount_and_unknown_paths`
- 通过：`cargo check -p resolution-issuance --features runtime-benchmarks`
- 通过：`cargo check -p runtime-upgrade --features runtime-benchmarks`
- 通过：`WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo build --release --features runtime-benchmarks`
- 未生成正式 benchmark 权重：`./target/release/citizenchain benchmark pallet --chain=citizenchain --pallet=voting_engine --extrinsic='*' --steps=10 --repeat=3 --output=/tmp/voting_engine.weights.rs` 失败，原因是当前链 spec/WASM 不含 Benchmark Runtime API。已用保守 fallback 覆盖旧占位，并在文档记录正式重生条件。

## 完成信息

- 完成时间：2026-05-02 19:42:28
- 完成摘要：完成 voting-engine benchmark 签名修复、internal_vote 收费与保守权重兜底、admins-change ORG_DUOQIAN 边界收紧；补齐关联 benchmark 参数、文档和验证记录。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
