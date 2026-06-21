# 任务卡：CPMS 行政区文件收敛到 address/ 目录

## 任务需求

`citizenpassport/backend/china.rs`（CID 行政区源只读适配）只被 `address.rs`（CPMS 地址业务）消费，二者同属"地址/行政区"内聚关注点。把两个顶层文件收进一个 `address/` 模块目录，china 作为 address 的源适配子文件，保留"源/业务"分层。

## 背景结论（分析已确认）

- `china.rs`(158 行)= 纯 CID `china.sqlite` 只读适配；`address.rs`(246 行)= CPMS 自有地址业务（HTTP 路由 + postgres `address_towns/villages` + 安装重建 + 安装码校验），是 china 的唯一消费方。
- 全工程 `china::` 仅 `address.rs` 调用，main.rs 仅 `mod china;` 声明 → china 是 address 的专属实现细节。
- 用户确认：收进 `address/` 目录，源适配子文件保留命名 `china.rs`。
- 安全点：`china.rs` 的 `CARGO_MANIFEST_DIR` dev 兜底路径 + 测试 `point_to_china_source()` 均基于 crate 根 `citizenpassport/backend`，与文件在模块树位置无关 → 移动不影响。

## 建议模块

- CPMS 后端 `address`
- CPMS 技术文档

## 影响范围

- `citizenpassport/backend/address.rs` → `citizenpassport/backend/address/mod.rs`（git mv 保历史）。涉及代码移动。
- `citizenpassport/backend/china.rs` → `citizenpassport/backend/address/china.rs`（git mv 保历史）。涉及代码移动。
- `citizenpassport/backend/main.rs`：删 `mod china;` 及其上方中文注释（china 文档已在 china.rs 自带）。`mod address;` 自动解析到 `address/mod.rs`。涉及代码 + 残留清理。
- `citizenpassport/backend/address/mod.rs`：顶部加 `mod china;`；`use crate::{...}` 去掉 `china`；`china::find_city` 等调用改指本模块子模块（语法不变）。涉及代码。
- `citizenpassport/CITIZENPASSPORT_TECHNICAL.md`：模块表 `china | china.rs` 与 `address | address.rs` 两行改为 `address/china.rs`、`address/mod.rs`（或合并描述）。涉及文档。

## 主要风险点

- `address.rs` 与 `address/` 不能并存（Rust 模块歧义）；移动后只留 `address/`。
- `china.rs` 无 `crate::` 依赖（仅 std + rusqlite），移动安全；`pub(crate)` 可见性保持。
- 不改任何行为，纯结构收敛；验收靠 `cargo check/clippy/test` 全绿 + 残留扫描。

## 是否需要先沟通

- 形状与命名已确认：收进 address/ 目录，子文件保留 china.rs。

## 执行清单

- [ ] mkdir address/；`git mv` address.rs→address/mod.rs、china.rs→address/china.rs。
- [ ] main.rs 删 `mod china;` + 注释。
- [ ] address/mod.rs 加 `mod china;`，crate:: import 去 china。
- [ ] CPMS_TECHNICAL.md 模块表更新。
- [ ] `cargo fmt --check && cargo clippy --all-targets && cargo test`（32 通过保持）。
- [ ] 残留扫描：无顶层 `mod china;`、无 `crate::china`。

## 完成记录

- 2026-06-06：创建任务卡，开始执行。
- 2026-06-06：执行完成。china.rs + address.rs 收进 `citizenpassport/backend/address/`（`address/mod.rs` 业务 + `address/china.rs` CID 源适配子模块，git 识别 rename 保历史），main.rs 顶层 `mod china;` 删除。纯结构收敛、0 行为变化；`CARGO_MANIFEST_DIR` dev 兜底路径与测试 helper 基于 crate 根不受影响。fmt/clippy/test 全绿（32 passed），残留扫描无 `crate::china`。
