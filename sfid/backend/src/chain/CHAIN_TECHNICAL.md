# CHAIN_TECHNICAL

## 0. 区块链端方案对齐（冻结，优先级最高）
1. 本文档第 0 步严格按《SFID-Chain 五项能力对齐技术方案（Runtime 对齐版）》执行。
2. 功能 1/2/3 统一按 Runtime 固定 payload、统一摘要算法 `blake2_256(scale_encode(payload))`、统一签名算法 `sr25519`。
3. 功能 1/2/3 的 nonce 防重放分别按 Runtime 对应键消费，任何重复 nonce 必须被链上拒绝。
4. 若本文件其余章节与本节冲突，以本节为准。

## 1. 模块目标
1. `chain` 模块负责 SFID 面向区块链的凭证与查询接口。
2. 本模块聚焦功能 1/2/3（绑定、投票、人口快照）的 Runtime 对齐，不承载机构治理与密钥治理后台流程。

## 2. Runtime 对齐基线（冻结）
1. 以链上 Runtime 为唯一验签真值。
2. 功能 1/2/3 摘要算法统一为 `blake2_256(scale_encode(payload))`，签名算法统一为 `sr25519`。
3. Runtime 代码锚点：
   - 绑定 payload：`citizenchain/runtime/src/configs/mod.rs:676`
   - 投票 payload：`citizenchain/runtime/src/configs/mod.rs:720`
   - 人口快照 payload：`citizenchain/runtime/src/configs/mod.rs:780`
4. 防重放锚点：
   - 绑定：`citizenchain/otherpallet/sfid-code-auth/src/lib.rs:294`（`hash(nonce)`）
   - 投票：`citizenchain/otherpallet/sfid-code-auth/src/lib.rs:434`（`proposal_id + sfid_hash + hash(vote_nonce)`）

## 3. API 矩阵（模块内）
1. `POST /api/v1/bind/request`
2. `GET /api/v1/bind/result`
   - 绑定成功时返回：`sfid_code_hash`、`nonce`、`expires_at_block`、`signature`（并可附带 `key_id/key_version/alg`）。
   - `sfid_signature` 为历史兼容字段，保持返回旧语义绑定证明签名（JSON payload 签名），不等同于 Runtime `signature`。
   - 同一 `account_pubkey` 在已绑定状态下，若凭证未过期则返回同一份持久化凭证；过期后自动重签发新凭证。
3. `POST /api/v1/vote/verify`
   - `proposal_id` 必填；返回：`sfid_hash`、`proposal_id`、`vote_nonce`、`signature`（并可附带 `key_id/key_version/alg`）。
   - 隐私约束：不返回 `sfid_code` 明文。
4. `GET /api/v1/chain/voters/count?account_pubkey=<who>`
   - `who(account)` 必填（兼容别名 `who`），返回：`eligible_total`、`snapshot_nonce`、`snapshot_signature`（兼容 `snapshot_attestation`）。
5. `POST /api/v1/chain/binding/validate`
6. `POST /api/v1/chain/reward/ack`
7. `GET /api/v1/chain/reward/state`
8. `GET /api/v1/attestor/public-key`

## 4. 功能 1/2/3 对齐契约（Runtime 口径）

### 4.1 功能 1：SFID 绑定验签
1. 固定 payload：`("GMB_SFID_BIND_V2", genesis_hash, who, sfid_code_hash, nonce, expires_at_block)`。
2. `genesis_hash` 对应 Runtime `block_hash(0)`。
3. SFID 对链输出字段（链上消费）：`sfid_code_hash`、`nonce`、`expires_at_block`、`signature`。
4. 链上交易参数保持：`bind_sfid(who, sfid_code, credential)`；其中 Runtime 负责校验 `hash(sfid_code) == sfid_code_hash`。
5. `nonce` 必须一次性，链上按 `hash(nonce)` 去重并在过期后自动清理。
6. SFID 在绑定确认时生成并持久化运行时绑定凭证，`bind_result` 仅回传已持久化凭证，避免重复查询产生不同 `nonce`。
7. 若当前 signer 公钥或 `key_id/key_version/alg` 与已持久化凭证不一致，SFID 必须重新签发 Runtime 绑定凭证并覆盖旧值。
8. SFID 可返回扩展运维字段（`key_id`、`key_version`、`alg`），但不得改变链上验签字段。
9. `bind_result.signature` 为 Runtime 凭证签名；`bind_result.sfid_signature` 为历史兼容字段，保持旧 JSON 绑定证明语义。

