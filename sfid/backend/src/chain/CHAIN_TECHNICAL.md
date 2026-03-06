# CHAIN_TECHNICAL

## 1. 模块目标
1. `chain` 模块负责 SFID 面向区块链的数据服务接口。
2. 本模块不处理管理员后台治理动作（如机构登记、密钥轮换发起），只提供链侧可调用 API 与数据回执接口。

## 2. 代码归属
1. `backend/src/chain/binding.rs`：绑定申请/结果、绑定有效性校验、奖励回执与状态查询。
2. `backend/src/chain/vote.rs`：投票资格校验与投票凭证签发。
3. `backend/src/chain/voters.rs`：可投票公民人数统计。
4. 路由挂载：`backend/src/main.rs`（`chain_routes`）。

## 3. API 矩阵（已实现）
1. `POST /api/v1/bind/request`
2. `GET /api/v1/bind/result`
3. `POST /api/v1/vote/verify`
4. `GET /api/v1/chain/voters/count`
5. `POST /api/v1/chain/binding/validate`
6. `POST /api/v1/chain/reward/ack`
7. `GET /api/v1/chain/reward/state`
8. `GET /api/v1/attestor/public-key`

## 4. 区块链参数对齐（核心三项）

### 4.1 公民身份绑定（必传：`sfid_code`、`credential.sfid_code_hash`、`credential.nonce`、`credential.signature`）
1. SFID 数据来源接口：`GET /api/v1/bind/result`。
2. 当前接口返回：`sfid_code`、`sfid_signature`（字段名：`sfid_signature`）。
3. 对齐映射：
   - `sfid_code` <- `bind/result.data.sfid_code`
   - `credential.signature` <- `bind/result.data.sfid_signature`
   - `credential.sfid_code_hash` <- 由链侧按 Runtime 规则对 `sfid_code` 计算（SFID 当前不单独返回该字段）
   - `credential.nonce` <- 由链侧交易流程提供（SFID 当前绑定结果接口不返回该字段）
4. 结论：当前为“部分对齐”，可直接提供 `sfid_code + signature`，其余字段由链侧组装。

### 4.2 公民投票凭证（必传：`sfid`、`proposal_id`、`nonce`、`signature`）
1. SFID 数据来源接口：`POST /api/v1/vote/verify`。
2. 请求字段：`account_pubkey`、`proposal_id`、`challenge`（可选；为空时后端自动生成）。
3. 返回字段：`vote_token`（`SignatureEnvelope`），含 `payload` + `signature_hex`。
4. 对齐映射：
   - `sfid` <- `vote_token.payload.sfid_code`
   - `proposal_id` <- `vote_token.payload.proposal_id`
   - `nonce` <- `vote_token.payload.challenge`
   - `signature` <- `vote_token.signature_hex`
5. 结论：该项可在现有接口上对齐，链侧按映射取值即可。

### 4.3 联合投票人口快照（必传：`eligible_total`、`snapshot_nonce`、`snapshot_signature`）
1. SFID 数据来源接口：`GET /api/v1/chain/voters/count`。
2. 当前接口返回：`total_voters`、`as_of`。
3. 对齐映射：
   - `eligible_total` <- `chain/voters/count.data.total_voters`
4. 差异项：
   - `snapshot_nonce`：当前接口未返回
   - `snapshot_signature`：当前接口未返回
5. 结论：该项目前“未完全对齐”，文档口径先按差异登记，后续需扩展快照签名字段。

## 5. 链路鉴权与防重放
1. 所有链路接口统一要求：
   - `x-chain-token`
   - `x-chain-request-id`
   - `x-chain-nonce`
   - `x-chain-timestamp`
   - `x-chain-signature`
2. 鉴权与防重放能力：
   - 令牌校验（`SFID_CHAIN_TOKEN`）
   - 请求签名校验（`SFID_CHAIN_SIGNING_SECRET`）
   - 时间窗校验（默认 `±300s`）
   - `request_id` 幂等与 `nonce` 去重
3. 失败场景会返回标准错误码，并计入链路指标与审计。

## 6. 与其他模块边界
1. 机构 SFID 登记（`sfid_id` 主数据）归属 `super-admins` 模块，不在 `chain` 模块。
2. 主备账户管理与轮换归属 `key-admins` 模块，不在 `chain` 模块。
3. `chain` 模块只消费已建立的数据（绑定关系、状态、签名密钥）并对链提供查询/凭证服务。
