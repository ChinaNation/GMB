# DUOQIAN_TECHNICAL

模块：`duoqian-transaction-pow`  
范围：SFID 机构登记、注册型多签机构创建、注册型多签机构关闭（当前实现）

## 0. 功能需求

### 0.1 核心职责
- 提供 `sfid_id -> duoqian_address` 的链上登记能力。
- 提供注册型多签机构的创建提案、创建投票和创建落地能力。
- 提供注册型多签机构的关闭提案、关闭投票和关闭落地能力。
- 机构创建和关闭不走离线聚合签名，当前实现统一走内部投票引擎 `ORG_DUOQIAN`。

### 0.2 地址与域隔离需求
- `duoqian_address` 必须由链上按 `BLAKE2b("DUOQIAN_SFID_V1" || ss58_prefix_le || sfid_id)` 派生。
- `ss58_prefix_le` 为 SS58 前缀 2027 的小端 `u16` 字节，用于链域隔离。
- 派生地址必须通过地址合法性校验，且不能落入制度保留地址或受保护地址集合。

### 0.3 SFID 登记需求
- `register_sfid_institution` 必须只接受 `sfid_id + register_nonce + signature`。
- `signature` 必须通过 `("GMB_SFID_INSTITUTION_V1", genesis_hash, sfid_id, register_nonce)` 验签。
- 机构登记只认当前 SFID `MAIN` 公钥，不再信任交易发送者身份。
- 登记成功后必须写入 `sfid_id <-> duoqian_address` 双向映射，并消费 `register_nonce`。

### 0.4 创建流程需求
- `propose_create` 必须只接受已登记的 `sfid_id`，并从登记映射中解析目标地址。
- 管理员数量必须 `>= 2`，且 `duoqian_admins.len() == admin_count`。
- 阈值必须满足 `ceil(admin_count / 2) <= threshold <= admin_count`，同时最小不少于 2。
- 管理员公钥必须逐个校验格式，且不能重复。
- 创建金额必须 `>= MinCreateAmount`。
- 发起人必须是管理员之一。
- 创建提案成功后，提案和业务动作统一写入内部投票引擎。

### 0.5 关闭流程需求
- `propose_close` 必须只作用于已存在且状态为 `Active` 的多签账户。
- 发起人必须是该多签账户管理员之一。
- 多签账户余额必须同时满足 `free_balance >= MinCloseBalance` 和 `reserved_balance == 0`。
- `beneficiary` 不能等于多签地址自身，且不能是保留地址、非法地址或受保护地址。
- 关闭提案成功后，提案和业务动作统一写入内部投票引擎。

### 0.6 存储与制度边界
- 链上维护 `sfid_id -> duoqian_address` 和 `duoqian_address -> { sfid_id, nonce }` 双向映射。
- 链上维护 `DuoqianAccounts`，保存管理员、阈值、状态、初始金额等运行中信息。
- 本模块只负责注册型多签机构的登记、创建和关闭，不负责机构转账。

## 1. 当前实现结论

当前代码实际提供 5 个公开入口：

1. `register_sfid_institution(sfid_id, register_nonce, signature)`
2. `propose_create(sfid_id, admin_count, duoqian_admins, threshold, amount)`
3. `vote_create(proposal_id, approve)`
4. `propose_close(duoqian_address, beneficiary)`
5. `vote_close(proposal_id, approve)`

当前代码没有实现以下旧口径：

- 没有 `create_duoqian(... expires_at, approvals)` 这种离线 M-of-N 一次性提交。
- 没有 `close_duoqian(... expires_at, approvals)` 这种离线 M-of-N 一次性提交。
- 没有 `DUOQIAN_CREATE_V3 / DUOQIAN_CLOSE_V3` 业务签名 payload。

## 2. 地址派生与链域

地址派生公式（当前）：

`duoqian_address = Blake2b256("DUOQIAN_SFID_V1" || ss58_prefix_le || sfid_id_bytes)`

说明：
1. `ss58_prefix_le` 为 SS58 前缀 2027 的小端 `u16` 字节（`[0xEB, 0x07]`）。
2. 该派生逻辑由链上固定实现，前端和 SFID 不能自定义多签地址。

## 3. 链上存储

1. `DuoqianAccounts<duoqian_address, DuoqianAccount>`
2. `SfidRegisteredAddress<sfid_id, duoqian_address>`
3. `AddressRegisteredSfid<duoqian_address, RegisteredInstitution { sfid_id, nonce }>`
4. `UsedRegisterNonce<register_nonce, bool>`

补充：
1. `DuoqianAccount.status` 当前只有 `Pending / Active` 两种状态。
2. `AddressRegisteredSfid.nonce` 当前跟随机构地址保存，用于机构生命周期变更计数。
3. 当前制度下，关闭多签账户不会删除 `sfid_id <-> duoqian_address` 的登记映射。

## 4. Extrinsic 规则

### 4.1 register_sfid_institution(sfid_id, register_nonce, signature)

校验：
1. `sfid_id` 非空。
2. `register_nonce` 未被消费。
3. `signature` 必须能通过 `("GMB_SFID_INSTITUTION_V1", genesis_hash, sfid_id, register_nonce)` 验签，且只认当前 SFID `MAIN`。
4. `sfid_id` 未登记。
5. 派生地址未登记、非保留地址、非受保护地址、地址格式合法。

执行：
1. 写入双向映射。
2. 记录 `register_nonce` 已消费。
3. 发出 `SfidInstitutionRegistered`。
4. 初始化该地址的 `RegisteredInstitution.nonce = 0`。

