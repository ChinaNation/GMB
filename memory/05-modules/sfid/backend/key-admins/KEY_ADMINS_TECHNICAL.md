# KEY_ADMINS_TECHNICAL

## 0. 区块链端方案对齐（冻结，优先级最高）
1. 本文档第 0 步严格按《SFID-Chain 五项能力对齐技术方案（Runtime 对齐版）》执行。
2. 功能 5 轮换以链上标准 extrinsic 为准（如 `sfid-code-auth::rotate_sfid_keys`），不依赖私有 RPC 方法。
3. 链上三把公钥是唯一权威来源；`sfid` 本地不得用数据库恢复活动主签名人覆盖链上状态。
4. 全流程必须记录并回写 `chain_tx_hash`、`block_number`，并记录审计事件与版本号。
5. 若本文件其余章节与本节冲突，以本节为准。

## 1. 模块目标
- 本模块负责 SFID 系统密钥管理员能力：管理员治理能力、主备密钥轮换、链证明签名输出。
- 密钥轮换规则固定为：`只能备升主，新增的是备`。

## 2. 角色与槽位
- `MAIN`：主密钥管理员（链上主公钥 + `sfid` 本地唯一在线主私钥）。
- `BACKUP_A`：备用密钥管理员 A（链上备用公钥；私钥不进入 `sfid`）。
- `BACKUP_B`：备用密钥管理员 B（链上备用公钥；私钥不进入 `sfid`）。

### 2.1 链上参数命名对齐
1. `sfidMainAccount` 对应本模块 `MAIN.main_pubkey`。
2. `sfidBackupAccount1` 对应本模块 `BACKUP_A.backup_a_pubkey`。
3. `sfidBackupAccount2` 对应本模块 `BACKUP_B.backup_b_pubkey`。
4. 主备轮换时链上参数 `new_backup` 对应本模块接口字段 `new_backup_pubkey`。

## 3. 权限定义（最终口径）
- 三个密钥管理员共同拥有系统全部权限：
1. 更换省级管理员（43 省）。
2. 全局管理市级管理员（跨省查看、增删改查、启停用）。
3. 全局业务操作与查询（不受省隔离限制，按接口细分）。
4. 机构管理（生成、扫码登记、更新、禁用、撤销、删除、查询）。

- 差异化权限：
1. `MAIN` 才能执行链证明签名（投票状态、绑定证明、公民数证明）。
2. `BACKUP_A/B` 不能执行链证明签名。
3. 只有 `BACKUP_A/B` 可以发起主密钥轮换。
4. `MAIN` 无权发起主密钥轮换。

## 4. 主密钥轮换状态机
- 初始状态：`(main, backup_a, backup_b)`。
- 输入：`initiator_pubkey`（必须是 backup_a 或 backup_b）+ `new_backup_pubkey`（新增备用）。
- 规则：
1. 发起方必须是当前备用之一（大小写不敏感比较）。
2. `new_backup_pubkey` 必填，且不能与当前三把公钥重复（大小写不敏感比较）。
3. 若发起方是 `BACKUP_A`：
   - 新主 = `backup_a`
   - 新 `backup_a` = `new_backup_pubkey`
   - `backup_b` 保持不变
4. 若发起方是 `BACKUP_B`：
   - 新主 = `backup_b`
   - 新 `backup_b` = `new_backup_pubkey`
   - `backup_a` 保持不变
5. 旧主退出活动密钥集，结果始终保持一主两备。
6. 链侧交易发起 `origin` 必须为 `backup_a` 或 `backup_b` 之一；`main` 不可直接发起轮换。
7. `version` 递增使用饱和加法（`saturating_add`），避免极端整数溢出风险。

## 5. 轮换执行流程（两阶段，强约束）
1. `rotate/challenge`：
   - 后端生成一次性 challenge（含 `challenge_id`、`version`、`nonce`、`iat`、`exp`、`sigfmt=raw-v1`）。
   - `initiator_pubkey` 在 challenge 文本中按规范化格式写入（`0x` + 小写 hex）。
   - challenge 与发起人绑定，短时有效。
   - TTL 可配置：`SFID_KEYRING_CHALLENGE_TTL_MINUTES`（默认 2 分钟）。
   - 每个 KEY_ADMIN 未消费 challenge 并发上限可配置：`SFID_KEYRING_CHALLENGE_MAX_ACTIVE`（默认 2）。
2. `rotate/verify`：
   - 前端扫码签名结果后，先调用该接口校验“确实是备用密钥对 `challenge_text` 的签名”。
   - 校验成功后，服务端写入 `verified_at`。
