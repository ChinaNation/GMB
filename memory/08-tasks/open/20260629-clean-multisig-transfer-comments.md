# 清理多签转账模块文案

## 任务目标

清理 `multisig-transfer` 中仍停留在“机构多签转账”的旧文案，统一表达为“多签资金账户转账”。

## 执行范围

- `citizenchain/runtime/transaction/multisig-transfer/`
  - 只清理模块说明、字段注释、错误注释、benchmark/weight 注释和测试注释。
  - 不修改业务逻辑、存储结构、call_index 或权重数值。
- `memory/05-modules/citizenchain/runtime/transaction/multisig-transfer/`
  - 同步技术文档中的个人多签接入口径。

## 验收要求

- 文案明确 `multisig-transfer` 只负责转账提案与执行。
- 文案明确个人多签生命周期归 `personal-manage`。
- 文案明确个人多签管理员真源归 `personal-admins`。
- 文案明确个人多签通过 `personal-manage::PersonalMultisigQuery` 接入转账。

## 状态

- 已完成。
