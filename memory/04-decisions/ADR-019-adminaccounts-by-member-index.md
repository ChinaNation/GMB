# ADR-019：管理员账户反向发现索引

- 状态：Superseded（2026-07-24，被 ADR-039 与 ADR-040 取代）
- 关联：ADR-018、ADR-039、ADR-040

## 原提案结论

本 ADR 曾计划为旧 `AdminAccounts` 聚合账户模型增加按成员反向索引，以避免 CitizenApp 全表扫描。该模型已经被拆分后的 public/private/personal admins、机构岗位任职和统一 `account_id` 契约取代，原提案不再实施。

## 当前边界

- 机构管理员名册以机构 `cid_number` 为键，成员统一使用 `Admin { account_id, cid_number, family_name, given_name }`。
- 机构业务权限由 `cid_number + role_code + account_id` 和有效任职共同成立，不能通过“某账户属于哪些 admins”反向索引推导。
- 个人多签保持独立授权主体；客户端发现策略必须读取当前真实 storage，不得恢复旧聚合账户模型。
- 正式创世前所有版本保持 `0`，本废弃提案不新增 storage、migration 或 runtime 升级。