3. `rotate/commit`：
   - 必须基于已 `verify` 成功的 challenge 才可提交（未验签会返回 `rotation challenge not verified`）。
   - 提交参数：
     - `new_backup_pubkey`（新备用公钥）
   - commit 阶段验签消息为：`{challenge_text}|phase=commit|new_backup={normalized_pubkey}`。
   - `verify` 与 `commit` 签名不可复用（防止 verify 报文签名被直接用于 commit）。
   - 说明：功能 5 固定要求 `backup_1/backup_2` 发起；若由服务端代提 `rotate_sfid_keys`，则服务端必须具备被选备用账户的 signer 能力，否则应改由外部 signer 提交。
   - commit 在服务内按互斥锁串行执行，避免并发 commit 的本地覆盖风险。
   - 链提交等待最终确认带超时：`SFID_CHAIN_ROTATE_FINALIZE_TIMEOUT_SECONDS`（默认 `90` 秒）。
   - 记录并回写 `tx_hash`、`block_number`（可先回写受理高度，最终高度异步确认）。
4. 签名格式兼容：
   - 当前服务端统一生成 `sigfmt=raw-v1`（默认 raw 消息签名）。
   - 代码保留 `bytes-wrap-v1` 解析分支作为未来兼容扩展，但当前流程不生成该模式。

## 6. 上链一致性要求
- 区块链验证公钥仅使用 `MAIN`。
- 轮换上链以 `sfid-code-auth` pallet 标准入口为准（如 `rotate_sfid_keys`），不依赖私有 RPC 方法。
- 轮换策略固定：先写链上 `backup`，再提升为 `main`，再补位新 `backup`。
- 上链回执要求：
1. 轮换提交必须记录 `chain_tx_hash`。
2. 轮换提交必须记录 `block_number`（受理高度或最终高度）。
3. 若链上提交失败，必须返回明确错误并保留可重试上下文（challenge_id、version、initiator、new_backup_pubkey）。

### 6.1 本地一致性顺序（关键）
1. `sfid` 本地唯一权威密钥材料只有当前主私钥 `SFID_SIGNING_SEED_HEX`。
2. `SFID_SIGNING_SEED_HEX` 必须按 Substrate `sp-core::sr25519::Pair::from_seed_slice` 规则派生主公钥与签名，不允许使用裸 `schnorrkel::MiniSecretKey::expand_to_keypair(Uniform)` 自行扩展。
3. `runtime_meta` 不再持久化也不再恢复：
   - `signing_seed_hex`
   - `known_key_seeds`
   - `public_key_hex`
4. 启动顺序固定为：
   - 从 `SFID_SIGNING_SEED_HEX` 派生本地主公钥
   - 从链上同步 `main/backup_a/backup_b`
   - 校验“本地主公钥 == 链上 main 公钥”
   - 不一致直接拒绝启动签名服务
5. `store.chain_keyring_state` 只是链上三把公钥的本地镜像，不得反向覆盖链上状态。

### 6.2 与区块链“验签主备账户管理”对齐口径
1. 创世固定三账户：`sfidMainAccount`、`sfidBackupAccount1`、`sfidBackupAccount2`。
2. 轮换动作由 `backup_1/backup_2` 发起，提交 `new_backup`（本模块字段 `new_backup_pubkey`）。
3. 本模块输出的 keyring 状态（`main_pubkey`、`backup_a_pubkey`、`backup_b_pubkey`）是链侧账户映射的权威来源。
4. 本模块 `rotate/commit` 返回 `chain_submit_ok`、`chain_tx_hash`、`block_number`，用于链侧运维对账。

## 7. 链证明签名要求
- 仅 `MAIN` 私钥执行签名。
- 签名算法统一 `sr25519`。
- 适用对象：
1. 投票状态证明。
2. 公钥绑定/SFID 证明。
3. 公民数统计证明。

## 8. 安全控制
- 防重放：challenge 一次性消费 + TTL + 版本绑定。
- 防滥用：每 KEY_ADMIN 的活跃 rotation challenge 数量有限制（默认 2）。
- 防越权：轮换接口仅允许 `BACKUP_A/B` 身份调用。
- 防跳步：`commit` 必须建立在 `verify` 成功后（`verified_at` 非空）。
- 防签名复用：`verify` 与 `commit` 使用不同签名消息文本。
- 防并发覆盖：`commit` 串行执行 + 上链成功后版本重检（发现并发则拒绝本地覆盖）。
- 审计必记：发起人、旧主、新主、新增备用、challenge_id、交易哈希、时间、结果。
- 密钥材料约束：
1. seed 只接受严格 `64` hex 字符（可含 `0x` 前缀），不再允许任意字符串哈希兜底。
2. 主私钥 seed 的派生与签名必须与链 runtime 使用的 Substrate sr25519 实现一致，避免“seed 正确但派生公钥错误”造成链上验签失败。
3. 生产模式（`SFID_ENV=prod|production`）必须显式配置两把备用公钥，不允许 deterministic fallback。
4. 备用私钥不得进入 `sfid` 进程、数据库或运行态缓存。
5. 内存中敏感字段使用 `SensitiveSeed`（Drop 时 zeroize），`Debug` 输出脱敏；仅通过 `expose_secret()` 供加密路径调用。
6. 进程启动时尝试设置 `RLIMIT_CORE=0` 禁用 core dump（best-effort）。

