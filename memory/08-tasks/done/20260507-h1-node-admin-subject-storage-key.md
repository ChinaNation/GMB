# 任务卡:修复节点端管理员主体 storage key

## 任务需求

只修复 H-1：节点端管理员读取路径仍指向旧 `AdminsChange::Institutions` 和 raw padded subject_id，需改为重新创世后的 `AdminsChange::Subjects` 与带 kind tag 的 Builtin subject_id。

## 预计修改目录

- `citizenchain/node/src/governance/`：修复节点端治理模块管理员 storage key 构造；涉及 Rust 代码和单元测试。
- `memory/05-modules/citizenchain/node/governance/`：同步节点治理技术文档，明确管理员读取使用 `AdminsChange::Subjects`；涉及文档。
- `memory/08-tasks/`：记录本次 H-1 修复并归档；涉及任务文档。

## 验收标准

- `admin_subjects_key` 使用 `AdminsChange::Subjects`。
- `admin_subjects_key` 使用带 `0x01` Builtin kind tag 的 subject_id。
- 节点端本地 subject_id 派生与 `primitives::derive::subject_id_from_sfid_number` 字节级一致。
- 不修改 M-1、M-2、L-1、L-2。
- Rust 格式化和目标测试通过。

## 执行记录

- [x] 修复节点端 storage key 构造。
- [x] 补强 storage key 单元测试。
- [x] 更新节点治理文档。
- [x] 执行验收。

## 验收记录

- `admin_subjects_key` 已改为读取 `AdminsChange::Subjects`。
- `admin_subjects_key` 已改为复用本文件 `subject_id_from_sfid_number`，不再 raw padded。
- 单元测试已增加与 `primitives::derive::subject_id_from_sfid_number` 的字节级一致性断言。
- 静态扫描确认节点治理 H-1 范围内不再残留 `AdminsChange::Institutions`、`twox_128(b"Institutions")`、raw padded、旧 migration 注释。
- `cargo fmt --manifest-path citizenchain/Cargo.toml --all` 通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo check --manifest-path citizenchain/Cargo.toml -p node` 通过。
- 目标单测命令被既有无关 test 编译问题阻塞：`node/src/offchain/settlement/packer.rs` 测试代码访问 `OffchainLedger::inner` 私有字段；本轮按要求不修复 H-1 以外问题。

## 结论

H-1 已修复。节点桌面端管理员读取路径现在与重新创世后的 `AdminsChange::Subjects[0x01 || padded_sfid]` 协议一致。
