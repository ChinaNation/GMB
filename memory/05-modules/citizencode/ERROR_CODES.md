# CID 错误码规范

- 最后更新:2026-05-31
- 任务卡:`memory/08-tasks/done/20260531-cid-shi-admin-city-limit.md`

## 1. 总原则

CID HTTP 状态码只表达协议层结果,稳定业务错误码使用响应体中的
`error_code` 字段表达。前端不得解析 `message` 做逻辑判断。

错误响应结构:

```json
{
  "code": 2004,
  "error_code": "CID_BIND_SIGNATURE_VERIFY_FAILED",
  "message": "signature verify failed",
  "trace_id": "..."
}
```

`message` 只用于展示或日志排查,`trace_id` 用于关联后端日志。

## 2. HTTP 状态码边界

| HTTP 状态 | 使用边界 |
|---|---|
| 400 | 请求字段缺失、格式错误、JSON 或 hex 无法解析 |
| 401 | 只用于管理员登录态无效,包括缺 token、token 无效、token 过期 |
| 403 | 已认证但权限不足,包括角色不允许、跨省/跨市范围、管理员禁用 |
| 404 | challenge、机构、公民记录等业务对象不存在 |
| 409 | 资源状态冲突,包括重复绑定、重复消费、名称或公钥冲突 |
| 410 | 一次性或限时资源失效,包括 challenge 过期、QR 过期 |
| 422 | 请求格式正确但业务校验失败,包括签名失败、账户不匹配 |
| 429 | 请求频率超过限制 |
| 500 | 未预期服务端错误 |
| 503 | 数据库、分片缓存、链节点或外部依赖不可用 |

死规则:

- `401` 只表示当前管理员登录态无效。
- 公民绑定 challenge 过期必须返回 `410 + CID_BIND_CHALLENGE_EXPIRED`。
- 公民绑定签名失败必须返回 `422 + CID_BIND_SIGNATURE_VERIFY_FAILED`。
- 公民绑定账户不匹配必须返回 `422 + CID_BIND_ACCOUNT_MISMATCH`。
- 前端收到业务错误只展示错误,不得自动退出登录。

## 3. 当前错误码

### 认证

| error_code | HTTP | 含义 |
|---|---:|---|
| `CID_AUTH_MISSING_TOKEN` | 401 | 未携带管理员 token |
| `CID_AUTH_INVALID_ACCESS_TOKEN` | 401 | 管理员 token 无效 |
| `CID_AUTH_ACCESS_TOKEN_EXPIRED` | 401 | 管理员 token 过期 |
| `CID_AUTH_ADMIN_DISABLED` | 403 | 管理员已禁用 |
| `CID_AUTH_PERMISSION_DENIED` | 403 | 管理员权限不足 |
| `CID_AUTH_FORBIDDEN` | 403 | 已认证但访问被拒绝 |

### 公民绑定

| error_code | HTTP | 含义 |
|---|---:|---|
| `CID_BIND_CHALLENGE_NOT_FOUND` | 404 | 绑定 challenge 不存在 |
| `CID_BIND_CHALLENGE_CONSUMED` | 409 | 绑定 challenge 已消费 |
| `CID_BIND_CHALLENGE_EXPIRED` | 410 | 绑定 challenge 已过期 |
| `CID_BIND_ACCOUNT_MISMATCH` | 422 | 签名账户与 challenge 锁定账户不一致 |
| `CID_BIND_SIGNATURE_FORMAT_INVALID` | 400 | 签名不是合法 hex |
| `CID_BIND_SIGNATURE_VERIFY_FAILED` | 422 | 钱包签名验签失败 |
| `CID_BIND_PUBKEY_ALREADY_BOUND` | 409 | 公钥已绑定身份ID |

### 管理员账号

| error_code | HTTP | 含义 |
|---|---:|---|
| `CID_ADMIN_ACCOUNT_EXISTS_AS_FEDERAL_REGISTRY` | 409 | 管理员账户已作为联邦注册局管理员存在 |
| `CID_ADMIN_ACCOUNT_EXISTS_AS_CITY_REGISTRY` | 409 | 管理员账户已作为市注册局管理员存在 |
| `CID_ADMIN_FEDERAL_REGISTRY_PROVINCE_LIMIT_REACHED` | 409 | 本省联邦注册局管理员已达到 5 人上限 |
| `CID_ADMIN_CITY_REGISTRY_CITY_LIMIT_REACHED` | 409 | 本市市注册局管理员已达到 30 人上限 |
| `CID_STORE_PERSIST_FAILED` | 500 | 写操作持久化失败，接口不得返回业务成功 |

管理员公钥全局唯一。新增联邦注册局机构管理员或市注册局机构管理员时，后端必须先按规范化公钥查全局
`admins` 账号表；命中后按已有角色返回上述稳定错误码，前端按新增目标角色展示中文提示。
联邦注册局机构管理员每省最多 5 人，市注册局机构管理员每省每市最多 30 人，达到上限时必须在后端拒绝新增。

## 4. 前端处理规则

`citizencode/frontend/utils/http.ts` 必须满足:

- 成功时返回 `T`。
- 认证失效时抛 `AuthExpiredError`,并触发全局退出。
- 业务失败时抛 `ApiError`,页面按 `errorCode` 展示业务错误。
- 禁止用 `undefined as T` 表示失败。

公民绑定页只根据 `errorCode` 定制展示文案,不能根据 `message` 做分支。
