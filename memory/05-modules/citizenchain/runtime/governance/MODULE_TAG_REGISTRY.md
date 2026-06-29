# MODULE_TAG 注册表

## 用途

MODULE_TAG 是各业务模块在 `votingengine` 的 `ProposalData` / `ProposalOwner` 中写入的字节标识。投票引擎本身不解析提案数据内容，但会用 `ProposalOwner` 做 owner 校验，禁止跨模块覆写。各模块在读取时仍需校验前缀或独立存储键，防止误解码。

## 适用场景

MODULE_TAG 仅用于共享 `ProposalData` 存储的模块。若模块使用独立 `StorageMap` 存储提案动作数据，则不需要 MODULE_TAG。

## TAG 注册表

| MODULE_TAG | 字节值 | 所属 Pallet | 数据格式 |
|------------|--------|------------|----------|
| `b"multisig-transfer"` | `[109,117,108,116,105,115,105,103,45,116,114,97,110,115,102,101,114]` | `multisig-transfer` | `MODULE_TAG + TransferAction (SCALE)` |
| `b"pub-mgmt"` | `[112,117,98,45,109,103,109,116]` | `public-manage` | `MODULE_TAG + ACTION_CODE (1 byte) + payload (SCALE)` |
| `b"pri-mgmt"` | `[112,114,105,45,109,103,109,116]` | `private-manage` | `MODULE_TAG + ACTION_CODE (1 byte) + payload (SCALE)` |
| `b"per-mgmt"` | `[112,101,114,45,109,103,109,116]` | `personal-admins` | `MODULE_TAG + ACTION_CODE (1 byte) + payload (SCALE)` |
| `b"res-iss"` | `[114,101,115,45,105,115,115]` | `resolution-issuance` | `MODULE_TAG + IssuanceProposalData (SCALE)` |
| `b"res-dst"` | `[114,101,115,45,100,115,116]` | `resolution-destro` | `MODULE_TAG + DestroyAction (SCALE)` |
| `b"rt-upg"` | `[114,116,45,117,112,103]` | `runtime-upgrade` | `MODULE_TAG + Proposal (SCALE)`; 大对象另存 ProposalObject |
| `b"gra-key"` | `[103,114,97,45,107,101,121]` | `grandpakey-change` | `MODULE_TAG + KeyReplaceProposal (SCALE)` |
| `b"gen-adm1"` | `[103,101,110,45,97,100,109,49]` | `genesis-admins` | `AdminSetChangeAction<AdminsOf<T>> (SCALE)` |
| `b"pub-adm1"` | `[112,117,98,45,97,100,109,49]` | `public-admins` | `AdminSetChangeAction<AdminsOf<T>> (SCALE)` |
| `b"pri-adm1"` | `[112,114,105,45,97,100,109,49]` | `private-admins` | `AdminSetChangeAction<AdminsOf<T>> (SCALE)` |

## 使用独立 StorageMap（不需要 MODULE_TAG）的模块

| Pallet | 独立存储 | 说明 |
|--------|---------|------|
| `multisig-transfer` | `SafetyFundProposalActions`, `SweepProposalActions` | 安全基金转账和手续费划转使用独立存储；普通转账仍使用 ProposalData + MODULE_TAG |
| `offchain-transaction` | `RateProposalActions` | 费率设置提案使用独立存储 |

## 编解码协议

**写入**（propose 阶段）：
```
let mut encoded = Vec::from(MODULE_TAG);
encoded.extend_from_slice(&action.encode());
let proposal_id = T::InternalVoteEngine::create_internal_proposal_with_data(
    proposer,
    org,
    institution,
    MODULE_TAG,
    encoded,
)?;
```

联合提案使用 `create_joint_proposal_with_data`；Pending 主体和管理员集合变更分别使用对应的 `*_with_data` 变体。禁止业务模块直接调用旧的 `store_proposal_data`。

说明：`genesis-admins`、`public-admins`、`private-admins` 的管理员集合变更提案只把 `MODULE_TAG` 写入 `ProposalOwner`，`ProposalData` 直接保存 `AdminSetChangeAction`，不再重复嵌入 tag 前缀。

**读取**（execute/callback 阶段）：
```
let raw = get_proposal_data(proposal_id);
let tag = MODULE_TAG;
assert!(raw.starts_with(tag), "MODULE_TAG mismatch");
let action = Action::decode(&mut &raw[tag.len()..]);
```

## 设计原则

1. TAG 均为 ASCII 可读字符；需要升级数据结构时必须显式增加版本后缀
2. 校验失败时返回错误或忽略非本模块提案，不做回退尝试
3. `public-manage` / `private-manage` 在 TAG 后增加 1 字节 ACTION_CODE 区分 create/close/其他操作
4. `runtime-upgrade` 的 runtime wasm 大对象通过 `ProposalObject` 单独存储，ProposalData 中仅存摘要
