# CID System Technical Notes

## 1. 模块定位

`cid-system` 是公民链 runtime 中负责 CID 绑定与投票资格消费的 FRAME pallet。

当前边界：

- 只保存链上账户与 `binding_id` 的当前绑定关系。
- 只维护绑定 nonce 与投票 nonce 防重放状态。
- 不保存 CID 明文。
- 不保存任何特殊管理员名册。
- 不保存签发机构私钥或链下缓存。
- 凭证签发人身份统一由 runtime 验签桥接到 `admins` 分类模块判断。

代码位置：

- `/Users/rhett/GMB/citizenchain/runtime/otherpallet/cid-system/src/lib.rs`
- `/Users/rhett/GMB/citizenchain/runtime/otherpallet/cid-system/src/weights.rs`
- `/Users/rhett/GMB/citizenchain/runtime/otherpallet/cid-system/src/benchmarks.rs`
- `/Users/rhett/GMB/citizenchain/runtime/otherpallet/cid-system/src/tests/`
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`

## 2. 凭证模型

绑定、投票和人口快照的签发身份字段统一为：

- `issuer_cid_number`：签发机构 CID 号。
- `issuer_main_account`：签发机构主账户，也是 runtime 管理员查询门面的查询键。
- `signer_pubkey`：实际签名管理员公钥。
- `scope_province_name`：业务作用域省名称。
- `scope_city_name`：业务作用域市名称。
- `signature`：管理员用 `sr25519` 对业务 payload 摘要签名。

`scope_province_name / scope_city_name` 只表示业务作用域，不表示签发人身份。

runtime 验签规则：

1. 用 `issuer_main_account` 通过 `RuntimeAdminAccountQuery` 读取对应管理员集合。
2. 要求该管理员账户状态为 `Active`。
3. 要求 `signer_pubkey` 对应账户存在于该机构 `admins` 集合。
4. 用 `signer_pubkey` 验证 `sr25519` 签名。

## 3. 核心类型

`BindCredential<AccountId, Hash, Nonce, Signature>` 字段：

- `binding_id`
- `bind_nonce`
- `issuer_cid_number`
- `issuer_main_account`
- `signer_pubkey`
- `scope_province_name`
- `scope_city_name`
- `signature`

runtime 注入接口：

- `CidVerifier`：验证绑定凭证。
- `CidVoteVerifier`：验证投票凭证。
- `OnCidBound`：绑定成功后通知发行模块。
- `CidEligibilityProvider`：给投票模块使用的资格查询和凭证消费接口。

## 4. 存储结构

- `BindingIdToAccount<Hash -> AccountId>`：`binding_id` 到链上账户的正向映射。
- `AccountToBindingId<AccountId -> Hash>`：链上账户到 `binding_id` 的反向映射。
- `BoundCount<u64>`：当前已绑定账户数量。
- `UsedBindNonce<Hash -> bool>`：绑定 nonce 防重放。
- `UsedVoteNonce<(proposal_id, binding_id, nonce_hash) -> bool>`：投票 nonce 防重放。

本 pallet 不再保存独立管理员、注册局管理员、签发公钥或签发人槽位。

## 5. Extrinsic 规则

### 5.1 `bind_cid(origin, credential)`

校验顺序：

1. `origin` 必须是签名账户。
2. `credential.bind_nonce` 非空。
3. `UsedBindNonce[hash(bind_nonce)]` 未使用。
4. `T::CidVerifier::verify(&who, &credential)` 必须通过。
5. `binding_id` 不得已绑定到其他账户。
6. 当前账户不得重复绑定相同 `binding_id`。

状态变更：

1. 若账户此前绑定过旧 `binding_id`，删除旧正向映射。
2. 若账户此前未绑定，`BoundCount += 1`。
3. 写入 `BindingIdToAccount` 和 `AccountToBindingId`。
4. 标记 `UsedBindNonce[hash(bind_nonce)] = true`。
5. 调用 `T::OnCidBound::on_cid_bound(&who, binding_id)`。
6. 发出 `CidBound` 事件。

### 5.2 `unbind_cid(origin, target)`

校验顺序：

1. `origin` 必须满足 runtime 注入的 `T::UnbindOrigin`。
2. `target` 必须当前已绑定。

状态变更：

1. 删除 `target` 的 `AccountToBindingId`。
2. 删除对应 `BindingIdToAccount`。
3. `BoundCount` 饱和减一。
4. 发出 `CidUnbound` 事件。

当前生产配置中 `UnbindOrigin` 由 runtime 决定；本 pallet 不自行判断具体机构管理员。

## 6. 投票资格接口

`is_eligible(binding_id, who)`：

- 只检查 `BindingIdToAccount[binding_id] == who`。

`verify_and_consume_vote_credential(...)`：

1. 检查账户和 `binding_id` 当前绑定关系。
2. 计算 `(proposal_id, binding_id, nonce_hash)`。
3. 拒绝重复 nonce。
4. 调用 `T::CidVoteVerifier::verify_vote(...)`。
5. 验签通过后写入 `UsedVoteNonce`。

`cleanup_vote_credentials(proposal_id)`：

- 清理指定提案维度下的投票 nonce 防重放状态。
- 该接口由投票模块在提案终态后调用，避免长期状态膨胀。

## 7. Runtime 接线

runtime 侧接线在 `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`：

- `RuntimeCidVerifier`：验证 CID 绑定凭证。
- `RuntimeCidVoteVerifier`：验证公民投票凭证。
- `RuntimePopulationSnapshotVerifier`：验证联合提案人口快照凭证。
- `RuntimeCidInstitutionVerifier`：验证机构注册凭证。

以上验签器全部使用相同签发身份规则：

1. 通过 `RuntimeAdminAccountQuery` 读取 `issuer_main_account` 对应的管理员集合。
2. 确认 `signer_pubkey` 属于该机构 `admins`。
3. 按各业务 payload 固定顺序重算 `blake2_256(scale_encode(payload))`。
4. 用 `sr25519_verify(signature, hash, signer_pubkey)` 验签。

## 8. 安全规则

- 凭证 payload 必须包含 `genesis_hash`，防止跨链重放。
- `bind_nonce` 当前永久记账，不做过期回收。
- 投票 nonce 必须由提案生命周期显式清理。
- `issuer_cid_number` 用于审计和展示；链上管理员真值以 `issuer_main_account` 下的 `admins` 为准。
- 所有机构和个人多签管理员集合的唯一真源是 `admins` 分类模块；runtime 统一通过 `RuntimeAdminAccountQuery` 查询。

## 9. 验收要求

本模块改动后必须执行：

- `cargo test --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml -p cid-system`
- `cargo check --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml -p citizenchain`
- 旧签发名册、旧签发字段和旧管理员槽位残留扫描。
