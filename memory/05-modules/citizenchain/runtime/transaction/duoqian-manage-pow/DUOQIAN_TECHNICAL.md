# DUOQIAN_TECHNICAL

模块：`duoqian-manage-pow`  
范围：SFID 机构登记、注册型多签机构创建、注册型多签机构关闭（当前实现）

## 0. 功能需求

### 0.1 核心职责
- 提供 `sfid_id -> duoqian_address` 的链上登记能力。
- 提供注册型多签机构(及个人多签)的创建提案、离线签名聚合 finalize、自动入金激活能力。
- 提供注册型多签机构(及个人多签)的关闭提案、在线投票关闭能力。
- 机构/个人多签**创建**走"Tx 1 propose + 离线 QR 聚合 N 签 + Tx 2 finalize 代投"的离线聚合路径(Step 1,2026-04-21)。关闭当前仍走"每个管理员在线 vote_close"路径(Step 2 待改造)。
- 两个流程都透过内部投票引擎 `ORG_DUOQIAN` 统一处理提案生命周期。

### 0.2 地址与域隔离需求
- `duoqian_address` 必须由链上按 `BLAKE2b(DUOQIAN_DOMAIN || OP_MAIN || ss58_prefix_le || sfid_id)`（OP_MAIN/OP_FEE 路径）或 `BLAKE2b(DUOQIAN_DOMAIN || OP_INSTITUTION || ss58_prefix_le || sfid_id || account_name)`（OP_INSTITUTION 路径）派生。
- `DUOQIAN_DOMAIN = b"DUOQIAN_V1"`（10 字节），`OP_MAIN = 0x00`，均见 `primitives::core_const`。
- `ss58_prefix_le` 为 SS58 前缀 2027 的小端 `u16` 字节，用于链域隔离。
- 派生地址必须通过地址合法性校验，且不能落入制度保留地址或受保护地址集合。

### 0.3 SFID 登记需求
- `register_sfid_institution` 必须接受 `sfid_id + account_name + register_nonce + signature`。
- `signature` 必须通过 `(DUOQIAN_DOMAIN, OP_SIGN_INST, genesis_hash, sfid_id, account_name, register_nonce)` 验签。
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

当前代码实际提供 7 个公开入口(Step 1 · 多签注册改造后,2026-04-21):

1. `register_sfid_institution(sfid_id, account_name, register_nonce, signature)` — `call_index(2)` SFID 机构登记,独立于多签创建。
2. `propose_create(sfid_id, account_name, admin_count, duoqian_admins, threshold, amount)` — `call_index(0)` Tx 1:创建机构多签提案 + 预写入 Pending。
3. `finalize_create(proposal_id, sigs)` — `call_index(3)` **Tx 2**:离线聚合 N 个管理员 sr25519 签名 → 循环代投 → 自动 execute_create。
4. `propose_close(duoqian_address, beneficiary)` — `call_index(1)` 关闭提案。
5. `vote_close(proposal_id, approve)` — `call_index(4)` 关闭在线投票(Step 2 将改造为离线聚合)。
6. `propose_create_personal(account_name, admin_count, duoqian_admins, threshold, amount)` — `call_index(5)` Tx 1:创建个人多签(无 SFID 归属),流程与 propose_create 对称。
7. `cleanup_rejected_proposal(proposal_id)` — `call_index(6)` 清理超时 reject 后的 Pending 残留。

Step 1 **已删除**:
- `vote_create(proposal_id, approve)` — 被 `finalize_create` 取代。原 `call_index(3)` 现由 `finalize_create` 占用。
- `CreateVoteSubmitted` 事件 — 已被 `CreateFinalized` 覆盖。

Step 1 **已实现**:
- `CreateVoteIntent` SCALE 结构 + `signing_hash(ss58_prefix)` 方法(`DUOQIAN_V1 || OP_SIGN_CREATE || ss58_le || blake2_256(intent)` 两层 hash)。
- `AdminSignatureOf<T>` / `AdminSignaturesOf<T>` BoundedVec 类型别名。
- `compute_admins_root` / `pubkey_from_accountid` 两个 pub helper。
- `OP_SIGN_CREATE = 0x14` 域前缀(位于 `primitives::core_const`)。

