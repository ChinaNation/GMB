# SFID Code Auth Technical Notes

## 0. 功能需求
### 0.1 核心职责
`sfid-system` 的功能需求是：
- 维护 SFID 与链上账户的一对一绑定关系。
- 为公民投票提供基于 SFID 的资格校验与投票凭证验签能力。
- 维护省管理员 3-tier 名册 `ShengAdmins[Province][Slot]`。
- 维护省级签名公钥 `ShengSigningPubkey[Province][AdminPubkey]`。
- 为上游发行/治理模块提供统一、可复用的资格接口，避免多个模块各自保存一份 SFID 真值状态。
- 将链上“是否为已认证公民”的判断收敛到单一真相源，避免链下口径和链上状态漂移。

### 0.2 绑定与解绑需求
- 绑定时不得保存 SFID 明文，只保存 `binding_id`。
- 同一 SFID 同一时刻只能绑定到一个账户。
- 同一账户允许换绑，但旧映射必须原子释放。
- 绑定凭证 nonce 必须防重放（按 `hash(bind_nonce)` 永久记账，无过期回收）。
- 绑定成功后，模块必须能够向上游模块发出“已绑定”回调，但回调模块不得破坏本模块的一对一绑定不变量。
- 解绑仅限管理员（SFID 主账户或省级签名账户）执行，用户不允许自行解绑。解绑只影响”当前绑定关系”，不应隐式清理历史领奖、历史投票或外部业务审计记录。

### 0.3 投票资格需求
- 投票资格校验必须以链上绑定关系为真值。
- 投票凭证摘要算法统一为 `blake2_256(scale_encode(payload))`。
- 投票签名算法统一为 `sr25519`。
- 每个 `(proposal_id, binding_id, nonce_hash)` 只能使用一次。
- 投票 nonce 的生命周期由提案生命周期管理；提案结束后，上游治理模块应显式调用清理接口释放防重放状态。
- 未绑定账户、签名错误、重复 nonce 等资格失败场景应返回 `false`，不得污染治理模块的主交易错误语义。

### 0.4 安全与运维需求
- 模块不得依赖链下“在线状态”或链下缓存来判断公民资格，资格判断必须完全可由链上状态重建。
- 省管理员三槽名册必须保证 Main 为本省 trust anchor,Backup1/Backup2 只能由 Main 签名授权增删。
- 绑定凭证与投票凭证必须带链域隔离信息（`genesis_hash`），防止跨链重放。
- 绑定 nonce（`UsedBindNonce`）当前为永久存储，无回收机制。以 `CITIZEN_ISSUANCE_MAX_COUNT` 为上界，存储增长有限。
- 投票 nonce 当前没有按块自动回收机制，运维流程必须在提案结束时通过 `cleanup_vote_credentials` 触发清理，否则会产生长期状态膨胀。当前清理实现使用 `clear_prefix(u32::MAX)` 一次性清除，如果单提案投票量极大，需考虑分批清理。

### 0.5 Runtime 对齐基线（冻结）
1. 以链上 Runtime 为唯一验签真值。
2. 功能 1/2/3 的摘要算法统一为 `blake2_256(scale_encode(payload))`，签名算法统一为 `sr25519`。
3. Runtime 绑定点（代码锚点）：
   - 绑定：`runtime/src/configs/mod.rs:676`
   - 投票：`runtime/src/configs/mod.rs:720`
   - 人口快照：`runtime/src/configs/mod.rs:780`
4. 绑定 nonce 防重放按 `hash(nonce)` 消费（代码锚点：`runtime/otherpallet/sfid-system/src/lib.rs:294`）。

## 1. 模块定位
`sfid-system` 是一个 FRAME pallet，负责四件核心事：
- SFID 与链上账户的一对一绑定/解绑。
- 公民投票资格校验（基于 SFID 绑定关系 + SFID 系统签名凭证）。
- 维护省管理员三槽名册。
- 维护每个省管理员槽独立的签名公钥。

