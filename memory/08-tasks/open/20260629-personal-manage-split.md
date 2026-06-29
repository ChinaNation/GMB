# 修复个人多签模块

## 任务目标

只拆分个人多签模块职责，不触碰其他线程正在处理的无关工作。

- `citizenchain/runtime/private/personal-manage/` 负责个人多签账户生命周期。
- `citizenchain/runtime/admins/personal-admins/` 负责个人多签管理员。
- `citizenchain/runtime/transaction/multisig-transfer/` 只负责多签转账。

## 执行边界

- 只修改个人多签拆分必需的 runtime 代码、测试和文档。
- 不回退、不覆盖工作区中其他线程已有改动。
- 提交时只暂存本次拆分相关文件。

## 验收要求

- 个人多签创建、关闭、拒绝清理入口归属 `personal-manage`。
- 个人多签管理员集合和管理员变更入口归属 `personal-admins`。
- `multisig-transfer` 只通过查询 trait 获取个人多签账户状态和管理员配置。
- 更新文档、完善中文注释、清理旧口径残留。
- 执行可行的 cargo 检查；无法执行或失败时记录原因。

## 状态

- 已完成。

## 验收记录

- `cargo check -p personal-admins`
- `cargo check -p personal-manage`
- `cargo check -p multisig-transfer`
- `cargo check -p citizenchain`
- `cargo check -p personal-admins --tests`
- `cargo check -p personal-manage --tests`
- `cargo check -p multisig-transfer --tests`
- `cargo test -p personal-admins -p personal-manage`
- `cargo test -p multisig-transfer`
- `cargo test -p citizenchain`
