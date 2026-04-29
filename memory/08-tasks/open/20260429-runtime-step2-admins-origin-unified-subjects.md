# 任务卡：第2步区块链段统一管理员真源

## 状态

- open

## 背景

第1步已在 `duoqian-manage-pow` 内新增机构级创建模型，但管理员与阈值仍在多签模块内保留旧语义。根据最新确认，`admins-origin-gov` 应作为所有机构/多签主体的管理员真源，覆盖内置机构、SFID 注册机构多签和个人多签。

## 目标

- 将管理员主体统一收口到 `admins-origin-gov`。
- 支持内置机构、SFID 机构多签、个人多签三类主体。
- `voting-engine-system` 的 runtime provider 统一从 `admins-origin-gov` 读取管理员、阈值、人数。
- `duoqian-manage-pow` 只负责账户、资金、生命周期，不再作为管理员长期真源。
- 修正 `duoqian-transfer-pow` 与 `offchain-transaction-pos` 的旧 `DuoqianAccounts` 管理员/激活读取残留。
- 更新文档、完善中文注释、清理残留。

## 不在本步范围

- 不改 SFID 后端。
- 不改 node UI。
- 不改 wuminapp。
- 不改 wumin。

## 涉及模块

- `citizenchain/runtime/governance/admins-origin-gov`
- `citizenchain/runtime/transaction/duoqian-manage-pow`
- `citizenchain/runtime/transaction/duoqian-transfer-pow`
- `citizenchain/runtime/transaction/offchain-transaction-pos`
- `citizenchain/runtime/src/configs/mod.rs`

## 验收标准

- `ORG_NRC / ORG_PRC / ORG_PRB / ORG_DUOQIAN` 管理员均由 `admins-origin-gov` 提供。
- SFID 机构多签与个人多签创建 pending 时可被投票引擎快照管理员。
- 创建通过后管理员主体变 Active，拒绝/失败后清理 pending 主体。
- 相关 Rust 测试通过。