设计边界：
- 本模块不保存 SFID 明文，只保存 `binding_id`。
- 本模块不保存任何私钥，链上只保存账户公钥（`AccountId`）。
- 绑定成功后的奖励发行不在本模块实现，而是通过回调给上游模块处理。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/otherpallet/sfid-system/src/lib.rs`
- `/Users/rhett/GMB/citizenchain/runtime/otherpallet/sfid-system/src/duoqian_info/`
- `/Users/rhett/GMB/citizenchain/runtime/otherpallet/sfid-system/src/sheng_admins/`

### 1.1 `duoqian_info/` 目录边界(2026-05-02)

`duoqian_info/` 是 DUOQIAN 链接收 SFID 机构信息的链上落点,先承接第 1 步机构备案的类型和基础校验。

当前目录结构:

```text
duoqian_info/
├── mod.rs       # 模块聚合与边界说明
├── types.rs     # InstitutionFilingPayload / InstitutionFilingRecord
├── validate.rs  # 备案三字段非空校验
├── filing.rs    # 备案记录与 payload 对比辅助
└── tests.rs     # 基础单测
```

边界:

- 备案 payload 只包含 `sfid_id`、`institution_name`、`account_name`。
- 备案记录不能写入 `duoqian-manage` 的正式机构 storage。
- 备案记录不能激活机构账户。
- 正式多签机构注册仍由后续 `duoqian-manage` 流程完成。

---

### 1.2 `sheng_admins/` 目录边界(2026-05-02)

`sheng_admins/` 是 `sfid-system` pallet 内省管理员相关纯类型与辅助逻辑目录,
与 SFID 后端 `sfid/backend/src/chain/sheng_admins/`、前端
`sfid/frontend/chain/sheng_admins/` 同名对齐。FRAME storage 与 call 壳仍在
`lib.rs`,纯类型、payload 与迁移提示下沉到本目录。

当前目录结构:

```text
sheng_admins/
├── mod.rs        # 模块聚合与边界说明
├── types.rs      # Slot { Main, Backup1, Backup2 }
├── payload.rs    # 4 个省管理员 unsigned extrinsic 的 domain 与 payload hash
└── migration.rs  # ADR-008 历史 storage 清理提示
```

边界:

- `types.rs` 承载链上 `ShengAdmins[Province][Slot]` 的 Slot 类型。
- `payload.rs` 的 domain 常量和字段顺序必须与 SFID 后端
  `sfid/backend/src/chain/sheng_admins/` 保持完全一致。
- `lib.rs` 继续保留 storage、event、error、call 与 ValidateUnsigned,避免破坏
  FRAME pallet 宏边界。

---

## 2. Runtime 接线位置
Runtime 配置与验签桥接：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`

关键接线：
- `type SfidVerifier = RuntimeSfidVerifier`
- `type SfidVoteVerifier = RuntimeSfidVoteVerifier`
- `type OnSfidBound = CitizenIssuance`

说明：
- `bind_sfid` 成功后会触发 `OnSfidBound::on_sfid_bound(...)`，由奖励模块处理发放逻辑。
- `RuntimeSfidVerifier` / `RuntimeSfidVoteVerifier` 使用当前 SFID 主账户公钥做 `sr25519` 验签。

---

## 3. 核心类型与存储

### 3.1 核心类型
- `BindCredential { binding_id, bind_nonce, signature }`
- `NonceOf<T> = BoundedVec<u8, MaxCredentialNonceLength>`
- `SignatureOf<T> = BoundedVec<u8, MaxCredentialSignatureLength>`

### 3.2 存储结构
- `BindingIdToAccount<Hash -> AccountId>`
  - `binding_id` 到账户的正向映射。
- `AccountToBindingId<AccountId -> Hash>`
  - 账户到 `binding_id` 的反向映射。
- `BoundCount<u64>`
  - 当前已绑定账户数量（可作为公民投票基数参考）。
- `UsedBindNonce<Hash -> bool>`
  - 绑定凭证 `bind_nonce` 防重放（按 `hash(bind_nonce)` 记账）。
- `UsedVoteNonce<(proposal_id, binding_id, nonce_hash) -> bool>`
  - 投票凭证防重放（提案 + 身份 + nonce 三维度）。
- `ShengAdmins<Province, Slot -> [u8; 32]>`
  - 省管理员三槽名册。Main 首激活占位,Backup1/Backup2 由 Main 签名增删。
