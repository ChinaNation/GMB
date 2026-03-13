# Governance 治理模块技术文档（区块链规范版）

## 1. 模块目标

`lib/governance/` 负责 WuminApp 的链上治理能力规范，覆盖：

- 提案（proposal）发起
- 投票（vote）提交
- 提案状态跟踪与结果展示

说明：

- 本文档定义的是“链上字段/格式/标准/流程”。
- 当前 App 里治理业务 UI 仍是占位，本文作为后续开发对齐基线。

## 2. 链上入口与权限边界

### 2.1 关键约束（必须遵守）

- `voting-engine-system` 的 `create_internal_proposal`、`create_joint_proposal` 和 `internal_vote` 外部调用被禁用，直接调用会返回 `NoPermission`。
- 联合提案必须由业务治理 pallet 通过 `JointVoteEngine` trait 发起。
- 内部投票必须由业务治理 pallet 通过 `InternalVoteEngine` trait 转发。

### 2.2 可直接由交易发起的投票引擎入口

- `submit_joint_institution_vote(proposal_id, institution, internal_passed, expires_at, approvals)`
- `citizen_vote(proposal_id, sfid_hash, nonce, signature, approve)`

## 3. 通用字段与格式标准

### 3.1 基础类型

| 字段 | 链上类型 | App 传输规范 |
| --- | --- | --- |
| `account` | `AccountId32` | SS58 地址字符串（当前链 `ss58 = 2027`） |
| `institution` | `[u8; 48]` | `0x` + 96 hex（机构 pallet id） |
| `proposal_id` | `u64` | 十进制整数 |
| `approve/internal_passed` | `bool` | `true/false` |
| `expires_at` | `BlockNumber` | 十进制整数，必须是链上仍未过期的 proof 截止块 |
| `nonce` | `BoundedVec<u8, 64>` | `0x` hex，解码后字节长度 `1..64` |
| `signature` | `BoundedVec<u8, 64>` | `0x` hex，解码后字节长度 `1..64` |
| `sfid_hash` | `Hash` | `0x` + 64 hex |

联合门限证明项：

| 字段 | 链上类型 | App 传输规范 |
| --- | --- | --- |
| `approvals[].public_key` | `[u8; 32]` | `0x` + 64 hex（管理员 `sr25519` 公钥） |
| `approvals[].signature` | `[u8; 64]` | `0x` + 128 hex（管理员对联合决定 payload 的原始签名） |

### 3.2 枚举与编码

- `org`：`0 = NRC(国储会)`，`1 = PRC(省储会)`，`2 = PRB(省储行)`。
- proposal kind：`0 = internal`，`1 = joint`。
- stage：`0 = internal`，`1 = joint`，`2 = citizen`。
- status：`0 = voting`，`1 = passed`，`2 = rejected`。

### 3.3 时效与阈值

- 单阶段投票时长：`VOTING_DURATION_BLOCKS`（当前为 30 天对应区块数）。
- 内部投票通过阈值：
  - NRC：`13`
  - PRC：`6`
  - PRB：`6`
- 联合投票权重：
  - NRC：`19`
  - 每个 PRC：`1`
  - 每个 PRB：`1`
  - 总票权：`105`
- 联合机构提交时，`approvals` 必须达到该机构当前链上管理员门限：
  - NRC：`13`
  - PRC：`6`
  - PRB：`6`
- 联合投票 `yes >= 105` 立即通过，否则在“全机构结果齐备”或超时后进入公民投票阶段。
- 公民投票通过规则：`yes * 100 > eligible_total * 50`（严格大于 50%）。

## 4. 提案字段规范（按业务类型）

| 业务类型 | 提案入口 | 必填字段 | 发起权限 | 投票入口 |
| --- | --- | --- | --- | --- |
| 决议发行 | `propose_resolution_issuance` | `reason, total_amount, allocations[], eligible_total, snapshot_nonce, snapshot_signature` | 国储会管理员（NRC） | 联合+公民 |
| Runtime 升级 | `propose_runtime_upgrade` | `reason, code, eligible_total, snapshot_nonce, snapshot_signature` | 国储会管理员（NRC） | 联合+公民 |
| 管理员更换 | `propose_admin_replacement` | `org, institution, old_admin, new_admin` | 目标机构管理员 | 内部 |
| 决议销毁 | `propose_destroy` | `org, institution, amount` | 目标机构管理员 | 内部 |
| GRANDPA 密钥更换 | `propose_replace_grandpa_key` | `institution, new_key(32B)` | NRC/PRC 机构管理员 | 内部 |
| 省储行业务治理 | `propose_institution_rate / propose_verify_key / propose_sweep_to_main / propose_relay_submitters` | 见 4.4 | PRB 机构管理员 | 内部 |

### 4.1 联合提案额外字段标准（决议发行 / Runtime 升级）

- `eligible_total`：`u64`，必须 `> 0`。
- `snapshot_nonce`：`1..64` 字节。
- `snapshot_signature`：`1..64` 字节，运行时当前要求 64 字节 `sr25519` 原始签名。

人口快照验签消息标准（runtime）：

```text
payload = (
  "GMB_SFID_POPULATION_V2",
  genesis_hash,
  who,
  eligible_total,
  snapshot_nonce
)
message = blake2_256(SCALE.encode(payload))
```

### 4.2 决议发行 allocations 约束

