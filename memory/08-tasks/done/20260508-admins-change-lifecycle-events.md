# 任务卡:修复 admins-change 生命周期清理与事件 org 字段

## 任务需求

修复 admins-change 审查中的 L-1 / L-2:

- L-1:`do_remove_pending_subject` 不得在主体不存在时静默成功。
- L-2:`AdminAccountActivated` / `AdminAccountPendingRemoved` / `AdminAccountClosed` 事件必须携带 `org` 字段,方便客户端和索引器按组织分桶。

## 预计修改目录

- `citizenchain/runtime/governance/admins-change/`：修改生命周期事件结构、执行函数和单测；涉及 Rust 代码、测试和注释。
- `memory/05-modules/citizenchain/runtime/governance/admins-change/`：同步技术文档；涉及文档。
- `memory/07-ai/unified-protocols.md`：如存在 admins-change 事件协议描述,同步事件字段；涉及统一协议文档。
- `memory/08-tasks/`：记录执行与验收结果；涉及任务文档。

## 验收标准

- 不存在 Pending 主体时,`do_remove_pending_subject` 返回 `InvalidInstitution`。
- Pending 主体非 Pending 状态时仍返回 `SubjectNotPending`。
- 激活、移除 Pending、关闭主体 3 类事件都包含 `org`。
- admins-change 单测通过。
- 残留扫描确认旧事件结构未在当前模块文档中继续作为现行协议出现。

## 执行记录

- [x] 修改链端事件和生命周期函数。
- [x] 补充/调整单测。
- [x] 更新文档。
- [x] 运行验证。

## 验证结果

- `cargo fmt --manifest-path citizenchain/Cargo.toml -p admins-change`：通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib`：41 passed。
- 残留扫描：当前模块、统一协议文档和 ADR 已无旧生命周期事件现行结构残留。
