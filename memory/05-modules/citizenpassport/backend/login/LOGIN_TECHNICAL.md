# CPMS Backend Login 技术说明

- 更新日期:2026-06-22
- 协议事实源:`memory/01-architecture/qr/qr-protocol-spec.md`

## 1. 职责

`citizenpassport/backend/login/` 负责 CPMS 管理员登录:

1. 生成 `QR_V1 k=1,a=1` 登录签名请求。
2. 接收 CitizenWallet 公民钱包返回的 `QR_V1 k=2` 签名响应。
3. 验证管理员公钥、签名、会话和过期时间。
4. 创建 HttpOnly 登录 Cookie。
5. 查询当前登录管理员。

## 2. API

| 方法 | 路径 | 注释 |
|---|---|---|
| `POST` | `/api/v1/admin/auth/qr/sign-request` | 生成登录签名请求二维码 |
| `POST` | `/api/v1/admin/auth/qr/complete` | 提交签名响应并完成登录 |
| `GET` | `/api/v1/admin/auth/qr/result` | 浏览器轮询登录结果 |
| `GET` | `/api/v1/admin/auth/me` | 查询当前登录态 |
| `POST` | `/api/v1/admin/auth/logout` | 退出登录 |

## 3. 运行态表

| 表 | 注释 |
|---|---|
| `sessions` | 登录会话 |
| `login_sign_requests` | 登录签名请求短期状态 |
| `qr_login_results` | 浏览器轮询结果 |

这些表只保存短生命周期状态,由 `StoreDb::cleanup_auth_runtime()` 清理。

## 4. QR 字段

登录请求:

```json
{"p":"QR_V1","k":1,"i":"...","e":1780000000,"b":{"a":1,"g":1,"u":"...","d":"..."}}
```

`b.d` 为 UTF-8 `cpms|system_signature`。系统签名用于 CitizenWallet 确认二维码确实由 CPMS 签发。

签名响应:

```json
{"p":"QR_V1","k":2,"i":"...","e":1780000000,"b":{"u":"...","s":"..."}}
```

后端必须用本地 `login_sign_requests` 找回原请求,不得从签名响应读取业务 payload。

## 5. 验签规则

1. `challenge_id/session_id` 与本地请求一致。
2. 请求未过期且未消费。
3. `b.u` 对应的管理员存在且具备登录资格。
4. 签名原文按 `QR_V1|2|i|cpms|e|pubkey_without_0x` 重建。
5. sr25519 验签通过后才消费请求并创建 session。

任何旧 QR 字段、旧路由或旧表名都不得恢复。
