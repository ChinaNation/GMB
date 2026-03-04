# KEY_ADMINS_TECHNICAL

## 1. 模块目标
- 本模块负责 SFID 系统密钥管理员能力：管理员治理能力、主备密钥轮换、链证明签名输出。
- 密钥轮换规则固定为：`只能备升主，新增的是备`。

## 2. 角色与槽位
- `MAIN`：主密钥管理员（主公钥 + 主私钥）。
- `BACKUP_A`：备用密钥管理员 A（仅备用公钥，或由钱包持有私钥参与轮换签名）。
- `BACKUP_B`：备用密钥管理员 B（仅备用公钥，或由钱包持有私钥参与轮换签名）。

## 3. 权限定义（最终口径）
- 三个密钥管理员共同拥有系统最高业务权限：
1. 更换超级管理员（43 省）。
2. 全局管理操作管理员（跨省查看、增删改查、启停用）。
3. 全局业务操作与查询（不受省隔离限制，按接口细分）。
4. 机构列表查询（`/api/v1/admin/cpms-keys`）。

- 明确不属于 `KEY_ADMIN` 的能力：
1. 机构新增扫码登记（`/api/v1/admin/cpms-keys/register-scan`）仅 `SUPER_ADMIN` 可用。

- 差异化权限：
1. `MAIN` 才能执行链证明签名（投票状态、绑定证明、公民数证明）。
2. `BACKUP_A/B` 不能执行链证明签名。
3. 只有 `BACKUP_A/B` 可以发起主密钥轮换。
4. `MAIN` 无权发起主密钥轮换。

## 4. 主密钥轮换状态机
- 初始状态：`(main, backup_a, backup_b)`。
- 输入：`initiator_pubkey`（必须是 backup_a 或 backup_b）+ `new_backup_pubkey`（新增备用）。
- 规则：
1. 发起方必须是当前备用之一。
2. `new_backup_pubkey` 必填，且不能与当前三把公钥重复。
3. 若发起方是 `BACKUP_A`：
   - 新主 = `backup_a`
   - 新 `backup_a` = `new_backup_pubkey`
   - `backup_b` 保持不变
4. 若发起方是 `BACKUP_B`：
   - 新主 = `backup_b`
   - 新 `backup_b` = `new_backup_pubkey`
   - `backup_a` 保持不变
5. 旧主退出活动密钥集，结果始终保持一主两备。

## 5. 轮换执行流程（两阶段，强约束）
1. `rotate/challenge`：
   - 后端生成一次性 challenge（含 `challenge_id`、`version`、`nonce`、`iat`、`exp`）。
   - challenge 与发起人绑定，短时有效。
2. `rotate/verify`：
   - 前端扫码签名结果后，先调用该接口校验“确实是备用密钥对 challenge 的签名”。
   - 校验成功后，服务端写入 `verified_at`。
3. `rotate/commit`：
   - 必须基于已 `verify` 成功的 challenge 才可提交（未验签会返回 `rotation challenge not verified`）。
   - 提交参数：
     - `new_backup_pubkey`（新备用公钥）
   - 说明：发起备用私钥不再通过前端上传；服务端从受控密钥库按 `initiator_pubkey` 查找对应 seed。
   - 后端先完成本地主备替换，再异步推送新主公钥到区块链。
   - 不等待链上确认，不阻塞本地主密钥切换。

## 6. 上链一致性要求
- 区块链验证公钥仅使用 `MAIN`。
- 本地替换优先：轮换提交成功即本地完成主私钥/主公钥替换，并更新一主两备状态。
- 链推送由后端直接执行 JSON-RPC 提交，不走外部 Hook。
- 上链执行方式：
1. 配置 `SFID_CHAIN_RPC_URL`（区块链 JSON-RPC URL）。
2. 配置 `SFID_CHAIN_RPC_METHOD`（默认 `sfid_set_main_pubkey`），参数固定 `[new_main_pubkey, version, ticket]`。
3. 可选配置 `SFID_CHAIN_RPC_TOKEN`（Bearer Token）。
4. 提交成功返回 `chain_tx_hash`；提交失败返回 `chain_submit_ok=false` 与 `chain_submit_error`，便于运维补偿重提。

## 7. 链证明签名要求
- 仅 `MAIN` 私钥执行签名。
- 签名算法统一 `sr25519`。
- 适用对象：
1. 投票状态证明。
2. 公钥绑定/SFID 证明。
3. 公民数统计证明。

## 8. 安全控制
- 防重放：challenge 一次性消费 + TTL + 版本绑定。
- 防越权：轮换接口仅允许 `BACKUP_A/B` 身份调用。
- 防跳步：`commit` 必须建立在 `verify` 成功后（`verified_at` 非空）。
- 审计必记：发起人、旧主、新主、新增备用、challenge_id、交易哈希、时间、结果。
- 密钥存储建议：`MAIN` 私钥在服务端受控存储；`BACKUP` 私钥优先钱包持有，不落盘后端。

## 9. 错误语义
- `initiator must be backup`
- `initiator signer seed is not present on server`
- `server signer seed does not match initiator_pubkey`
- `new_backup_pubkey is required`
- `new_backup_pubkey conflicts with current keyring`
- `rotation signature verify failed`
- `rotation challenge expired`
- `rotation challenge not verified`
- `chain keyring version changed, retry challenge`

## 10. 当前已实现功能清单（2026-03）
1. 超级管理员治理：
   - 查询省级超级管理员：`GET /api/v1/admin/super-admins`
   - 更换省级超级管理员：`PUT /api/v1/admin/super-admins/:province`
2. 操作管理员治理（全局）：
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
   - 机构列表查询：`GET /api/v1/admin/cpms-keys`
5. 链证明相关：
   - 公钥输出：`GET /api/v1/attestor/public-key`
   - 绑定证明、投票资格证明、公民计数证明签名由当前主签名密钥执行。

## 11. 测试清单（最低）
1. 备用 A 发起轮换成功，状态满足一主两备。
2. 备用 B 发起轮换成功，状态满足一主两备。
3. 主密钥发起轮换被拒绝。
4. 新备用与现有公钥冲突被拒绝。
5. challenge 过期/重复提交被拒绝。
6. 签名错误被拒绝。
7. 备用私钥与发起备用公钥不匹配时拒绝。
8. 链 RPC 不可达时，本地仍可轮换，响应中必须明确 `chain_submit_ok=false`。
9. 只有主密钥可签业务证明，备用调用被拒绝。
10. 未经过 `verify` 的 challenge 调用 `commit` 必须被拒绝。

## 12. 文件归属
- `chain_keyring.rs`：一主两备状态机、轮换验签、签名密钥加载。
- `chain_proof.rs`：链证明签名封装、公钥输出。
- `mod.rs`：密钥管理员 API、权限控制、审计落库、路由处理。
