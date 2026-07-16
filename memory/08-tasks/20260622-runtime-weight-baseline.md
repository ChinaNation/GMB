# runtime 权重基线盘点与 benchmark 生成

## 状态

完成：第一阶段 11 个 runtime pallet benchmark 已全部真实运行通过，并已用正式 benchmark 输出覆盖对应 `weights.rs`。

## 任务需求

- 盘点 `citizenchain/runtime` 中所有 pallet 的权重实现、benchmark 覆盖和 runtime 接线情况。
- 区分已启用真实业务逻辑、当前禁用但可提前校准、仍为空壳或 stub 的模块。
- 后续在获得 runtime 二次确认后，按盘点结果运行 benchmark 并写入正式权重。

## 当前阶段

第一阶段盘点、前置修复、正式 benchmark 生成和编译验收已完成。

- 已生成 `citizenchain/node/frontend/dist/`，满足 Tauri node 编译前置条件。
- 已修正 `citizenchain/scripts/benchmark.sh` 的第一阶段 pallet 清单、二进制名称、前端 dist 检查和本地 runtime WASM 参数。
- 已新增 `citizenchain/scripts/benchmark-weight-template.hbs`，让 benchmark CLI 生成本仓库本地 `WeightInfo` trait / `SubstrateWeight<T>` / `impl WeightInfo for ()` 结构，避免默认模板生成不可编译的外部 crate impl。
- 已修复 `citizenchain/runtime/private/organization-manage/src/benchmarks.rs` 在 `runtime-benchmarks` feature 下的编译问题。
- 已修复 `WASM_BUILD_FROM_SOURCE=1` 构建 benchmark runtime WASM 时的 `institution-asset/std` feature 泄漏。
- 已修复 `cid-system` benchmark 源码：`wasm32v1-none` 下 `vec!` 宏未导入。
- 已修复 `organization-manage` benchmark 源码：`wasm32v1-none` 下 `Vec` 类型未导入。
- 已确认 `WASM_BUILD_FROM_SOURCE=1 cargo build --release --features runtime-benchmarks --bin citizenchain` 曾通过。
- 已获得第一阶段 11 个 `weights.rs` 写入确认并执行 `./scripts/benchmark.sh`。
- `./scripts/benchmark.sh` 第一次执行时，因 benchmark CLI 默认 `dev` chain spec 未被节点 `load_spec` 支持，11 个 pallet 均未进入实际 benchmark。
- 已修复 `citizenchain/node/src/core/command.rs`，让 `dev/local/staging` 内置别名落到 CitizenChain 冻结 chainspec。
- `organization-manage` 注销机构功能修复结束后，再次执行 `./scripts/benchmark.sh`，已进入真实 benchmark 采样。
- 已成功生成 8 个 `weights.rs`：`shengbank_interest`、`fullnode_issuance`、`citizen_issuance`、`cid_system`、`pow_difficulty`、`admins_change`、`resolution_destro`、`grandpakey_change`。
- 已修复剩余 3 个 benchmark fixture：
  - `duoqian_transfer`：benchmark 转账金额从 `100` 改为全链统一 ED `111`。
  - `resolution_issuance`：当时为联合投票 benchmark 准备预备快照夹具；该中转存储现已删除，benchmark 改为提案内联快照。
  - `runtime_upgrade`：当时同样使用预备快照夹具覆盖 5MB runtime object；现已改为提案内联快照。
- 已完整重跑第一阶段 11 个 pallet：`shengbank_interest`、`fullnode_issuance`、`citizen_issuance`、`resolution_issuance`、`cid_system`、`pow_difficulty`、`admins_change`、`resolution_destro`、`grandpakey_change`、`duoqian_transfer`、`runtime_upgrade` 全部通过并写入 `weights.rs`。

## 盘点结论

### 可直接重跑并写入正式权重的候选模块

这些模块已挂入 `runtime/src/benchmarks.rs`，且 benchmark 覆盖当前 `WeightInfo` 方法，适合作为第一批正式权重生成对象：

