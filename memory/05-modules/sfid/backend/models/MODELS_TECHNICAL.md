# MODELS 模块技术文档

## 1. 模块定位

- 路径：`backend/src/models`
- 职责：统一维护 SFID 后端的数据结构定义（领域模型 + API DTO + 状态枚举）。
- 目标：把数据协议与业务逻辑解耦，避免 `main.rs` 持续膨胀。

## 2. 结构分类

- 运行态模型（Runtime Store）
  - `Store`
  - `PersistedRuntimeMeta`
  - `AdminUser`
  - `BindingRecord`
  - `RewardStateRecord`
  - `VoteVerifyCacheEntry`
  - `AuditLogEntry`

- 安全与链路模型
  - `ChainRequestAuth`
  - `ChainRequestReceipt`
  - `BindCallbackJob`
  - `BindCallbackPayload`
  - `BindCallbackSignablePayload`

- 业务状态枚举
  - `AdminRole`
  - `AdminStatus`
  - `CitizenStatus`
  - `CpmsSiteStatus`
  - `RewardStatus`
  - `RewardAckStatusInput`

- 接口输入输出 DTO
  - 认证/绑定/查询/投票/奖励/CPMS/密钥轮换相关请求与响应结构
  - 统一响应结构：`ApiResponse<T>`、`ApiError`、`HealthData`

## 3. 使用方式

- `main.rs` 通过 `pub(crate) use models::*;` 统一导出。
- 业务模块继续通过 `crate::*` 获取类型，不需要逐模块改大量引用路径。

## 4. 边界

- `models` 只定义“数据长什么样”，不执行业务流程。
- 业务处理逻辑分别位于：
  - `operate`（操作业务）
  - `chain`（区块链业务）
  - `business`（查询/审计/作用域）
  - `login`（认证）
  - `key-admins` / `super-admins` / `operator-admins`（角色业务）

## 5. 链路字段同步（2026-03）

- `BindingRecord` 新增 `runtime_bind_*` 字段族（`sfid_code_hash/nonce/expires_at_block/signature/key_id/key_version/alg/signer_pubkey`），用于持久化 Runtime 绑定凭证与签发者元信息。
- `BindingRecord.sfid_signature` 继续保留旧 JSON 绑定证明签名语义；`bind_result.signature` 返回 Runtime 凭证签名，二者不可混用。
- `VoteVerifyInput.proposal_id` 改为必填 `u64`，已移除废弃 `challenge` 字段。
- `VoteVerifyOutput` 仅返回投票凭证字段（`sfid_hash/proposal_id/vote_nonce/signature/key_*`），不再返回 `sfid_code` 明文。
- `ChainVotersCountOutput` 统一以 `eligible_total` 为主统计字段，`as_of` 与统计快照同源生成；保留 `snapshot_signature` 兼容字段并推荐使用 `snapshot_attestation.signature_hex`。
- 涉及新增字段均使用 `#[serde(default)]` 兼容历史持久化数据反序列化。
