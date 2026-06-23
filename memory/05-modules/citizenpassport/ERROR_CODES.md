# CPMS 错误码规范

- 最后更新:2026-06-20
- 任务卡:`memory/08-tasks/open/20260530-cpms-security-deploy-hardening.md`

## 1. 总原则

CPMS 是完全离线实名系统。错误码只描述本机安装、管理员认证、实名档案、
ARCHIVE 签发、打印和审计状态,不得引入在线认证或 CID 远程调用语义。

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
| 401 | 只用于当前管理员登录态无效,包括缺 session、session 无效、session 过期 |
| 403 | 已识别管理员但权限不足、违反离线信任边界 |
| 404 | 管理员、档案、镇或地址段等对象不存在 |
| 409 | 重复档案号、重复管理员、公民状态或审核状态冲突 |
| 410 | challenge、扫码登录结果等限时资源已过期 |
| 422 | 请求格式正确但业务校验失败,包括签名验签失败、审核条件不满足 |
| 429 | 本机高风险入口限流 |
| 500 | 未预期服务端错误 |
| 503 | 本地数据库、签发密钥或本机存储不可用 |

死规则:

- `401` 不表示签名失败、challenge 过期或业务状态冲突。
- CPMS 不因任何错误码设计而联网。
- ARCHIVE 不包含实名原文,错误响应不得泄露实名原始数据。

## 3. 当前错误码

### 认证

| error_code | HTTP | 含义 |
|---|---:|---|
| `CPMS_RATE_LIMITED` | 429 | 登录、初始化、删除签名或资料上传入口请求过于频繁 |
| `CPMS_AUTH_MISSING_SESSION` | 401 | 未携带管理员 Cookie session |
| `CPMS_AUTH_INVALID_SESSION` | 401 | 管理员 Cookie session 无效 |
| `CPMS_AUTH_SESSION_EXPIRED` | 401 | 管理员 Cookie session 过期 |
| `CPMS_AUTH_PERMISSION_DENIED` | 403 | 管理员权限不足 |
| `CPMS_AUTH_CHALLENGE_NOT_FOUND` | 404/400 | 登录 challenge 不存在 |
| `CPMS_AUTH_CHALLENGE_CONSUMED` | 409/400 | 登录 challenge 已消费 |
| `CPMS_AUTH_CHALLENGE_EXPIRED` | 410 | 登录 challenge 已过期 |
| `CPMS_AUTH_CHALLENGE_MISMATCH` | 422/400 | challenge 与公钥或会话不匹配 |
| `CPMS_AUTH_SIGNATURE_VERIFY_FAILED` | 422 | 管理员签名验签失败 |

### 管理员

| error_code | HTTP | 含义 |
|---|---:|---|
| `CPMS_ADMIN_USER_GROUP_INVALID` | 400 | 管理员用户分组不是 `admins` 或 `operators` |
| `CPMS_ADMIN_NAME_REQUIRED` | 400 | 管理员姓名为空 |
| `CPMS_ADMIN_NAME_TOO_LONG` | 400 | 管理员姓名超过 50 个字符 |
| `CPMS_ADMIN_LIMIT_REACHED` | 409 | 管理员总数已达到 5 个 |
| `CPMS_ADMIN_INITIAL_ADMIN_IMMUTABLE` | 409 | 初始化绑定的管理员不可删除 |
| `CPMS_ADMIN_NOT_FOUND` | 404 | 管理员不存在 |
| `CPMS_ADMIN_PUBKEY_DUPLICATED` | 409 | 管理员账户公钥已存在 |

### 档案与签发

