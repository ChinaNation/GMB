# CPMS 错误码规范

- 最后更新:2026-05-26
- 任务卡:`memory/08-tasks/open/20260526-error-codes.md`

## 1. 总原则

CPMS 是完全离线实名系统。错误码只描述本机安装、管理员认证、实名档案、
ARCHIVE 签发、打印和审计状态,不得引入在线认证或 SFID 远程调用语义。

错误响应结构:

```json
{
  "code": 2007,
  "error_code": "CPMS_AUTH_SIGNATURE_VERIFY_FAILED",
  "message": "signature verify failed",
  "trace_id": "..."
}
```

`error_code` 是前端和排障使用的稳定业务错误码;`message` 只用于展示或日志辅助。

## 2. HTTP 状态码边界

| HTTP 状态 | 使用边界 |
|---|---|
| 400 | 请求字段缺失、格式错误、安装码/二维码 JSON 格式错误 |
| 401 | 只用于当前管理员登录态无效,包括缺 token、token 无效、token 过期 |
| 403 | 已识别管理员但权限不足、管理员停用、违反离线信任边界 |
| 404 | 操作员、档案、镇村等对象不存在 |
| 409 | 重复档案号、重复管理员、公民状态或审核状态冲突 |
| 410 | challenge、扫码登录结果等限时资源已过期 |
| 422 | 请求格式正确但业务校验失败,包括签名验签失败、审核条件不满足 |
| 500 | 未预期服务端错误 |
| 503 | 本地数据库、签发密钥或本机存储不可用 |

死规则:

- `401` 不表示签名失败、challenge 过期、管理员停用或业务状态冲突。
- CPMS 不因任何错误码设计而联网。
- ARCHIVE 不包含实名原文,错误响应不得泄露实名原始数据。

## 3. 当前错误码

### 认证

| error_code | HTTP | 含义 |
|---|---:|---|
| `CPMS_AUTH_MISSING_TOKEN` | 401 | 未携带管理员 token |
| `CPMS_AUTH_INVALID_TOKEN` | 401 | 管理员 token 无效 |
| `CPMS_AUTH_TOKEN_EXPIRED` | 401 | 管理员 token 过期 |
| `CPMS_AUTH_ADMIN_INACTIVE` | 403 | 管理员未激活或已停用 |
| `CPMS_AUTH_PERMISSION_DENIED` | 403 | 管理员权限不足 |
| `CPMS_AUTH_CHALLENGE_NOT_FOUND` | 404/400 | 登录 challenge 不存在 |
| `CPMS_AUTH_CHALLENGE_CONSUMED` | 409/400 | 登录 challenge 已消费 |
| `CPMS_AUTH_CHALLENGE_EXPIRED` | 410 | 登录 challenge 已过期 |
| `CPMS_AUTH_CHALLENGE_MISMATCH` | 422/400 | challenge 与公钥或会话不匹配 |
| `CPMS_AUTH_SIGNATURE_VERIFY_FAILED` | 422 | 管理员签名验签失败 |

### 档案与签发

| error_code | HTTP | 含义 |
|---|---:|---|
| `CPMS_INTAKE_ARCHIVE_NOT_FOUND` | 404 | 档案不存在 |
| `CPMS_INTAKE_ARCHIVE_DUPLICATED` | 409 | 档案号冲突 |
| `CPMS_INTAKE_CITIZEN_STATUS_INVALID` | 400 | 公民状态值非法 |
| `CPMS_ISSUE_QR_GENERATE_FAILED` | 500 | ARCHIVE 二维码生成失败 |
| `CPMS_AUDIT_WRITE_FAILED` | 500 | 审计或打印记录写入失败 |

## 4. 前端处理规则

CPMS 前端只在收到 `401` 且本地存在 token 时清除登录态并回到登录页。
其他 `4xx/5xx` 业务错误只展示 `message`,不得自动退出。
