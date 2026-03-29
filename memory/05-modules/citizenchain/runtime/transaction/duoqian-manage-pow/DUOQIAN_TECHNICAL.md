# DUOQIAN_TECHNICAL

模块：`duoqian-manage-pow`  
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
- `register_sfid_institution` 必须接受 `sfid_id + name + register_nonce + signature`。
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
- `duoqian_address` 的资金转出必须同时通过 `institution-asset-guard` 白名单检查。
- 关闭提案成功后，提案和业务动作统一写入内部投票引擎。

### 0.6 存储与制度边界
- 链上维护 `sfid_id -> duoqian_address` 和 `duoqian_address -> { sfid_id, nonce }` 双向映射。
- 链上维护 `DuoqianAccounts`，保存管理员、阈值、状态、初始金额等运行中信息。
- 本模块只负责注册型多签机构的登记、创建和关闭，不负责机构转账。

## 1. 当前实现结论

当前代码实际提供 6 个公开入口：

1. `register_sfid_institution(sfid_id, name, register_nonce, signature)`
2. `propose_create(sfid_id, admin_count, duoqian_admins, threshold, amount)`
3. `vote_create(proposal_id, approve)`
4. `propose_close(duoqian_address, beneficiary)`
5. `vote_close(proposal_id, approve)`
6. `propose_create_personal(name, admin_count, duoqian_admins, threshold, amount)`

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
3. `AddressRegisteredSfid<duoqian_address, RegisteredInstitution { sfid_id, name }>`
4. `UsedRegisterNonce<register_nonce_hash, bool>`
5. `PersonalDuoqianInfo<duoqian_address, PersonalDuoqianMeta { creator, name }>`：个人多签反向索引
6. `PendingCloseProposal<duoqian_address, proposal_id>`：每个多签账户当前进行中的关闭提案 ID，防止并发注销

补充：
1. `DuoqianAccount.status` 当前只有 `Pending / Active` 两种状态。
2. 当前制度下，关闭多签账户不会删除 `sfid_id <-> duoqian_address` 的登记映射。

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

### 4.6 propose_create_personal(name, admin_count, duoqian_admins, threshold, amount)

关键校验：
1. 调用者 `who` 非受保护源。
2. `name` 非空。
3. `admin_count >= 2` 且 `duoqian_admins.len() == admin_count`。
4. `threshold` 满足 `ceil(admin_count / 2) <= threshold <= admin_count`，且最小至少 2。
5. `amount >= MinCreateAmount`，余额覆盖 `amount + fee + ED`。
6. 所有管理员公钥必须唯一，`who` 必须在管理员列表中。
7. 地址由 `Blake2b256("DUOQIAN_PERSONAL_V1" || ss58_prefix_le || creator.encode() || name_utf8)` 派生。
8. 派生地址必须合法、非保留、非受保护，且当前不存在于 `DuoqianAccounts`。

执行：
1. 先写入 `DuoqianAccounts(status = Pending)` 和 `PersonalDuoqianInfo { creator, name }`。
2. 以 `ORG_DUOQIAN` 创建内部投票提案。
3. 把 `CreateDuoqianAction` 编码写入投票引擎的 `ProposalData`（复用 `ACTION_CREATE`）。
4. 发出 `PersonalDuoqianProposed`。

投票通过后由 `vote_create` 复用 `execute_create` 完成入金和激活。

## 5. 内部执行逻辑

### 5.1 execute_create

执行内容（在 `with_transaction` 内原子执行）：
1. 计算手续费（复用 `onchain-transaction-pow` 公共费率）。
2. 从提案发起者向 `duoqian_address` 转入初始金额（`KeepAlive`）。
3. 从提案发起者额外扣取手续费，通过 `FeeRouter` 分账。
4. 把 `DuoqianAccounts.status` 从 `Pending` 改成 `Active`。
5. 发出 `DuoqianCreated`（含 fee 字段）。
6. 调用投票引擎把提案状态写成 `STATUS_EXECUTED`。

失败处理：若执行失败，`with_transaction` 回滚资金操作，外层清理 Pending 条目和 PersonalDuoqianInfo，释放地址锁定。

### 5.2 execute_close

执行内容（在 `with_transaction` 内原子执行）：
1. 校验 `InstitutionAssetGuard::can_spend`。
2. 计算手续费，确保扣费后转给 beneficiary 的金额 >= ED。
3. 从 `duoqian_address` 扣取手续费，通过 `FeeRouter` 分账。
4. 将剩余余额转给 `beneficiary`（`AllowDeath`）。
5. 删除 `DuoqianAccounts`、`PersonalDuoqianInfo`、`PendingCloseProposal`。
6. 发出 `DuoqianClosed`（含 fee 字段）。
7. 调用投票引擎把提案状态写成 `STATUS_EXECUTED`。

失败处理：若执行失败，`with_transaction` 回滚资金操作，外层清理 `PendingCloseProposal`，允许重新发起关闭提案。

## 6. 与投票引擎的关系

- 本模块不自己实现投票。
- 创建和关闭都委托给 `voting-engine-system` 的内部投票引擎。
- 使用的组织类型是 `ORG_DUOQIAN`。
- `ORG_DUOQIAN` 的管理员和阈值不是固定表，而是 runtime 从 `DuoqianAccounts` 动态读取。

## 7. 与 duoqian-transfer-pow 的关系

两者职责不同：

| 模块 | 职责 | 地址类型 | 当前审批方式 |
| --- | --- | --- | --- |
| `duoqian-manage-pow` | SFID 登记、机构创建、机构关闭 | 注册型多签机构 | `sfid` 主签名登记 + 内部投票引擎 `ORG_DUOQIAN` |
| `duoqian-transfer-pow` | 机构多签地址转账 | 当前只覆盖内置治理机构 | 内部投票引擎 |

本模块当前不负责机构转账。

## 8. 已修复的历史风险

### 8.1 创建状态机闭环（已修复）

`propose_create` 会先把 `DuoqianAccounts` 写成 `Pending`，然后才创建内部投票提案。

已修复：
- `vote_create` 检测到 `STATUS_PASSED` 时执行入金激活；执行失败时清理 Pending 条目。
- `vote_create` 检测到 `STATUS_REJECTED` 时清理 Pending 条目和 PersonalDuoqianInfo，释放地址锁定。
- 投票引擎超时 reject（`on_initialize`）后无人再投票时，任意账户可调用 `cleanup_rejected_proposal` 清理。

### 8.2 关闭状态机闭环（已修复）

`propose_close` 写入 `PendingCloseProposal` 防止并发注销。

已修复：
- `vote_close` 检测到 `STATUS_PASSED` 时执行关闭；执行失败时清理 PendingCloseProposal。
- `vote_close` 检测到 `STATUS_REJECTED` 时清理 PendingCloseProposal。
- 投票引擎超时 reject 后，任意账户可调用 `cleanup_rejected_proposal` 清理。

## 9. 前端 / 系统接入要求

1. 机构创建前必须先完成 `register_sfid_institution`。
2. 前端不能手填 `duoqian_address`，只能提交 `sfid_id`。
3. 机构创建和关闭当前都不是离线聚合签名一次性提交，而是链上提案 + 管理员逐票投票。
4. 如果业务需要“注册型多签机构转账”，不能误以为本模块已经提供，需要另行接入 `duoqian-transfer-pow` 或扩展转账能力。
