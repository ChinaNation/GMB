# runtime 权重基线盘点与 benchmark 生成

## 状态

阻塞：benchmark runtime WASM 已可从本地源码构建，但 clean wbuild 暴露 `organization-manage` 注销凭证改造残留，正式权重尚未生成。

## 任务需求

- 盘点 `citizenchain/runtime` 中所有 pallet 的权重实现、benchmark 覆盖和 runtime 接线情况。
- 区分已启用真实业务逻辑、当前禁用但可提前校准、仍为空壳或 stub 的模块。
- 后续在获得 runtime 二次确认后，按盘点结果运行 benchmark 并写入正式权重。

## 当前阶段

第一阶段盘点和前置修复已完成，正式 benchmark 执行被 runtime WASM 构建阻塞。

- 已生成 `citizenchain/node/frontend/dist/`，满足 Tauri node 编译前置条件。
- 已修正 `citizenchain/scripts/benchmark.sh` 的第一阶段 pallet 清单、二进制名称、前端 dist 检查和本地 runtime WASM 参数。
- 已修复 `citizenchain/runtime/governance/organization-manage/src/benchmarks.rs` 在 `runtime-benchmarks` feature 下的编译问题。
- 未写入任何 `weights.rs` 正式权重，禁止在 benchmark 未完成时手写或伪造权重值。
- 已修复 `WASM_BUILD_FROM_SOURCE=1` 构建 benchmark runtime WASM 时的 `institution-asset/std` feature 泄漏。
- 已修复 `cid-system` benchmark 源码：`wasm32v1-none` 下 `vec!` 宏未导入。
- 已修复 `organization-manage` benchmark 源码：`wasm32v1-none` 下 `Vec` 类型未导入。
- 已确认 `WASM_BUILD_FROM_SOURCE=1 cargo build --release --features runtime-benchmarks --bin citizenchain` 曾通过。
- 已获得第一阶段 11 个 `weights.rs` 写入确认并执行 `./scripts/benchmark.sh`。
- `./scripts/benchmark.sh` 第一次执行时，因 benchmark CLI 默认 `dev` chain spec 未被节点 `load_spec` 支持，11 个 pallet 均未进入实际 benchmark。
- 已修复 `citizenchain/node/src/core/command.rs`，让 `dev/local/staging` 内置别名落到 CitizenChain 冻结 chainspec。
- 再次执行 `./scripts/benchmark.sh pow_difficulty` 时，clean wbuild 暴露 `organization-manage` 注销凭证改造残留，benchmark 尚未进入权重采样阶段。

## 盘点结论

### 可直接重跑并写入正式权重的候选模块

这些模块已挂入 `runtime/src/benchmarks.rs`，且 benchmark 覆盖当前 `WeightInfo` 方法，适合作为第一批正式权重生成对象：

- `shengbank_interest` → `citizenchain/runtime/issuance/shengbank-interest/src/weights.rs`
- `fullnode_issuance` → `citizenchain/runtime/issuance/fullnode-issuance/src/weights.rs`
- `citizen_issuance` → `citizenchain/runtime/issuance/citizen-issuance/src/weights.rs`
- `resolution_issuance` → `citizenchain/runtime/issuance/resolution-issuance/src/weights.rs`
- `cid_system` → `citizenchain/runtime/otherpallet/cid-system/src/weights.rs`
- `pow_difficulty` → `citizenchain/runtime/otherpallet/pow-difficulty/src/weights.rs`
- `admins_change` → `citizenchain/runtime/governance/admins-change/src/weights.rs`
- `resolution_destro` → `citizenchain/runtime/governance/resolution-destro/src/weights.rs`
- `grandpakey_change` → `citizenchain/runtime/governance/grandpakey-change/src/weights.rs`
- `duoqian_transfer` → `citizenchain/runtime/transaction/duoqian-transfer/src/weights.rs`
- `runtime_upgrade` → `citizenchain/runtime/governance/runtime-upgrade/src/weights.rs`

### 不能直接重跑覆盖的模块

- `organization_manage`：已挂 benchmark，但当前只实现 `register_cid_institution`；`weights.rs` 还包含 `propose_create_institution`、`propose_close`、`cleanup_rejected_proposal`。直接覆盖会丢失方法。
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
- `citizenchain/runtime/governance/organization-manage/src/benchmarks.rs` 已更新 `register_cid_institution` benchmark 调用参数，匹配当前 11 参数接口。
- `citizenchain/runtime/governance/organization-manage/src/benchmarks.rs` 已清理不再使用的 benchmark helper 和 import，`cargo check -p organization-manage --features runtime-benchmarks` 通过。

### 当前阻塞问题

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
--> runtime/governance/organization-manage/src/benchmarks.rs:60:27
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

## 预计后续修改目录

- `citizenchain/scripts/benchmark.sh`：已修正第一阶段 benchmark 入口；后续需在 WASM 构建修复后复跑。
- `citizenchain/runtime/Cargo.toml`：已移除 `runtime-benchmarks` 和 `try-runtime` 中对 `institution-asset/std` 的 WASM 污染接线。
- `citizenchain/runtime/transaction/institution-asset/Cargo.toml`：已补空的 `runtime-benchmarks` / `try-runtime` feature。
- `citizenchain/runtime/otherpallet/cid-system/src/benchmarks.rs`：已导入 no_std 可用的 `vec!` 宏，继续推进 benchmark runtime WASM 构建。
- `citizenchain/runtime/governance/organization-manage/src/benchmarks.rs`：已导入 no_std 可用的 `Vec` 类型，benchmark runtime WASM 构建通过。
- `citizenchain/node/src/core/command.rs`：已补齐 `dev/local/staging` chain spec 别名，避免 benchmark CLI 默认 `dev` 被误当文件路径。
- `citizenchain/runtime/governance/organization-manage/src/lib.rs`：后续需修正注销关闭 extrinsic 参数类型和 no_std Vec 路径。
- `citizenchain/runtime/governance/organization-manage/src/close.rs`：后续需同步注销凭证校验、nonce 防重放、scope 写入 `CloseInstitutionAction`。
- `citizenchain/runtime/**/weights.rs`：仅在真实 benchmark 跑通后写入正式权重。
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
- `git diff --check -- citizenchain/runtime/Cargo.toml citizenchain/runtime/transaction/institution-asset/Cargo.toml citizenchain/runtime/otherpallet/cid-system/src/benchmarks.rs citizenchain/runtime/governance/organization-manage/src/benchmarks.rs citizenchain/scripts/benchmark.sh memory/08-tasks/20260622-runtime-weight-baseline.md`：通过。
- `./scripts/benchmark.sh`：第一次全量运行未进入实际采样，因 `dev` chain spec 未识别失败。
- `./scripts/benchmark.sh pow_difficulty`：修复 chain spec 别名后重新运行，clean wbuild 暴露 `organization-manage` 注销凭证改造残留，未进入实际采样。