- `ShengSigningPubkey<Province, AdminPubkey -> [u8; 32]>`
  - 每个省管理员槽独立的业务签名公钥。
- `UsedShengNonce<Hash -> ()>`
  - 4 个省管理员 unsigned extrinsic 的 32 字节 nonce 防重放。

---

## 4. 创世配置与密钥模型
ADR-008 后 `sfid-system` 不再通过 GenesisConfig 注入旧全局管理员 / SFID 主备账户。
省管理员链上状态采用 first-come-first-serve:

1. 某省首次调用 `activate_sheng_signing_pubkey` 的 `admin_pubkey` 占据
   `ShengAdmins[Province][Main]`。
2. Backup1 / Backup2 后续只能由该省 Main 对 payload 签名授权后写入。
3. 每个 admin slot 单独写入 `ShengSigningPubkey[Province][AdminPubkey]`。

---

## 5. Extrinsic 规则

### 5.1 `bind_sfid(origin, credential)`（call index = 0）
校验顺序：
1. `origin` 必须是签名账户。
2. `credential.bind_nonce` 非空（`EmptyBindNonce`）。
3. `UsedBindNonce[hash(bind_nonce)]` 不得已使用（`BindNonceAlreadyUsed`）。
4. `T::SfidVerifier::verify(&who, &credential)` 必须通过（`InvalidSfidBindingSignature`）。
5. 若 `binding_id` 已绑定他人，拒绝（`BindingIdAlreadyBoundToAnotherAccount`）。
6. 若 `binding_id` 已绑定当前账户，拒绝（`SameBindingIdAlreadyBound`）。

状态变更：
1. 若账户之前已绑旧 binding_id：移除旧正向映射（允许换绑），不减少 `BoundCount`。
2. 若账户此前未绑定：`BoundCount += 1`。
3. 写入新双向映射（`BindingIdToAccount` + `AccountToBindingId`）。
4. 标记 `UsedBindNonce[hash(bind_nonce)] = true`（永久存储，无过期）。
5. 触发 `T::OnSfidBound::on_sfid_bound(&who, binding_id)` 回调。
6. 发事件 `SfidBound { who, binding_id, bind_nonce_hash }`。

weight：
- `T::WeightInfo::bind_sfid().saturating_add(T::OnSfidBound::on_sfid_bound_weight())`
- `src/weights.rs` 当前仍是旧代码路径生成的产物，proof 注释仍引用已删除的旧存储名（如 `UsedCredentialNonce`、`SfidToAccount`、`AccountToSfid`、`CredentialNoncesByExpiry`）。
- 当前文件只能视为待重生的历史 benchmark 结果，不能当作完全贴合现状的存储访问说明。

### 5.2 `unbind_sfid(origin, target)`（call index = 1）— 管理员代解绑
校验：
1. `origin` 必须满足 runtime 注入的 `T::UnbindOrigin`。
2. `target` 必须当前已绑定（`NotBound`）。

状态变更：
1. 删除 `target` 的 `AccountToBindingId` 与 `BindingIdToAccount`。
2. `BoundCount -= 1`（`saturating_sub`）。
3. 发事件 `SfidUnbound { admin, who, binding_id }`。

权限说明：用户不允许自行解绑，必须由 SFID 管理员发起。

### 5.3 省管理员 4 个 unsigned extrinsic（call index = 2..5）

| call | 说明 | 验签口径 |
|---|---|---|
| `add_sheng_admin_backup` | 写入 Backup1 / Backup2 槽 | Main 对 `ADD_BACKUP_DOMAIN + province + slot + new_pubkey + nonce` 签名 |
| `remove_sheng_admin_backup` | 清空 Backup1 / Backup2 槽,并级联清签名公钥 | Main 对 `REMOVE_BACKUP_DOMAIN + province + slot + nonce` 签名 |
| `activate_sheng_signing_pubkey` | 首激活 Main 或激活在册 admin 的签名公钥 | admin_pubkey 对 `ACTIVATE_DOMAIN + province + admin_pubkey + signing_pubkey + nonce` 签名 |
| `rotate_sheng_signing_pubkey` | 轮换在册 admin 的签名公钥 | admin_pubkey 对 `ROTATE_DOMAIN + province + admin_pubkey + new_signing_pubkey + nonce` 签名 |