- `shengbank_interest` → `citizenchain/runtime/issuance/shengbank-interest/src/weights.rs`
- `fullnode_issuance` → `citizenchain/runtime/issuance/fullnode-issuance/src/weights.rs`
- `citizen_issuance` → `citizenchain/runtime/issuance/citizen-issuance/src/weights.rs`
- `resolution_issuance` → `citizenchain/runtime/issuance/resolution-issuance/src/weights.rs`
- `cid_system` → `citizenchain/runtime/otherpallet/cid-system/src/weights.rs`
- `pow_difficulty` → `citizenchain/runtime/otherpallet/pow-difficulty/src/weights.rs`
- `admins_change` → `citizenchain/runtime/admins/admin-management/src/weights.rs`
- `resolution_destro` → `citizenchain/runtime/governance/resolution-destro/src/weights.rs`
- `grandpakey_change` → `citizenchain/runtime/governance/grandpakey-change/src/weights.rs`
- `duoqian_transfer` → `citizenchain/runtime/transaction/duoqian-transfer/src/weights.rs`
- `runtime_upgrade` → `citizenchain/runtime/governance/runtime-upgrade/src/weights.rs`

### 不能直接重跑覆盖的模块

- `organization_manage`：该记录形成时只实现 `register_cid_institution`，权重文件还包含创建、关闭及业务模块手工清理入口；现行实现已删除手工清理入口并由投票引擎统一清理，重新生成前须按当前 ABI 复核 benchmark 覆盖。
- `personal_manage`：`benchmarks.rs` 是空骨架，且未挂入 `runtime/src/benchmarks.rs`。
- `offchain_transaction`：`benchmarks.rs` 明确无可执行 benchmark，且未挂入 `runtime/src/benchmarks.rs`。
- `votingengine` / `internal_vote` / `joint_vote`：benchmark 文件为空或仅占位，且当前未挂入 `runtime/src/benchmarks.rs`。
- `onchain_issuance`：业务仍是 stub，runtime 仍接 `ZeroWeight`；在业务实装前不得生成“正式权重”。
- `genesis_pallet`：无 extrinsic，`WeightInfo` 为空实现。

### benchmark 脚本问题

- `citizenchain/scripts/benchmark.sh` 仍引用已删除的 `duoqian_manage`。
- `citizenchain/scripts/benchmark.sh` 仍引用旧 `runtime/governance/voting-engine/src/weights.rs` 路径。
- `citizenchain/scripts/benchmark.sh` 未覆盖当前 runtime benchmark 注册表中的全部可生成模块。

### 当前已修正的问题

- `citizenchain/scripts/benchmark.sh` 已移除旧 `duoqian_manage`、旧 `voting_engine` 路径，只保留第一阶段可安全生成的 11 个 pallet。
- `citizenchain/scripts/benchmark.sh` 已改为编译并调用 `target/release/citizenchain`。
- `citizenchain/scripts/benchmark.sh` 已在缺少 `node/frontend/dist` 时自动运行 `npm --prefix node/frontend run build`。
- `citizenchain/scripts/benchmark.sh` 已开启 `WASM_BUILD_FROM_SOURCE=1`，并向 `benchmark pallet` 传入 `--runtime=<本地生成 wasm>` 与 `--genesis-builder=runtime`。
- `citizenchain/runtime/private/organization-manage/src/benchmarks.rs` 已更新 `register_cid_institution` benchmark 调用参数，匹配当前 11 参数接口。
- `citizenchain/runtime/private/organization-manage/src/benchmarks.rs` 已清理不再使用的 benchmark helper 和 import，`cargo check -p organization-manage --features runtime-benchmarks` 通过。

### 已解除的阻塞问题

不开启 `WASM_BUILD_FROM_SOURCE=1` 时，`cargo build --release --features runtime-benchmarks --bin citizenchain` 可以生成节点二进制，但 runtime 没有嵌入 benchmark runtime API，11 个 pallet benchmark 都失败：

```text
Invalid input: Did not find the benchmarking runtime api.
```

按 CLI 提示改为传入本地 runtime WASM 后，构建 `citizenchain/runtime` 的 benchmark WASM 失败：

```text
error[E0463]: can't find crate for `std`
std is required by byte_slice_cast because it does not declare #![no_std]
```

已确认一条实际 feature 链路：`citizenchain/runtime/Cargo.toml` 的 `runtime-benchmarks` 直接启用 `institution-asset/std`，进而触发 `scale-info/std`、`parity-scale-codec/std` 和 `byte-slice-cast/std`。这与既有文档 `memory/05-modules/citizenchain/runtime/issuance/resolution-issuance/RESOLUTIONISSUANCE_TECHNICAL.md` 中记录的 `WASM_BUILD_FROM_SOURCE=1` 阻塞一致。

已修复该链路：

