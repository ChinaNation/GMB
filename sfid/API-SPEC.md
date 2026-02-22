# SFID API 规范（v1）

## 1. 约定
- Base Path: `/api/v1`
- Content-Type: `application/json`
- 成功响应：`{ "code": 0, "message": "ok", "data": ... }`
- 失败响应：`{ "code": <non-zero>, "message": "...", "trace_id": "..." }`

## 2. 鉴权
- 管理员接口通过请求头：
  - `x-admin-user`
  - `x-admin-level`（`NATIONAL | PROVINCE | CITY_BUREAU`）
  - `x-admin-2fa-code`
  - `x-admin-province-code`（省级/市级必填）
  - `x-admin-city-code`（市级必填）
- 轻节点接口当前为匿名调用（后续升级 token）。

## 3. 错误码建议
- `1001` 参数错误
- `1002` 状态不允许
- `2001` 管理员鉴权缺失
- `2002` 管理员级别无效
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

### 4.2 查询绑定申请状态
`GET /bind/request/{request_id}`

### 4.3 管理员确认绑定（人工）
`POST /admin/bind/confirm`

请求体：
```json
{
  "request_id": "br_xxx",
  "archive_index": "CIV-XXXX-XXXX",
  "province_code": "GD",
  "city_code": "GZ",
  "remark": "manual verified"
}
```

### 4.4 获取绑定凭证
`GET /bind/credential?request_id=...&account_pubkey=...`

返回字段包含：`identity_hash`, `nonce`, `signature`, `key_id`, `signer_scope`, `expired_at`。

### 4.5 获取投票凭证（自动）
`POST /vote/credential`

返回字段包含：`identity_hash`, `nonce`, `signature`, `key_id`, `signer_scope`, `expired_at`。

### 4.6 查询绑定关系（管理）
`GET /admin/bindings?account_pubkey=...` 或 `?archive_index=...`

### 4.7 解绑（管理）
`POST /admin/bind/unbind`

请求体：
```json
{
  "account_pubkey": "0x...",
  "reason": "manual unbind"
}
```

### 4.8 冻结（管理）
`POST /admin/bind/suspend`

请求体：
```json
{
  "account_pubkey": "0x...",
  "reason": "risk control"
}
```

### 4.9 查询凭证签发记录（管理）
`GET /admin/credentials?account_pubkey=...&credential_type=BIND|VOTE&limit=50`

## 5. 签名载荷建议
- Bind 域：`GMB_SFID_BIND_V1`
- Vote 域：`GMB_SFID_VOTE_V1`
- payload 最少包含：`domain, account_pubkey, identity_hash, nonce`
- Vote 额外包含：`proposal_id`

## 6. 幂等与防重放
- `bind/request` 支持 `client_nonce` 幂等（下一版本完善）。
- 凭证短时效（建议 2-5 分钟）。
- `nonce` 一次性消费由链上完成防重放。