这 4 个调用均为 unsigned + `Pays::No`,防重放统一走 `UsedShengNonce`。
payload 代码锚点:`src/sheng_admins/payload.rs`。

---

## 6. 投票资格接口（内部接口）
本模块实现 `SfidEligibilityProvider<AccountId, Hash>`，供投票模块调用。

### 6.1 `is_eligible(binding_id, who)`
- 直接使用 `binding_id` 检查其是否绑定到 `who`。

### 6.2 `verify_and_consume_vote_credential(binding_id, who, proposal_id, nonce, signature)`
逻辑：
1. `nonce` / `signature` 非空。
2. `binding_id` 必须已绑定到 `who`。
3. `(proposal_id, binding_id, nonce_hash)` 未被使用。
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
- payload: `(DUOQIAN_DOMAIN, OP_SIGN_BIND, genesis_hash, who, binding_id, bind_nonce)`
- `genesis_hash = block_hash(0)`（链域隔离）
- message: `blake2_256(scale_encode(payload))`
- algorithm: `sr25519`

### 7.2 公民投票凭证域
`RuntimeSfidVoteVerifier` 的 payload：
- payload: `(DUOQIAN_DOMAIN, OP_SIGN_VOTE, genesis_hash, who, binding_id, proposal_id, vote_nonce)`
- `genesis_hash = block_hash(0)`（链域隔离）
- message: `blake2_256(scale_encode(payload))`
- algorithm: `sr25519`

### 7.3 人口快照凭证域（同一信任根）
该逻辑位于 `voting-engine` 的 snapshot verifier：
- payload: `(DUOQIAN_DOMAIN, OP_SIGN_POP, genesis_hash, who, eligible_total, snapshot_nonce)`
- `genesis_hash = block_hash(0)`（链域隔离）
- message: `blake2_256(scale_encode(payload))`
- algorithm: `sr25519`

说明：
- 7.3 不在 `sfid-system` pallet 内，但使用同一 SFID 主验签公钥体系。
- `who(account)` 必须参与签名；`voters/count` 不能只签 `eligible_total`。

---

## 8. SFID 系统对区块链提供的数据（按 5 大功能）
以下是 Runtime 对齐口径下，区块链侧需要 SFID 系统提供或配合的 5 类数据/能力。

### 功能 1：SFID 绑定
需要提供：
1. 固定签名域：`(DUOQIAN_DOMAIN, OP_SIGN_BIND, genesis_hash, who, binding_id, bind_nonce)`。
2. 链上消费字段：`binding_id`、`bind_nonce`、`signature`。
3. `bind_nonce` 一次性；链上按 `hash(bind_nonce)` 去重。
4. SFID 可保留扩展运维字段（如 `key_id`、`key_version`、`alg`），但不改变链上验签字段。
5. 链上交易签名为 `bind_sfid(origin, credential)`，credential 封装 `binding_id` + `bind_nonce` + `signature`。

### 功能 2：公民投票凭证校验
需要提供：
1. 固定签名域：`(DUOQIAN_DOMAIN, OP_SIGN_VOTE, genesis_hash, who, binding_id, proposal_id, vote_nonce)`。
2. SFID 输出字段：`binding_id`、`proposal_id`、`vote_nonce`、`signature`。
3. 防重放键：`(proposal_id, binding_id, hash(vote_nonce))`。
4. `vote_nonce` 每次新生成，不复用。

### 功能 3：人口快照签名
需要提供：
1. 固定签名域：`(DUOQIAN_DOMAIN, OP_SIGN_POP, genesis_hash, who, eligible_total, snapshot_nonce)`。
2. SFID 输出字段：`eligible_total`、`snapshot_nonce`、`signature`。
3. 提交者账户 `who`（治理发起者链上账户）必须进入签名 payload。

### 功能 4：机构 SFID 登记（多签模块）
需要提供：
1. `sfid_id`
2. 由省级签名公钥签发的机构注册凭证

说明：
- 当前实现不校验“sfid_id 哈希与链下回传是否一致”这类二次证明；
- 当前是“链上唯一性 + 省级签名凭证 + 派生地址”模型。