- `citizenchain/runtime/Cargo.toml`：`runtime-benchmarks` 改为启用 `institution-asset/runtime-benchmarks`。
- `citizenchain/runtime/Cargo.toml`：`try-runtime` 改为启用 `institution-asset/try-runtime`。
- `citizenchain/runtime/transaction/institution-asset/Cargo.toml`：新增空的 `runtime-benchmarks` / `try-runtime` feature，避免 benchmark / try-runtime 间接打开 `std`。

复跑 `WASM_BUILD_FROM_SOURCE=1 cargo build --release --features runtime-benchmarks --bin citizenchain` 后，原 `byte-slice-cast` 的 `std` 报错消失，新的阻塞为：

```text
error: cannot find macro `vec` in this scope
--> runtime/otherpallet/cid-system/src/benchmarks.rs:33:41
```

已修复 `cid-system` 的导入路径为 `sp_runtime::sp_std::vec`。再次复跑后，构建推进到：

```text
error[E0425]: cannot find type `Vec` in this scope
--> runtime/private/organization-manage/src/benchmarks.rs:60:27
```

已修复 `organization-manage` 的 `Vec` 导入为 `sp_std::{vec, vec::Vec}`。再次复跑后：

```text
Finished `release` profile [optimized] target(s) in 56.02s
```

生成的 benchmark runtime WASM：

```text
citizenchain/target/release/wbuild/citizenchain/citizenchain.compact.compressed.wasm
```

执行 `./scripts/benchmark.sh` 后发现 `--chain` 与 `--runtime` 互斥；根因是 Substrate benchmark CLI 默认 `dev`，而节点 `load_spec` 未实现 `dev/local/staging` 别名。已将这些别名接到冻结 chainspec。随后 clean wbuild 暴露 `organization-manage` 注销凭证改造残留：

```text
missing field `scope` in initializer of `CloseInstitutionAction<_>`
failed to resolve: use of unresolved module or unlinked crate `alloc`
this function takes 3 arguments but 8 arguments were supplied
```

只读定位显示：`CloseInstitutionAction` 已新增 `scope`，`propose_close` 已新增注销凭证参数，`CidInstitutionVerifier::verify_institution_deregistration` / `UsedDeregisterNonce` / `SCOPE_INSTITUTION` / `SCOPE_ACCOUNT` 已存在，但 `close.rs::do_propose_institution_close` 仍是旧 3 参数实现。

`organization-manage` 修复后继续运行，`--genesis-builder=runtime` 因 runtime 没有 `development` preset 失败。低步数验证确认 `--runtime=<本地 WASM>` + `--genesis-builder=spec-genesis` 可进入真实 benchmark；脚本已切到该组合。该组合会复用冻结 chainspec genesis，CLI 仍提示该模式未来可能被弃用，后续若要消除警告，应给 runtime 补专用 benchmark genesis preset。

第一阶段首次真实运行结果：

- 成功：`shengbank_interest`
- 成功：`fullnode_issuance`
- 成功：`citizen_issuance`
- 失败：`resolution_issuance::propose_resolution_issuance`，错误 `JointVoteCreateFailed`
- 成功：`cid_system`
- 成功：`pow_difficulty`
- 成功：`admins_change`
- 成功：`resolution_destro`
- 成功：`grandpakey_change`
- 失败：`duoqian_transfer::propose_transfer`，错误 `AmountBelowExistentialDeposit`
- 失败：`runtime_upgrade::propose_runtime_upgrade`，错误 `JointVoteCreateFailed`

只读定位结论：

- `resolution_issuance` / `runtime_upgrade` 的 benchmark 当时受联合投票预备快照中转约束；该约束现已删除，创建接口直接内联快照。
- `duoqian_transfer` 的 benchmark 把转账金额写死为 `100`，而 runtime `ExistentialDeposit` 为 `111`，因此触发 `AmountBelowExistentialDeposit`。

最终修复结论：

- `citizenchain/scripts/benchmark.sh` 改为显式传入 `--template="$CHAIN_ROOT/scripts/benchmark-weight-template.hbs"`，避免 Substrate 默认模板覆盖出不可编译的 `pub struct WeightInfo<T>` 外部 impl。
- `citizenchain/.gitignore` 改为继续忽略 `citizenchain/scripts/*` 下本地脚本，但单独放行 `benchmark-weight-template.hbs`。
- `resolution-issuance` / `runtime-upgrade` benchmark fixture 曾写入预备快照中转存储并对齐测量块；现已删除该中转与时效错误分支，benchmark 直接测量提案内联快照。
- `resolution_issuance` 与 `runtime_upgrade` 的测量点改为 `#[block]` 直接调用 pallet extrinsic 函数，覆盖完整 origin 校验、业务校验、联合提案创建、数据/对象写入路径。

