# MODULE_TAG 注册表

## 用途

MODULE_TAG 是各业务模块在 `voting-engine` 的 `ProposalData` 中写入的字节前缀。投票引擎本身不解析提案数据内容，仅提供通用 `Vec<u8>` 存储。各模块在写入时加前缀、在读取时校验前缀，防止跨模块误解码。

## 适用场景

MODULE_TAG 仅用于共享 `ProposalData` 存储的模块。若模块使用独立 `StorageMap` 存储提案动作数据，则不需要 MODULE_TAG。

## TAG 注册表

| MODULE_TAG | 字节值 | 所属 Pallet | 数据格式 |
|------------|--------|------------|----------|
| `b"dq-xfer"` | `[100,113,45,120,102,101,114]` | `duoqian-transfer` | `MODULE_TAG + TransferAction (SCALE)` |
| `b"dq-mgmt"` | `[100,113,45,109,103,109,116]` | `duoqian-manage` | `MODULE_TAG + ACTION_CODE (1 byte) + payload (SCALE)` |
| `b"res-iss"` | `[114,101,115,45,105,115,115]` | `resolution-issuance` | `MODULE_TAG + IssuanceProposalData (SCALE)` |
| `b"res-dst"` | `[114,101,115,45,100,115,116]` | `resolution-destro` | `MODULE_TAG + DestroyAction (SCALE)` |
| `b"rt-upg"` | `[114,116,45,117,112,103]` | `runtime-upgrade` | `MODULE_TAG + Proposal (SCALE)`; 大对象另存 ProposalObject |
| `b"gra-key"` | `[103,114,97,45,107,101,121]` | `grandpakey-change` | `MODULE_TAG + KeyReplaceProposal (SCALE)` |
| `b"adm-rep"` | `[97,100,109,45,114,101,112]` | `admins-change` | `MODULE_TAG + AdminReplacementProposal (SCALE)` |

## 使用独立 StorageMap（不需要 MODULE_TAG）的模块

| Pallet | 独立存储 | 说明 |
|--------|---------|------|
| `duoqian-transfer` | `SafetyFundProposalActions`, `SweepProposalActions` | 安全基金转账和手续费划转使用独立存储；普通转账仍使用 ProposalData + MODULE_TAG |
| `offchain-transaction` | `RateProposalActions` | 费率设置提案使用独立存储 |

## 编解码协议

**写入**（propose 阶段）：
```
let mut encoded = Vec::from(MODULE_TAG);
encoded.extend_from_slice(&action.encode());
store_proposal_data(proposal_id, encoded);
```

**读取**（execute/callback 阶段）：
```
let raw = get_proposal_data(proposal_id);
let tag = MODULE_TAG;
assert!(raw.starts_with(tag), "MODULE_TAG mismatch");
let action = Action::decode(&mut &raw[tag.len()..]);
```

## 设计原则

1. TAG 长度固定 6-7 字节，均为 ASCII 可读字符
2. 校验失败时 panic 或返回错误，不做回退尝试
3. `duoqian-manage` 在 TAG 后增加 1 字节 ACTION_CODE 区分 create/close/其他操作
4. `runtime-upgrade` 的 runtime wasm 大对象通过 `ProposalObject` 单独存储，ProposalData 中仅存摘要
