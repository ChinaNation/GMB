# 管理员更换提案与其他提案互斥

- **日期**: 2026-04-04
- **模块**: Blockchain Agent — admins-origin-gov, voting-engine-system
- **优先级**: 中

## 设计

同一机构内，管理员更换提案与其他提案互斥：

1. **发起管理员更换提案时**：该机构不能有任何活跃中的提案（转账/费率/划转/安全基金/销毁/升级等），必须全部结束才能发起
2. **有管理员更换提案活跃时**：该机构不能发起其他类型的提案，必须等管理员更换提案结束（通过/拒绝/超时）才能发起

## 目的

避免不同时期管理员混合投票的语义模糊问题。确保每个提案的投票人池在整个投票期间保持一致。

## 实现思路

- voting-engine-system 新增查询接口：`has_active_proposals(institution) -> bool`
- voting-engine-system 新增查询接口：`has_active_admin_replacement(institution) -> bool`（需要 admins-origin-gov 通过 trait 注册提案类型标记）
- admins-origin-gov 在 `propose_replace_admin` 前调用 `has_active_proposals` 校验
- 其他 pallet 在 `create_internal_proposal` 前调用 `has_active_admin_replacement` 校验
- 或者统一在 voting-engine-system 的 `create_internal_proposal` 中加互斥检查

## 涉及文件

- `voting-engine-system/src/lib.rs` — 新增活跃提案查询 + 互斥校验
- `admins-origin-gov/src/lib.rs` — propose 前校验
- 可能需要在 Proposal 结构或独立 Storage 中标记提案类型（admin_replacement vs other）