## 预计后续修改目录

- `citizenchain/scripts/benchmark.sh`：已修正第一阶段 benchmark 入口，使用本地 runtime WASM、`spec-genesis` 和仓库自定义权重模板。
- `citizenchain/scripts/benchmark-weight-template.hbs`：新增 benchmark 权重模板，输出本仓库需要的本地 trait 权重结构。
- `.gitignore`：单独放行 `citizenchain/scripts/benchmark-weight-template.hbs`，避免模板被 `citizenchain/scripts/*` 忽略。
- `citizenchain/runtime/Cargo.toml`：已移除 `runtime-benchmarks` 和 `try-runtime` 中对 `institution-asset/std` 的 WASM 污染接线。
- `citizenchain/runtime/transaction/institution-asset/Cargo.toml`：已补空的 `runtime-benchmarks` / `try-runtime` feature。
- `citizenchain/runtime/otherpallet/cid-system/src/benchmarks.rs`：已导入 no_std 可用的 `vec!` 宏，继续推进 benchmark runtime WASM 构建。
- `citizenchain/runtime/private/organization-manage/src/benchmarks.rs`：已导入 no_std 可用的 `Vec` 类型，benchmark runtime WASM 构建通过。
- `citizenchain/node/src/core/command.rs`：已补齐 `dev/local/staging` chain spec 别名，避免 benchmark CLI 默认 `dev` 被误当文件路径。
- `citizenchain/runtime/private/organization-manage/src/lib.rs`：后续需修正注销关闭 extrinsic 参数类型和 no_std Vec 路径。
- `citizenchain/runtime/private/organization-manage/src/close.rs`：后续需同步注销凭证校验、nonce 防重放、scope 写入 `CloseInstitutionAction`。
- `citizenchain/runtime/issuance/resolution-issuance/src/benchmarks.rs`：已改为由联合提案创建路径内联生成快照，不写预备中转存储。
- `citizenchain/runtime/governance/runtime-upgrade/src/benchmarks.rs`：已改为由联合提案创建路径内联生成快照，不写预备中转存储。
- `citizenchain/runtime/transaction/duoqian-transfer/src/benchmarks.rs`：已把 benchmark 转账金额改为全链统一 ED `111`。
- `citizenchain/runtime/**/weights.rs`：已由真实 benchmark 跑通后写入第一阶段 11 个正式权重。
- `citizenchain/runtime/**/benchmarks.rs`：后续补齐不能代表真实路径的 benchmark，尤其是 `organization_manage` 剩余 3 个权重方法。
- `citizenchain/runtime/src/configs/mod.rs`：后续仅在权重接线需要替换占位实现时修改。
- `memory/05-modules/citizenchain/runtime/`：后续记录权重基线范围、例外模块和验收命令。

## 风险边界

- `citizenchain/runtime/` 的任何实际修改都必须先列出完整路径、预计改动和原因，并获得用户第二次确认。
- 空壳/stub 模块不得写入“看似正式”的 benchmark 权重。
- 被 `RuntimeCallFilter` 禁用的模块需要单独标记，不能把“已生成权重”等同于“可启用”。

## 验收记录

