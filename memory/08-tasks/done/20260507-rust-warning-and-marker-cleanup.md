# 任务卡:清理 Rust warning 与待办标记

## 任务需求

修复剩余第 4、5 项：

- 自有 Rust 代码 warning 清零。
- 自有代码和自有文档中的待办/修复/临时标记清零。
- fork/vendor 目录建立单独基线，不和自有代码门禁混算。

## 预计修改目录

- `citizenchain/runtime/`：清理 runtime 自有 Rust warning 和 benchmark 占位说明；涉及代码。
- `citizenchain/node/src/`：清理 node 自有 Rust warning，必要时补窄范围说明；涉及代码。
- `sfid/backend/`：清理后端 Rust warning；涉及代码。
- `cpms/backend/`：清理后端 Rust warning；涉及代码。
- `wuminapp/rust/`：清理 smoldot FFI 自有实现 warning；涉及代码。
- `wuminapp/smoldot-dart/rust/`：清理本地 Dart 包自带 Rust FFI warning；涉及代码/配置。
- `wuminapp/lib/proposal/runtime_upgrade/`：清理联合公投提交占位标记；涉及 Dart 代码。
- `memory/05-modules/`：清理自有文档中的待办标记；涉及文档。
- `memory/07-ai/`：补充 fork/vendor 基线和门禁规则；涉及 AI 系统文档。
- `memory/08-tasks/`：记录任务过程并归档；涉及任务文档。

## 验收标准

- 自有代码/文档待办类标记扫描为 0。
- fork/vendor 待办类标记只统计，不阻断。
- 自有 Rust 包完成 warning 检查；能修复的 warning 被修复，不能纳入本轮的阻塞项明确记录。
- `git diff --check` 通过。

## 执行记录

- [x] 扫描自有待办类标记。
- [x] 清理自有待办类标记。
- [x] 建立 fork/vendor 基线规则。
- [x] 扫描 Rust warning 基线。
- [x] 修复 Rust warning。
- [x] 执行验收。

## 验收记录

- 自有代码和自有文档待办类标记扫描为 0；fork/vendor、历史任务卡、资源哈希按基线规则排除。
- `cargo check --manifest-path sfid/backend/Cargo.toml` 通过。
- `cargo check --manifest-path cpms/backend/Cargo.toml` 通过。
- `cargo check --manifest-path wuminapp/rust/Cargo.toml` 通过。
- `cargo check --manifest-path wuminapp/smoldot-dart/rust/Cargo.toml` 通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo check --manifest-path citizenchain/Cargo.toml` 通过。
- `RUSTFLAGS="-D warnings" cargo check --manifest-path sfid/backend/Cargo.toml` 通过。
- `RUSTFLAGS="-D warnings" cargo check --manifest-path cpms/backend/Cargo.toml` 通过。
- `RUSTFLAGS="-D warnings" cargo check --manifest-path wuminapp/rust/Cargo.toml` 通过。
- `RUSTFLAGS="-D warnings" cargo check --manifest-path wuminapp/smoldot-dart/rust/Cargo.toml` 通过。
- `RUSTFLAGS="-D warnings" WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo check --manifest-path citizenchain/Cargo.toml` 通过。
- `git diff --check` 通过。

## 结论

剩余第 4 项 Rust warning 已按自有代码门禁清理完成；剩余第 5 项待办类残留已按自有范围清零。fork/vendor 和第三方依赖 future-incompat 报告已建立单独基线，不再和自有业务代码清零门禁混算。
