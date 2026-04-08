# CHAIN_TECHNICAL

## 0. 区块链端方案对齐（冻结，优先级最高）
1. 本文档第 0 步严格按《SFID-Chain 五项能力对齐技术方案（Runtime 对齐版）》执行。
2. 功能 1/2/3 统一按 Runtime 固定 payload、统一摘要算法 `blake2_256(scale_encode(payload))`、统一签名算法 `sr25519`。
3. 功能 1/2/3 的 nonce 防重放分别按 Runtime 对应键消费，任何重复 nonce 必须被链上拒绝。
4. 若本文件其余章节与本节冲突，以本节为准。

## 0.1 五项能力总对齐矩阵（冻结）
1. 功能 1：公民身份绑定凭证
   - 链上需要：`genesis_hash`、`who`、`binding_id`、`bind_nonce`、`signature`。
   - SFID 实际提供：`GET /api/v1/bind/result` 返回 Runtime 绑定凭证；核心签名 payload 与链上 verifier 一致。
   - 对齐结论：已对齐；链上调用已改为 `bind_sfid(origin, credential)`，不再消费 `sfid_code` 与过期字段。
2. 功能 2：公民投票凭证
   - 链上需要：`binding_id`、`proposal_id`、`vote_nonce`、`signature`。
   - SFID 实际提供：`POST /api/v1/vote/verify` 返回投票凭证；核心签名 payload 与链上 verifier 一致。
   - 对齐结论：已对齐；`is_bound/has_vote_eligibility/message/key_*` 属状态或运维字段，不参与链上验签。
3. 功能 3：联合投票人口快照
   - 链上需要：`eligible_total`、`snapshot_nonce`、`signature`，且发起账户 `who` 必须进入签名 payload。
   - SFID 实际提供：`GET /api/v1/chain/voters/count` 返回人口快照签名；核心签名 payload 与链上 verifier 一致。
   - 对齐结论：已对齐；对链接口现固定返回 `genesis_hash`、`who`、`eligible_total`、`snapshot_nonce`、`signature`。
4. 功能 4：机构 `sfid_id` 登记
   - 链上需要：proof 型字段包 `("GMB_SFID_INSTITUTION_V2", genesis_hash, sfid_id, name, register_nonce)`，以及 extrinsic `register_sfid_institution(sfid_id, name, register_nonce, signature)`。V2 新增 `name` 参数（机构名称，BoundedVec<u8, 128>），纳入签名载荷防篡改。创建/注销多签账户时按 0.1%（最低 10 分）收取手续费，走 FeeRouter 分账。
   - SFID 实际提供：`super-admins` 模块在机构扫码录入成功后，生成 `genesis_hash + sfid_id + name + register_nonce + signature`，并用这组字段调用链上登记入口，同时在响应中回写 proof 字段与 `tx_hash/block_number`。
   - 对齐结论：已对齐（V2 升级待下次 runtime 升级生效）。
5. 功能 5：SFID 验签主备账户管理
   - 链上需要：创世三账户 `main + backup_1 + backup_2`，以及标准 extrinsic `rotate_sfid_keys(new_backup)`，要求由备用账户发起。
   - SFID 实际提供：`key-admins` 模块以链上三把公钥为唯一真相；`rotate/challenge` 与 `rotate/commit` 都强制发起者是 `backup_1/backup_2`，若服务端代提链上交易，则必须具备所选备用账户的 signer 能力。
   - 对齐结论：已对齐到“功能 5 只能由备用账户发起”的口径；`/api/v1/attestor/public-key` 继续对外公布当前本地主公钥，但仅用于功能 1/2/3/4 的主签名输出。

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
   - 投票：`citizenchain/otherpallet/sfid-code-auth/src/lib.rs:434`（`proposal_id + binding_id + hash(vote_nonce)`）

## 3. API 矩阵（模块内）
1. `POST /api/v1/bind/request`
2. `GET /api/v1/bind/result`
   - 绑定成功时返回：`genesis_hash`、`who`、`binding_id`、`bind_nonce`、`signature`。
   - 同一 `account_pubkey` 在已绑定状态下，若 active signer 未变化则返回同一份持久化凭证；若 signer / key 元信息变化则重新签发。
3. `POST /api/v1/vote/verify`
   - `proposal_id` 必填；返回：`binding_id`、`proposal_id`、`vote_nonce`、`signature`（并可附带 `key_id/key_version/alg`）。
   - 隐私约束：不返回 `sfid_code` 明文。
4. `GET /api/v1/chain/voters/count?account_pubkey=<who>`
   - `who(account)` 必填（兼容别名 `who`），返回：`genesis_hash`、`who`、`eligible_total`、`snapshot_nonce`、`signature`。