## 9. 错误语义
- `initiator must be backup`
- `initiator backup private key is not present on server; rotate_sfid_keys must be submitted by backup_1 or backup_2`
- `server signer seed does not match initiator_pubkey`
- `new_backup_pubkey is required`
- `new_backup_pubkey conflicts with current keyring`
- `rotation signature verify failed`
- `rotation challenge expired`
- `rotation challenge not verified`
- `chain keyring version changed, retry challenge`
- `concurrent rotation completed, local state refresh required`
- `too many active rotation challenges`
- `failed to persist runtime signer state`
- `rotate_sfid_keys submit failed: timed out waiting for finalization`
- `rotate_sfid_keys submit failed`
- `rotate_sfid_keys included failed`

## 10. 当前已实现功能清单（2026-03，含 Runtime 对齐改造项）
1. 省级管理员治理：
   - 查询省级省级管理员：`GET /api/v1/admin/sheng-admins`
   - 更换省级省级管理员：`PUT /api/v1/admin/sheng-admins/:province`
2. 市级管理员治理（全局）：
   - 列表/新增/删除/启停：`/api/v1/admin/operators*`
3. 密钥管理（一主两备）：
   - 查询 keyring：`GET /api/v1/admin/attestor/keyring`
   - 轮换挑战：`POST /api/v1/admin/attestor/rotate/challenge`
   - 轮换验签：`POST /api/v1/admin/attestor/rotate/verify`
   - 轮换提交：`POST /api/v1/admin/attestor/rotate/commit`
4. 业务与查询（全局）：
   - 公民列表/绑定查询：`GET /api/v1/admin/citizens`、`GET /api/v1/admin/bind/query`
   - 绑定扫码/确认/解绑：`POST /api/v1/admin/bind/scan|confirm|unbind`
   - SFID 元数据/城市/生成：`GET /api/v1/admin/sfid/meta|cities`、`POST /api/v1/admin/sfid/generate`
   - CPMS 状态变更扫码：`POST /api/v1/admin/cpms-status/scan`
   - 机构管理（全部操作）：`/api/v1/admin/cpms-keys*`
5. 链证明相关：
   - 公钥输出：`GET /api/v1/attestor/public-key`
   - 绑定证明、投票资格证明、公民计数证明签名由当前主签名密钥执行。
6. 一致性与安全加固：
   - 启动改为“链上 keyring 同步 + 本地主私钥硬校验”，不再自动修复旧 signer。
   - `runtime_meta` 不再保存活动主私钥/主公钥，避免数据库旧状态覆盖部署环境。
   - key admin pubkey 统一规范化存储（`0x` + 小写）。
   - 轮换上链改为标准 extrinsic（`rotate_sfid_keys`）并回写 `tx_hash/block_number`。

## 11. 测试清单（最低）
1. 备用 A 发起轮换成功，状态满足一主两备。
2. 备用 B 发起轮换成功，状态满足一主两备。
3. 主密钥发起轮换被拒绝。
4. 新备用与现有公钥冲突被拒绝。
5. challenge 过期/重复提交被拒绝。
6. 签名错误被拒绝。
7. 备用私钥与发起备用公钥不匹配时拒绝。
8. `rotate_sfid_keys` 提交失败时必须明确返回 `chain_submit_ok=false`，且不得丢失重试上下文。
9. 只有主密钥可签业务证明，备用调用被拒绝。
10. 未经过 `verify` 的 challenge 调用 `commit` 必须被拒绝。
11. `verify` 签名不可复用于 `commit`。
12. `SensitiveSeed` / `PersistedRuntimeMeta` 的 `Debug` 输出不含明文 seed。
13. 启动校验：本地主私钥派生公钥与链上 `main` 不一致时必须拒绝启动。

## 12. 文件归属
- `chain_keyring.rs`：一主两备状态机、轮换验签、签名密钥加载。
- `chain_proof.rs`：链证明签名封装、公钥输出。
- `mod.rs`：密钥管理员 API、权限控制、审计落库、路由处理。