Step 2 **待改造**(不在本任务范围):
- `vote_close` → `finalize_close(proposal_id, sigs)`:复用本步的 QR 协议骨架,只替换 op_tag 为 `OP_SIGN_TRANSFER = 0x15`。

## 2. 地址派生与链域

SFID 登记机构的账户地址按**角色**分三条路径派生（`InstitutionAccountRole` 枚举）：

| 角色 | op_tag | 派生公式 |
|---|---|---|
| `Role::Main` | `OP_MAIN = 0x00` | `Blake2b256(DUOQIAN_DOMAIN \|\| 0x00 \|\| ss58_prefix_le \|\| sfid_id)` |
| `Role::Fee` | `OP_FEE = 0x01` | `Blake2b256(DUOQIAN_DOMAIN \|\| 0x01 \|\| ss58_prefix_le \|\| sfid_id)` |
| `Role::Named(account_name)` | `OP_INSTITUTION = 0x05` | `Blake2b256(DUOQIAN_DOMAIN \|\| 0x05 \|\| ss58_prefix_le \|\| sfid_id \|\| account_name)` |

个人多签独立使用 `OP_PERSONAL = 0x04`，payload 为 `ss58 || creator_32 || account_name`。

说明：
1. `DUOQIAN_DOMAIN = b"DUOQIAN_V1"`（10 字节），所有 op_tag 定义见 `primitives::core_const`。
2. `ss58_prefix_le` 为 SS58 前缀 2027 的小端 `u16` 字节（`[0xEB, 0x07]`）。
3. **角色翻译**：链上 `role_from_account_name(account_name)` 把账户名翻译到 Role：
   - `"主账户"` → `Role::Main`（preimage 不含 account_name）
   - `"费用账户"` → `Role::Fee`（preimage 不含 account_name）
   - 其他非空 → `Role::Named(account_name)`（preimage 含 account_name）
   - 空 account_name → 返回 `EmptyAccountName` 错误
4. **保留名校验**：`Role::Named("主账户")` / `Role::Named("费用账户")` 在 `derive_institution_address` 里返回 `ReservedAccountName` 错误——保证 "主账户"/"费用账户" 这两个语义只会落到 `Role::Main`/`Role::Fee`，与其他自定义命名不可冲突。
5. **宪法机构 + SFID 机构的主账户/费用账户派生公式完全一致**（只有 `sfid_id` 值不同，`GFR-...` / `SFR-...` / `FFR-...` 命名空间天然隔离）。
6. 该派生逻辑由链上固定实现，前端和 SFID 后端不能自定义多签地址，只能通过角色路由调用。

## 3. 链上存储

1. `DuoqianAccounts<duoqian_address, DuoqianAccount>`
2. `SfidRegisteredAddress<sfid_id, duoqian_address>`
3. `AddressRegisteredSfid<duoqian_address, RegisteredInstitution { sfid_id, account_name }>`
4. `UsedRegisterNonce<register_nonce_hash, bool>`
5. `PersonalDuoqianInfo<duoqian_address, PersonalDuoqianMeta { creator, account_name }>`：个人多签反向索引
6. `PendingCloseProposal<duoqian_address, proposal_id>`：每个多签账户当前进行中的关闭提案 ID，防止并发注销

补充：
1. `DuoqianAccount.status` 当前只有 `Pending / Active` 两种状态。
2. 当前制度下，关闭多签账户不会删除 `sfid_id <-> duoqian_address` 的登记映射。

## 4. Extrinsic 规则

### 4.1 register_sfid_institution(sfid_id, register_nonce, signature)

校验：
1. `sfid_id` 非空。
2. `register_nonce` 未被消费。
3. `signature` 必须能通过 `(DUOQIAN_DOMAIN, OP_SIGN_INST, genesis_hash, sfid_id, account_name, register_nonce)` 验签，且只认当前 SFID `MAIN`。
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
4. 从投票引擎回读 `proposal.end` 作为 `expires_at`。
5. 发出 `CreateDuoqianProposed { proposal_id, duoqian_address, proposer, sfid_id, account_name, admins, admin_count, threshold, amount, expires_at }`,wuminapp 扫描此事件即可构造 QR。