### 4.2 功能 2：投票凭证验签与防重放
1. 固定 payload：`("GMB_SFID_VOTE_V2", genesis_hash, who, sfid_hash, proposal_id, vote_nonce)`。
2. SFID 对链输出字段：`sfid_hash`、`proposal_id`、`vote_nonce`、`signature`。
3. 链上防重放键固定为：`(proposal_id, sfid_hash, hash(vote_nonce))`。
4. `vote_nonce` 每次新生成，严禁复用。

### 4.3 功能 3：人口快照签名
1. 固定 payload：`("GMB_SFID_POPULATION_V2", genesis_hash, who, eligible_total, snapshot_nonce)`。
2. `voters/count` 必须接收 `who(account)` 并进入签名 payload，不能仅对 `eligible_total` 签名。
3. SFID 对链输出字段：`eligible_total`、`snapshot_nonce`、`snapshot_signature`。
4. 兼容口径：可临时并行返回 `snapshot_attestation`（含 `key_id/key_version/alg/payload/signature_hex`），并在文档中标注 `snapshot_signature` 为过渡期保留字段。
5. `snapshot_attestation.domain` 与签名 payload domain 必须使用同一来源常量，避免字符串字面量漂移。
6. `as_of` 必须与 `eligible_total` 同一统计快照生成（同一次读锁窗口）。

### 4.4 运行时环境约束
1. Runtime 签名所需 `genesis_hash` 来自环境变量 `SFID_CHAIN_GENESIS_HASH`（32 字节 hex，对应链上 `block_hash(0)`）。
2. 签名密钥缓存中的 seed 必须使用可清零敏感类型存储（`SensitiveSeed`），禁止以普通 `String` 持有。

## 5. 功能 4/5 模块边界（SFID 系统内）

### 5.1 功能 4：机构 `sfid_id` 登记
1. 归属 `super-admins` 模块：负责机构主数据与审批。
2. 标准流程：`SFID 审批完成 -> 授权 Origin 发链上登记交易 -> 回写 tx_hash/block_number`。
3. `sfid_id` 格式（长度、字符集、大小写）由 SFID 与链侧双端校验。

### 5.2 功能 5：验签密钥运维（主备轮换）
1. 归属 `key-admins` 模块：负责密钥治理、轮换流程、审计记录。
2. 链侧动作以标准 extrinsic 为准（如 `rotate_sfid_keys`），不依赖私有 RPC 方法。
3. 轮换策略：先写链上 backup，再提升为 main，再下发新 backup；全程记录审计事件与版本号。
4. `rotate/commit` 必须串行执行，防止并发提交导致本地 keyring 状态回写覆盖。
5. 上链成功后本地写入前必须重检 keyring `version`；若版本已变化，返回冲突并禁止本地覆盖。
6. `wait_for_finalized` 必须设置超时（默认 `90` 秒，可配置），避免链终局停滞导致服务请求无限挂起。

## 6. 实施步骤（文档与接口同步）
1. 冻结协议字段顺序与 domain 常量，输出签名向量文档（输入、哈希、签名样例）。
2. 改造 SFID 后端功能 1/2/3 接口 payload 结构，严格对齐 Runtime。
3. 改造链侧调用参数组装，确保 `who`、`sfid_hash/sfid_code_hash`、`nonce` 类型一致。
4. 完成本地链 + SFID 集成联调，覆盖重放攻击与密钥轮换回归。
5. 清理不一致口径：同步更新 `SFIDCODEAUTH_TECHNICAL.md`、`CHAIN_TECHNICAL.md`、`SUPER_ADMINS_TECHNICAL.md`、`KEY_ADMINS_TECHNICAL.md`。

## 7. 验收标准
1. 功能 1/2/3 链上验签通过率 `100%`，摘要算法全部为 `blake2_256`。
2. 任意重复 nonce 必须被链上拒绝。
3. 密钥轮换后新签名可验，旧签名按策略失效或按窗口兼容。
4. 文档字段、代码字段、链上 verifier 三者一致。

## 8. 与其他模块边界
1. 机构登记能力见：`backend/src/super-admins/SUPER_ADMINS_TECHNICAL.md`。
2. 密钥轮换能力见：`backend/src/key-admins/KEY_ADMINS_TECHNICAL.md`。
3. 本模块仅提供链路接口与凭证输出，不负责后台治理审批。
