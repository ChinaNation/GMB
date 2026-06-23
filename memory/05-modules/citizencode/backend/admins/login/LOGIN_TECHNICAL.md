# CID Backend Admin Login 技术说明

- 更新日期:2026-06-22
- 协议事实源:`memory/01-architecture/qr/qr-protocol-spec.md`

## 1. 职责

`citizencode/backend/admins/login/` 负责 CID 管理员认证:

1. 普通管理员身份识别、登录签名、会话守卫。
2. 生成 `QR_V1 k=1,a=1` 管理员扫码登录签名请求。
3. 接收 CitizenWallet 公民钱包返回的 `QR_V1 k=2` 签名响应。
4. 校验管理员、公钥、签名、会话和过期时间。
5. 返回登录结果给前端轮询。

## 2. 文件边界

| 文件 | 注释 |
|---|---|
| `model.rs` | 登录会话、登录签名请求、QR 登录结果和 DTO |
| `handler.rs` | 普通登录与会话接口 |
| `qr_login.rs` | QR 登录签名请求、签名响应提交、结果轮询 |
| `signature.rs` | 登录签名原文、系统签名和管理员验签 |
| `guards.rs` | 登录态守卫 |

## 3. QR API

| 方法 | 路径 | 注释 |
|---|---|---|
| `POST` | `/api/v1/admin/auth/qr/sign-request` | 生成登录签名请求二维码 |
| `POST` | `/api/v1/admin/auth/qr/complete` | 提交签名响应并完成登录 |
| `GET` | `/api/v1/admin/auth/qr/result` | 前端轮询登录结果 |

## 4. 运行态表

| 表 | 注释 |
|---|---|
| `admin_login_sign_requests` | QR/普通登录的一次性签名请求状态 |
| `admin_qr_login_results` | QR 登录轮询结果 |
| `admin_sessions` | 管理员登录会话 |

运行态清理由 `cleanup_login_state_conn()` 执行。不得恢复旧登录二维码表名。

## 5. QR 字段

登录签名请求:

```json
{"p":"QR_V1","k":1,"i":"...","e":1780000000,"b":{"a":1,"g":1,"u":"...","d":"..."}}
```

`b.d` 为 UTF-8 `cid|system_signature`。CitizenWallet 先验系统签名,再让管理员确认登录。

签名响应:

```json
{"p":"QR_V1","k":2,"i":"...","e":1780000000,"b":{"u":"...","s":"..."}}
```

CID 后端必须按本地 `admin_login_sign_requests` 重建签名原文:

```text
QR_V1|2|i|cid|e|pubkey_without_0x
```

## 6. 安全规则

1. CitizenApp 不承担管理员扫码登录职责。
2. CitizenWallet 公民钱包是管理员扫码登录唯一签名端。
3. 签名响应不携带 payload、payload hash 或展示字段。
4. 前端只负责展示二维码、扫描签名响应和提交 API。
5. 联邦注册局/市注册局身份仍由 `registry_org_code` 表达,不得恢复独立管理员授权真源。
