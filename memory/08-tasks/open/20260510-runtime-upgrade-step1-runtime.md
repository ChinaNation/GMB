# 20260510 runtime-upgrade 第1步 runtime 修复

## 任务目标

- 只修复 citizenchain runtime 里的 `runtime-upgrade` 模块。
- 协议升级提案继续保留联合提案边界。
- 开发直升 runtime 只允许国储会管理员执行。
- 更新测试、中文注释和模块文档。
- 清理本次修复产生的残留，权重数值由 CI 基准流程更新。

## 预计修改目录

- `citizenchain/runtime/governance/runtime-upgrade/src`
  - 修改 runtime-upgrade pallet 实现、测试和中文注释；不迁移目录、不新增业务目录。
- `citizenchain/runtime/src/configs`
  - 接入开发直升权限来源，保持协议升级提案的联合提案权限。
- `memory/05-modules/citizenchain/runtime/governance/runtime-upgrade`
  - 更新 runtime-upgrade 文档，删除过时描述并补齐当前权限说明。

## 执行记录

- 已在 `runtime-upgrade` pallet 中拆分 `ProposeOrigin` 与 `DeveloperUpgradeOrigin`。
- 已在 runtime 配置中接入 `DeveloperUpgradeOrigin = EnsureNrcAdmin`。
- 已保留协议升级提案权限为 `ProposeOrigin = EnsureJointProposer`。
- 已更新单测：国储会管理员可开发直升，省储会管理员和非国储会管理员均拒绝。
- 已更新模块技术文档，删除开发直升仍待 runtime 侧收窄的过时说明。
- 已清理“状态升级”旧注释，将 runtime-upgrade 包注释改为“运行时协议升级治理模块”。
- 权重数值未手工修改，按要求交由 CI 基准流程更新。
- 本轮继续修复协议升级边界：`propose_runtime_upgrade` 已删除人口快照、联合签名、省份和签名管理员公钥参数。
- 本轮已删除 `runtime-upgrade` 业务摘要中的本地状态字段，协议升级真实状态只读取 `votingengine::Proposals.status`。
- 本轮已把协议升级提案创建入口改为投票引擎待补的业务无感联合提案接口，等待下一步修复 `votingengine` 后统一编译。
- 本轮已修复投票引擎联合投票模块：人口快照准备、验签、防重放和消费全部收回 `joint-vote`，业务模块不再传 `eligible_total`、`snapshot_nonce`、`signature`、`province`、`signer_pubkey`。
- 本轮已删除 `JointVoteEngine` 旧签名，不保留兼容入口；业务模块只能提交业务摘要和可选对象。
- 本轮同步修复受接口收口影响的 `runtime-upgrade` 与 `resolution-issuance` 调用、测试和 benchmark。
- 本轮已更新 `runtime-upgrade`、`votingengine`、`resolution-issuance` 技术文档，并清理旧接口说明。
- 2026-05-10 追加修复：runtime 集成测试已删除旧 `runtime_upgrade::ProposalStatus` 和业务 `status` 字段残留。

## 验证记录

- 2026-05-10 `cargo check --manifest-path citizenchain/Cargo.toml -p joint-vote`：通过。
- 2026-05-10 `cargo check --manifest-path citizenchain/Cargo.toml -p runtime-upgrade`：通过。
- 2026-05-10 `cargo check --manifest-path citizenchain/Cargo.toml -p resolution-issuance`：通过。
- 2026-05-10 `cargo test --manifest-path citizenchain/Cargo.toml -p internal-vote --lib`：通过，89 passed。
- 2026-05-10 `cargo test --manifest-path citizenchain/Cargo.toml -p joint-vote --lib`：通过，5 passed。
- 2026-05-10 `cargo test --manifest-path citizenchain/Cargo.toml -p runtime-upgrade --lib`：通过，17 passed。
- 2026-05-10 `cargo test --manifest-path citizenchain/Cargo.toml -p resolution-issuance --lib`：通过，16 passed。
- 已执行格式整理与残留扫描。

### 此前验证记录

- `cargo fmt --manifest-path citizenchain/Cargo.toml -p citizenchain -p runtime-upgrade --check`：通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p runtime-upgrade --lib`：通过，17 passed。
- `cargo check --manifest-path citizenchain/Cargo.toml -p runtime-upgrade --features runtime-benchmarks`：通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain --features runtime-benchmarks`：本地被 runtime `build.rs` 拦截，原因是未设置 CI/本地启动脚本注入的 `WASM_FILE`，未手工伪造 wasm 文件。
