# SFID 错误码规范

- 最后更新:2026-05-31
- 任务卡:`memory/08-tasks/done/20260531-sfid-shi-admin-city-limit.md`

## 1. 总原则

SFID HTTP 状态码只表达协议层结果,稳定业务错误码使用响应体中的
`error_code` 字段表达。前端不得解析 `message` 做逻辑判断。

错误响应结构:

```json
{
  "code": 2004,
  "error_code": "SFID_BIND_SIGNATURE_VERIFY_FAILED",
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
| 404 | challenge、档案、机构、公民记录等业务对象不存在 |
| 409 | 资源状态冲突,包括重复绑定、重复消费、名称或公钥冲突 |
| 410 | 一次性或限时资源失效,包括 challenge 过期、QR 过期 |
| 422 | 请求格式正确但业务校验失败,包括签名失败、账户不匹配、ARCHIVE 验真失败 |
| 429 | 请求频率超过限制 |
| 500 | 未预期服务端错误 |
| 503 | 数据库、分片缓存、链节点或外部依赖不可用 |

死规则:

- `401` 只表示当前管理员登录态无效。
- 公民绑定 challenge 过期必须返回 `410 + SFID_BIND_CHALLENGE_EXPIRED`。
- 公民绑定签名失败必须返回 `422 + SFID_BIND_SIGNATURE_VERIFY_FAILED`。
- 公民绑定账户不匹配必须返回 `422 + SFID_BIND_ACCOUNT_MISMATCH`。
- 前端收到业务错误只展示错误,不得自动退出登录。

## 3. 当前错误码

### 认证

| error_code | HTTP | 含义 |
|---|---:|---|
| `SFID_AUTH_MISSING_TOKEN` | 401 | 未携带管理员 token |
| `SFID_AUTH_INVALID_ACCESS_TOKEN` | 401 | 管理员 token 无效 |
| `SFID_AUTH_ACCESS_TOKEN_EXPIRED` | 401 | 管理员 token 过期 |
| `SFID_AUTH_ADMIN_DISABLED` | 403 | 管理员已禁用 |
| `SFID_AUTH_PERMISSION_DENIED` | 403 | 管理员权限不足 |
| `SFID_AUTH_FORBIDDEN` | 403 | 已认证但访问被拒绝 |

### 公民绑定

| error_code | HTTP | 含义 |
|---|---:|---|
| `SFID_BIND_CHALLENGE_NOT_FOUND` | 404 | 绑定 challenge 不存在 |
| `SFID_BIND_CHALLENGE_CONSUMED` | 409 | 绑定 challenge 已消费 |
| `SFID_BIND_CHALLENGE_EXPIRED` | 410 | 绑定 challenge 已过期 |
| `SFID_BIND_ACCOUNT_MISMATCH` | 422 | 签名账户与 challenge 锁定账户不一致 |
| `SFID_BIND_SIGNATURE_FORMAT_INVALID` | 400 | 签名不是合法 hex |
| `SFID_BIND_SIGNATURE_VERIFY_FAILED` | 422 | 钱包签名验签失败 |
| `SFID_BIND_ARCHIVE_ALREADY_BOUND` | 409 | 档案号已绑定 |
| `SFID_BIND_PUBKEY_ALREADY_BOUND` | 409 | 公钥已绑定档案 |

### CPMS ARCHIVE 验真

| error_code | HTTP | 含义 |
|---|---:|---|
| `SFID_CITIZEN_ARCHIVE_SIGNATURE_BAD` | 422 | ARCHIVE 的 CPMS 签名无效 |
| `SFID_CITIZEN_ARCHIVE_GEO_SEAL_INVALID` | 422 | `geo_seal` 不能用已授权 CPMS 解密 |
| `SFID_CITIZEN_ARCHIVE_SCOPE_MISMATCH` | 422 | `geo_seal` 中的授权范围不匹配 |
| `SFID_CITIZEN_ARCHIVE_PUBKEY_MISMATCH` | 422 | CPMS 本机公钥与已绑定授权不一致 |
| `SFID_CITIZEN_QR_EXPIRED` | 410 | CPMS 状态 QR 已过期 |
| `SFID_CITIZEN_QR_HEADER_INVALID` | 400 | CPMS 状态 QR 头部字段非法 |

### 管理员账号

| error_code | HTTP | 含义 |
|---|---:|---|
| `SFID_ADMIN_PUBKEY_EXISTS_AS_SHENG_ADMIN` | 409 | 管理员公钥已作为省级管理员存在 |
| `SFID_ADMIN_PUBKEY_EXISTS_AS_SHI_ADMIN` | 409 | 管理员公钥已作为市级管理员存在 |
| `SFID_ADMIN_SHENG_ADMIN_PROVINCE_LIMIT_REACHED` | 409 | 本省省级管理员已达到 5 人上限 |
| `SFID_ADMIN_SHI_ADMIN_CITY_LIMIT_REACHED` | 409 | 本市市级管理员已达到 30 人上限 |
| `SFID_STORE_PERSIST_FAILED` | 500 | 写操作持久化失败，接口不得返回业务成功 |

管理员公钥全局唯一。新增省级管理员或市级管理员时，后端必须先按规范化公钥查全局
`admins` 账号表；命中后按已有角色返回上述稳定错误码，前端按新增目标角色展示中文提示。
省级管理员每省最多 5 人，市级管理员每省每市最多 30 人，达到上限时必须在后端拒绝新增。

## 4. 前端处理规则

`sfid/frontend/utils/http.ts` 必须满足:

- 成功时返回 `T`。
- 认证失效时抛 `AuthExpiredError`,并触发全局退出。
- 业务失败时抛 `ApiError`,页面按 `errorCode` 展示业务错误。
- 禁止用 `undefined as T` 表示失败。

公民绑定页只根据 `errorCode` 定制展示文案,不能根据 `message` 做分支。