### 4.2 propose_create(sfid_id, admin_count, duoqian_admins, threshold, amount)

关键校验：
1. 调用者 `who` 非受保护源。
2. `admin_count >= 2` 且 `duoqian_admins.len() == admin_count`。
3. `threshold` 满足 `ceil(admin_count / 2) <= threshold <= admin_count`，且最小至少 2。
4. `amount >= MinCreateAmount`。
5. 所有管理员公钥必须唯一。
6. `who` 必须在管理员列表中。
7. `sfid_id` 已登记，且从链上登记映射派生出的 `duoqian_address` 一致。
8. 派生地址必须合法、非保留、非受保护，且当前不能已存在于 `DuoqianAccounts`。

执行：
1. 先写入 `DuoqianAccounts(status = Pending)`。
2. 以 `ORG_DUOQIAN` 创建内部投票提案。
3. 把 `CreateDuoqianAction` 编码写入投票引擎的 `ProposalData`。
4. 发出 `DuoqianCreateProposed`。

### 4.3 vote_create(proposal_id, approve)

关键校验：
1. `proposal_id` 必须存在，且提案业务数据能解码为 `CreateDuoqianAction`。
2. 当前投票人必须是该 `duoqian_address` 的管理员。

执行：
1. 调用内部投票引擎投票。
2. 若提案状态进入 `PASSED`，自动尝试执行 `execute_create`。
3. 执行成功后把提案状态改成 `EXECUTED`。

### 4.4 propose_close(duoqian_address, beneficiary)

关键校验：
1. 调用者 `who` 非受保护源。
2. `duoqian_address` 不能是受保护地址。
3. `beneficiary` 不能等于 `duoqian_address`，且必须合法、非保留、非受保护。
4. 目标机构必须存在于 `DuoqianAccounts`，且状态为 `Active`。
5. 调用者必须是当前管理员之一。
6. `free_balance >= MinCloseBalance`。
7. `reserved_balance == 0`。

执行：
1. 以 `ORG_DUOQIAN` 创建内部投票提案。
2. 把 `CloseDuoqianAction` 编码写入投票引擎的 `ProposalData`。
3. 发出 `DuoqianCloseProposed`。

### 4.5 vote_close(proposal_id, approve)

关键校验：
1. `proposal_id` 必须存在，且提案业务数据能解码为 `CloseDuoqianAction`。
2. 当前投票人必须是该 `duoqian_address` 的管理员。

执行：
1. 调用内部投票引擎投票。
2. 若提案状态进入 `PASSED`，自动尝试执行 `execute_close`。
3. 执行成功后把提案状态改成 `EXECUTED`。

## 5. 内部执行逻辑

### 5.1 execute_create

执行内容：
1. 从提案发起者向 `duoqian_address` 转入初始金额。
2. 把 `DuoqianAccounts.status` 从 `Pending` 改成 `Active`。
3. 把 `AddressRegisteredSfid.nonce += 1`。
4. 发出 `DuoqianCreated`。
5. 调用投票引擎把提案状态写成 `STATUS_EXECUTED`。

### 5.2 execute_close

执行内容：
1. 把 `duoqian_address` 全额转给 `beneficiary`。
2. 删除 `DuoqianAccounts`。
3. 把 `AddressRegisteredSfid.nonce += 1`。
4. 发出 `DuoqianClosed`。
5. 调用投票引擎把提案状态写成 `STATUS_EXECUTED`。

## 6. 与投票引擎的关系

- 本模块不自己实现投票。
- 创建和关闭都委托给 `voting-engine-system` 的内部投票引擎。
- 使用的组织类型是 `ORG_DUOQIAN`。
- `ORG_DUOQIAN` 的管理员和阈值不是固定表，而是 runtime 从 `DuoqianAccounts` 动态读取。

## 7. 与 duoqian-transfer-pow 的关系

两者职责不同：

| 模块 | 职责 | 地址类型 | 当前审批方式 |
| --- | --- | --- | --- |
| `duoqian-transaction-pow` | SFID 登记、机构创建、机构关闭 | 注册型多签机构 | `sfid` 主签名登记 + 内部投票引擎 `ORG_DUOQIAN` |
| `duoqian-transfer-pow` | 机构多签地址转账 | 当前只覆盖内置治理机构 | 内部投票引擎 |

本模块当前不负责机构转账。

## 8. 已知实现风险

### 8.1 Pending 残留风险

在 `propose_create` 中，模块会先把 `DuoqianAccounts` 写成 `Pending`，然后才创建内部投票提案。

当前代码已确认：
1. `execute_create` 只会把这条记录改成 `Active`。
2. `execute_close` 只会在机构已 `Active` 且通过关闭提案后删除记录。
3. 本模块中未看到“创建提案被拒绝 / 超时后，清理这条 Pending 记录”的路径。

这意味着存在以下风险：
- 创建提案未通过后，链上残留一条 `Pending` 机构记录。
- 后续再次对同一 `sfid_id` / `duoqian_address` 发起创建时，可能命中 `AddressAlreadyExists`。

这个问题需要单独修复，不能靠文档规避。

## 9. 前端 / 系统接入要求

1. 机构创建前必须先完成 `register_sfid_institution`。
2. 前端不能手填 `duoqian_address`，只能提交 `sfid_id`。
3. 机构创建和关闭当前都不是离线聚合签名一次性提交，而是链上提案 + 管理员逐票投票。
4. 如果业务需要“注册型多签机构转账”，不能误以为本模块已经提供，需要另行接入 `duoqian-transfer-pow` 或扩展转账能力。
