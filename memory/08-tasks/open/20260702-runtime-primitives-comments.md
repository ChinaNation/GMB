# runtime primitives 注释精简

- 日期:2026-07-02
- 状态:完成
- 范围:`citizenchain/runtime/primitives` 中除 `sign.rs` 外的常量与协议文件

## 目标

- 精简 runtime/primitives 冗长注释。
- 保留必要的协议边界、单位、单一真源和测试意图。
- 不修改代码逻辑、常量值、类型定义、测试断言或生成数据。

## 验收

- 注释更短、更直接。
- `cargo fmt --all --check` 通过。
- `cargo check -p primitives` 通过。
- `git diff --check` 通过。

## 完成记录

- 已精简 `src`、`cid`、`cid/china` 与 primitives 金标测试中的冗长注释。
- 保留 CID 格式、费率单位、账户派生、内置机构和金标测试的必要边界说明。
- 未修改 `src/sign.rs`、常量值、类型定义、测试断言或生成数组内容。