### 功能 5：省管理员名册与签名密钥运维
需要提供：
1. `activate_sheng_signing_pubkey` 首激活 Main 或激活在册 admin 签名公钥。
2. `add_sheng_admin_backup` / `remove_sheng_admin_backup` 由 Main 授权维护 Backup1/Backup2。
3. `rotate_sheng_signing_pubkey` 由当前 admin 自签轮换自身签名公钥。
4. 4 个调用均走 unsigned + `Pays::No` + `UsedShengNonce` 防重放。

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
- 一对一绑定：`BindingIdToAccount` + `AccountToBindingId` 双向约束。
- 防重放：
  - 绑定：`UsedBindNonce(hash(bind_nonce))`（永久存储）
  - 投票：`UsedVoteNonce(proposal_id, (binding_id, hash(vote_nonce)))`（提案结束后可清理）
- 链域隔离：payload 包含 `block_hash(0)`。
- 域隔离：绑定/投票/快照使用不同 domain 常量。
- 省级签名根可轮换：每个省管理员槽独立签名公钥,通过 `rotate_sheng_signing_pubkey` 更新。

注意：
- `cleanup_vote_credentials` 当前使用 `clear_prefix(u32::MAX)` 一次性清除，如果单提案投票量极大，可能影响出块稳定性。

---

## 11. 事件与错误码
事件：
- `SfidBound { who, binding_id, bind_nonce_hash }`
- `SfidUnbound { who, binding_id }`
- `SfidKeysRotated { operator, new_main, backup_1, backup_2 }`

错误码：
- `EmptyBindNonce`：绑定凭证中 bind_nonce 为空。
- `BindNonceAlreadyUsed`：该 bind_nonce 已被使用（防重放）。
- `InvalidSfidBindingSignature`：SFID 绑定签名验证失败。
- `BindingIdAlreadyBoundToAnotherAccount`：该 binding_id 已被另一个账户绑定。
- `SameBindingIdAlreadyBound`：该账户已绑定到同一 binding_id。
- `NotBound`：账户当前未绑定 SFID。
- `UnauthorizedSfidOperator`：调用者不是备用账户，无权轮换。
- `DuplicateSfidKey`：新备用账户与现有账户重复。

---

## 12. 测试覆盖（当前）

### 12.1 单元测试
`cargo test -p sfid-system` 覆盖（17 个用例）：
- 绑定成功与 `BoundCount` 计数
- 绑定 nonce 防重放
- 同账户换绑不增加 `BoundCount`
- 投票资格判断与 vote nonce 防重放
- `current_sfid_verify_pubkey` 编码长度边界
- 备用账户轮换成功路径（backup_1 和 backup_2 各一条）
- 空 bind_nonce 拒绝（`EmptyBindNonce`）
- 签名验证失败拒绝（`InvalidSfidBindingSignature`）
- binding_id 已绑他人拒绝（`BindingIdAlreadyBoundToAnotherAccount`）
- 同 binding_id 重复绑定拒绝（`SameBindingIdAlreadyBound`）
- 未绑定账户解绑拒绝（`NotBound`）
- 主账户无权轮换拒绝（`UnauthorizedSfidOperator`）
- 新备用账户重复拒绝（`DuplicateSfidKey`，三条路径）
- `cleanup_vote_credentials` 清理后 nonce 可复用

### 12.2 跨模块集成测试
`cargo test -p citizen-issuance --test integration_bind_sfid` 覆盖：
- `bind_sfid` → `OnSfidBound` → 奖励发放完整链路
- 换绑时奖励跳过但绑定成功
- 不同账户独立领奖
- 达到上限后绑定成功但奖励跳过

---

## 13. 联调检查清单（给 SFID 系统）
1. 确认三把 SFID 账户已在创世或链上初始化完成。
2. 绑定/投票/快照都使用对应 domain 常量，不可混用。
3. 每次签名使用新 nonce，避免被链上防重放拒绝。
4. 绑定签名 payload 中 `account` 必须是实际发交易账户。
5. 投票签名 payload 中 `proposal_id` 必须与链上提案一致。
6. 机构登记由 SFID 当前 `MAIN` 发起，并只提交 `sfid_id`。