5. `POST /api/v1/chain/binding/validate`
6. `POST /api/v1/chain/reward/ack`
7. `GET /api/v1/chain/reward/state`
8. `GET /api/v1/attestor/public-key`
9. `GET /api/v1/admin/chain/balance?account_pubkey=<hex>`
   - 权限：任意已登录管理员（`require_admin_any`）。
   - 功能：通过 `chain/balance.rs` 直接读取本地全节点 `System.Account` 存储项，截取 `data.free`（u128，单位"分"）并按 `xxx.xx` 元格式化。
   - 存储键算法：`twox_128("System") || twox_128("Account") || blake2_128(account32) || account32`。
   - 返回：`{ account_pubkey, balance_min_units(string), balance_text("xxx.xx"), unit("元") }`。
   - 用途：密钥管理页"主账户"行右侧展示链上余额；前端每次进入 `keyring` 视图都拉一次，不缓存（SFID 服务器与全节点同机部署，无超时降级）。

## 4. 功能 1/2/3 对齐契约（Runtime 口径）

### 4.1 功能 1：SFID 绑定验签
1. 固定 payload：`("GMB_SFID_BIND_V3", genesis_hash, who, binding_id, bind_nonce)`。
2. `genesis_hash` 对应 Runtime `block_hash(0)`。
3. SFID 对链输出字段（链上消费）：`genesis_hash`、`who`、`binding_id`、`bind_nonce`、`signature`。
4. 链上交易参数更新为：`bind_sfid(origin, credential)`；Runtime 直接使用 `credential.binding_id` 绑定，不再消费旧版明文与过期字段。
5. `bind_nonce` 必须一次性，链上按 `hash(bind_nonce)` 去重。
6. SFID 在绑定确认时生成并持久化运行时绑定凭证，`bind_result` 仅回传已持久化凭证，避免重复查询产生不同 `bind_nonce`。
7. 若当前 signer 公钥或 `key_id/key_version/alg` 与已持久化凭证不一致，SFID 必须重新签发 Runtime 绑定凭证并覆盖旧值。
8. 运维侧可在本地持久化 `key_id`、`key_version`、`alg`，但对链接口只返回 Runtime 核心字段。
9. `bind_result.signature` 为 Runtime 凭证签名；`sfid_signature` 不再对链接口暴露。

### 4.2 功能 2：投票凭证验签与防重放
1. 固定 payload：`("GMB_SFID_VOTE_V3", genesis_hash, who, binding_id, proposal_id, vote_nonce)`。
2. SFID 对链输出字段：`genesis_hash`、`who`、`binding_id`、`proposal_id`、`vote_nonce`、`signature`。
3. 链上防重放键固定为：`(proposal_id, binding_id, hash(vote_nonce))`。
4. `vote_nonce` 每次新生成，严禁复用。

### 4.3 功能 3：人口快照签名
1. 固定 payload：`("GMB_SFID_POPULATION_V3", genesis_hash, who, eligible_total, snapshot_nonce)`。
2. `voters/count` 必须接收 `who(account)` 并进入签名 payload，不能仅对 `eligible_total` 签名。
3. SFID 对链输出字段：`genesis_hash`、`who`、`eligible_total`、`snapshot_nonce`、`signature`。
4. 人口快照对链接口已收口为最小字段集，不再并行返回旧版兼容字段。

### 4.4 运行时环境约束
1. Runtime 签名所需 `genesis_hash` 在开发环境通过 `SFID_CHAIN_RPC_URL / SFID_CHAIN_WS_URL` 启动时自动发现；生产环境通过代码内 `TRUSTED_PRODUCTION_CHAINS` 白名单与链上实际 `block_hash(0)` 校验后缓存。
2. 签名密钥缓存中的 seed 必须使用可清零敏感类型存储（`SensitiveSeed`），禁止以普通 `String` 持有。
3. `runtime_meta` 不再持久化或恢复活动主私钥 / 主公钥 / 已知 seed 映射，防止数据库旧状态覆盖部署环境。

### 4.5 功能 4：机构登记签名（INSTITUTION_V2）
1. 固定 payload：`("GMB_SFID_INSTITUTION_V2", genesis_hash, sfid_id, name, register_nonce)`。
2. V2 相对 V1 新增 `name`（机构名称）字段，用于链上存储展示。
3. SFID 对链输出字段：`genesis_hash`、`sfid_id`、`name`、`register_nonce`、`signature`。
4. 链上交易参数：`register_sfid_institution(sfid_id, name, register_nonce, signature)`。
5. `register_nonce` 每次新生成（UUID），链上按 `hash(register_nonce)` 去重。

## 5. 功能 4/5 模块边界（SFID 系统内）