| error_code | HTTP | 含义 |
|---|---:|---|
| `CPMS_INTAKE_ARCHIVE_NOT_FOUND` | 404 | 档案不存在 |
| `CPMS_INTAKE_ARCHIVE_DUPLICATED` | 409 | 档案号冲突 |
| `CPMS_INTAKE_PASSPORT_DUPLICATED` | 409 | 护照号冲突 |
| `CPMS_INTAKE_PASSPORT_CAPACITY_EXHAUSTED` | 409 | 当前市护照号容量已耗尽 |
| `CPMS_INTAKE_PASSPORT_AREA_INVALID` | 400 | 安装码省市代码不能用于生成护照号 |
| `CPMS_INTAKE_ADDRESS_AREA_NOT_FOUND` | 404 | 镇或地址段不属于当前 CPMS 安装城市或不存在 |
| `CPMS_INTAKE_CITIZEN_STATUS_INVALID` | 400 | 公民状态值非法 |
| `CPMS_ANNUAL_STATUS_EXPORT_REQUIRED` | 423 | 已超过年度报告宽限日,操作员需等待管理员完成导出 |
| `CPMS_ANNUAL_STATUS_EXPORT_NOT_REQUIRED` | 409 | 当前没有待导出的年度报告 |
| `CPMS_ISSUE_QR_GENERATE_FAILED` | 500 | ARCHIVE 二维码生成失败 |
| `CPMS_AUDIT_WRITE_FAILED` | 500 | 审计或打印记录写入失败 |
| `CPMS_ARCHIVE_WALLET_REQUIRED` | 400 | 档案缺少钱包地址,不能签出 ARCHIVE |
| `CPMS_ARCHIVE_WALLET_ADDRESS_INVALID` | 400 | 钱包地址不是合法 SS58 地址 |
| `CPMS_ARCHIVE_WALLET_ALREADY_BOUND` | 409 | 钱包账户已绑定其他未硬删除档案 |
| `CPMS_ARCHIVE_ALREADY_DELETED` | 409 | 档案已软删除,不能继续业务操作 |
| `CPMS_ARCHIVE_DELETE_CHALLENGE_NOT_FOUND` | 404 | 删除签名 challenge 不存在 |
| `CPMS_ARCHIVE_DELETE_CHALLENGE_CONSUMED` | 409 | 删除签名 challenge 已消费 |
| `CPMS_ARCHIVE_DELETE_CHALLENGE_EXPIRED` | 410 | 删除签名 challenge 已过期 |
| `CPMS_ARCHIVE_DELETE_CHALLENGE_MISMATCH` | 422 | 删除签名 challenge 与当前档案或管理员不匹配 |
| `CPMS_ARCHIVE_DELETE_SIGNER_MISMATCH` | 422 | CitizenWallet 签名账户不是当前登录管理员账户 |
| `CPMS_ARCHIVE_DELETE_PAYLOAD_HASH_MISMATCH` | 422 | 生成方本地 session 的 payload_hash 与删除 payload 不一致 |
| `CPMS_ARCHIVE_DELETE_SIGNATURE_INVALID` | 422 | CitizenWallet 删除签名验签失败 |
| `CPMS_ARCHIVE_MATERIAL_NOT_FOUND` | 404 | 公民资料库记录不存在或已删除 |
| `CPMS_ARCHIVE_MATERIAL_FILE_NOT_FOUND` | 404 | 公民资料库文件正文不存在 |
| `CPMS_ARCHIVE_MATERIAL_TYPE_INVALID` | 400 | 公民资料类型非法 |
| `CPMS_ARCHIVE_MATERIAL_MIME_INVALID` | 400 | 公民资料文件 MIME 与资料类型不匹配 |
| `CPMS_ARCHIVE_MATERIAL_FILE_REQUIRED` | 400 | 上传公民资料时未提供文件 |
| `CPMS_ARCHIVE_MATERIAL_FILE_TOO_LARGE` | 413 | 上传公民资料超过 100 MB |
| `CPMS_ARCHIVE_MATERIAL_FILE_EMPTY` | 400 | 上传公民资料为空文件 |

## 4. 前端处理规则

CPMS 前端收到 `401` 时只清除本地用户镜像并通知认证上下文，实际会话以 HttpOnly
Cookie 为准；是否进入 `/login` 由路由守卫决定，未初始化系统必须优先进入 `/install`。
其他 `4xx/5xx` 业务错误只展示 `message`,不得自动退出。
