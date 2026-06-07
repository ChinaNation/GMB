# 任务卡：citizen_constitution.rs 合并进 genesis.rs 并删除

## 任务需求

把 `primitives/src/citizen_constitution.rs` 的功能（公民宪法 HTML 常量 + Runtime API 声明）并入同目录 `genesis.rs`，然后删除 `citizen_constitution.rs`，并修正所有引用路径。0 行为变化。

## 修改范围 / 执行记录

- `primitives/src/genesis.rs`：新增 `use sp_std::vec::Vec;` + `CITIZEN_CONSTITUTION_HTML` 常量（`include_str!("CitizenConstitution.html")` 同目录相对路径不变）+ `sp_api::decl_runtime_apis! { CitizenConstitutionApi }`（编为"四、五"两节）。
- `primitives/src/citizen_constitution.rs`：已删除。
- `primitives/src/lib.rs`：删 `pub mod citizen_constitution;`。
- `runtime/src/apis.rs`：`primitives::citizen_constitution::` → `primitives::genesis::`（3 处）。
- `node/src/core/rpc.rs`：`primitives::citizen_constitution::` → `primitives::genesis::`（2 处）。
- 未动：trait 方法名 `citizen_constitution_html` / `citizen_constitution_blake2_256`，rpc.rs:261/267 方法调用，HTML 本体。

## 验证记录

- `citizen_constitution.rs` 已删；全仓 `primitives::citizen_constitution::` 残留 0、`mod citizen_constitution` 残留 0。
- `cargo check --manifest-path citizenchain/runtime/primitives/Cargo.toml`：`primitives v1.0.0 ... Finished dev in 24.36s`，exit 0，无 error/warning。
- 注：首次 `cargo check -p primitives` 从仓库根跑报 "package did not match"（根工作区不含该包），改用 manifest-path 后通过。
- runtime(apis.rs)/node(rpc.rs) 未单独编译（重）；路径改动与 apis.rs 对称，且 primitives 已过编译，API trait 路径有效。

## 后续

- runtime/node 全量编译 + 正式链 runtime 升级仍由后续统一发布；本卡只做 primitives 内部重组，0 行为变化。
