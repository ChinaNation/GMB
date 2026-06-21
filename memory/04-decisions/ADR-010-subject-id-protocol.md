# ADR-010 DUOQIAN AccountId 派生唯一协议

- 状态:Superseded by target-state AccountId governance
- 决议日期:2026-06-07
- 关联任务卡:`memory/08-tasks/done/20260607-duoqian-accountid-governance.md`

## 决议

治理、投票、管理员和发行资格主体统一使用机构或个人多签 `AccountId`。历史 48 字节主体包装协议已废止；资产编号 `asset_id` 只表示资产本身，不能承担治理身份，也不能派生投票身份。

## 唯一派生协议

唯一真源位于 `citizenchain/runtime/primitives/src/core_const.rs`。

```rust
pub const DUOQIAN: &[u8; 7] = b"DUOQIAN";

pub const OP_MAIN: u8 = 0x00;
pub const OP_FEE: u8 = 0x01;
pub const OP_STAKE: u8 = 0x02;
pub const OP_AN: u8 = 0x03;
pub const OP_HE: u8 = 0x04;
pub const OP_PERSONAL: u8 = 0x05;
pub const OP_INSTITUTION: u8 = 0x06;

pub fn derive_duoqian_account(op_tag: u8, ss58: u16, payload: &[u8]) -> [u8; 32] {
    // DUOQIAN || op_tag || ss58.to_le_bytes() || payload -> blake2_256
}
```

公开 API 参数命名为 `ss58`；写入 preimage 时在函数内部转为 `ss58.to_le_bytes()`。

## 主体边界

- 内置机构：创世表内置，`cid_number -> DUOQIAN -> AccountId`。
- 注册机构：CID 注册，`cid_number + account_name -> DUOQIAN -> AccountId`。
- 个人多签：`creator + account_name -> DUOQIAN -> AccountId`。
- onchain-issuance：发行资格主体、治理主体和管理员主体均为机构多签 `AccountId`。
- `asset_id`：只作为资产编号，标识被治理的资产，不承担投票身份。

## 模块契约

- `votingengine` 只接收 `AccountId` 作为内部投票主体。
- `admins-change` 的管理员账户 storage key 为 `AccountId`。
- `duoqian-transfer` 的支出机构为 `AccountId`。
- `onchain-issuance` 的 issuer/governance/admin 均为 `AccountId`；`asset_id` 仅作为资产编号。
- CID、citizenwallet、citizenapp、tools 不得再定义第二套 DUOQIAN domain、op_tag 或 hash preimage。

## 迁移结论

旧 48 字节主体包装协议、资产编号派生治理身份、以及第二套 DUOQIAN 派生常量均不再作为目标态的一部分。历史 ADR 内容如需追溯，应以 Git 历史查看；当前文档只记录目标态。
