# 任务卡：primitives traits.rs + types.rs 合并为 multisig 模块

## 任务需求

把 `primitives/src/traits.rs`（多签账户校验/资金保护 3 个 Config 注入 trait）与 `primitives/src/types.rs`（MultisigConfig / MultisigConfigSnapshot 类型）合并为单一模块 `primitives::multisig`，删除两个原文件，更新全部下游引用。0 行为变化。

（用户原话是 traits→china_zb，分析后改为 traits+types→multisig：两者服务同一组多签治理 pallet，china_zb 是 china 机构常量文件、语义/路径不合。用户选 A 方案。）

## 修改范围 / 执行记录

- 新建 `primitives/src/multisig.rs`：3 trait（AccountValidator / ReservedAccountGuard / ProtectedSourceChecker）+ 2 类型（MultisigConfig / MultisigConfigSnapshot），doc 合并。
- 删 `primitives/src/traits.rs`、`primitives/src/types.rs`。
- `primitives/src/lib.rs`：删 `pub mod traits;` `pub mod types;`，加 `pub mod multisig;`。
- 下游 8 文件 `primitives::traits` / `primitives::types` → `primitives::multisig`（含 organization-manage 本地 traits.rs 的 re-export `pub use` 与 doc 注释）：
  - organization-manage/src/{lib.rs, traits.rs, tests/mod.rs}
  - personal-manage/src/{lib.rs, create.rs, close.rs, traits.rs, tests/mod.rs}
- 未动：china_zb.rs、duoqian-transfer（不引用）、各 pallet 本地业务 trait（InstitutionMultisigQuery / CidInstitutionVerifier / PersonalMultisigQuery 等）。
- 踩坑记录：批量替换首次用 shell 变量 `$FILES` 传参，zsh 不做无引号分词，perl 把 8 路径当成 1 个文件名→替换未执行；改为把 8 路径直接作为 perl 参数后成功。

## 验证记录

- 全仓 `primitives::traits` / `primitives::types` 残留 **0**；`primitives::multisig` 引用 22 处；旧文件已删；lib.rs 仅剩 `pub mod multisig;`。
- `cargo check --manifest-path personal-manage/Cargo.toml`：Finished dev 15.46s。
- `cargo check --manifest-path organization-manage/Cargo.toml`：Finished dev 1.13s。
- 两次合计 **0 error / 0 warning**；连带编译了 `primitives::multisig`（模块本身 + 下游路径切换均通过）。

## 后续

- runtime/node 全量编译与正式链 runtime 升级由后续统一发布；本卡纯 primitives 模块重组，0 行为变化。