- `rg -n "type WeightInfo\s*=|ZeroWeight|SubstrateWeight<Runtime>|weights::" citizenchain/runtime/src citizenchain/runtime -g '*.rs'`：完成权重接线扫描。
- `find citizenchain/runtime -path '*/benchmarks.rs' -o -path '*/benchmarking.rs'`：确认 benchmark 文件分布。
- `find citizenchain/runtime -path '*/weights.rs'`：确认 runtime 内权重文件分布。
- `sed -n '1,260p' citizenchain/scripts/benchmark.sh`：确认脚本存在旧 pallet / 旧路径残留。
- `sed -n '1,260p' citizenchain/runtime/src/benchmarks.rs`：确认当前 benchmark 注册表。
- `npm --prefix citizenchain/node/frontend run build`：通过，已生成 Tauri 前端 dist。
- `cargo check -p organization-manage --features runtime-benchmarks`：通过。
- `bash -n citizenchain/scripts/benchmark.sh`：通过。
- `./scripts/benchmark.sh`：节点 release 编译通过后，因未嵌入 Benchmark Runtime API 导致 11 个 pallet benchmark 全部失败。
- `WASM_BUILD_FROM_SOURCE=1 ./scripts/benchmark.sh`：在 runtime WASM 构建阶段失败，错误为 `byte-slice-cast` 需要 `std`，不适用于 `wasm32v1-none`。
- `cargo tree --manifest-path citizenchain/target/release/wbuild/citizenchain/Cargo.toml --target wasm32v1-none -e features -i parity-scale-codec`：确认 `institution-asset/std` 会触发 `parity-scale-codec/std`。
- `cargo tree --manifest-path citizenchain/target/release/wbuild/citizenchain/Cargo.toml --target wasm32v1-none -e features -i institution-asset`：确认修复后 `institution-asset` 只启用 `runtime-benchmarks`，不再启用 `std`。
- `WASM_BUILD_FROM_SOURCE=1 cargo build --release --features runtime-benchmarks --bin citizenchain`：原 `byte-slice-cast` 报错消失，构建推进到 `cid-system` benchmark 的 `vec!` 宏导入错误。
- `WASM_BUILD_FROM_SOURCE=1 cargo build --release --features runtime-benchmarks --bin citizenchain`：修复 `cid-system` 后再次复跑，构建推进到 `organization-manage` benchmark 的 `Vec` 类型导入错误。
- `WASM_BUILD_FROM_SOURCE=1 cargo build --release --features runtime-benchmarks --bin citizenchain`：修复 `organization-manage` 后再次复跑，通过。
- `find citizenchain/target/release/wbuild -type f -name '*.compact.compressed.wasm' ! -path '*/frame-storage-access-test-runtime/*'`：确认已生成本地 benchmark runtime WASM。
- `bash -n citizenchain/scripts/benchmark.sh`：通过。
- `git diff --check -- citizenchain/runtime/Cargo.toml citizenchain/runtime/transaction/institution-asset/Cargo.toml citizenchain/runtime/otherpallet/cid-system/src/benchmarks.rs citizenchain/runtime/private/organization-manage/src/benchmarks.rs citizenchain/scripts/benchmark.sh memory/08-tasks/20260622-runtime-weight-baseline.md`：通过。
- `./scripts/benchmark.sh`：第一次全量运行未进入实际采样，因 `dev` chain spec 未识别失败。
- `./scripts/benchmark.sh pow_difficulty`：修复 chain spec 别名后重新运行，clean wbuild 暴露 `organization-manage` 注销凭证改造残留，未进入实际采样。
- `./target/release/citizenchain benchmark pallet --runtime=<本地 WASM> --genesis-builder=spec-genesis --pallet=pow_difficulty --steps=2 --repeat=1 --output=/tmp/citizenchain-pow-difficulty-test.rs`：通过，确认 CLI 参数组合可进入真实 benchmark。
- `./scripts/benchmark.sh`：真实运行后 8 个 pallet 成功写入 `weights.rs`，3 个 fixture 失败待修。
- `cargo check -p resolution-issuance -p runtime-upgrade --features runtime-benchmarks`：通过，确认 `joint-vote` benchmark fixture 依赖和 where 约束可编译。
- `./scripts/benchmark.sh resolution_issuance`：历史基线通过；当前实现已删除预备快照时效字段，联合提案 benchmark 直接覆盖内联快照。
- `./scripts/benchmark.sh runtime_upgrade`：通过，确认 runtime upgrade 联合提案和 `ProposalObject` 5MB 路径可生成权重。
- `./scripts/benchmark.sh`：通过，第一阶段 11 个 pallet 全部完成正式 benchmark 并写入 `weights.rs`。
- `rg -n "pub struct WeightInfo<T>|impl<T: frame_system::Config> [a-z_]+::WeightInfo|\\+/-" citizenchain/runtime -g 'weights.rs'`：无输出，确认没有默认模板残留或非 ASCII 误差符号残留。
- `bash -n citizenchain/scripts/benchmark.sh`：通过。
- `git diff --check -- . ':!citizencode/backend/admins/operation_auth.rs' ':!citizencode/backend/core/chain_runtime.rs' ':!citizencode/backend/core/db.rs' ':!memory/08-tasks/open/20260621-admins-change-builtin-pup-selfgovern.md'`：通过。
- `cargo check --release --features runtime-benchmarks --bin citizenchain`：通过，确认新生成的权重文件和 benchmark fixture 参与 release 编译无错误。
