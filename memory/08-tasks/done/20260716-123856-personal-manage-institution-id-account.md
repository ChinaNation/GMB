# 任务卡：personal-manage 管理员生命周期参数改名 institution_id 到 account

- 任务编号：20260716-123856
- 状态：done
- 所属模块：citizenchain/entity/personal-manage
- 当前负责人：Blockchain Agent
- 创建时间：2026-07-16 12:38:56

## 任务需求

个人多签与机构是不同分类且无 CID，personal-manage 内 4 个管理员账户生命周期辅助函数的参数 institution_id:T::AccountId 属机构侧误导命名残留，字面改名为 account，与同文件 ensure_lifecycle_proposal 及调用方对齐

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 定位：`institution_id` 全仓仅 12 处，均在 `citizenchain/runtime/entity/personal-manage/src/lib.rs` 的 4 个管理员账户生命周期辅助函数（`create_pending_admin_account_for_proposal` / `activate_admin_account` / `remove_pending_admin_account` / `close_admin_account`），类型均为 `T::AccountId`，是从机构侧复制的误导命名，个人多签本无 CID。
- 改名：这 12 处 `institution_id` 字面改为 `account`，与同文件 `ensure_lifecycle_proposal(account)`、调用方 `execute.rs`（`account` / `action.account`）保持局部一致；仅命名变更，无行为变化。
- 调用方无需改动：Rust 位置参数，`execute.rs` / `create.rs` 调用点不依赖参数名。
- 下游 trait `admin-primitives` 对应参数名为 `personal_account`，本文件既有约定为 `account`，故收敛为 `account`。

## 验证记录

- `grep -c institution_id personal-manage/src/lib.rs` → 0，零残留。
- `cargo check -p personal-manage` → Finished，编译通过。
- 全 citizenchain 机构身份键复查：公权/私权/非法人主键仍统一为 CID（本改动不触及身份键，个人多签本就以 AccountId 为主体）。

## 完成信息

- 完成时间：2026-07-16 12:40:19
- 完成摘要：personal-manage 4 个管理员生命周期辅助函数参数 institution_id→account 改名完成，12 处零残留，cargo check 通过
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
