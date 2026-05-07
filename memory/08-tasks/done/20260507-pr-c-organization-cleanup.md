# PR-C organization-manage 物理残留清理

## 任务目标

执行重新创世前总审计 PR-C：清理 `organization-manage` 中已废弃的机构创建代投残留，修正机构关闭目录边界，并在完成后重新扫描确认没有活跃代码残留。

## 当前真源

- 机构创建：`OrganizationManage.propose_create_institution`
- 机构关闭：`organization-manage/src/close.rs`
- 个人关闭：`personal-manage::propose_close`
- 仍活跃错误：`MalformedSignature`，由 `pubkey_from_accountid` 使用，必须保留。

## 预计修改目录

- `citizenchain/runtime/governance/organization-manage/`：删除机构创建代投死代码、删除错误关闭空壳、同步测试 mock；涉及 runtime 代码和测试。
- `citizenchain/runtime/src/`：删除 Runtime Config impl 中不再存在的旧配置项；涉及 runtime 配置。
- `citizenchain/runtime/transaction/duoqian-transfer/`：同步测试 mock，避免 Config trait 删除后编译失败；涉及测试支撑代码。
- `memory/08-tasks/open/`：记录 PR-C 执行范围、结果和验收；只涉及文档。

## 执行清单

- [x] 删除 `organization-manage::Config::MaxAdminSignatureLength`。
- [x] 删除 `AdminSignatureOf<T>` 与 `AdminSignaturesOf<T>`。
- [x] 删除无 emit 路径的 `CreateFinalized` 事件。
- [x] 删除无使用路径的 `UnauthorizedSignature` / `DuplicateSignature` / `InvalidSignature` / `InsufficientSignatures`。
- [x] 保留仍活跃的 `MalformedSignature`。
- [x] 删除 `institution/close.rs` 空壳，并从 `institution/mod.rs` 移除模块声明。
- [x] 同步 Runtime Config 和测试 mock。
- [x] 回写审计文档并运行验收。

## 验收标准

- `cargo check -p organization-manage` 通过。
- `cargo test -p organization-manage --lib` 通过。
- `cargo check -p citizenchain --lib` 通过。
- `rg -n 'finalize_create|CreateFinalized|AdminSignaturesOf|AdminSignatureOf|MaxAdminSignatureLength|UnauthorizedSignature|DuplicateSignature|InvalidSignature|InsufficientSignatures' citizenchain/runtime/governance/organization-manage citizenchain/runtime/src citizenchain/runtime/transaction/duoqian-transfer` 不再命中活跃代码残留。
- `rg -n 'MalformedSignature' citizenchain/runtime/governance/organization-manage/src/lib.rs` 仍命中定义和使用。
- `git diff --cached --check` 通过。

## 执行结果

2026-05-07 已执行：

- 删除 `organization-manage::Config::MaxAdminSignatureLength`。
- 删除 `AdminSignatureOf<T>` 与 `AdminSignaturesOf<T>`。
- 删除无 emit 路径的 `CreateFinalized` 事件。
- 删除无使用路径的 `UnauthorizedSignature`、`DuplicateSignature`、`InvalidSignature`、`InsufficientSignatures`。
- 保留 `MalformedSignature`，当前仍由 `pubkey_from_accountid` 使用。
- 删除 `citizenchain/runtime/governance/organization-manage/src/institution/close.rs` 空壳，并从 `institution/mod.rs` 移除模块声明。
- 同步删除 `citizenchain/runtime/src/configs/mod.rs`、`organization-manage/src/tests/mod.rs`、`duoqian-transfer/src/tests/mod.rs` 中的旧 Config impl 项。

验收记录：

- `cargo check -p organization-manage`：通过。
- `cargo test -p organization-manage --lib`：通过，24 个测试全绿。
- `cargo test -p duoqian-transfer --lib`：通过，20 个测试全绿。
- `cargo check -p citizenchain --lib`：首次未带 `WASM_FILE` 时被 runtime build.rs 硬规则阻断；随后使用本地已有 `target/wasm/citizenchain.compact.compressed.wasm` 作为 `WASM_FILE` 后通过。
- `rg -n 'finalize_create|CreateFinalized|AdminSignaturesOf|AdminSignatureOf|MaxAdminSignatureLength|UnauthorizedSignature|DuplicateSignature|InvalidSignature|InsufficientSignatures' citizenchain/runtime/governance/organization-manage citizenchain/runtime/src citizenchain/runtime/transaction/duoqian-transfer`：无输出。
- `rg -n 'MalformedSignature' citizenchain/runtime/governance/organization-manage/src/lib.rs`：仍命中定义和使用。
- `rg -n 'pub mod close|个人多签和机构多签共用同一条关闭逻辑|do_propose_close' citizenchain/runtime/governance/organization-manage/src/institution citizenchain/runtime/governance/organization-manage/src/institution/mod.rs`：无输出。
- `git diff --check`：通过。
- `git diff --cached --check`：通过。
