# SFID Code Auth Technical Notes

## 1. 模块定位
`sfid-code-auth` 是一个 FRAME pallet，负责三件核心事：
- SFID 与链上账户的一对一绑定/解绑。
- 公民投票资格校验（基于 SFID 绑定关系 + SFID 系统签名凭证）。
- 维护 SFID 验签主备账户（主账户验签、备用账户轮换）。

设计边界：
- 本模块不保存 SFID 明文，只保存 `sfid_hash`。
- 本模块不保存任何私钥，链上只保存账户公钥（`AccountId`）。
- 绑定成功后的奖励发行不在本模块实现，而是通过回调给上游模块处理。

代码位置：
- `/Users/rhett/GMB/citizenchain/otherpallet/sfid-code-auth/src/lib.rs`

---

## 2. Runtime 接线位置
Runtime 配置与验签桥接：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`

关键接线：
- `type SfidVerifier = RuntimeSfidVerifier`
- `type SfidVoteVerifier = RuntimeSfidVoteVerifier`
- `type OnSfidBound = CitizenLightnodeIssuance`

说明：
- `bind_sfid` 成功后会触发 `OnSfidBound::on_sfid_bound(...)`，由奖励模块处理发放逻辑。
- `RuntimeSfidVerifier` / `RuntimeSfidVoteVerifier` 使用当前 SFID 主账户公钥做 `sr25519` 验签。

---

## 3. 核心类型与存储

### 3.1 核心类型
- `BindCredential { sfid_code_hash, nonce, signature }`
- `SfidOf<T> = BoundedVec<u8, MaxSfidLength>`
- `NonceOf<T> = BoundedVec<u8, MaxCredentialNonceLength>`
- `SignatureOf<T> = BoundedVec<u8, MaxCredentialSignatureLength>`

### 3.2 存储结构
- `SfidToAccount<Hash -> AccountId>`
  - SFID 哈希到账户的正向映射。
- `AccountToSfid<AccountId -> Hash>`
  - 账户到 SFID 哈希的反向映射。
- `BoundCount<u64>`
  - 当前已绑定账户数量（可作为公民投票基数参考）。
- `UsedCredentialNonce<Hash -> bool>`
  - 绑定凭证 nonce 防重放（按 `hash(nonce)` 记账）。
- `UsedVoteNonce<(proposal_id, sfid_hash, nonce_hash) -> bool>`
  - 投票凭证防重放（提案 + 身份 + nonce 三维度）。
- `SfidMainAccount<Option<AccountId>>`
  - 当前 SFID 主验签账户。
- `SfidBackupAccount1<Option<AccountId>>`
- `SfidBackupAccount2<Option<AccountId>>`

---

## 4. 创世配置与密钥模型
`GenesisConfig` 包含三把 SFID 账户：
- `sfid_main_account`
- `sfid_backup_account_1`
- `sfid_backup_account_2`

规则：
1. 三个都不配置：允许（no-op）。
2. 只要配置了任意一个：必须三把都配置。
3. 三把账户必须两两不同。

含义：
- 主账户：当前验签公钥来源。
- 两个备用账户：可发起轮换，把自己提升为主账户并补位新备用账户。

---

## 5. Extrinsic 规则

### 5.1 `bind_sfid(origin, sfid_code, credential)`（call index = 0）
校验顺序：
1. `origin` 必须是签名账户。
2. `sfid_code` 非空。
3. `credential.nonce` 非空。
4. `hash(sfid_code)` 必须等于 `credential.sfid_code_hash`。
5. `UsedCredentialNonce[hash(nonce)]` 不得已使用。
6. `T::SfidVerifier::verify(&who, &credential)` 必须通过。
7. 若 `sfid_hash` 已绑定他人，拒绝。
8. 若 `sfid_hash` 已绑定当前账户，拒绝（`SameSfidAlreadyBound`）。

状态变更：
1. 若账户之前已绑旧 SFID：移除旧正向映射（允许换绑）。
2. 若账户此前未绑定：`BoundCount += 1`。
3. 写入新双向映射。
4. 标记 `UsedCredentialNonce` 已使用。
5. 触发 `OnSfidBound` 回调。
6. 发事件 `SfidBound { who, sfid_hash, credential_nonce_hash }`。

weight：
- `DbWeight.reads_writes(6, 6) + OnSfidBound::on_sfid_bound_weight()`

### 5.2 `unbind_sfid(origin)`（call index = 1）
校验：
1. `origin` 必须是签名账户。
2. 账户必须当前已绑定 SFID。

状态变更：
1. 删除 `AccountToSfid` 与 `SfidToAccount`。
2. `BoundCount -= 1`（`saturating_sub`）。
3. 发事件 `SfidUnbound`。

### 5.3 `rotate_sfid_keys(origin, new_backup)`（call index = 2）
校验：
1. 三把当前 SFID 账户都必须已配置。
2. 调用者必须是 `backup_1` 或 `backup_2`（主账户不能直接调用）。
3. `new_backup` 不能与 `main` / 调用者 / 幸存备用账户重复。

轮换规则：
1. 调用者升级为新 `main`。
2. 另一个备用账户成为新 `backup_1`。
3. `new_backup` 成为新 `backup_2`。
4. 发事件 `SfidKeysRotated`。

---

## 6. 投票资格接口（内部接口）
本模块实现 `SfidEligibilityProvider<AccountId>`，供投票模块调用。

### 6.1 `is_eligible(sfid, who)`
- 计算 `sfid_hash`，检查其是否绑定到 `who`。

### 6.2 `verify_and_consume_vote_credential(sfid, who, proposal_id, nonce, signature)`
逻辑：
1. `nonce` / `signature` 非空。
2. `sfid` 必须已绑定到 `who`。
3. `(proposal_id, sfid_hash, nonce_hash)` 未被使用。
4. `nonce` / `signature` 长度必须可转为对应 `BoundedVec`。
5. `T::SfidVoteVerifier::verify_vote(...)` 必须通过。
6. 成功后写入 `UsedVoteNonce`，并返回 `true`。

返回值语义：
- 任一校验失败都返回 `false`（不抛 dispatch 错误，因为这是内部资格接口）。

---

## 7. 验签 payload 约定（Runtime 实现）
以下约定由 Runtime 中的 verifier 实现定义：

### 7.1 绑定凭证域
`RuntimeSfidVerifier` 的 payload：
- domain: `GMB_SFID_BIND_V1`
- chain: `block_hash(0)`（链域隔离）
- fields: `account`, `sfid_code_hash`, `nonce`
- message: `blake2_256(scale_encode(payload))`
- algorithm: `sr25519`

### 7.2 公民投票凭证域
`RuntimeSfidVoteVerifier` 的 payload：
- domain: `GMB_SFID_VOTE_V1`
- chain: `block_hash(0)`
- fields: `account`, `sfid_hash`, `proposal_id`, `nonce`
- message: `blake2_256(scale_encode(payload))`
- algorithm: `sr25519`

### 7.3 人口快照凭证域（同一信任根）
该逻辑位于 `voting-engine-system` 的 snapshot verifier：
- domain: `GMB_SFID_POPULATION_V1`
- chain: `block_hash(0)`
- fields: `who`, `eligible_total`, `nonce`
- message: `blake2_256(scale_encode(payload))`
- algorithm: `sr25519`

说明：
- 7.3 不在 `sfid-code-auth` pallet 内，但使用同一 SFID 主验签公钥体系。

---

## 8. SFID 系统对区块链提供的数据（按 5 大功能）
以下是当前实现下，区块链侧需要 SFID 系统提供或配合的 5 类数据/能力。

### 功能 1：SFID 绑定
需要提供：
1. `sfid_code`（明文，仅用于本次上链，链上最终只存 hash）
2. `credential.sfid_code_hash`
3. `credential.nonce`（一次性）
4. `credential.signature`（绑定域签名）

### 功能 2：公民投票凭证校验
需要提供：
1. `sfid`（投票人 SFID）
2. `proposal_id`
3. `vote_nonce`（一次性）
4. `vote_signature`（投票域签名）

### 功能 3：人口快照签名
需要提供：
1. `eligible_total`
2. `snapshot_nonce`（一次性）
3. `snapshot_signature`（快照域签名）
4. 提交者账户 `who`（由治理发起者签名交易，进入 payload）

### 功能 4：机构 SFID 登记（多签模块）
需要提供：
1. `sfid_id`
2. 由 SFID 授权账户发起上链（当前由 SFID 主/备账户权限控制）

说明：
- 当前实现不校验“sfid_id 哈希与链下回传是否一致”这类二次证明；
- 当前是“链上唯一性 + 授权账户 + 派生地址”模型。

### 功能 5：SFID 验签密钥运维（主备轮换）
需要提供：
1. 创世阶段三把账户（主 + 备1 + 备2）
2. 轮换时由备用账户发起 `rotate_sfid_keys`
3. 新补位备用账户 `new_backup`

---

## 9. 私钥与链上数据边界
链上不需要、也不应存储私钥。

链上存储的是：
- SFID 账户公钥形式的 `AccountId`（主/备）。
- 各类签名结果（`signature`）与 nonce 的哈希防重放标记。

私钥只应存在于：
- SFID 系统离线/受控签名环境；
- 节点或业务系统的安全密钥托管设施。

---

## 10. 安全属性与注意事项
- 一对一绑定：`SfidToAccount` + `AccountToSfid` 双向约束。
- 防重放：
  - 绑定：`UsedCredentialNonce(hash(nonce))`
  - 投票：`UsedVoteNonce(proposal_id, sfid_hash, hash(nonce))`
- 链域隔离：payload 包含 `block_hash(0)`。
- 域隔离：绑定/投票/快照使用不同 domain 常量。
- 可轮换验签根：主备账户机制降低单点密钥风险。

注意：
- `current_sfid_verify_pubkey()` 要求 `AccountId` 编码长度恰好 32 字节，否则验签会失败。

---

## 11. 事件与错误码
事件：
- `SfidBound`
- `SfidUnbound`
- `SfidKeysRotated`

错误码：
- `EmptySfid`
- `EmptyCredentialNonce`
- `InvalidCredentialSfidCodeHash`
- `CredentialAlreadyUsed`
- `InvalidSfidCredentialSignature`
- `SfidAlreadyBoundToAnotherAccount`
- `SameSfidAlreadyBound`
- `NotBound`
- `UnauthorizedSfidOperator`
- `DuplicateSfidKey`

---

## 12. 测试覆盖（当前）
`sfid-code-auth` 模块单测已覆盖：
- 绑定成功与 `BoundCount` 计数
- 绑定 nonce 防重放
- 同 SFID 不能绑定给不同账户
- 同账户换绑 SFID 不增加 `BoundCount`
- 同账户重复绑定同 SFID 拒绝
- 解绑前置条件与计数回退
- 备用账户轮换成功路径
- 轮换权限与重复 key 拒绝路径
- 投票资格判断与 vote nonce 防重放
- 绑定参数与签名错误路径
- `current_sfid_verify_pubkey` 编码长度边界

---

## 13. 联调检查清单（给 SFID 系统）
1. 确认三把 SFID 账户已在创世或链上初始化完成。
2. 绑定/投票/快照都使用对应 domain 常量，不可混用。
3. 每次签名使用新 nonce，避免被链上防重放拒绝。
4. 绑定签名 payload 中 `account` 必须是实际发交易账户。
5. 投票签名 payload 中 `proposal_id` 必须与链上提案一致。
6. 机构登记由 SFID 授权账户发起，并只提交 `sfid_id`。