### 5.1 功能 4：机构 `sfid_id` 登记
1. 归属 `super-admins` 模块：负责机构主数据与审批。
2. 标准流程：`SFID 审批完成 -> 授权 Origin 发链上登记交易 -> 回写 tx_hash/block_number`。
3. `sfid_id` 格式（长度、字符集、大小写）由 SFID 与链侧双端校验。

### 5.2 功能 5：验签密钥运维（主备轮换）
1. 归属 `key-admins` 模块：负责密钥治理、轮换流程、审计记录。
2. 链侧动作以标准 extrinsic 为准（如 `rotate_sfid_keys`），不依赖私有 RPC 方法。
3. 轮换策略：先写链上 backup，再提升为 main，再下发新 backup；全程记录审计事件与版本号。
4. 当前正式口径下，备用私钥不进入 `sfid`；因此 `sfid` 不得假设本地持有 backup seed 自动代签上链。
5. `rotate/commit` 必须串行执行，防止并发提交导致本地 keyring 状态回写覆盖。
6. 上链成功后本地写入前必须重检 keyring `version`；若版本已变化，返回冲突并禁止本地覆盖。
7. `wait_for_finalized` 必须设置超时（默认 `90` 秒，可配置），避免链终局停滞导致服务请求无限挂起。

## 6. 实施步骤（文档与接口同步）
1. 冻结协议字段顺序与 domain 常量，输出签名向量文档（输入、哈希、签名样例）。
2. 改造 SFID 后端功能 1/2/3 接口 payload 结构，严格对齐 Runtime。
3. 改造链侧调用参数组装，确保 `who`、`binding_id/binding_id`、`nonce` 类型一致。
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

## 9. 本地手机联调地址约束
1. `sfid` 后端监听地址通过 `SFID_BIND_ADDR` 控制，格式固定为 `<host>:<port>`。
2. 手机真机联调时，`SFID_BIND_ADDR` 应优先使用 `0.0.0.0:<port>` 监听全部网卡，不能使用 `127.0.0.1`。
3. 手机访问地址应通过 `SFID_PUBLIC_BASE_URL` 或 `WUMINAPP_API_BASE_URL` 单独指定为电脑的局域网 IP，不能直接把一个不属于本机网卡的公网 IP 填进 `SFID_BIND_ADDR`。
4. `wuminapp/scripts/app-run.sh` 与 `wuminapp/scripts/app-clean-run.sh` 会优先复用 `sfid/.env.dev.local` 中的 `SFID_PUBLIC_BASE_URL`，其次才回退到 `SFID_BIND_ADDR`。

## 10. 创世哈希绑定策略
1. 开发环境（`SFID_ENV=dev`）默认通过 `SFID_CHAIN_RPC_URL / SFID_CHAIN_WS_URL` 在启动时自动获取创世哈希，适配经常清链重启的开发链。
2. 开发环境仍允许通过 `SFID_CHAIN_GENESIS_HASH` 临时覆盖，但仅用于特殊调试，不作为默认路径。
3. 当同时配置 `SFID_CHAIN_RPC_URL` 与 `SFID_CHAIN_WS_URL` 时，开发环境优先使用 HTTP JSON-RPC 调用 `chain_getBlockHash(0)` 获取创世哈希，只有缺少 HTTP RPC 时才回退到 WebSocket，避免本地 `ws://` 被客户端判定为 insecure。
4. 生产环境（`SFID_ENV=prod`）不再信任运行时传入的 `SFID_CHAIN_GENESIS_HASH`，而是要求链上实际返回的创世哈希命中代码内 `TRUSTED_PRODUCTION_CHAINS` 白名单。
5. 若生产环境连接到的链不在白名单内，`sfid` 必须拒绝初始化 Runtime 签名上下文。
6. 若后续需要支持多条正式链，只允许在 `TRUSTED_PRODUCTION_CHAINS` 中追加新链定义，保持源码级强绑定。

## 11. 主签名人唯一来源
1. `sfid` 当前在线主私钥只允许来自部署环境中的 `SFID_SIGNING_SEED_HEX`。
2. 这把主私钥必须使用 Substrate `sp-core::sr25519::Pair::from_seed_slice` 规则派生公钥与签名，不允许使用裸 schnorrkel uniform 扩展替代。
3. 启动时必须先从链上读取：
   - `SfidCodeAuth::SfidMainAccount`
   - `SfidCodeAuth::SfidBackupAccount1`
   - `SfidCodeAuth::SfidBackupAccount2`
4. `sfid` 必须把链上三把公钥同步到本地 `chain_keyring_state` 镜像，再做签名服务启动。
5. 若本地主私钥派生公钥不等于链上 `SfidMainAccount`，服务必须拒绝启动，不能继续提供绑定凭证、投票凭证、人口快照签名。