### 4.3 finalize_create(proposal_id, sigs)

**Step 1 改造新入口**,替代原 `vote_create`。发起人(可以是管理员或他人,因为 Tx 1 已锁定 proposer)一笔提交 N 个管理员对 `CreateVoteIntent` 的 sr25519 签名。

关键校验:
1. 提案业务数据存在,可解码为 `CreateDuoqianAction`。
2. 读出 Pending 状态 `DuoqianAccount`。
3. `sigs.len() >= duoqian.threshold`,否则 `InsufficientSignatures`。
4. 循环每对 `(admin, sig_bytes)`:
   - `admin ∈ duoqian.duoqian_admins`(否则 `UnauthorizedSignature`)
   - 同批次去重(否则 `DuplicateSignature`)
   - `sig_bytes.len() == 64`(否则 `MalformedSignature`)
   - `sr25519_verify(sig, signing_hash, pubkey)`(否则 `InvalidSignature`)
   - `T::InternalVoteEngine::cast_internal_vote(admin, proposal_id, true)`(投票引擎自动做快照检查 / 去重 / 阈值)

签名消息:
```
admins_root = blake2_256(SCALE.encode(sorted(admins)))
intent = CreateVoteIntent { proposal_id, duoqian_address, creator, admins_root, threshold, amount, approve: true }
preimage = DUOQIAN_DOMAIN (10B) || OP_SIGN_CREATE (1B) || SS58_PREFIX_LE (2B) || blake2_256(intent.encode())
signing_hash = blake2_256(preimage)
```

执行:
1. 循环结束后读投票引擎最新 `proposal.status`。
2. `STATUS_PASSED` → 事务内执行 `execute_create`;失败回滚并清理 Pending。
3. `STATUS_REJECTED` → 清理 Pending,发 `DuoqianCreateRejected`。
4. 其他(仍 Voting)→ 不清理,继续等待(但本函数已完整消费签名;实际上这种情况只会在阈值设置异常时出现)。
5. 最后 emit `CreateFinalized { proposal_id, signatures_accepted, final_status }`。

语义要点:
- **发起人不必是管理员**:Tx 1 已把 proposer 锁定,Tx 2 仅代投 + 代付 gas。
- **幂等保护**:同一 proposal_id 被第二次调用时,投票引擎的 `AlreadyVoted` 会让 cast_internal_vote 失败,整笔交易回滚,不会重复入金。
- **不支持分批补签**:一次提交必须 >= 阈值,否则 `InsufficientSignatures`。若部分签名失败则整个事务回滚,发起人需重新组织签名再试。

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

### 4.6 propose_create_personal(account_name, admin_count, duoqian_admins, threshold, amount)

关键校验：
1. 调用者 `who` 非受保护源。
2. `account_name` 非空。
3. `admin_count >= 2` 且 `duoqian_admins.len() == admin_count`。
4. `threshold` 满足 `ceil(admin_count / 2) <= threshold <= admin_count`，且最小至少 2。
5. `amount >= MinCreateAmount`，余额覆盖 `amount + fee + ED`。
6. 所有管理员公钥必须唯一，`who` 必须在管理员列表中。
7. 地址由 `Blake2b256(DUOQIAN_DOMAIN || OP_PERSONAL || ss58_prefix_le || creator.encode() || name_utf8)` 派生（`OP_PERSONAL = 0x04`）。
8. 派生地址必须合法、非保留、非受保护，且当前不存在于 `DuoqianAccounts`。

执行：
1. 先写入 `DuoqianAccounts(status = Pending)` 和 `PersonalDuoqianInfo { creator, account_name }`。
2. 以 `ORG_DUOQIAN` 创建内部投票提案。
3. 把 `CreateDuoqianAction` 编码写入投票引擎的 `ProposalData`（复用 `ACTION_CREATE`）。
4. 从投票引擎回读 `proposal.end` 作为 `expires_at`。
5. 发出 `PersonalDuoqianProposed { proposal_id, duoqian_address, proposer, account_name, admins, admin_count, threshold, amount, expires_at }`。