- `allocations` 不能为空。
- 每个 `recipient` 必须唯一，`amount > 0`。
- 分配接收者集合必须与链上 `AllowedRecipients` 完整一致（不能缺项、不能多项）。
- `sum(allocations.amount) == total_amount`。

### 4.3 GRANDPA 密钥更换约束

- `new_key` 不能全 0。
- `new_key` 必须是合法 Ed25519 压缩公钥（32 字节）。
- `new_key` 不得与当前 key 相同，不得与其他机构正在使用 key 冲突。
- 同一 `new_key` 不得被并发提案占用。

### 4.4 省储行业务治理约束（offchain-transaction-pos）

- `propose_institution_rate`：
  - `new_rate_bp` 范围 `1..10`（0.01%~0.1%）。
- `propose_verify_key`：
  - `new_key` 非空，长度不超过 `MaxVerifyKeyLen(当前 256)`。
- `propose_sweep_to_main`：
  - `amount > 0`；
  - 执行时还要满足保底与上限规则（保留费地址最低余额、单次最多提可用余额 80%）。
- `propose_relay_submitters`：
  - `submitters` 数量 `1..MaxRelaySubmitters(当前 8)`；
  - 账户不得重复。

## 5. 投票字段规范

### 5.1 内部投票（业务 pallet）

内部投票业务入口统一字段：

- `proposal_id: u64`
- `approve: bool`

典型函数：

- `vote_admin_replacement`
- `vote_destroy`
- `vote_replace_grandpa_key`
- `vote_institution_rate`
- `vote_verify_key`
- `vote_sweep_to_main`
- `vote_relay_submitters`

### 5.2 联合机构投票（投票引擎）

`submit_joint_institution_vote` 字段：

- `proposal_id: u64`
- `institution: [u8;48]`
- `internal_passed: bool`（该机构内部投票是否通过）
- `expires_at: BlockNumber`
- `approvals: Vec<{ public_key: [u8;32], signature: [u8;64] }>`

权限要求：

- 必须由“该机构多签账户”提交，不能由其他机构或普通管理员代提。
- 同时必须附带当前机构管理员对本次联合决定的门限签名证明；只有“多签地址提交 + proof 验证通过”两者都满足才会记票。

运行时联合决定验签消息标准：

```text
payload = (
  "GMB_JOINT_DECISION_V1",
  genesis_hash,
  proposal_id,
  institution,
  internal_passed,
  expires_at
)
message = blake2_256(SCALE.encode(payload))
```

联合 proof 校验要求：

- `approvals` 不能为空，且签名人必须属于该机构当前链上管理员集合。
- 同一管理员不能重复出现在 `approvals` 里。
- 签名数量必须达到该机构当前门限。
- `expires_at` 到期后该 proof 无效，不能重放旧内部决议。

### 5.3 公民投票（投票引擎）

`citizen_vote` 字段：

- `proposal_id: u64`
- `sfid_hash: Hash`
- `nonce: BoundedVec<u8,64>`
- `signature: BoundedVec<u8,64>`
- `approve: bool`

运行时投票凭证验签消息标准：

```text
payload = (
  "GMB_SFID_VOTE_V2",
  genesis_hash,
  account,
  sfid_hash,
  proposal_id,
  nonce
)
message = blake2_256(SCALE.encode(payload))
```

防重放要求：

- 同一 `proposal_id + sfid_hash` 只能投一次。
- 同一 `proposal_id + sfid_hash + nonce` 不能重放。

## 6. 标准流程

### 6.1 提案发起流程（App 侧）

1. 选择业务类型并收集业务字段。
2. 校验当前钱包是否具备该机构管理员权限。
3. 若为联合提案，先获取 `eligible_total + snapshot_nonce + snapshot_signature`。
4. 组装链上调用字段并签名提交。
5. 记录 `proposal_id` 与业务类型映射，订阅状态事件。

### 6.2 投票流程（App 侧）

1. 根据提案类型匹配投票入口（内部/联合/公民）。
2. 采集投票字段并做本地格式校验；若为联合投票，还要收集足够数量的管理员签名 proof。
3. 发起签名并提交交易。
4. 监听事件刷新状态：
  - `InternalVoteCast / JointInstitutionVoteCast / CitizenVoteCast`
  - `ProposalAdvancedToCitizen`
  - `ProposalFinalized`

### 6.3 超时与补偿

- 投票引擎在 `on_initialize` 自动做到期结算（支持分桶分批）。
- 业务 pallet 允许在部分场景手动执行或重试（如执行失败补偿）。
- App 端必须支持“最终状态后停止轮询”，避免重复提交。

## 7. 源码对齐基线

- `citizenchain/governance/voting-engine-system/src/lib.rs`
- `citizenchain/governance/voting-engine-system/src/joint_vote.rs`
- `citizenchain/governance/voting-engine-system/src/citizen_vote.rs`
- `citizenchain/governance/resolution-issuance-gov/src/lib.rs`
- `citizenchain/governance/runtime-root-upgrade/src/lib.rs`
- `citizenchain/governance/admins-origin-gov/src/lib.rs`
- `citizenchain/governance/resolution-destro-gov/src/lib.rs`
- `citizenchain/governance/grandpa-key-gov/src/lib.rs`
- `citizenchain/transaction/offchain-transaction-pos/src/lib.rs`
- `citizenchain/runtime/src/configs/mod.rs`
- `primitives/src/count_const.rs`
