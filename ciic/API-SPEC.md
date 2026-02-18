# CIIC API 规范（v1）

## 1. 约定
- Base Path: `/api/v1`
- Content-Type: `application/json`
- 成功响应：`{ "code": 0, "message": "ok", "data": ... }`
- 失败响应：`{ "code": <non-zero>, "message": "...", "trace_id": "..." }`

## 2. 鉴权
- 管理员接口：`Authorization: Bearer <token>` + 2FA 校验字段
- 轻节点接口：可先用匿名+签名挑战，后续升级为 token

## 3. 错误码建议
- `1001` 参数错误
- `1002` 状态不允许
- `2001` 未登录
- `2002` 权限不足
- `2003` 2FA 失败
- `3001` 索引号已绑定
- `3002` 公钥已绑定
- `3003` 绑定申请不存在
- `3004` 绑定申请已过期
- `3005` 未绑定或绑定无效
- `5001` 签名服务不可用
- `5002` 数据库错误

## 4. 接口清单

### 4.1 创建绑定申请
`POST /bind/request`

请求体：
```json
{
  "account_pubkey": "0x....",
  "chain_id": "gmb-mainnet",
  "client_nonce": "a1b2c3"
}
```

返回：
```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "request_id": "br_20260218_xxx",
    "status": "PENDING",
    "expires_at": "2026-02-18T10:30:00Z"
  }
}
```

### 4.2 查询绑定申请状态
`GET /bind/request/{request_id}`

返回 `PENDING | APPROVED | REJECTED | EXPIRED`。

### 4.3 管理员确认绑定（人工）
`POST /admin/bind/confirm`

请求体：
```json
{
  "request_id": "br_20260218_xxx",
  "archive_index": "CIV-XXXX-XXXX",
  "operator_2fa_code": "123456",
  "remark": "manual verified"
}
```

返回：
```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "request_id": "br_20260218_xxx",
    "status": "APPROVED",
    "binding_id": "bd_xxx"
  }
}
```

### 4.4 获取绑定凭证
`GET /bind/credential?request_id=...&account_pubkey=...`

返回：
```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "identity_hash": "0x...",
    "nonce": "n_xxx",
    "signature": "0x...",
    "key_id": "k1",
    "expired_at": "2026-02-18T10:35:00Z"
  }
}
```

### 4.5 获取投票凭证（自动）
`POST /vote/credential`

请求体：
```json
{
  "account_pubkey": "0x....",
  "proposal_id": 1001,
  "chain_id": "gmb-mainnet"
}
```

返回同绑定凭证结构，且 payload 绑定 `proposal_id`。

### 4.6 查询绑定关系（管理）
`GET /admin/bindings?account_pubkey=...` 或 `?archive_index=...`

返回绑定关系详情与状态。

### 4.7 解绑/冻结（管理，可选）
`POST /admin/bind/unbind`
`POST /admin/bind/suspend`

用于异常处理与风控。

## 5. 签名载荷建议
- Bind 域：`GMB_CIIC_BIND_V1`
- Vote 域：`GMB_CIIC_VOTE_V1`
- 统一使用：`blake2_256(encode(payload))` 后签名
- payload 最少包含：`domain, chain_genesis_hash, account_pubkey, identity_hash, nonce`
- Vote 额外包含：`proposal_id`

## 6. 幂等与防重放
- `bind/request` 支持 `client_nonce` 幂等。
- 凭证必须短时效（建议 2-5 分钟）。
- `nonce` 全局唯一并仅可消费一次。

