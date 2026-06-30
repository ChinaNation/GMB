# 20260627 CID runtime / registry 目录彻底收敛

## 任务需求

把 CID 相关代码按 runtime 与 registry 两侧对称目录收敛:

- runtime 侧统一放到 `citizenchain/runtime/primitives/cid/`
- registry 侧统一放到 `citizenchain/registry/src/cid/`
- 删除旧 `runtime/primitives/src/code.rs`、`runtime/primitives/china/`、`registry/src/china/`、`registry/src/number/`
- 不保留旧路径兼容壳,全仓引用一次性迁到新路径

## 设计边界

- runtime 只放确定性、no_std 可用的 CID 协议、代码常量、校验、核心生成规则和链上内置机构常量。
- registry 保留 SQLite 行政区、当前年份、随机 UUID、数据库查重、HTTP DTO/API 等运行态功能。
- `docs/citizenpassport/` 是备份目录,本任务不修改。

## 预计修改目录

- `citizenchain/runtime/primitives/cid/`:runtime CID 唯一目录,涉及代码。
- `citizenchain/runtime/primitives/src/`:移除旧 code 入口并调整 lib/count_const,涉及代码。
- `citizenchain/runtime/`:替换 `primitives::code` / `primitives::china` 引用,涉及代码。
- `citizenchain/registry/src/cid/`:registry CID 唯一目录,涉及代码与 SQLite LFS 文件。
- `citizenchain/registry/src/`:移除旧 china/number 顶层模块引用,涉及代码。
- `.gitattributes`:更新 `china.sqlite` LFS 路径,涉及配置。
- `citizenchain/scripts/`:更新本地 ONCHINA_CHINA_DB 路径,涉及开发脚本。
- `memory/`:清理旧 `number/`、顶层 `china/`、`primitives/src/code.rs` 文档残留,涉及文档。

## 验收要求

- 全仓不再出现业务代码引用 `crate::number`、`crate::china`、`primitives::code`、`primitives::china`。
- `registry/src/number/` 与 `registry/src/china/` 删除。
- `runtime/primitives/src/code.rs` 与 `runtime/primitives/china/` 删除。
- 至少运行 Rust 格式化与相关 cargo check / test,并对残留路径做 `rg` 扫描。

## 完成记录

- 完成 runtime primitives CID 目录收敛:`cid/code.rs`、`cid/china/`、`cid/number.rs`、`cid/generator.rs`、`cid/seed.rs`。
- 完成 registry CID 目录收敛:`src/cid/` 统一承载发号适配、动态种子、SQLite 行政区、分类和管理端元信息接口。
- 删除旧顶层 `china` / `number` 代码目录,不保留兼容壳。
- 更新 `.gitattributes`、`citizenchain/scripts/`、Tauri resource、前端 API、技术文档与本地文档生成文件。
- 清理 `.DS_Store`、`__pycache__` 等本机缓存残留。

## 验收结果

- `cargo fmt --all --manifest-path citizenchain/Cargo.toml`
- `cargo check -p primitives --manifest-path citizenchain/Cargo.toml`
- `cargo check -p registry --manifest-path citizenchain/Cargo.toml`
- `cargo check -p citizenchain --manifest-path citizenchain/Cargo.toml`
- `cargo check -p node --manifest-path citizenchain/Cargo.toml`
- `cargo test -p primitives --manifest-path citizenchain/Cargo.toml`
- `cargo test -p registry --manifest-path citizenchain/Cargo.toml`
- `npm run build` in `citizenchain/node/frontend`
- `node --check citizenapp/tools/generate_admin_division_bundle.mjs`
- `node --check scripts/generate_citizenapp_governance_registry.mjs`
- `python3 -m py_compile scripts/rebake_china_codes.py scripts/fill_china_admins.py scripts/gmb.py`
- `node scripts/generate_citizenapp_governance_registry.mjs`
- 旧路径残留扫描通过,`docs/citizenpassport/` 按要求未纳入本次清理。