Tx 2 由 `finalize_create` 统一处理,与机构多签共享同一 extrinsic 和同一套 `CreateVoteIntent` 签名协议(个人多签的 QR 载荷里 kind 字段标记为 `Personal`,但链上 finalize 不区分 kind,从 `PersonalDuoqianInfo` 判断)。

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
- `finalize_create` 检测到 `STATUS_PASSED` 时执行入金激活；执行失败时清理 Pending 条目(Step 1 由 `vote_create` 改造而来)。
- `finalize_create` 检测到 `STATUS_REJECTED` 时清理 Pending 条目和 PersonalDuoqianInfo,释放地址锁定。
- 投票引擎超时 reject（`on_initialize`）后无人再调用 finalize_create 时,任意账户可调用 `cleanup_rejected_proposal` 清理。

### 8.2 关闭状态机闭环（已修复）

`propose_close` 写入 `PendingCloseProposal` 防止并发注销。

已修复：
- `vote_close` 检测到 `STATUS_PASSED` 时执行关闭；执行失败时清理 PendingCloseProposal。
- `vote_close` 检测到 `STATUS_REJECTED` 时清理 PendingCloseProposal。
- 投票引擎超时 reject 后，任意账户可调用 `cleanup_rejected_proposal` 清理。

## 9. 前端 / 系统接入要求

1. 机构创建前必须先完成 `register_sfid_institution`(与多签创建正交,独立调用)。
2. 前端不能手填 `duoqian_address`,只能提交 `sfid_id + account_name`(机构)或 `account_name`(个人)。
3. **创建走离线聚合**:发起人发 Tx 1 `propose_create` / `propose_create_personal` 后,wuminapp 扫读 `CreateDuoqianProposed` / `PersonalDuoqianProposed` 事件 → 生成 QR → 其他管理员逐人扫 QR、核对载荷、用各自 sr25519 私钥签名 `CreateVoteIntent::signing_hash(ss58)` → 发起人收齐 >= threshold 个签名 → 一笔 `finalize_create(proposal_id, sigs)` 代投。
4. 关闭当前仍走在线投票 `vote_close`(Step 2 改造后会替换为 `finalize_close`,复用同样的离线聚合协议)。

## 10. Step 1 改造(2026-04-21)关键变更

- **call_index 稳定性**:`finalize_create` 占用原 `vote_create` 的 `call_index(3)`,其他 index 不动。老客户端发 `vote_create` 调用会解码失败(未知 call_index)——这是**强制切换**,无兼容方案。
- **签名消息字节布局**:与 wuminapp 端对齐铁律:
  - `DUOQIAN_DOMAIN = b"DUOQIAN_V1"` 10 字节 ASCII
  - `OP_SIGN_CREATE = 0x14` 1 字节
  - `SS58_PREFIX_LE`:生产 `2027_u16.to_le_bytes() = [0xEB, 0x07]`;测试 `42_u16.to_le_bytes() = [0x2A, 0x00]`
  - `CreateVoteIntent` SCALE 编码:字段顺序严格按 struct 声明顺序(proposal_id / duoqian_address / creator / admins_root / threshold / amount / approve)
  - admins 排序:字节序(AccountId32 默认 Ord 即字典序)
  - blake2_256 各用于 intent 本身、admins_root 计算、以及最终 signing_hash 两层包裹
- **Provider 不过滤 status 铁律**:runtime `RuntimeInternalAdminProvider::get_admin_list` / `pass_threshold` 必须对 `DuoqianAccounts` 的 `Pending` 和 `Active` 状态都返回数据,否则 Tx 1 创建提案时投票引擎快照抓空。本 runtime 已正确实现此语义。
- **测试固件**:单元测试中 `admin(seed)` 从 `sr25519::Pair::from_seed([seed, 0, ..., 0])` 派生,公钥即为 AccountId32。这保证测试中管理员既能做链上 origin,又能对 `CreateVoteIntent` 产出可验证签名。
4. 如果业务需要“注册型多签机构转账”，不能误以为本模块已经提供，需要另行接入 `duoqian-transfer-pow` 或扩展转账能力。
